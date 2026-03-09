---
default: patch
---

Compensate for one-way network latency when snapping beat at join time. The `forceBeatAtTime` call now adds `RTT/2 * BPM/60` beats to account for the transit time of the first `StateSnapshot`, reducing join-time beat offset at higher latencies.
