---
default: patch
---

Remove AudioSendGate that could permanently block audio after signaling reconnect. Add INFO/WARN logging for audio transmission milestones and frame drops.
