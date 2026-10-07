# Performance

The Pina Bonding Curve is built to be cheap to trade on and cheap to deploy.

## Compute units

Measured on the real SBF artifacts by the end-to-end suite (`compute_units_stay_within_budget` in `programs/pina_bonding_curve/tests/surfpool`). The numbers include every CPI each instruction makes.

| Instruction    | Compute units | CPIs                                                                        | CI ceiling |
| -------------- | ------------- | --------------------------------------------------------------------------- | ---------- |
| `Buy`          | ~8,400        | 2 transfers                                                                 | 10,000     |
| `Sell`         | ~9,000        | 2 transfers                                                                 | 11,000     |
| `CreateLaunch` | ~21,300       | creates 3 accounts, mints, revokes the mint authority                       | 26,000     |
| `Graduate`     | ~65,400       | the AMM's `CreatePool` (5 accounts, 2 transfers, 1 mint), LP and base burns | 80,000     |

`Graduate` costs about 91,000 compute units when it also creates LP token accounts for the creator and partner. The suite fails if any instruction exceeds its ceiling, so a regression is caught in the pull request that introduces it.

A buy's cost grows slightly with the number of segments it crosses, because each crossing prices one more segment. Most buys stay within one segment.

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
