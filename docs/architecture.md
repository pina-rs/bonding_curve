# Architecture

The Pina Bonding Curve is one program with two account types and nine instructions. A **configuration** describes a launchpad; a **launch** is one token sold under it. This page explains how they fit together and why each design decision was made. For the curve formulas see [curves.md](curves.md); for every account list see [instructions.md](instructions.md).

## Accounts

### `LaunchConfig` — a launchpad

```text
PDA: [b"config", authority, index.to_le_bytes()]   (index: u64)
```

| Field                                               | Type         | Meaning                                                                        |
| --------------------------------------------------- | ------------ | ------------------------------------------------------------------------------ |
| `authority`                                         | `Address`    | The partner: earns the partner fee share and optional LP                       |
| `quote_mint`                                        | `Address`    | The token every launch raises                                                  |
| `amm_config`                                        | `Address`    | The Pina AMM fee tier launches graduate into                                   |
| `sqrt_start_price`                                  | `u128`       | Q64.64 square-root price every launch starts at                                |
| `migration_sqrt_price`                              | `u128`       | Q64.64 square-root price at which a launch completes (derived)                 |
| `curve_sqrt_prices`                                 | `[u128; 16]` | Upper square-root price of each segment; unused slots are zero                 |
| `curve_liquidities`                                 | `[u128; 16]` | Liquidity of each segment; unused slots are zero                               |
| `total_supply`                                      | `u64`        | Base minted for every launch                                                   |
| `sale_supply`                                       | `u64`        | Base sold up to the migration price (derived)                                  |
| `migration_supply`                                  | `u64`        | Base reserved to seed the AMM pool (derived)                                   |
| `creator_allocation`                                | `u64`        | Base reserved for each creator, released by vesting                            |
| `migration_quote_threshold`                         | `u64`        | Quote a launch raises before it completes                                      |
| `creator_vesting_cliff`, `creator_vesting_duration` | `u64`        | Vesting schedule in seconds, measured from activation                          |
| `fee_decay_duration`                                | `u64`        | Seconds over which the trading fee falls to its end rate                       |
| `index`                                             | `u64`        | The partner's configuration number, part of the PDA seeds                      |
| `start_fee_rate`, `end_fee_rate`                    | `u32`        | Trading fee at activation and after the decay, parts per million               |
| `creator_fee_share`                                 | `u32`        | Creator's share of every fee; the partner receives the rest                    |
| `migration_fee_rate`                                | `u32`        | Fee on the raised quote at graduation                                          |
| `creator_lp_share`, `partner_lp_share`              | `u32`        | Shares of graduation LP paid out; the rest is burned                           |
| `base_decimals`                                     | `u8`         | Decimals every launched mint must have                                         |
| `segment_count`                                     | `u8`         | Segments in use, 1 to 16                                                       |
| `pool_creator_mode`                                 | `u8`         | `0` pays the AMM pool's creator fees to the launch creator, `1` to the partner |
| `bump`                                              | `u8`         | Canonical PDA bump                                                             |

A configuration is immutable. Anyone can create configurations for themselves, numbered by `index`.

### `Launch` — one token

```text
PDA: [b"launch", base_mint]
```

| Field                          | Type      | Meaning                                                        |
| ------------------------------ | --------- | -------------------------------------------------------------- |
| `config`                       | `Address` | The configuration this launch follows                          |
| `creator`                      | `Address` | Earns the creator fee share and the vested allocation          |
| `base_mint`, `quote_mint`      | `Address` | The launched token and the raised token                        |
| `base_vault`, `quote_vault`    | `Address` | Launch-owned token accounts                                    |
| `pool`                         | `Address` | The Pina AMM pool after graduation; the default address before |
| `sqrt_price`                   | `u128`    | Current Q64.64 square-root price                               |
| `quote_reserve`                | `u64`     | Quote backing the curve, excluding fees                        |
| `partner_fees`, `creator_fees` | `u64`     | Fees accrued in the quote vault and not yet claimed            |
| `creator_claimed`              | `u64`     | Allocation already claimed                                     |
| `activation_time`              | `i64`     | When trading opened; the fee decay and vesting start here      |
| `status`                       | `u8`      | `0` trading, `1` completed, `2` graduated                      |
| `bump`                         | `u8`      | Canonical PDA bump                                             |

Seeding the launch by its mint gives each token exactly one launch.

### Data-free PDAs

| Address       | Seeds                             | Purpose                                                       |
| ------------- | --------------------------------- | ------------------------------------------------------------- |
| Launch vault  | `[b"launch_vault", launch, mint]` | Token account for one side of a launch, owned by the launch   |
| AMM authority | `[b"amm_authority"]`              | Signs as the restricted AMM tier's pool creator at graduation |

The launch PDA owns both of its vaults, so each launch signs only for its own funds. Every account carries a one-byte discriminator followed by a one-byte schema version written by [`pina migrations`](https://github.com/pina-rs/pina), so layouts can evolve after deployment through Pina's reserved `Migrate` instruction.

## Lifecycle

```text
partner ──CreateConfig──▶ LaunchConfig (immutable)
                              │
creator ──CreateLaunch──▶ Launch + vaults; fixed supply minted, mint authority revoked
                              │
traders ──Buy / Sell──────▶ price moves along the curve, fees accrue
                              │ a buy reaches the migration price
                              ▼
                          Completed ──Graduate (anyone)──▶ Pina AMM pool; LP paid out or burned
                              │
partner ──ClaimPartnerFees    creator ──ClaimCreatorFees / ClaimCreatorAllocation / SetLaunchCreator
```

## Design decisions

### Curves are concentrated-liquidity segments

A segment is a constant-product curve between two prices, so one segment reproduces the virtual-reserve curve that pump-style launchpads use, and sixteen segments approximate any increasing price schedule. The same two formulas price every segment, the arithmetic is exact integer math with documented rounding, and the derived graduation price lines up with the AMM's constant-product pool. See [curves.md](curves.md).

### A launch mints its whole supply up front

`CreateLaunch` requires a brand-new mint controlled by the creator, mints `total_supply` into the base vault, and revokes the mint authority in the same instruction. Nobody can mint more afterwards, the supply is visible from the first block, and unsold base is burned at graduation. The creator still owns everything else about the mint, such as its metadata, which is created before the launch.

### Configurations are immutable and self-serve

Every launch under a configuration shares its terms, so a trader evaluates one account per launchpad. There is no admin, no allowlist, and no way to change a live launch's fees. Partners create new configurations for new terms.

### Fees stay in the launch until claimed

Fees accrue on the launch account and stay in its quote vault, excluded from the curve's reserve. A trade makes exactly two token transfers, and fee recipients claim whenever they want, at any stage.

### Graduation commits before it calls out

`Graduate` writes the launch's new state before calling the AMM. A failure anywhere rolls back everything, and the launch can never be observed half-graduated.

### Graduation can never be front-run

The configuration's AMM tier must be restricted to the program's AMM authority PDA, so only the curve can create pools in it. See [graduation.md](graduation.md#the-restricted-amm-tier).

## Token programs and extensions

Base and quote mints may each be SPL Token or Token-2022. Token-2022 mints may carry only extensions that cannot change how many tokens a transfer moves or who can move them:

| Allowed                                                  | Rejected                                                                              |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| Metadata pointer, token metadata                         | Transfer fee                                                                          |
| Group pointer, group, group member pointer, group member | Transfer hook                                                                         |
| Interest-bearing config, scaled UI amount                | Permanent delegate, pausable                                                          |
|                                                          | Non-transferable, default account state, confidential transfers, mint close authority |

These match the Pina AMM's rules, so every launch can graduate. A base mint must also have no freeze authority, so no one can freeze a launch's vault or a holder's tokens.

## Threats and mitigations

| Threat                                     | Mitigation                                                                                   |
| ------------------------------------------ | -------------------------------------------------------------------------------------------- |
| A creator mints more tokens after launch   | The mint authority is revoked inside `CreateLaunch`                                          |
| A creator freezes holders                  | Base mints with a freeze authority are rejected                                              |
| Bots buy the launch block                  | The decaying opening fee                                                                     |
| Someone creates the graduation pool first  | The AMM tier is restricted to the curve's AMM authority PDA                                  |
| Rounding drains the curve over many trades | Every rounding favours the curve; property tests check the reserve covers the curve integral |
| A malicious token program fakes a transfer | Token programs must be SPL Token or Token-2022 and own the accounts they move                |
| A vault from another launch is passed in   | Vault addresses are compared with the launch's stored vaults                                 |
| A partner changes terms on a live launch   | Configurations are immutable                                                                 |

[security.md](security.md) lists the full threat model and the tests that check each invariant.

## What is deliberately missing

- **No admin, pause, or withdrawal path.** The upgrade authority can ship a fix; a pause switch would only add a way to freeze funds.
- **No metadata creation.** Creators set up their mint (including Token-2022 metadata) before launching, with whatever tool they prefer.
- **No referral or platform fees beyond the partner share.** A platform that wants its own cut owns the configuration.
- **No pre-sale or allowlist.** Use `activation_time` to schedule a launch and the fee decay to discourage snipers.
- **No graduation to other AMMs.** One integration keeps the account list fixed and the graduation path audited.
