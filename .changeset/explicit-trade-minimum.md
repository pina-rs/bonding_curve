---
bonding_curve: breaking
---

# Require explicit trade output limits

CLI buys require `--minimum-base-out` and sales require `--minimum-quote-out`, both nonzero. Remove `--slippage-bps`: simulation events from the selected RPC are advisory and never determine the signed floor. Display the exact minimum before signing and in the result.
