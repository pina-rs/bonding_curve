# The `pina-curve` CLI

`pina-curve` drives every Pina Bonding Curve instruction from the terminal and helps design curves offline. It derives every PDA, vault, AMM pool address, and associated token account, so you supply only what you choose: a terms file, a launch's mint, and amounts.

```sh
cargo install pina_bonding_curve_cli
pina-curve --help
```

## Global options

| Option          | Default                                            | Meaning                                                                                                           |
| --------------- | -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `-u, --url`     | `devnet` (`$SOLANA_URL`)                           | `mainnet`, `devnet`, `testnet`, `localhost`, or an `https://` URL. Plain `http://` is only accepted for localhost |
| `-k, --keypair` | `~/.config/solana/id.json` (`$PINA_CURVE_KEYPAIR`) | Signer and fee payer, in the Solana CLI's JSON format                                                             |
| `--simulate`    | off                                                | Simulate and print the result instead of sending                                                                  |
| `--json`        | off                                                | Print one JSON object per command                                                                                 |

Amounts are always **base units**, the integer a token account stores. Prices are quote base units per base base unit. Rates in terms files are parts per million.

## Designing a curve

The `design` commands are pure arithmetic; they need no cluster or keypair.

```sh
pina-curve design sqrt-price --price 0.000028           # price → Q64.64 square root
pina-curve design price --sqrt-price 97610000000000000  # and back
pina-curve design segment --from-price 0.000028 --to-price 0.000497 --quote 85000000000
```

`design segment` sizes the liquidity that raises `--quote` between two prices and estimates the base it sells. [curves.md](curves.md) explains the model and walks through a full design.

## Launchpads

```sh
# Preview: runs the real program and prints the derived supplies and price.
pina-curve --simulate config create --index 0 \
  --quote-mint So11111111111111111111111111111111111111112 \
  --amm-config <AMM_TIER> --terms terms.json

pina-curve config create --index 0 \
  --quote-mint So11111111111111111111111111111111111111112 \
  --amm-config <AMM_TIER> --terms terms.json

pina-curve config show --config <CONFIG>
```

The signer becomes the configuration's partner. `config create` prints the configuration address, `sale_supply`, `migration_supply`, and the migration price. `config show` prints every term, with rates as fractions (`0.01` is 1%) and each segment's price.

### Terms files

`--terms` names a JSON file whose fields match the `CreateConfig` instruction. Square-root prices and liquidities are 128-bit; write them as decimal strings, because most JSON tools lose precision above 2^53. Unknown fields are rejected, so a typo fails instead of silently using a default.

| Field                       | Required | Default     | Meaning                                                   |
| --------------------------- | -------- | ----------- | --------------------------------------------------------- |
| `sqrt_start_price`          | yes      |             | Q64.64 square-root start price (string)                   |
| `segments`                  | yes      |             | 1 to 16 `{ "sqrt_price": "...", "liquidity": "..." }`     |
| `migration_quote_threshold` | yes      |             | Quote raised before completion                            |
| `total_supply`              | yes      |             | Base minted per launch                                    |
| `base_decimals`             | yes      |             | Decimals launched mints must have                         |
| `start_fee_rate`            | yes      |             | ppm at activation                                         |
| `end_fee_rate`              | yes      |             | ppm after the decay                                       |
| `fee_decay_duration`        | no       | `0`         | Seconds; required above zero when the start fee is higher |
| `creator_allocation`        | no       | `0`         | Base reserved per creator                                 |
| `creator_vesting_cliff`     | no       | `0`         | Seconds after activation                                  |
| `creator_vesting_duration`  | no       | `0`         | Seconds after the cliff                                   |
| `creator_fee_share`         | no       | `0`         | ppm of every fee paid to the creator                      |
| `migration_fee_rate`        | no       | `0`         | ppm of the raised quote                                   |
| `creator_lp_share`          | no       | `0`         | ppm of graduation LP to the creator                       |
| `partner_lp_share`          | no       | `0`         | ppm of graduation LP to the partner                       |
| `pool_creator`              | no       | `"creator"` | `"creator"` or `"partner"`: who collects AMM creator fees |

[launchpads.md](launchpads.md) explains each term and shows a complete example.

## Launching a token

```sh
# Create a new SPL Token mint and launch it in one transaction.
pina-curve launch create --config <CONFIG>

# The same with a Token-2022 mint.
pina-curve launch create --config <CONFIG> --token-2022

# Launch a mint you created yourself (for example with Token-2022 metadata),
# opening trading at a future time.
pina-curve launch create --config <CONFIG> --base-mint <MINT> --activation-time 1767225600
```

The signer is the creator. A mint you bring must be empty, have no freeze authority, use the configuration's decimals, and have the signer as its mint authority; the launch revokes that authority after minting the fixed supply. Without `--base-mint`, the CLI generates the mint keypair and signs with it.

```sh
pina-curve launch show --base-mint <MINT>
pina-curve launch set-creator --base-mint <MINT> --new-creator <ADDRESS>
```

`launch show` prints the status, the current price, `progress` toward graduation (`quote_reserve / migration_quote_threshold`), unsold base, unclaimed fees, and the pool once graduated.

## Trading

```sh
pina-curve quote --base-mint <MINT> --buy 1000000000     # spend 1 SOL, fee included
pina-curve quote --base-mint <MINT> --sell 5000000000    # sell 5,000 tokens
pina-curve buy   --base-mint <MINT> --quote-amount 1000000000 --slippage-bps 100
pina-curve sell  --base-mint <MINT> --base-amount 5000000000 --slippage-bps 100
```

Each trade is first simulated with an open limit; the `Traded` event the program emits is the quote. The CLI then sends the trade with the limit lowered by `--slippage-bps` (default 100, 1%). Missing token accounts are created in the same transaction. When the quote mint is wrapped SOL, `buy` wraps exactly `--quote-amount` lamports first, so you trade straight from your SOL balance. A buy that completes the launch only spends what it uses; any unspent wrapped SOL stays in your wrapped SOL account.

## Graduating and claiming

```sh
pina-curve graduate --base-mint <MINT>               # any signer; pays the pool's rent

pina-curve claim partner-fees --base-mint <MINT>     # signer: the configuration authority
pina-curve claim creator-fees --base-mint <MINT>     # signer: the launch creator
pina-curve claim allocation   --base-mint <MINT>     # signer: the launch creator
```

`graduate` reads the AMM program from the owner of the configuration's tier, derives the pool accounts, and includes the creator and partner LP accounts when their shares are above zero. Claims pay into the signer's associated token accounts, created if needed.

## Scripting

With `--json`, every command prints exactly one JSON object on stdout, in one of three shapes.

Commands that act (`config create`, `launch create`, `launch set-creator`, `quote`, `buy`, `sell`, `graduate`, and `claim ...`) print an envelope. `signature` is `null` for `quote` and under `--simulate`:

```console
$ pina-curve --json quote --base-mint <MINT> --buy 1000000000
{"action":"quote","details":{"base_amount":34082533733470,"compute_units":30432,"fee":10000000,"launch":"9wbt...","price_after":0.000030134047060842826,"quote_amount":1000000000,"side":"buy"},"signature":null}
```

Commands that read (`config show` and `launch show`) print the decoded fields at the top level:

```console
$ pina-curve --json launch show --base-mint <MINT>
{"activation_time":1791372726,"address":"9wbt...","creator_fees":7520455,"migration_quote_threshold":85000000000,"partner_fees":7520456,"pool":null,"price":0.00002903733852360539,"progress":0.005716575541176471,"quote_reserve":485908921,"sqrt_price":"99402687498361881","status":"trading","unsold_base":982958733133265,...}
```

`design` commands print their results at the top level too. 128-bit values (square-root prices and liquidities) are strings in every shape.

The CLI exits with status 1 and a message on stderr when anything fails, including a failed simulation, which prints the program logs.
