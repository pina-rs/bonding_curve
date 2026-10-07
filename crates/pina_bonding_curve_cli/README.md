# `pina-curve`

Command-line interface for the [Pina Bonding Curve](https://github.com/pina-rs/bonding_curve). Design curves offline, run a launchpad, launch tokens, and trade them. The CLI derives every PDA, vault, pool, and token account, quotes trades by simulating them, and sends them with a slippage limit.

```sh
cargo install pina_bonding_curve_cli

pina-curve design segment --from-price 0.000028 --to-price 0.000497 --quote 85000000000
pina-curve -u mainnet --simulate config create --index 0 --quote-mint <MINT> --amm-config <TIER> --terms terms.json
pina-curve -u mainnet launch create --config <CONFIG>
pina-curve -u mainnet buy --base-mint <MINT> --quote-amount 1000000000 --slippage-bps 100
pina-curve -u mainnet graduate --base-mint <MINT>
```

Every command accepts `--json` for scripting and `--simulate` for a dry run. Full reference: [docs/cli.md](https://github.com/pina-rs/bonding_curve/blob/main/docs/cli.md).
