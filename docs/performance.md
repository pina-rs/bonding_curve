# Performance

The Pina Bonding Curve is built to be cheap to trade on and cheap to deploy.

## Compute units

Measured on the real SBF artifacts by the end-to-end suite (`compute_units_stay_within_budget` in `programs/pina_bonding_curve/tests/surfpool`). The numbers include every CPI each instruction makes.

| Instruction    | Compute units | CPIs                                                                        | CI ceiling |
| -------------- | ------------- | --------------------------------------------------------------------------- | ---------- |
| `Buy`          | ~8,400        | 2 transfers                                                                 | 10,000     |
| `Sell`         | ~9,000        | 2 transfers                                                                 | 11,000     |
| `CreateLaunch` | 19,800–21,300 | creates 3 accounts, mints, revokes the mint authority                       | 32,000     |
| `Graduate`     | 65,000–90,000 | the AMM's `CreatePool` (5 accounts, 2 transfers, 1 mint), LP and base burns | 130,000    |

Costs that derive PDAs vary with the addresses involved: every extra bump attempt in a PDA search costs about 1,500 compute units, and graduation searches for the pool, LP mint, vault, and token account bumps. Creating LP token accounts for the creator and partner adds about 25,000 more. The suite fails if any instruction exceeds its ceiling, so a regression is caught in the pull request that introduces it.

A buy's cost grows slightly with the number of segments it crosses, because each crossing prices one more segment. Most buys stay within one segment.

## Continuous benchmarking

Every pull request that touches the program, the clients, the vendored AMM, or the harness is benchmarked automatically. The `performance` workflow builds the real SBF binary for the pull request **and** for its base — each side against its own pinned `amm.rev` — runs the all-instructions journey three times against each binary on an offline Surfnet, and posts one consolidated comment on the pull request:

- the deployed binary size, base against head;
- every instruction's compute units, base against head, as the median of three runs.

The journey uses fixed keypair seeds for every address that feeds a PDA derivation, so identical binaries measure identical compute units and the comparison shows program changes, not address luck.

The policy in `scripts/benchmark-policy.json` gates the pull request:

- any increase in the deployed binary's size fails the check;
- any instruction whose compute units rise by more than **2%** fails the check;
- an instruction that loses its measurement fails the check;
- a new instruction is reported as a new baseline and never blocks.

A regression blocks auto-merge until the pull request carries the `performance-approved` label — the maintainer's explicit permission for that trade. Re-run the numbers locally with:

```sh
devenv shell -- pnpm exec tsx scripts/benchmark.ts --out target/perf/head
devenv shell -- pnpm exec tsx scripts/compare-benchmarks.ts \
  --base <base-dir> --head target/perf/head \
  --markdown report.md --json report.json
```

The per-instruction ceilings in the end-to-end suite stay as they are: they catch a regression even when the workflow is skipped.

## Program size

About **125 KB** for the deployed `pina_bonding_curve.so`, built with fat LTO, one codegen unit, and `opt-level = 3`. Rent for the program-data account scales with this size. The largest contributors are the 256-bit arithmetic that keeps every curve calculation exact and the graduation path's AMM integration.

The size-optimised profiles were measured and rejected:

| Profile           | Size   | `Buy`            | Result                                                      |
| ----------------- | ------ | ---------------- | ----------------------------------------------------------- |
| `opt-level = 3`   | 125 KB | ~8,400 CU        | Used                                                        |
| `opt-level = "s"` | 105 KB | ~9,800 CU (+17%) | Every trade pays for a one-time rent saving                 |
| `opt-level = "z"` | 94 KB  | —                | The entrypoint overflows the 4 KB SBF stack frame and fails |

A launchpad's program is deployed once and called on every trade, so it uses the speed profile. `pina build` applies that profile itself; build with `pina build --no-size-profile` to try another.

## Where the savings come from

| Choice                                   | Effect                                                                                                         |
| ---------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Typed loads instead of PDA re-derivation | A launch is trusted because the program owns it and its discriminator matches; only creation derives addresses |
| Stored vault addresses                   | Validating a vault is a 32-byte comparison, not a hash                                                         |
| Fees accrued, not transferred            | A trade makes exactly two token CPIs                                                                           |
| 256-bit math with a native fast path     | Wide division only runs when an intermediate actually exceeds 128 bits                                         |
| One curve walk per trade                 | The buy or sale prices the segments it crosses and nothing else                                                |
| Plain `Transfer` CPIs                    | No mint accounts or decimals in the trade account list; allowed Token-2022 extensions never need them          |
| Launch PDA as the vault authority        | No global authority account in trading instructions                                                            |
| Pina's dispatch entrypoint               | Routes on the discriminator before parsing any account                                                         |

## Measuring yourself

```sh
devenv shell build:program
devenv shell test:surfpool    # prints "<instruction>: <units> CU (budget <limit>)"
pina profile target/deploy/pina_bonding_curve.so   # static compute-unit analysis
```
