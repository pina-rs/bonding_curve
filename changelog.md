# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/pina-rs/bonding_curve/releases/tag/v0.2.0) (2026-10-12)

Grouped release for `bonding_curve`.

### Breaking changes

#### Harden graduation payouts and creator handover

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #9](https://github.com/pina-rs/bonding_curve/pull/9)

- A partner who launches under their own configuration no longer bricks graduation: when the launch creator and the partner are the same address, the creator and partner LP shares are paid as one transfer to that address's LP token account instead of naming the same writable account twice, which the runtime rejects.
- `SetLaunchCreator` rejects the default address, which could otherwise hold creator rights that can never sign, leaving fees and the vested allocation unclaimable forever.
- `Graduate` asserts the launch's price equals the configuration's migration price before seeding the pool, turning any future state drift into a failed transaction instead of a mispriced pool.

#### Creator-handover events and a dust sweep

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #10](https://github.com/pina-rs/bonding_curve/pull/10) · _Related issues:_ [#9](https://github.com/pina-rs/bonding_curve/issues/9)

- `SetLaunchCreator` emits `LaunchCreatorChanged` with the previous and new creator, so indexers follow creator-rights changes without polling accounts.
- A new permissionless `SweepQuoteDust` instruction splits quote that arrived in a launch's quote vault above its accounting — a donation or a mis-sent transfer — into the creator's and partner's fee balances by the configuration's `creator_fee_share`. The reserve and accrued fees are never touched, and it works in every launch status. Emits `DustSwept`.
- Clients, IDL, and ABI layout regenerated for the new instruction and events.

#### Move the program to its mainnet identity and graduate into the mainnet AMM

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #18](https://github.com/pina-rs/bonding_curve/pull/18) · _Related issues:_ [#16](https://github.com/pina-rs/bonding_curve/issues/16)

The program id is now `Curve1GqpqznjxpJzH313L9CRhAehHaikb4SJMFQeBvP`, so every PDA moves with it: the AMM authority that graduation tiers must restrict pool creation to is now `HUjATWLMQR85P71zHVoHgbRPTY7xx4gEdyDg7EMin7XX`. `pina migrations create` rebound the unpublished migration history to the new id; `publications.json` still has no receipts.

Graduation now calls the Pina AMM at `pAMMzVPRwtrxQs2CjLTUUpfCAp9XdvVmwwVhpAuCobi`. `amm.rev` pins the AMM commit that carries that identity, and `vendor/pina_amm` is re-imported from its IDL. The CPI, Rust, TypeScript, and Dart clients are regenerated with both ids, and `addresses.rs`, the docs, the website, and the tests name the new addresses.

The deployed binary now embeds a `security.txt` through `solana-security-txt` 1.1.3, only under `bpf-entrypoint`. It names the project, the private advisory form as contact, `SECURITY.md` as policy, and the source repository.

#### Reject the launch itself as its creator

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #16](https://github.com/pina-rs/bonding_curve/pull/16)

`SetLaunchCreator` now fails with the new append-only error `LaunchAsCreator` (24) when `new_creator` is the launch's own address. The launch PDA can never sign, so its creator fees and vested allocation would be unclaimable, and with a nonzero `creator_lp_share` its LP payout targets the launch's own LP token account. `Graduate` then names that writable account twice, which Pina rejects as `DuplicateMutableAccount`, so no account list could graduate the launch and every trader's quote would stay locked in a completed launch. The Surfpool suite covers the rejection, and `docs/instructions.md` now lists error codes 22 to 24.

### Features

#### Client journeys and a performance gate

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #11](https://github.com/pina-rs/bonding_curve/pull/11) · _Related issues:_ [#10](https://github.com/pina-rs/bonding_curve/issues/10), [#9](https://github.com/pina-rs/bonding_curve/issues/9)

- Every instruction now has one ordered journey on a live Surfnet through the generated Rust client (`every_instruction_runs_in_one_journey`) and through the generated TypeScript client (`typescript_client_runs_every_instruction`), which the Rust side drives with funded fixtures — the restricted AMM tier, both mints, and every derived address — and then checks the resulting on-chain state, so the TypeScript client's wire format is proven end to end against the deployed program beside the pinned AMM. The TypeScript journey asserts its PDA helpers reproduce the Rust-derived addresses before sending anything.
- The journeys' keypairs are seeded, so identical binaries measure identical compute units.
- A `performance` workflow benchmarks the pull request against its base (each side against its own `amm.rev`): deployed binary size plus every instruction's compute units (median of three journey runs), posted as one consolidated PR comment. The policy fails the check on any binary-size increase or a compute-unit regression above 2%, unless the pull request carries `performance-approved`. `scripts/benchmark.ts` and `scripts/compare-benchmarks.ts` run the same pipeline locally.
- `docs/performance.md` documents the pipeline and the policy.

### Fixes

- **Trim the deployed program and trade compute.** Stops repricing configuration work on every trade: stored curves are trusted instead of revalidated, the 256-bit division is compiled once, trades copy only the pricing fields out of a configuration, and signer seeds are built through one shared helper. The deployed binary shrinks by about 11 KB and buys and sells get about 830 and 760 compute units cheaper. No account, instruction, event, or wire format changes.
  _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #15](https://github.com/pina-rs/bonding_curve/pull/15)

## [0.1.0](https://github.com/pina-rs/bonding_curve/releases/tag/v0.1.0) (2026-10-08)

Grouped release for `bonding_curve`.

### Features

#### Launch the Pina Bonding Curve program, clients, and CLI

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #1](https://github.com/pina-rs/bonding_curve/pull/1)

The first release of the Pina Bonding Curve: self-serve launchpads built with Pina 0.23 that graduate into the Pina AMM.

- **Program** (`CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9`): immutable partner configurations with up to 16 concentrated-liquidity curve segments in Q64.64 square-root prices, fixed-supply launches that revoke the mint authority, buys and sells with a linearly decaying anti-sniping fee split between creator and partner, completion at the derived migration price with partial fills, permissionless graduation into a Pina AMM tier restricted to the curve's AMM authority PDA (price-continuous seeding, migration fee, LP payout or burn, surplus base burn), vested creator allocations, and fee claims. A buy costs about 8,400 compute units.
- **`pina_bonding_curve_client`**, **`pina_bonding_curve_cpi`**, **`@pina-rs/bonding-curve`**, and **`pina_bonding_curve`** (Dart): generated from the program IDL with the version-0 migration baseline.
- **`pina_bonding_curve_cli`**: the `pina-curve` binary, which designs curves offline, previews configurations by simulation, quotes trades by simulation, applies slippage limits, and wraps SOL for wrapped-SOL launches.

The Pina AMM dependency is pinned in `amm.rev` and vendored as a generated CPI crate. Unit and property tests cover the curve math and rounding; an end-to-end Surfpool suite deploys the curve beside the pinned AMM, drives every instruction through the generated Rust client and the CLI, and enforces compute-unit ceilings.
