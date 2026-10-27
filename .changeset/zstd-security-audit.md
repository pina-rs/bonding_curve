---
bonding_curve: fix
---

# Patch audited dependencies and document compression reachability

Upgrade the CLI and separate Surfpool test workspace's transitive `zstd-sys` dependency to 2.1.2. Record three API-specific advisory exceptions for the SDK's required zstd 0.13 / zstd-safe 7 versions, with the exact unaffected RPC decoder path and disabled seekable feature documented in the security guide. All other advisory checks remain enforced.

Update the website's Zod and Undici dependencies to patched releases. Miniflare's exact Undici pin receives a version-specific override on the same 7.x line; the other updates stay within their existing parent ranges. No JavaScript advisory is suppressed.
