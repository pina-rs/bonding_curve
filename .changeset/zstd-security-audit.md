---
bonding_curve: fix
---

# Update compression dependency and document audit reachability

Upgrade the CLI and separate Surfpool test workspace's transitive `zstd-sys` dependency to 2.1.2. Record three API-specific advisory exceptions for the SDK's required zstd 0.13 / zstd-safe 7 versions, with the exact unaffected RPC decoder path and disabled seekable feature documented in the security guide. All other advisory checks remain enforced.
