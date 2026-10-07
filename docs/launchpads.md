# Running a launchpad

A launchpad is a **configuration**: one `LaunchConfig` account that fixes the curve, supply, fees, creator terms, and graduation terms for every token launched under it. Anyone can create configurations for themselves; the creator of a configuration is its **partner**. This page walks through every term and how to choose it. For the curve itself see [curves.md](curves.md).

## The parties

| Party   | Who                                       | Earns                                                                 |
| ------- | ----------------------------------------- | --------------------------------------------------------------------- |
| Partner | The configuration's `authority`           | The partner share of trading and migration fees, and optional LP      |
| Creator | Whoever launches a token under the config | The creator share of fees, an optional vested allocation, optional LP |
| Traders | Anyone                                    | Base bought on the curve, tradable on the Pina AMM after graduation   |

A configuration is **immutable**. Traders can judge a launchpad by its one configuration instead of auditing every token, and nobody can change the terms of a live launch. To change terms, create a new configuration with the next `index`; launches already running keep their old terms.

## Every term

Rates and shares are parts per million: `10_000` is 1%, `1_000_000` is 100%. Durations are seconds.

### Curve and supply

| Term                        | Limits                                    | Meaning                                                                          |
| --------------------------- | ----------------------------------------- | -------------------------------------------------------------------------------- |
| `sqrt_start_price`          | Supported square-root price range         | Q64.64 square-root price every launch starts at                                  |
| `segments`                  | 1 to 16, strictly increasing upper prices | Each segment's upper square-root price and liquidity; see [curves.md](curves.md) |
| `migration_quote_threshold` | Reachable within the segments             | Quote a launch raises before it completes                                        |
| `total_supply`              | Covers sale + migration + allocation      | Base minted for every launch; the mint authority is revoked afterwards           |
| `base_decimals`             | 0 to 18                                   | Decimals every launched mint must have                                           |

### Trading fees

| Term                 | Limits                                       | Meaning                                             |
| -------------------- | -------------------------------------------- | --------------------------------------------------- |
| `start_fee_rate`     | At most 990,000 (99%), at least the end rate | Fee charged at the moment trading opens             |
| `end_fee_rate`       | At most 100,000 (10%)                        | Fee once the decay finishes                         |
| `fee_decay_duration` | Above zero exactly when start > end          | Seconds over which the fee falls in a straight line |
| `creator_fee_share`  | At most 1,000,000                            | Creator's share of every trading and migration fee  |

A high opening fee that decays over the first seconds or minutes is the anti-sniping tool: bots that buy in the launch block pay most of their purchase as fees, while everyone a minute later pays the normal rate. Set `start_fee_rate` equal to `end_fee_rate` and `fee_decay_duration` to `0` for a flat fee. See [fees.md](fees.md).

### Creator allocation

| Term                       | Limits                      | Meaning                                                        |
| -------------------------- | --------------------------- | -------------------------------------------------------------- |
| `creator_allocation`       | Covered by `total_supply`   | Base reserved for each launch's creator                        |
| `creator_vesting_cliff`    | Cliff + duration fits `i64` | Seconds after activation before anything vests                 |
| `creator_vesting_duration` | Cliff + duration fits `i64` | Seconds after the cliff over which the allocation vests evenly |

The allocation stays in the launch's base vault until claimed with `ClaimCreatorAllocation`, whatever stage the launch is in. A zero duration releases everything at the cliff; a zero allocation turns the feature off.

### Graduation

| Term                 | Limits                                  | Meaning                                                                         |
| -------------------- | --------------------------------------- | ------------------------------------------------------------------------------- |
| `amm_config`         | A Pina AMM tier restricted to the curve | The fee tier graduated pools are created in                                     |
| `migration_fee_rate` | At most 100,000 (10%)                   | Share of the raised quote taken as a fee at graduation, split like trading fees |
| `creator_lp_share`   | Creator + partner at most 1,000,000     | Share of the pool's LP tokens sent to the creator                               |
| `partner_lp_share`   | Creator + partner at most 1,000,000     | Share of the pool's LP tokens sent to the partner                               |
| `pool_creator_mode`  | `0` creator, `1` partner                | Who collects the AMM pool's creator fees after graduation                       |

LP tokens not paid to the creator or partner are **burned**, so that share of the pool's liquidity is locked forever. With both shares at zero, the whole graduated pool is permanently locked liquidity, which is what most traders want to see. See [graduation.md](graduation.md).

### Quote mint and AMM tier

The quote mint is the token every launch raises, usually wrapped SOL (`So11111111111111111111111111111111111111112`) or a stablecoin. It may be SPL Token or Token-2022 with transfer-neutral extensions only.

`amm_config` must be a Pina AMM fee tier whose `pool_creator_authority` is this program's AMM authority PDA, `[b"amm_authority"]` under the curve program. A restricted tier guarantees nobody can create a launch's pool before graduation does. The AMM's upgrade authority creates these tiers; see [deploying.md](deploying.md#the-amm-tier). The tier's trade, protocol, and creator fee rates become the graduated pool's fees.

## Creating a configuration

Write the terms to a JSON file:

```json
{
	"sqrt_start_price": "97610000000000000",
	"segments": [
		{ "sqrt_price": "780880000000000000", "liquidity": "5000000000000" }
	],
	"migration_quote_threshold": 85000000000,
	"total_supply": 1000000000000000,
	"base_decimals": 6,
	"creator_allocation": 50000000000000,
	"creator_vesting_cliff": 0,
	"creator_vesting_duration": 2592000,
	"start_fee_rate": 500000,
	"end_fee_rate": 10000,
	"fee_decay_duration": 60,
	"creator_fee_share": 500000,
	"migration_fee_rate": 20000,
	"creator_lp_share": 0,
	"partner_lp_share": 0,
	"pool_creator": "creator"
}
```

This launchpad sells about 720 million of a billion tokens for 85 SOL. Fees open at 50% and fall to 1% over the first minute, split evenly between partner and creator. Each creator receives 50 million tokens vesting over 30 days. Graduation takes 2%, and the whole pool is locked.

Preview, then create:

```sh
pina-curve --simulate config create --index 0 --quote-mint So11111111111111111111111111111111111111112 \
  --amm-config <AMM_TIER> --terms terms.json
pina-curve config create --index 0 --quote-mint So11111111111111111111111111111111111111112 \
  --amm-config <AMM_TIER> --terms terms.json
```

The configuration's address is the PDA `[b"config", partner, index]`, printed by the command. Share it: creators launch with `pina-curve launch create --config <CONFIG>`, and front ends list launches by filtering `Launch` accounts on their `config` field.

## Presets to start from

| Launchpad style        | Curve                               | Fees                                   | Creator                  | Graduation                    |
| ---------------------- | ----------------------------------- | -------------------------------------- | ------------------------ | ----------------------------- |
| Community meme launch  | One segment, ~18× price rise        | 50% → 1% over 60 s, 50/50 split        | No allocation            | 2% fee, all LP burned         |
| Creator-backed project | Shallow then deep segments          | 1% flat, 80% to the creator            | 5% vesting over 6 months | 1% fee, 10% LP to the creator |
| Partner-run platform   | One segment                         | 25% → 1% over 30 s, 70% to the partner | None                     | 5% fee, 20% LP to the partner |
| Fixed-price sale       | One very deep segment, narrow range | 0.5% flat                              | Optional                 | 0% fee, all LP burned         |

Every preset is a starting point. Preview each one with `--simulate` before using it.

## Operating

| Task                               | Command                                                          |
| ---------------------------------- | ---------------------------------------------------------------- |
| Inspect a configuration            | `pina-curve config show --config <CONFIG>`                       |
| Collect partner fees from a launch | `pina-curve claim partner-fees --base-mint <MINT>` (partner key) |
| Graduate a completed launch        | `pina-curve graduate --base-mint <MINT>` (any key)               |

Partner fees accrue per launch in that launch's quote vault and can be claimed at any stage, including after graduation. Graduation is permissionless: a partner can run a crank that graduates every completed launch, or leave it to whoever trades next.

## Integrating a launchpad into another product

A product that wants its own launchpad (a game, a social app, a creator platform) only needs:

1. a configuration owned by a key it controls, created once;
2. the client for its stack ([typescript.md](typescript.md), [dart.md](dart.md), [rust-client.md](rust-client.md), or [cpi.md](cpi.md) for an on-chain program);
3. a way to show launches and their progress: `quote_reserve / migration_quote_threshold`.

Nothing in the program is specific to one product, and one deployment serves every launchpad.
