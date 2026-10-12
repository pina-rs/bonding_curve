---
bonding_curve: major
---

# Reject the launch itself as its creator

`SetLaunchCreator` now fails with the new append-only error `LaunchAsCreator` (24) when `new_creator` is the launch's own address. The launch PDA can never sign, so its creator fees and vested allocation would be unclaimable, and with a nonzero `creator_lp_share` its LP payout targets the launch's own LP token account. `Graduate` then names that writable account twice, which Pina rejects as `DuplicateMutableAccount`, so no account list could graduate the launch and every trader's quote would stay locked in a completed launch. The Surfpool suite covers the rejection, and `docs/instructions.md` now lists error codes 22 to 24.
