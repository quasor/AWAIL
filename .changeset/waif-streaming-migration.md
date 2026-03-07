---
default: patch
---

Migrate send/recv plugins and recorder to WAIF streaming format. Send plugin now streams 20ms WAIF frames instead of full WAIL intervals. Recv plugin and recorder use shared FrameAssembler to reassemble frames into complete intervals. Fixed frame loss from undersized channel buffers and non-blocking sends.
