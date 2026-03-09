---
default: patch
---

Suppress noisy third-party crate logs (e.g. webrtc-rs ICE messages) from the UI log panel. Only WARN+ events from non-wail crates are forwarded to the frontend.
