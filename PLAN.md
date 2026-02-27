# Implementation Plan: Intervalic Audio via CLAP/VST3 + Opus over WebRTC DataChannels

## Architecture Overview

```
DAW Track → [WAIL CLAP Plugin] → capture f32 audio per interval
                                → Opus encode (48kHz, high quality)
                                → Local IPC (Unix socket) to WAIL App
                                → WebRTC DataChannel to remote peers
                                → Remote WAIL App receives
                                → IPC to remote plugin
                                → Opus decode → f32 playback in DAW
```

The NINJAM-style approach: record for one interval, transmit, remote plays back
previous interval while recording current. Latency = exactly 1 interval (by design).
DataChannels are ideal here — no need for WebRTC media tracks since intervalic
delivery doesn't require sub-frame latency.

## New Crates

### 1. `wail-plugin` (CLAP/VST3 via nih-plug)
Stereo audio plugin that captures DAW audio per interval and plays back remote audio.

### 2. `wail-audio` (audio encoding/protocol)
Opus encoding/decoding + interval audio message types. Shared by plugin and app.

## Changes to Existing Crates

### 3. `wail-core` — Add AudioInterval protocol messages
### 4. `wail-net` — Add binary DataChannel for audio (alongside existing JSON "sync" channel)
### 5. `wail-app` — Bridge plugin IPC ↔ WebRTC audio DataChannel

## Phase 1: Link 4 Submodule + Audio Infrastructure

- Add Ableton/link as git submodule at `vendor/link` (tag Link-4.0.0b1)
- Create `wail-audio` crate with Opus encode/decode
- Add `AudioInterval` message to protocol
- Add binary "audio" DataChannel to wail-net

## Phase 2: CLAP/VST3 Plugin

- Create `wail-plugin` crate with nih-plug
- Audio capture per interval (ring buffer)
- Opus encode captured intervals
- Local IPC server/client for plugin ↔ app communication
- Playback of received remote intervals

## Phase 3: App Integration

- Update wail-app to handle audio intervals
- Bridge IPC ↔ WebRTC DataChannel
- Interval-synchronized audio relay
