---
bonding_curve: user
---

# Mis-sent quote is no longer stranded

## User impact

Quote tokens sent straight to a launch's vault used to sit above the launch's accounting forever. Anyone can now call `SweepQuoteDust` to split that stray balance between the token's creator and the launchpad by the configured fee share, before or after graduation, and creator handovers are visible in transaction logs.
