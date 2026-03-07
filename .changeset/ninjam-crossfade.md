---
default: minor
---

Add NINJAM-style crossfade at every interval boundary to prevent clicks/pops. The tail of the outgoing interval is linearly blended with the head of the incoming interval over a 10ms window, matching NINJAM's overlap-add splicing pattern.
