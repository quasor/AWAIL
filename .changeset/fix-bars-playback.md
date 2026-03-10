---
default: patch
---

Fix peers hearing partial/compressed audio in NINJAM intervals. The outgoing audio guard incorrectly blocked all frames during interval index 0, and the Opus decoder crashed on missing frames instead of using Packet Loss Concealment.
