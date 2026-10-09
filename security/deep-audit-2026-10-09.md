# Deep security audit — 2026-10-09

Base: `main` @ 2c7675d (pinned AMM `amm.rev` = `fe0f261`, whose generated CPI client is byte-identical to the AMM repository's current `main`). Scope: full repository, centered on a line-by-line review of the on-chain program (`programs/pina_bonding_curve/src/`, all ~2,100 lines: curve and math modules with their property tests, state, every processor, token helpers), the graduation CPI boundary into the Pina AMM (including a diff of `vendor/pina_amm` against the pinned revision), the CLI, CI and `devenv` pipeline, and `docs/`. Method: the solana-audit skill taxonomy (operations / code / economics layers), applied class by class, with a tooling pass (`cargo test -p pina_bonding_curve --lib` — 19/19 pass) and an incident cross-check (pump.fun and launchpad patterns, Mango/Nirvana price-manipulation patterns, Cetus overflow, Wormhole/Cashio account-trust patterns).

Reviewed together with the Pina AMM repository, whose audit report carries the AMM-side half of the integration findings.

## Summary

| Severity      | Count                                           |
| ------------- | ----------------------------------------------- |
| Critical      | 0                                               |
| High          | 0                                               |
| Medium        | 1 (fixed in this PR)                            |
| Low           | 1 (fixed in this PR) + 1 defensive assert added |
| Informational | 5                                               |

**Verdict:** the program's core is sound. The piecewise concentrated-liquidity curve is fully checked 256-bit arithmetic with every rounding favouring the curve, and the property tests genuinely pin the load-bearing invariants (reserves always cover the curve's integral, round trips never profit, split buys never beat single buys, the supply is fixed). Vault solvency is maintained end to end: the tracked `quote_reserve` plus fee counters always equals the vault balance, sells cannot exceed what the curve priced, and graduation pays only from balances it re-reads. The supply-chain story is unusually good for a cross-program dependency: `amm.rev` pins a commit, `vendor/pina_amm` matches it byte for byte, and `check:clients` regenerates and diffs both in CI.

## Medium (fixed in this PR)

### BC-1 — Graduation is unexecutable when the launch creator is the configuration's partner and both LP shares are nonzero

1. **Location:** `programs/pina_bonding_curve/src/processors/graduate.rs` (`pay_lp` called once for the creator pair, once for the partner pair); account lists at `GraduateAccounts.creator/creator_lp_token/partner/partner_lp_token`.
2. **Mechanism:** with `creator_lp_share > 0` and `partner_lp_share > 0` — the normal launchpad split — graduation must pay both recipients' associated token accounts for the new LP mint. When `launch.creator == config.authority` (a partner launching under their own configuration), both payouts target the _same_ ATA of the same wallet and mint. Passing that account twice fails at parse time with `DuplicateMutableAccount` (pina rejects a repeated writable account, `pina/src/traits.rs:1250`); omitting either pair fails with `MissingLpAccount`. No account list can satisfy the instruction, so the launch can never graduate.
3. **Exploit scenario:** not attacker-profitable — it triggers on an innocent, plausible configuration (every partner dogfooding their own launchpad with split LP shares). After the launch completes, trading stops (`NotTrading`), and the raised quote plus unsold base stay locked in the vaults until someone notices that the only escape is re-assigning `launch.creator` to a different key via `SetLaunchCreator` and graduating with the new creator.
4. **Severity justification:** availability of all value on the curve, silent failure, realistic preconditions, recoverable only by an unintuitive workaround that costs the creator their fee identity. Medium.
5. **Fix (this PR):** when the two recipients coincide, the shares merge into one payment through whichever pair is supplied (`share_floor(creator) + share_floor(partner)` paid to the creator's ATA; the partner payment becomes a zero no-op, so its account pair is omitted — omitted trailing optionals are already the documented client behavior). `docs/graduation.md` and `docs/security.md` now state the rule, and the Surfpool suite gains `graduation_merges_lp_shares_when_the_creator_is_the_partner` driving the full journey: partner launches, completes, graduates with the creator pair only, and the merged ATA holds every LP token not burned.

## Low (fixed in this PR)

### BC-2 — `SetLaunchCreator` accepts the default address, permanently locking fees, the vested allocation, and graduation itself

1. **Location:** `programs/pina_bonding_curve/src/processors/launch.rs` (`SetLaunchCreator`).
2. **Mechanism:** the default address cannot sign, so creator fees and the vested base allocation become unclaimable forever (they still leave the curve's accounting, so the value is stranded, not redistributed), and with `creator_lp_share > 0` the launch can additionally never graduate (`MissingLpAccount` with no possible account list, since `launch.creator` is also the AMM pool's creator-fee recipient).
3. **Severity:** self-inflicted, irreversible, one signer away. Low.
4. **Fix (this PR):** reject `Address::default()` with the new append-only error `DefaultCreator` (22); exercised in the Surfpool suite.

### BC-3 (defensive) — `Graduate` now asserts the launch price equals the configuration's migration price

`Completed` is only reachable through a completing buy, which stops exactly at the derived price, so the two cannot drift today. The added `MigrationPriceMismatch` (23) check turns any future state bug into a failed transaction instead of a pool seeded at a stale price — the one input to `seed_pool` that is trusted rather than re-derived.

## Informational

- **BC-4 — AMM tier rates can change between configuration and graduation.** `CreateConfig` verifies the tier is restricted to this program's AMM authority (and the AMM's `UpdateConfig` provably cannot un-restrict it), but a tier's _rates_ are read by the AMM at pool creation. A graduation pool therefore pays whatever the tier says at graduation time, bounded by the AMM's 10% combined cap. Now recorded in `docs/security.md` known limitations; partners should choose tiers whose authority they trust.
- **BC-5 — Quote-vault donation dust is stranded after graduation.** Donated quote above `quote_reserve` + fee counters never enters the program's accounting and stays in the vault forever (donated base, by contrast, is burned at graduation). Grief-donation only; no value can be extracted.
- **BC-6 — `LaunchStatus` decode failure maps to `AccountMismatch`.** Unreachable today (the program only ever writes 0–2) but the error name misleads. Cosmetic.
- **BC-7 — Launch and configuration accounts are never closed.** Rent is intentionally spent on immutable history; documented behaviour, noted for completeness.
- **BC-8 — Instant vesting is possible by configuration** (`creator_vesting_cliff = 0`, `duration = 0` releases the creator allocation at activation, which can then be sold into the curve). The program enforces that terms are _consistent and fundable_, not fair; `docs/security.md` already says configuration quality is the partner's responsibility. Worth remembering when reading a configuration before trading.

## Verified-correct (selected, this pass)

- Curve engine: `Curve::new` canonicalizes and rejects malformed segments; buys cap at the migration price and charge only for quote used (the completing-buy gross-up is clamped to the offered amount); sells cannot cross the start price; `quote_floor` coverage holds under interleaved trades (property-tested); `derive_quantities` rejects thresholds at or beyond the curve's end.
- 256-bit arithmetic: `full_mul`/`div_wide` match a schoolbook reference across 4,096 random cases, including the `high >= divisor` overflow rejection and both rounding modes.
- Vault solvency: vault balance = `quote_reserve` + fee counters through every instruction; fee claims move exactly their counters; graduation re-reads balances and handles both directions of rounding drift in `seed_pool` without ever failing a funded launch.
- Launch integrity: base mints must be new (zero supply, creator as mint authority, no freeze authority, configured decimals, allowed extensions only), the fixed supply is minted into the vault, and the mint authority is revoked in the same instruction — no launch can inflate or freeze its token.
- Graduation front-running: the tier restriction is checked at configuration creation, the restriction field is immutable in the AMM, the AMM program account is the pinned CPI identity, and every derived address (pool, LP mint, vaults, recipient ATAs) is re-derived or ATA-verified inside the callee.
- Anti-sniping: the fee starts high (≤ 99%) and decays linearly from activation; wash-trading one's own launch cannot extract value because fees are recycled, not burned.

## What was not reviewed

On-chain bytecode (not deployed), deployed-key custody, the website beyond a read (static docs plus a health endpoint), and the Dart client's generated code beyond spot checks.

## Recommended actions before release

1. Merge this PR (BC-1/BC-2/BC-3 plus docs and tests).
2. Keep `amm.rev` and `vendor/pina_amm` moving in lockstep with the AMM repository's audit state; re-run the integration-boundary review whenever either wire format changes.
3. Consider surfacing configuration terms (vesting schedule, fee shares, LP split) in an off-chain registry so traders can judge a launchpad before trading under it — program-side enforcement of "fair" terms is deliberately out of scope.
