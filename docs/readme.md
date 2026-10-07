# Pina Bonding Curve documentation

Start with the guide for what you are doing:

| You want to                               | Read                                                        |
| ----------------------------------------- | ----------------------------------------------------------- |
| Run your own launchpad                    | [launchpads.md](launchpads.md), then [curves.md](curves.md) |
| Launch or trade a token from the terminal | [cli.md](cli.md)                                            |
| Build an app or bot on top of launches    | [integrating.md](integrating.md)                            |
| Deploy the program                        | [deploying.md](deploying.md)                                |

Every guide:

| Guide                              | Read it when you want to                                              |
| ---------------------------------- | --------------------------------------------------------------------- |
| [architecture.md](architecture.md) | Understand the accounts, PDAs, lifecycle, and design decisions        |
| [curves.md](curves.md)             | Understand the curve model, its formulas, and how to design a curve   |
| [launchpads.md](launchpads.md)     | Choose every term of a launchpad configuration                        |
| [fees.md](fees.md)                 | Understand the decaying trading fee, the split, and the migration fee |
| [graduation.md](graduation.md)     | Follow a launch from completion into its Pina AMM pool                |
| [instructions.md](instructions.md) | Look up an instruction's data, accounts, errors, or events            |
| [integrating.md](integrating.md)   | Choose a client and follow a buy end to end                           |
| [typescript.md](typescript.md)     | Use `@pina-rs/bonding-curve` with `@solana/kit`                       |
| [dart.md](dart.md)                 | Use `pina_bonding_curve` from Dart or Flutter                         |
| [rust-client.md](rust-client.md)   | Use `pina_bonding_curve_client` from Rust                             |
| [cpi.md](cpi.md)                   | Call the curve from another on-chain program                          |
| [cli.md](cli.md)                   | Design curves and operate launchpads from the terminal                |
| [performance.md](performance.md)   | See compute-unit and size measurements                                |
| [security.md](security.md)         | Review the threat model and invariants                                |
| [deploying.md](deploying.md)       | Deploy the program and prepare its Pina AMM tier                      |
| [releasing.md](releasing.md)       | Write changesets and publish a release                                |

Graduated tokens trade on the [Pina AMM](https://github.com/pina-rs/amm), which has its own documentation.
