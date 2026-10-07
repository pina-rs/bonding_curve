# Designing curves

A launch's price follows a curve that the configuration fixes before any launch exists. This page explains the curve model, the exact formulas the program uses, and how to design a curve that raises what you want at the prices you want. For the economic terms around the curve (fees, allocations, graduation) see [launchpads.md](launchpads.md).

## Units

| Quantity          | Unit                                                    | Example                                                 |
| ----------------- | ------------------------------------------------------- | ------------------------------------------------------- |
| Price             | Quote base units per base base unit                     | `0.000028` lamports per base unit of a 6-decimal token  |
| Square-root price | Q64.64 fixed point: `sqrt(price) * 2^64`, stored `u128` | `97_610_000_000_000_000` is a price of about `0.000028` |
| Liquidity `L`     | Unitless `u128`                                         | `5_000_000_000_000`                                     |
| Amounts           | Raw base units, the integer a token account stores      | `1_000_000` is one whole 6-decimal token                |

Prices are always in base units, never whole tokens, so a curve means the same thing for every pair of decimals. To convert a price per whole token to base units, multiply by `10^quote_decimals / 10^base_decimals`. A price of `0.028` SOL per million tokens of a 6-decimal token is `0.028 * 10^9 / (10^6 * 10^6) = 0.000028` lamports per base unit.

## The model: segments of concentrated liquidity

A curve is up to **16 contiguous segments**. Each segment holds constant liquidity `L` between two square-root prices, exactly like one concentrated-liquidity position, so within a segment the curve behaves like `x * y = L^2`. Buying spends quote and raises the price; selling returns base and lowers it.

```text
price
  │                                   ┌── segment 3: deep (large L), price rises slowly
  │                          ┌────────┘
  │                ┌─────────┘ segment 2
  │        ┌───────┘
  │ ───────┘ segment 1: shallow (small L), price rises quickly
  └──────────────────────────────────────────────────────── quote raised
  start                                        migration price
```

- **One segment** from the start price to a high upper bound reproduces a classic virtual-reserve bonding curve.
- **More segments** approximate any increasing price schedule: a shallow first segment makes early buyers move the price quickly; a deep later segment keeps the price steady while the launch fills.

A configuration stores the start square-root price, the upper square-root price of each segment, and each segment's liquidity. Upper bounds must strictly increase, every used segment needs liquidity above zero, and unused slots must be zero, so every curve has exactly one encoding.

## Formulas

For a segment with liquidity `L` and the price moving between square-root prices `a < b`:

```text
quote = L * (b - a) / 2^64
base  = L * (b - a) * 2^64 / (a * b)
```

The program evaluates `base` as `L * 2^64 / a - L * 2^64 / b`, rounding each term once, with 256-bit intermediates so no product overflows. A buy walks segments from the current price upward; a sale walks them downward.

Within one segment, the new square-root price after spending `q` quote or selling `x` base is:

```text
after a buy:   s' = s + q * 2^64 / L                    (rounded down)
after a sale:  s' = L * s / (L + x * s / 2^64)          (rounded up)
```

### Rounding

Every rounding choice favours the curve:

| Calculation                          | Direction | Effect                                      |
| ------------------------------------ | --------- | ------------------------------------------- |
| Quote needed to cross a segment      | up        | A buyer pays at least the exact integral    |
| Base received on a buy               | down      | A buyer never gets more than they paid for  |
| Price after a buy                    | down      | The next buyer is never undercharged        |
| Base needed to cross down a segment  | up        | A seller gives at least the exact integral  |
| Quote received on a sale             | down      | A seller never takes out more than went in  |
| Price after a sale                   | up        | The curve never gives back more than it got |
| Derived supplies and migration quote | up        | Configurations reserve enough base          |

The result, checked by the `reserves_always_cover_the_curve` property test, is that the quote a launch holds always covers the exact integral of its curve from the start price to its current price. No sequence of buys and sells can drain the curve, and `round_trips_never_profit` and `splitting_buys_never_beats_the_curve` check that neither a round trip nor splitting a buy into pieces beats a single trade.

## What a configuration derives

`CreateConfig` walks the curve from the start price until it has raised `migration_quote_threshold` quote and derives:

| Derived value          | Meaning                                                                       |
| ---------------------- | ----------------------------------------------------------------------------- |
| `migration_sqrt_price` | The square-root price at which a launch completes                             |
| `sale_supply`          | Base sold along the curve from the start to the migration price (rounded up)  |
| `migration_supply`     | Base needed to seed the AMM pool with the raised quote at the migration price |

The configuration is rejected unless:

- the threshold is above zero and reachable within the curve's segments (`InvalidMigrationThreshold`);
- `total_supply >= sale_supply + migration_supply + creator_allocation` (`InvalidSupply`);
- the graduation pool would mint more than the AMM's 1,000 locked LP units, even after the migration fee (`InsufficientMigrationLiquidity`).

Base left over at graduation is burned, so a `total_supply` above the minimum only shrinks the final supply; it never leaves tokens with the program. See [graduation.md](graduation.md).

## A worked example

This is the reference curve the test suite uses: a 1-billion-token launch with 6 decimals, raising 85 SOL.

| Term                        | Value                                                |
| --------------------------- | ---------------------------------------------------- |
| `sqrt_start_price`          | `97_610_000_000_000_000` (price ≈ `0.000028`)        |
| Segment 1 `sqrt_price`      | `780_880_000_000_000_000` (8× the start square root) |
| Segment 1 `liquidity`       | `5_000_000_000_000`                                  |
| `migration_quote_threshold` | `85_000_000_000` lamports (85 SOL)                   |
| `total_supply`              | `1_000_000_000_000_000` (1 billion tokens)           |
| `creator_allocation`        | `50_000_000_000_000` (50 million tokens)             |

What the program derives:

| Value                    | Result                                         | In tokens and SOL                                |
| ------------------------ | ---------------------------------------------- | ------------------------------------------------ |
| Start price              | `0.000028` lamports per base unit              | 28 SOL for all 1 billion tokens                  |
| `migration_sqrt_price`   | `411_204_649_253_062_377` (price ≈ `0.000497`) | about 17.7× the start price; 497 SOL valuation   |
| `sale_supply`            | `720_619_552_472_693`                          | about 720.6 million tokens sold for 85 SOL       |
| `migration_supply`       | `171_057_585_668_463`                          | about 171.1 million tokens reserved for the pool |
| Supply the terms require | `sale + migration + allocation`                | about 941.7 million of the 1 billion             |

With a 2% migration fee, the pool receives 83.3 SOL and about 167.6 million tokens at the migration price; the remaining 61.7 million unsold tokens are burned at graduation.

The single segment ends at 64× the start price, so this curve could raise up to about 185 SOL. The threshold stops it well inside the segment.

## Designing with the CLI

The `pina-curve design` commands do the conversions offline:

```console
$ pina-curve design sqrt-price --price 0.000028
price: 0.000028
sqrt_price: 97610994635780032

$ pina-curve design price --sqrt-price 97610000000000000
price: 0.000027999429374527884
sqrt_price: 97610000000000000

$ pina-curve design segment --from-price 0.000028 --to-price 0.000497 --quote 85000000000
approximate_base_sold: 720546006760645.2
liquidity: 4999413543066
quote: 85000000000
sqrt_price: 411242430154898304
sqrt_start_price: 97610994635780032
```

`design segment` prints the segment's start and end square-root prices, the liquidity (rounded up, so the segment raises at least `--quote`), and roughly how much base it sells. For a multi-segment curve, size each segment in turn, using the previous segment's end price as the next segment's start. Add `--json` for machine-readable output.

Then put the numbers in a terms file and preview the exact result before creating anything:

```sh
pina-curve --simulate config create --index 0 --quote-mint <QUOTE_MINT> \
  --amm-config <AMM_TIER> --terms terms.json
```

The simulation runs the real program and prints `sale_supply`, `migration_supply`, and the migration price it derived, or the error that rejects the terms. See [cli.md](cli.md#terms-files) for the terms file format.

## Shaping a curve

| Goal                                       | Shape                                                                  |
| ------------------------------------------ | ---------------------------------------------------------------------- |
| Classic pump-style launch                  | One segment; the threshold sets how far the price climbs               |
| Reward early buyers less, late buyers more | Deep first segment, shallow last segment                               |
| Fast price discovery, then a stable fill   | Shallow first segment, deep last segment                               |
| A flat sale price                          | A very deep single segment over a narrow price range                   |
| Stepped tiers                              | Several segments with increasing liquidity, each covering a price band |

The `stepped_curve` test fixture is a three-segment example: segments end at 2×, 3×, and 8× the start square-root price with liquidities of `1e12`, `4e12`, and `9e12`. The price climbs quickly through the first band and slows as each deeper band fills.

Keep two constraints in mind:

- **Supported range.** Square-root prices must lie between `4_295_048_016` and `79_226_673_521_066_979_257_578_248_091` (prices of about `2^-64` to `2^64` base units).
- **Precision.** Price conversions in `design` go through `f64`, accurate to about one part in 10^15. The integers you put in the terms file are what the program uses; `--simulate` shows their exact effect.
