---
bonding_curve: user
---

# Preview trades without authorizing them

## User impact

Quotes and commands run with `--simulate` no longer authorize an on-chain transaction. An RPC receiving a preview cannot reuse it to spend your tokens.
