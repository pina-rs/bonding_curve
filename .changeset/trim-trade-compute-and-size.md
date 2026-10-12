---
bonding_curve:
  bump: minor
  type: fix
---

# Trim the deployed program and trade compute

Stops repricing configuration work on every trade: stored curves are trusted instead of revalidated, the 256-bit division is compiled once, trades copy only the pricing fields out of a configuration, and signer seeds are built through one shared helper. The deployed binary shrinks by about 13 KB and buys and sells get 440 and 375 compute units cheaper. No account, instruction, event, or wire format changes.
