# `pina_bonding_curve`

The on-chain Pina Bonding Curve program: self-serve launchpads whose tokens sell along piecewise concentrated-liquidity curves and graduate into the [Pina AMM](https://github.com/pina-rs/amm). Its program address is `CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9`; it ships through `pina deploy`, never through a package registry.

| Path                  | Contents                                                            |
| --------------------- | ------------------------------------------------------------------- |
| `src/state.rs`        | `LaunchConfig` and `Launch` layouts, PDA seeds, limits              |
| `src/instructions.rs` | Instruction discriminators and data layouts                         |
| `src/processors/`     | Account lists and handlers                                          |
| `src/curve.rs`        | Segmented curve pricing and graduation seeding, with property tests |
| `src/math.rs`         | 256-bit arithmetic, fees, and vesting, with property tests          |
| `src/token.rs`        | Token-program checks and CPIs                                       |
| `src/events.rs`       | Events written to the transaction log                               |
| `src/errors.rs`       | Error codes                                                         |
| `migrations/`         | Pina's schema history; never edit by hand                           |
| `tests/surfpool/`     | End-to-end suite against the compiled curve and the pinned AMM      |

```sh
devenv shell build:program            # target/deploy/pina_bonding_curve.so and its IDL
cargo test -p pina_bonding_curve      # unit and property tests
devenv shell test:surfpool            # end-to-end suite
```

See the [repository README](../../README.md) and [docs/](../../docs/readme.md).
