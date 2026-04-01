//! Shared test tone generation and audio validation utilities.
//!
//! Used by both `wail-e2e` (two-machine tests) and `wail-tauri` (test mode)
//! to generate synthetic audio and validate received audio without a DAW.

use anyhow::{bail, Result};

use crate::codec::AudioEncoder;
use crate::interval::AudioFrame;
use crate::wire::AudioFrameWire;

const SAMPLE_RATE: u32 = 48000;
const CHANNELS: u16 = 2;
const OPUS_BITRATE: u32 = 128;

/// Result of validating received audio data.
pub struct AudioValidation {
    /// Wire format: "WAIF" (frame)
    pub format: String,
    /// Total wire size in bytes
    pub size_bytes: usize,
    /// RMS energy of decoded PCM (0.0 = silence)
    pub rms: f32,
    /// Human-readable detail string
    pub detail: String,
}

/// Encode a synthetic sine-wave test tone as multiple WAIF frames.
///
/// Computes the real frame count from BPM/bars/quantum (matching what a
/// DAW plugin would produce), then generates that many 20ms Opus frames.
/// Each frame is wrapped in WAIF wire format; the last is marked final
/// with interval metadata.
pub fn encode_test_interval(
    index: i64,
    freq: f32,
    bpm: f64,
    bars: u32,
    quantum: f64,
) -> Result<Vec<Vec<u8>>> {
    // Compute interval duration from musical parameters (same as real plugins)
    let beats_per_interval = bars as f64 * quantum;
    let interval_seconds = beats_per_interval / (bpm / 60.0);
    let frame_duration_s = 0.020; // 20ms per Opus frame
    let total_frames = (interval_seconds / frame_duration_s).round().max(1.0) as u32;

    let samples_per_frame = 960usize; // 20ms at 48kHz
    let num_samples = samples_per_frame * CHANNELS as usize;
    let mut encoder = AudioEncoder::new(SAMPLE_RATE, CHANNELS, OPUS_BITRATE)?;
    let mut phase: f64 = 0.0;
    let phase_inc = freq as f64 / SAMPLE_RATE as f64;
    let mut frames = Vec::with_capacity(total_frames as usize);

    for frame_num in 0..total_frames {
        let mut samples = vec![0.0f32; num_samples];
        for i in 0..samples_per_frame {
            let val = (2.0 * std::f64::consts::PI * phase).sin() as f32 * 0.5;
            samples[i * 2] = val;
            samples[i * 2 + 1] = val;
            phase += phase_inc;
        }

        let opus_data = encoder.encode_frame(&samples)?;
        let is_final = frame_num == total_frames - 1;

        let frame = AudioFrame {
            interval_index: index,
            stream_id: 0,
            frame_number: frame_num,
            channels: CHANNELS,
            opus_data,
            is_final,
            sample_rate: if is_final { SAMPLE_RATE } else { 0 },
            total_frames: if is_final { total_frames } else { 0 },
            bpm: if is_final { bpm } else { 0.0 },
            quantum: if is_final { quantum } else { 0.0 },
            bars: if is_final { bars } else { 0 },
        };
        frames.push(AudioFrameWire::encode(&frame));
    }
    Ok(frames)
}

/// Validate received audio wire data: decode, check format, return details.
pub fn validate_audio(data: &[u8]) -> Result<AudioValidation> {
    if data.len() < 4 {
        bail!("audio data too short ({} bytes)", data.len());
    }

    if &data[0..4] == b"WAIF" {
        let frame = AudioFrameWire::decode(data)?;
        let detail = format!(
            "WAIF frame: {} bytes, frame #{}, interval {}, final={}",
            data.len(),
            frame.frame_number,
            frame.interval_index,
            frame.is_final,
        );

        Ok(AudioValidation {
            format: "WAIF".into(),
            size_bytes: data.len(),
            rms: 0.0,
            detail,
        })
    } else {
        bail!(
            "unknown wire format: magic={:?}",
            &data[..data.len().min(4)]
        );
    }
}

/// RMS (root mean square) energy of an audio buffer.
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum / samples.len() as f64).sqrt() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let frames = encode_test_interval(42, 440.0, 120.0, 4, 4.0).unwrap();
        // At 120 BPM, 4 bars, quantum 4: 16 beats = 8 seconds = 400 frames
        assert_eq!(frames.len(), 400);
        // Validate last frame (final)
        let validation = validate_audio(&frames[frames.len() - 1]).unwrap();
        assert_eq!(validation.format, "WAIF");
        assert!(validation.detail.contains("interval 42"));
        assert!(validation.detail.contains("final=true"));
        // Validate first frame (not final)
        let first = validate_audio(&frames[0]).unwrap();
        assert!(first.detail.contains("final=false"));
    }

    #[test]
    fn rms_of_silence_is_zero() {
        let silence = vec![0.0f32; 1920];
        assert_eq!(rms(&silence), 0.0);
    }

    #[test]
    fn rms_of_signal_is_nonzero() {
        let mut samples = vec![0.0f32; 1920];
        for i in 0..960 {
            let val = (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin() * 0.5;
            samples[i * 2] = val;
            samples[i * 2 + 1] = val;
        }
        assert!(rms(&samples) > 0.1);
    }

    #[test]
    fn validate_rejects_garbage() {
        let garbage = vec![0u8; 10];
        assert!(validate_audio(&garbage).is_err());
    }

    #[test]
    fn validate_rejects_short_data() {
        let short = vec![0u8; 2];
        assert!(validate_audio(&short).is_err());
    }
}
