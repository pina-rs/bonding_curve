# Security model

This page records what the program defends against, how, and which test proves it. It is not an audit. The program has not been independently reviewed yet; see [SECURITY.md](../SECURITY.md) for reporting.

## Trust assumptions

| Party                      | Can                                                                           | Cannot                                                              |
| -------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| Upgrade authority          | Upgrade the program                                                           | Touch funds or change a configuration without shipping new code     |
| Partner (config authority) | Create configurations; claim partner fees; receive partner LP                 | Change a configuration; reach the curve's reserve or creators' fees |
| Creator                    | Launch tokens; claim creator fees and the vested allocation; hand rights over | Mint more tokens; freeze holders; reach the curve's reserve         |
| Anyone                     | Trade, graduate completed launches                                            | Graduate early; create a launch's AMM pool first                    |
| Pina AMM                   | Hold graduated liquidity under its own rules                                  | Change a launch's state                                             |

The program has no admin role and no pause switch. The upgrade authority is the single point of trust, as for any upgradeable Solana program. Deployments should hold it in a multisig and announce upgrades.

## Invariants

| Invariant                                                 | Enforcement                                                 | Test                                                                                                                    |
| --------------------------------------------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| The quote reserve always covers the curve's integral      | Every rounding favours the curve                            | `reserves_always_cover_the_curve` (property test)                                                                       |
| A buy-then-sell round trip never profits                  | Rounding plus fees                                          | `round_trips_never_profit` (property test), `trading_follows_the_curve_and_splits_fees`                                 |
| Splitting a buy never beats one buy                       | Price after a buy rounds down; base out rounds down         | `splitting_buys_never_beats_the_curve` (property test)                                                                  |
| A sale cannot push the price below the start              | The curve refuses the remainder                             | `selling_more_than_was_bought_fails`                                                                                    |
| A launch completes exactly at the derived migration price | Buys stop at the cap and charge only for quote used         | `a_full_buy_completes_at_the_derived_price`, `completion_stops_trading_and_graduation_seeds_the_amm_at_the_curve_price` |
| The pool opens at the curve's final price                 | Seeding pairs quote with the base it is worth at that price | `the_pool_opens_at_the_curve_price`, the end-to-end graduation test                                                     |
| The supply is fixed at launch                             | Mint authority revoked in `CreateLaunch`                    | `launches_mint_a_fixed_supply_and_revoke_the_mint_authority`                                                            |
| Fees never back the curve                                 | `quote_reserve` excludes accrued fees                       | `trading_follows_the_curve_and_splits_fees`                                                                             |
| Fees and allocations go only to their owners              | Signer compared with stored authority and creator           | `fees_and_vested_allocations_are_claimed_only_by_their_owners`                                                          |
| Vesting never releases more than the allocation           | Linear schedule capped at the duration                      | `vesting_never_exceeds_the_allocation` (property test)                                                                  |
| A gross-up always covers its own fee                      | Ceiling division                                            | `gross_up_always_covers_its_fee` (property test)                                                                        |
| 256-bit arithmetic is exact                               | Reference-checked multiply and divide                       | `mul_div_matches_the_reference` (property test), `div_wide_rounds_and_rejects_overflow`                                 |

## Account validation

| Account                     | Check                                                                                                          |
| --------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `LaunchConfig`, `Launch`    | Owned by the program, correct discriminator, schema version, and size (typed load); created only at their PDAs |
| Launch ↔ configuration      | Every instruction that takes both checks the launch's stored `config`                                          |
| Vaults                      | Compared with the addresses stored on the launch                                                               |
| Mints in `Graduate`         | Compared with the launch's stored mints; the AMM tier with the configuration's                                 |
| AMM tier in `CreateConfig`  | Owned by the Pina AMM, restricted to this program's AMM authority PDA                                          |
| AMM program in `Graduate`   | Must be the trusted Pina AMM program id; the CPI client re-checks it                                           |
| Base mint in `CreateLaunch` | Zero supply, no freeze authority, creator as mint authority, configured decimals, supported extensions         |
| Token programs              | Must be SPL Token or Token-2022, and must own the accounts they move                                           |
| Signers                     | Partner and creator compared with stored state                                                                 |
| Writable accounts           | Declared `&mut`; Pina rejects a writable account that appears twice                                            |

## Attacks considered

| Attack                                                                                        | Outcome                                                         | Test                                                                       |
| --------------------------------------------------------------------------------------------- | --------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Use an open AMM tier so anyone can create the pool first                                      | `InvalidAmmConfig` at configuration                             | `configurations_validate_their_terms`                                      |
| Launch a mint the creator does not control                                                    | `InvalidBaseMint`                                               | `launches_mint_a_fixed_supply_and_revoke_the_mint_authority`               |
| Launch a transfer-fee mint so transfers move less than accounted                              | `UnsupportedMint`                                               | `launches_mint_a_fixed_supply_and_revoke_the_mint_authority`               |
| Snipe the launch block                                                                        | Pays the decaying opening fee                                   | `the_fee_starts_high_and_decays_and_trading_waits_for_activation`          |
| Trade before activation                                                                       | `NotActive`                                                     | `the_fee_starts_high_and_decays_and_trading_waits_for_activation`          |
| Trade after completion to move the graduation price                                           | `NotTrading`                                                    | `completion_stops_trading_and_graduation_seeds_the_amm_at_the_curve_price` |
| Graduate before completion                                                                    | `NotCompleted`                                                  | `completion_stops_trading_and_graduation_seeds_the_amm_at_the_curve_price` |
| Graduate without paying the configured LP recipients                                          | `MissingLpAccount`                                              | `graduation_pays_lp_shares_to_the_creator_and_partner`                     |
| A partner launches under their own configuration, so both LP payouts target one token account | The shares merge into a single payment                          | `graduation_merges_lp_shares_when_the_creator_is_the_partner`              |
| Hand creator rights to the default address, which can never sign                              | `DefaultCreator`                                                | `fees_and_vested_allocations_are_claimed_only_by_their_owners`             |
| Graduate a launch whose price drifted from its configuration                                  | `MigrationPriceMismatch`                                        | held by the `Completed` state machine; asserted defensively                |
| Claim someone else's fees or allocation                                                       | `Unauthorized`                                                  | `fees_and_vested_allocations_are_claimed_only_by_their_owners`             |
| Create a configuration with unreachable or unfunded terms                                     | `InvalidMigrationThreshold`, `InvalidSupply`, `InvalidFeeRates` | `configurations_validate_their_terms`                                      |

## Static analysis

`devenv shell lint:all` runs Pina's official security lint set, which rejects unchecked asset arithmetic, missing signer or owner checks, and inconsistent token-program usage, alongside `clippy -D warnings`.

## Known limitations

- **AMM tier rates can change before graduation.** A configuration pins its tier's address, not its rates: the tier's authority may change them at any time, and a pool snapshots whatever the tier says when it is created. A graduating launch therefore pays the tier's current rates, bounded by the AMM's own 10% cap. Use a tier whose authority you trust, and re-read its rates before relying on them.
- **Quote mints with freeze authorities.** A quote mint such as USDC can have a launch's quote vault frozen by its issuer, halting that launch. This is inherent to such mints; the curve accepts them because rejecting them would exclude most stablecoins. Base mints may never have a freeze authority.
- **Configuration quality is the partner's responsibility.** The program enforces that terms are consistent and fundable, not that they are fair. Read a configuration before trading under it.
- **Spot price is not an oracle.** The curve price moves with every trade and is manipulable within a transaction.
- **No independent audit yet.** Do not route material value through the program until one is published.
