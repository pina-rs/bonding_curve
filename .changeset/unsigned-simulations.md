---
bonding_curve: fix
---

# Keep RPC simulations unsigned

The CLI constructs simulation transactions with zero signatures and a placeholder blockhash. Only the send path fetches a recent blockhash and signs, preventing an RPC from broadcasting a quote or preview transaction.
