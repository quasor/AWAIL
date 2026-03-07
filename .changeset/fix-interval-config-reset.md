---
default: patch
---

Fix IntervalTracker::set_config to only reset interval tracking when bars or quantum actually change. Previously, receiving a redundant IntervalConfig message (same values) would reset the tracker, briefly re-activating the warmup guard and potentially dropping outgoing audio. Also adds diagnostic logging at interval boundary swaps to help diagnose audio gap issues.
