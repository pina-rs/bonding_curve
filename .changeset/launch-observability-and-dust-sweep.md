---
bonding_curve: major
---

# Creator-handover events and a dust sweep

- `SetLaunchCreator` emits `LaunchCreatorChanged` with the previous and new creator, so indexers follow creator-rights changes without polling accounts.
- A new permissionless `SweepQuoteDust` instruction splits quote that arrived in a launch's quote vault above its accounting — a donation or a mis-sent transfer — into the creator's and partner's fee balances by the configuration's `creator_fee_share`. The reserve and accrued fees are never touched, and it works in every launch status. Emits `DustSwept`.
- Clients, IDL, and ABI layout regenerated for the new instruction and events.
