# Pina Bonding Curve

Self-serve token launchpads for Solana, built with [Pina](https://github.com/pina-rs/pina). Anyone can run a launchpad by writing one configuration: the price curve, the supply, the fees, the creator's terms, and how launches graduate. Every token launched under it sells along that curve and, once it raises its target, graduates into a [Pina AMM](https://github.com/pina-rs/amm) pool at exactly the price the curve reached.

```text
Program ID  CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9
```

> **Status:** pre-release. The program has not been deployed or audited, and the interface may still change before the first `v*` tag. See [SECURITY.md](SECURITY.md).

## What makes it different

- **Curves of any shape.** A curve is up to 16 segments of concentrated liquidity. One segment is the classic pump-style curve; more segments approximate any rising price schedule. The math is exact integer arithmetic that always rounds in the curve's favour. See [docs/curves.md](docs/curves.md).
- **A launchpad is one account.** Configurations are immutable and self-serve: create one for your app, game, or community and share its address. There is no admin, allowlist, or pause switch.
- **Fair by construction.** Each launch mints a fixed supply and revokes the mint authority in the same instruction. Base mints cannot have a freeze authority. An opening fee that decays over the first seconds taxes sniping bots instead of buyers.
- **Graduation nobody can front-run.** Launches graduate into a Pina AMM tier that only this program can create pools in, at the curve's final price. LP not paid to the creator or partner is burned, so the liquidity is locked forever.
- **Revenue for creators and partners.** Trading and migration fees split between the token's creator and the launchpad's partner, a creator allocation vests on a schedule, and the graduated pool can keep paying a creator fee.
- **Cheap.** A buy costs about **8,400 compute units** including both token transfers. See [docs/performance.md](docs/performance.md).

## Packages

| Package                     | Registry                                                        | What it is                                                 | Guide                                      |
| --------------------------- | --------------------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------ |
| `pina_bonding_curve_client` | [crates.io](https://crates.io/crates/pina_bonding_curve_client) | Rust client: instruction builders, decoders, PDAs, events  | [docs/rust-client.md](docs/rust-client.md) |
| `pina_bonding_curve_cpi`    | [crates.io](https://crates.io/crates/pina_bonding_curve_cpi)    | `no_std` client for calling the curve from another program | [docs/cpi.md](docs/cpi.md)                 |
| `pina_bonding_curve_cli`    | [crates.io](https://crates.io/crates/pina_bonding_curve_cli)    | The `pina-curve` command-line tool                         | [docs/cli.md](docs/cli.md)                 |
| `@pina-rs/bonding-curve`    | [npm](https://www.npmjs.com/package/@pina-rs/bonding-curve)     | TypeScript client for `@solana/kit`                        | [docs/typescript.md](docs/typescript.md)   |
| `pina_bonding_curve`        | [pub.dev](https://pub.dev/packages/pina_bonding_curve)          | Dart and Flutter client for `solana_kit`                   | [docs/dart.md](docs/dart.md)               |

The on-chain program itself lives in [`programs/pina_bonding_curve`](programs/pina_bonding_curve). It ships through `pina deploy` (see [docs/deploying.md](docs/deploying.md)), never through a package registry.

## Quick start

Install the CLI and design a curve that raises 85 SOL while the price rises about 18×:

```sh
cargo install pina_bonding_curve_cli
pina-curve design segment --from-price 0.000028 --to-price 0.000497 --quote 85000000000
```

Put the result in a terms file ([docs/launchpads.md](docs/launchpads.md) has a complete one), preview what the program derives, and create the launchpad:

```sh
pina-curve -u devnet --simulate config create --index 0 \
  --quote-mint So11111111111111111111111111111111111111112 --amm-config <AMM_TIER> --terms terms.json
pina-curve -u devnet config create --index 0 \
  --quote-mint So11111111111111111111111111111111111111112 --amm-config <AMM_TIER> --terms terms.json
```

Launch a token, buy it, and graduate it once it completes:

```sh
pina-curve -u devnet launch create --config <CONFIG>
pina-curve -u devnet buy --base-mint <MINT> --quote-amount 1000000000
pina-curve -u devnet launch show --base-mint <MINT>
pina-curve -u devnet graduate --base-mint <MINT>
```

A buy from TypeScript:

```ts
import { fetchLaunch, getBuyInstruction } from "@pina-rs/bonding-curve";
import { createSolanaRpc } from "@solana/kit";

const rpc = createSolanaRpc("https://api.devnet.solana.com");
const { data: launch } = await fetchLaunch(rpc, launchAddress);

const instruction = getBuyInstruction({
	trader: wallet, // a TransactionSigner
	config: launch.config,
	launch: launchAddress,
	traderBase: walletBaseToken,
	traderQuote: walletQuoteToken,
	baseVault: launch.baseVault,
	quoteVault: launch.quoteVault,
	baseTokenProgram,
	quoteTokenProgram,
	quoteAmountIn: 1_000_000_000n,
	minimumBaseOut: 33_000_000_000_000n,
});
```

Every client guide has complete examples. Start with [docs/integrating.md](docs/integrating.md) to choose one.

## How it works

| Concept                        | Summary                                                                                      | Details                                      |
| ------------------------------ | -------------------------------------------------------------------------------------------- | -------------------------------------------- |
| Configuration (`LaunchConfig`) | A partner's immutable launchpad terms, at `[b"config", authority, index]`                    | [docs/launchpads.md](docs/launchpads.md)     |
| Launch (`Launch`)              | One token on its curve, at `[b"launch", base_mint]`                                          | [docs/architecture.md](docs/architecture.md) |
| Curve                          | Up to 16 concentrated-liquidity segments in Q64.64 square-root prices                        | [docs/curves.md](docs/curves.md)             |
| Fees                           | A trading fee that decays from its opening rate, a migration fee, both split creator/partner | [docs/fees.md](docs/fees.md)                 |
| Graduation                     | Completion at the migration price, then a permissionless move into the Pina AMM              | [docs/graduation.md](docs/graduation.md)     |
| Instructions                   | Nine instructions with fixed account lists                                                   | [docs/instructions.md](docs/instructions.md) |

## Development

Everything runs inside [devenv](https://devenv.sh), which pins Rust, the Agave SBF toolchain, the Pina CLI, Node.js, Dart, and Monochange:

```sh
devenv shell install:all     # JavaScript and Dart dependencies
devenv shell build:program   # SBF artifact and IDL in target/
devenv shell fetch:amm       # the Pina AMM at the revision pinned in amm.rev
devenv shell test:unit       # Rust, TypeScript, and Dart unit tests
devenv shell test:surfpool   # end-to-end suite with the curve and the AMM on an offline Surfnet
devenv shell lint:all        # Pina security lints, clippy, docs, formatting, workflows
devenv shell verify:all      # everything CI runs
```

After any change to the program's accounts, instructions, or events, run `pina migrations create --project programs/pina_bonding_curve` and then `devenv shell generate:clients`. Generated sources under `clients/` and `vendor/` are never edited by hand. [AGENTS.md](AGENTS.md) lists the full set of repository rules.

## Releases

Releases are planned with [Monochange](https://github.com/monochange/monochange). Every pull request that changes a published package carries a changeset; user-visible changes carry a second, plain-language changeset for the public release notes. [docs/releasing.md](docs/releasing.md) covers the flow, and [docs/deploying.md](docs/deploying.md) covers deploying the program and preparing its AMM tier.

## License

[Apache-2.0](LICENSE)
