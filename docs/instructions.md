# Instruction reference

Every instruction's data is a one-byte discriminator followed by its fields, packed little-endian with no padding. Accounts are listed in order; `w` is writable and `s` is signer. The generated clients build these lists for you.

| #   | Instruction                                                  | Signer                  | Summary                                      |
| --- | ------------------------------------------------------------ | ----------------------- | -------------------------------------------- |
| 0   | [`CreateConfig`](#createconfig)                              | partner                 | Create a launchpad configuration             |
| 1   | [`CreateLaunch`](#createlaunch)                              | creator                 | Launch a token under a configuration         |
| 2   | [`Buy`](#buy-and-sell)                                       | trader                  | Spend quote on the curve                     |
| 3   | [`Sell`](#buy-and-sell)                                      | trader                  | Sell base back to the curve                  |
| 4   | [`Graduate`](#graduate)                                      | anyone                  | Move a completed launch into a Pina AMM pool |
| 5   | [`ClaimPartnerFees`](#claimpartnerfees-and-claimcreatorfees) | configuration authority | Pay out partner fees                         |
| 6   | [`ClaimCreatorFees`](#claimpartnerfees-and-claimcreatorfees) | launch creator          | Pay out creator fees                         |
| 7   | [`ClaimCreatorAllocation`](#claimcreatorallocation)          | launch creator          | Pay out the vested allocation                |
| 8   | [`SetLaunchCreator`](#setlaunchcreator)                      | launch creator          | Hand over creator rights                     |
| 255 | `Migrate`                                                    | —                       | Pina's reserved account-migration route      |

## `CreateConfig`

| Data                        | Type         | Notes                                                     |
| --------------------------- | ------------ | --------------------------------------------------------- |
| `index`                     | `u64`        | The partner's configuration number and PDA seed           |
| `sqrt_start_price`          | `u128`       | Q64.64                                                    |
| `curve_sqrt_prices`         | `[u128; 16]` | Segment upper bounds; unused slots zero                   |
| `curve_liquidities`         | `[u128; 16]` | Segment liquidities; unused slots zero                    |
| `migration_quote_threshold` | `u64`        | Quote raised before completion                            |
| `total_supply`              | `u64`        | Base minted per launch                                    |
| `creator_allocation`        | `u64`        | Base reserved per creator                                 |
| `creator_vesting_cliff`     | `u64`        | Seconds                                                   |
| `creator_vesting_duration`  | `u64`        | Seconds                                                   |
| `fee_decay_duration`        | `u64`        | Seconds; above zero exactly when start fee > end fee      |
| `start_fee_rate`            | `u32`        | ppm, at most 990,000                                      |
| `end_fee_rate`              | `u32`        | ppm, at most 100,000                                      |
| `creator_fee_share`         | `u32`        | ppm of every fee                                          |
| `migration_fee_rate`        | `u32`        | ppm of the raised quote, at most 100,000                  |
| `creator_lp_share`          | `u32`        | ppm of graduation LP                                      |
| `partner_lp_share`          | `u32`        | ppm of graduation LP; with the creator share at most 100% |
| `segment_count`             | `u8`         | 1 to 16                                                   |
| `base_decimals`             | `u8`         | At most 18                                                |
| `pool_creator_mode`         | `u8`         | `0` launch creator, `1` partner                           |

| # | Account               |      | Notes                                            |
| - | --------------------- | ---- | ------------------------------------------------ |
| 0 | `payer`               | w, s | Pays rent                                        |
| 1 | `authority`           | s    | The partner who owns the configuration           |
| 2 | `config`              | w    | `[b"config", authority, index]`, created         |
| 3 | `quote_mint`          |      | SPL Token or Token-2022 with allowed extensions  |
| 4 | `quote_token_program` |      | Owner of `quote_mint`                            |
| 5 | `amm_config`          |      | Pina AMM tier restricted to `[b"amm_authority"]` |
| 6 | `system_program`      |      |                                                  |

Errors: `InvalidCurve`, `InvalidFeeRates`, `InvalidShares`, `InvalidVesting`, `InvalidMigrationThreshold`, `InvalidSupply`, `InsufficientMigrationLiquidity`, `InvalidBaseMint` (decimals above 18), `InvalidPoolCreatorMode`, `UnsupportedMint`, `InvalidAmmConfig`. Emits `ConfigCreated` with the derived quantities.

## `CreateLaunch`

| Data              | Type  | Notes                                                 |
| ----------------- | ----- | ----------------------------------------------------- |
| `activation_time` | `u64` | Unix timestamp; zero or a past time opens trading now |

| #  | Account               |      | Notes                                                                       |
| -- | --------------------- | ---- | --------------------------------------------------------------------------- |
| 0  | `payer`               | w, s | Pays rent                                                                   |
| 1  | `creator`             | s    | The base mint's mint authority                                              |
| 2  | `config`              |      |                                                                             |
| 3  | `base_mint`           | w    | Zero supply, no freeze authority, configured decimals, creator as authority |
| 4  | `quote_mint`          |      | The configuration's quote mint                                              |
| 5  | `launch`              | w    | `[b"launch", base_mint]`, created                                           |
| 6  | `base_vault`          | w    | `[b"launch_vault", launch, base_mint]`, created                             |
| 7  | `quote_vault`         | w    | `[b"launch_vault", launch, quote_mint]`, created                            |
| 8  | `base_token_program`  |      | Owner of `base_mint`                                                        |
| 9  | `quote_token_program` |      | Owner of `quote_mint`                                                       |
| 10 | `system_program`      |      |                                                                             |

Mints `total_supply` to the base vault and sets the mint authority to none. Errors: `InvalidBaseMint`, `UnsupportedMint`, `AccountMismatch`. Emits `LaunchCreated`.

## `Buy` and `Sell`

| `Buy` data         | Type  | Notes                                       |
| ------------------ | ----- | ------------------------------------------- |
| `quote_amount_in`  | `u64` | Most quote to spend, fee included; non-zero |
| `minimum_base_out` | `u64` | Slippage limit                              |

| `Sell` data         | Type  | Notes                         |
| ------------------- | ----- | ----------------------------- |
| `base_amount_in`    | `u64` | Exact base to sell; non-zero  |
| `minimum_quote_out` | `u64` | Slippage limit, after the fee |

| # | Account               |   | Notes                                |
| - | --------------------- | - | ------------------------------------ |
| 0 | `trader`              | s | Owns the source token account        |
| 1 | `config`              |   | The launch's configuration           |
| 2 | `launch`              | w |                                      |
| 3 | `trader_base`         | w | Credited on a buy, debited on a sale |
| 4 | `trader_quote`        | w | Debited on a buy, credited on a sale |
| 5 | `base_vault`          | w | The launch's stored base vault       |
| 6 | `quote_vault`         | w | The launch's stored quote vault      |
| 7 | `base_token_program`  |   | Owner of the base mint               |
| 8 | `quote_token_program` |   | Owner of the quote mint              |

A buy that reaches the migration price completes the launch and charges only for the quote used. Errors: `NotTrading`, `NotActive`, `AccountMismatch`, `ZeroAmount`, `SlippageExceeded`, `InsufficientLiquidity` (sale past the start price), `MathOverflow`. Emits `Traded`, and `Completed` when the buy completes the launch.

## `Graduate`

No data.

| #  | Account                    |      | Notes                                                       |
| -- | -------------------------- | ---- | ----------------------------------------------------------- |
| 0  | `payer`                    | w, s | Pays rent for the pool and any LP token accounts            |
| 1  | `config`                   |      |                                                             |
| 2  | `launch`                   | w    | Must be `Completed`                                         |
| 3  | `base_mint`                | w    | Written when surplus base is burned                         |
| 4  | `quote_mint`               |      |                                                             |
| 5  | `base_vault`               | w    |                                                             |
| 6  | `quote_vault`              | w    |                                                             |
| 7  | `amm_authority`            |      | `[b"amm_authority"]`                                        |
| 8  | `amm_program`              |      | The Pina AMM                                                |
| 9  | `amm_config`               |      | The configuration's tier                                    |
| 10 | `pool`                     | w    | `[b"pool", amm_config, mint_0, mint_1]` in the AMM, created |
| 11 | `lp_mint`                  | w    | `[b"pool_lp_mint", pool]` in the AMM, created               |
| 12 | `pool_vault0`              | w    | `[b"pool_vault", pool, mint_0]` in the AMM, created         |
| 13 | `pool_vault1`              | w    | `[b"pool_vault", pool, mint_1]` in the AMM, created         |
| 14 | `launch_lp_token`          | w    | The launch's associated LP token account, created           |
| 15 | `base_token_program`       |      |                                                             |
| 16 | `quote_token_program`      |      |                                                             |
| 17 | `lp_token_program`         |      | SPL Token                                                   |
| 18 | `associated_token_program` |      |                                                             |
| 19 | `system_program`           |      |                                                             |
| 20 | `creator`                  |      | Optional: required when `creator_lp_share` is above zero    |
| 21 | `creator_lp_token`         | w    | Optional: the creator's associated LP token account         |
| 22 | `partner`                  |      | Optional: required when `partner_lp_share` is above zero    |
| 23 | `partner_lp_token`         | w    | Optional: the partner's associated LP token account         |

Absent optional accounts are passed as the program id. Errors: `NotCompleted`, `AccountMismatch`, `InsufficientMigrationLiquidity`, `MissingLpAccount`, plus any Pina AMM error from pool creation. Emits `Graduated`. See [graduation.md](graduation.md).

## `ClaimPartnerFees` and `ClaimCreatorFees`

No data.

| # | Account               |   | Notes                                                                 |
| - | --------------------- | - | --------------------------------------------------------------------- |
| 0 | `claimant`            | s | The configuration authority (partner) or the launch creator (creator) |
| 1 | `config`              |   |                                                                       |
| 2 | `launch`              | w |                                                                       |
| 3 | `quote_vault`         | w |                                                                       |
| 4 | `destination`         | w | Any quote token account                                               |
| 5 | `quote_token_program` |   |                                                                       |

Pays everything accrued. Errors: `Unauthorized`, `NothingToClaim`, `AccountMismatch`. Emits `Claimed` with `kind` `0` (partner) or `1` (creator).

## `ClaimCreatorAllocation`

No data.

| # | Account              |   | Notes                  |
| - | -------------------- | - | ---------------------- |
| 0 | `creator`            | s | The launch creator     |
| 1 | `config`             |   |                        |
| 2 | `launch`             | w |                        |
| 3 | `base_vault`         | w |                        |
| 4 | `destination`        | w | Any base token account |
| 5 | `base_token_program` |   |                        |

Pays the allocation vested so far minus what was already claimed. Errors: `Unauthorized`, `NothingToClaim`, `AccountMismatch`. Emits `Claimed` with `kind` `2`.

## `SetLaunchCreator`

| Data          | Type      | Notes           |
| ------------- | --------- | --------------- |
| `new_creator` | `Address` | The new creator |

| # | Account   |   | Notes               |
| - | --------- | - | ------------------- |
| 0 | `creator` | s | The current creator |
| 1 | `launch`  | w |                     |

Moves future creator fees and the unclaimed allocation to `new_creator`. Errors: `Unauthorized`.

## `SweepQuoteDust`

Splits quote sent straight to a launch's quote vault above its accounting into the creator's and partner's fee balances, by the configuration's `creator_fee_share`. Permissionless and safe in every status: the reserve and accrued fees are never touched, and a vault with nothing above its accounting fails with `NothingToClaim`. Useful after a donation or a mis-sent transfer, before or after graduation.

| Account               | Type          | Signer | Writable | Note                       |
| --------------------- | ------------- | ------ | -------- | -------------------------- |
| `config`              | LaunchConfig  |        |          | The launch's configuration |
| `launch`              | Launch        |        | ✓        | The launch to sweep        |
| `quote_vault`         | Token account |        |          | The launch's quote vault   |
| `quote_token_program` | Program       |        |          | Owns the quote mint        |

Emits `DustSwept`.

## Events

| # | Event                  | Fields                                                                                                               |
| - | ---------------------- | -------------------------------------------------------------------------------------------------------------------- |
| 1 | `ConfigCreated`        | `config`, `authority`, `quote_mint`, `amm_config`, `migration_sqrt_price`, `sale_supply`, `migration_supply`         |
| 2 | `LaunchCreated`        | `launch`, `config`, `creator`, `base_mint`, `total_supply`, `activation_time`                                        |
| 3 | `Traded`               | `launch`, `trader`, `is_buy`, `base_amount`, `quote_amount`, `fee`, `creator_fee`, `sqrt_price`, `quote_reserve`     |
| 4 | `Completed`            | `launch`, `quote_reserve`                                                                                            |
| 5 | `Graduated`            | `launch`, `pool`, `pool_base`, `pool_quote`, `migration_fee`, `burned_base`, `burned_lp`, `creator_lp`, `partner_lp` |
| 6 | `Claimed`              | `launch`, `claimant`, `kind`, `amount`                                                                               |
| 7 | `LaunchCreatorChanged` | `launch`, `previous_creator`, `new_creator`                                                                          |
| 8 | `DustSwept`            | `launch`, `amount`, `creator_fee`, `partner_fee`                                                                     |

In `Traded`, `quote_amount` is what the buyer paid (fee included) or what the seller received (fee deducted), and `sqrt_price` and `quote_reserve` are the values after the trade. Flags are `u8`: `1` for true and `0` for false. Decode events with the generated `parsePinaBondingCurveEventsFromLogs` (TypeScript and Dart), which only attributes records the curve itself emitted, not records from the AMM during graduation.

## Errors

| Code | Name                             | Meaning                                                                     |
| ---- | -------------------------------- | --------------------------------------------------------------------------- |
| 0    | `InvalidCurve`                   | Segments empty, not increasing, out of range, or without liquidity          |
| 1    | `InvalidFeeRates`                | A rate is out of range, or the decay does not match the start and end fees  |
| 2    | `InvalidSupply`                  | The total supply cannot cover sale, pool seed, and allocation               |
| 3    | `InvalidMigrationThreshold`      | The threshold is zero, too small, or beyond the curve                       |
| 4    | `InvalidShares`                  | A share is above 100%, or the two LP shares exceed 100% together            |
| 5    | `InvalidVesting`                 | Cliff plus duration does not fit a timestamp                                |
| 6    | `InvalidAmmConfig`               | The tier is not a Pina AMM tier restricted to this program                  |
| 7    | `UnsupportedMint`                | Token program or Token-2022 extension not supported                         |
| 8    | `InvalidBaseMint`                | The base mint is not new, has a freeze authority, or has wrong decimals     |
| 9    | `Unauthorized`                   | The signer is not the required authority                                    |
| 10   | `AccountMismatch`                | A vault, mint, configuration, or pool account does not belong to the launch |
| 11   | `NotTrading`                     | The launch has completed or graduated                                       |
| 12   | `NotActive`                      | Trading has not opened yet                                                  |
| 13   | `NotCompleted`                   | The launch cannot graduate yet                                              |
| 14   | `SlippageExceeded`               | The result is worse than the caller's limit                                 |
| 15   | `ZeroAmount`                     | An amount is zero or the trade rounds to nothing                            |
| 16   | `InsufficientLiquidity`          | The curve cannot absorb a sale this large                                   |
| 17   | `MathOverflow`                   | A value overflowed its type                                                 |
| 18   | `InsufficientMigrationLiquidity` | Graduation would seed too little liquidity                                  |
| 19   | `InvalidPoolCreatorMode`         | The mode is not 0 or 1                                                      |
| 20   | `NothingToClaim`                 | Nothing has accrued or vested                                               |
| 21   | `MissingLpAccount`               | An LP recipient is required because its share is above zero                 |

Pina's own framework errors use codes `0xFFFF0000` and above, for example `DuplicateMutableAccount` when the same writable account appears twice.
