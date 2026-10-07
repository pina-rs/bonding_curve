# Integrating the Pina Bonding Curve

Pick the client that matches where your code runs:

| You are writing                          | Use                                         | Guide                            |
| ---------------------------------------- | ------------------------------------------- | -------------------------------- |
| A web app, bot, or backend in TypeScript | `@pina-rs/bonding-curve` with `@solana/kit` | [typescript.md](typescript.md)   |
| A Flutter app or Dart backend            | `pina_bonding_curve` with `solana_kit`      | [dart.md](dart.md)               |
| A Rust service, keeper, or test          | `pina_bonding_curve_client`                 | [rust-client.md](rust-client.md) |
| Another on-chain program                 | `pina_bonding_curve_cpi`                    | [cpi.md](cpi.md)                 |
| Scripts and operations                   | the `pina-curve` CLI                        | [cli.md](cli.md)                 |

All clients are generated from the same IDL, so names match across languages (`getBuyInstruction` in TypeScript is `Buy` in Rust).

## Addresses you will need

| Address        | How to get it                                                                                                            |
| -------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Program        | `CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9`, exported by every client                                                 |
| Configuration  | `findLaunchConfigPda({ authority, index })`                                                                              |
| Launch         | `findLaunchPda({ baseMint })`                                                                                            |
| Vaults         | stored on the launch as `baseVault` and `quoteVault`, or `findLaunchVaultPda({ launch, mint })`                          |
| AMM authority  | `findAmmAuthorityPda()`                                                                                                  |
| Token programs | the owner of each mint account                                                                                           |
| Graduated pool | stored on the launch as `pool`; before graduation, derive it as shown in [graduation.md](graduation.md#sending-graduate) |

A launch is found from its token's mint alone, so a front end only needs the mint to show a token's page.

## A buy, step by step

1. **Fetch the launch** and confirm its owner is the curve program. Generated decoders only check the discriminator, so the owner check is yours. Check `status` is `0` (trading) and `activationTime` has passed.
2. **Quote.** Simulate the `Buy` with `minimumBaseOut = 0` and read the `Traded` event: `baseAmount` is what you receive and `quoteAmount` is what you pay. Or compute it yourself from [curves.md](curves.md) and [fees.md](fees.md).
3. **Apply slippage**: `minimumBaseOut = floor(baseAmount * (10_000 - bps) / 10_000)`.
4. **Make sure both token accounts exist.** Prepend idempotent associated-token-account creations; the base account uses the base mint's token program.
5. **Wrap SOL if needed.** When the quote mint is wrapped SOL, transfer `quoteAmountIn` lamports into the trader's wrapped SOL account and call `SyncNative` first.
6. **Send.** If the curve moved more than your tolerance, the program fails with `SlippageExceeded` and nothing changes.

A sale is the same with `Sell`, `baseAmountIn`, and `minimumQuoteOut`. The opening fee decays every second, so quote close to when you send, especially during the first minute of a launch.

## Showing a launch

| Value         | From                                                                      |
| ------------- | ------------------------------------------------------------------------- |
| Current price | `(sqrtPrice / 2^64)^2` quote base units per base base unit                |
| Progress      | `quoteReserve / config.migrationQuoteThreshold`                           |
| Market value  | price × `config.totalSupply`                                              |
| Current fee   | the linear decay in [fees.md](fees.md#rate-over-time) at the cluster time |
| Tokens sold   | `config.totalSupply` minus the base vault balance minus `creatorClaimed`  |

To list every launch under a configuration, filter `Launch` accounts by their `config` field (a `memcmp` at byte offset 2, after the discriminator and version bytes) with `getProgramAccounts`, or index `LaunchCreated` events.

## Compute budget

| Instruction    | Measured                               | Suggested limit |
| -------------- | -------------------------------------- | --------------- |
| `Buy`          | ~8,400 CU                              | 15,000          |
| `Sell`         | ~9,000 CU                              | 15,000          |
| `CreateLaunch` | ~21,300 CU                             | 40,000          |
| `Graduate`     | ~65,000 CU, ~91,000 CU with LP payouts | 150,000         |

Add the cost of anything else in the transaction, such as associated-token-account creation (~10,000 to 25,000 CU each). See [performance.md](performance.md).

## Reading events

Every state change emits an event: `ConfigCreated`, `LaunchCreated`, `Traded`, `Completed`, `Graduated`, and `Claimed`. Pass the **complete, ordered** log messages of a transaction to `parsePinaBondingCurveEventsFromLogs`. It follows the runtime's invoke frames and only returns records the curve itself wrote, so the AMM's `PoolCreated` during graduation, or a program that calls the curve, cannot be mistaken for a curve event. The per-event `parse*FromLog` helpers skip that attribution and are only safe for bytes you already know came from the curve.

## Errors

Program errors arrive as `Custom(code)`. Every client exports the codes and messages; the table is in [instructions.md](instructions.md#errors). The most common in practice:

| Error                        | Usual cause                                                                        |
| ---------------------------- | ---------------------------------------------------------------------------------- |
| `SlippageExceeded` (14)      | The curve moved, or the fee decayed less than expected; re-quote                   |
| `NotActive` (12)             | Trading opens at `activationTime`                                                  |
| `NotTrading` (11)            | The launch completed; trade on the AMM after graduation                            |
| `InsufficientLiquidity` (16) | A sale larger than everything bought since launch                                  |
| `InvalidBaseMint` (8)        | The mint has supply, a freeze authority, wrong decimals, or another mint authority |
| `MissingLpAccount` (21)      | `Graduate` without the creator or partner LP accounts the configuration requires   |
