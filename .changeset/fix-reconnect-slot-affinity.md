---
default: patch
---

Fix reconnecting peers being mapped to a different channel slot.

When a peer crashed and reconnected with a new peer_id before the old connection was cleaned up, the old slot remained occupied, forcing the reconnecting peer onto a new slot. The session now evicts the stale peer_id when a Hello arrives with an identity that already belongs to a different tracked peer, freeing the slot for the reconnecting peer to reclaim via affinity.
