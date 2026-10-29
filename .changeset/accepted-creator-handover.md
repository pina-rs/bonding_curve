---
bonding_curve: breaking
---

# Require accepted creator handovers

`SetLaunchCreator` adds a required `new_creator_signer` account matching the requested creator. Both creators sign, preventing handovers to deterministic graduation accounts that cannot accept them. Reject the base mint as a successor and reject launch creation when a nonzero LP recipient aliases that mint. The CLI adds `--new-creator-keypair`.
