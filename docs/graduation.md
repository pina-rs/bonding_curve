# Completion and graduation

A launch ends its life on the curve in two steps. **Completion** happens inside a buy, the moment the price reaches the migration price. **Graduation** is a separate, permissionless instruction that moves the raised quote and the reserved base into a new Pina AMM pool, where the token trades from then on.

```text
Trading ──Buy reaches the migration price──▶ Completed ──Graduate (anyone)──▶ Graduated
   ▲  │                                         │                               │
   └──┘ Buy / Sell                              no trading                      trades on the Pina AMM
```

## Completion

`CreateConfig` derives `migration_sqrt_price`, the price at which the curve has raised `migration_quote_threshold`. Every buy stops at that price:

- the buyer receives the base the curve sells up to the migration price;
- the buyer pays only the quote the curve used plus its fee; the rest of `quote_amount_in` stays in their account;
- the launch's status becomes `Completed` and the program emits `Completed`.

Completion therefore happens exactly at the migration price, never past it, and the last buyer is never overcharged. After completion, `Buy` and `Sell` fail with `NotTrading`, so the price stays where graduation will open the pool.

## Graduation

`Graduate` can be sent by anyone once a launch is `Completed`. The sender pays rent for the new pool, its vaults, its LP mint, and any LP token accounts it creates (about 0.01 SOL, plus about 0.002 SOL per LP recipient). It runs in this order:

1. **Plan.** Take the migration fee from `quote_reserve`. Compute the base available for the pool: the base vault's balance minus the creator allocation that has not been claimed yet.
2. **Seed at the curve price.** Pair the remaining quote with exactly the base it is worth at the final curve price. Whichever side is in excess is the surplus.
3. **Commit.** Mark the launch `Graduated`, record the pool, zero `quote_reserve`, and add the migration fee (plus any quote surplus) to the creator and partner fee balances. This happens before any cross-program call.
4. **Create the pool.** Call the Pina AMM's `CreatePool`, signed by the launch (the depositor and LP owner) and by the program's AMM authority PDA (the tier's pool creator). The pool's creator is the launch creator or the partner, per `pool_creator_mode`, and its creator fee is paid in the quote token.
5. **Distribute LP.** Send `creator_lp_share` of the minted LP to the creator and `partner_lp_share` to the partner, creating their associated LP token accounts when needed. When the creator and the partner are the same address, the two shares are paid as one transfer to that address's LP token account. Burn the rest.
6. **Burn surplus base.** Burn the base the pool did not need, shrinking the token's supply.
7. Emit `Graduated` with the amounts.

If any step fails, the whole transaction fails and the launch stays `Completed`.

### Price continuity

The pool opens at the curve's final price:

```text
pool_base  = floor(pool_quote * 2^128 / sqrt_price^2)
pool price = pool_quote / pool_base  ≈  (sqrt_price / 2^64)^2
```

Someone who bought the last token on the curve sees the same price on the AMM a block later. The end-to-end test `completion_stops_trading_and_graduation_seeds_the_amm_at_the_curve_price` checks the pool price against the curve price to within one part per million.

### Where every token goes

Using the reference configuration from [curves.md](curves.md#a-worked-example) (1 billion tokens, 85 SOL threshold, 50 million creator allocation, 2% migration fee):

| Tokens                             | Amount         |
| ---------------------------------- | -------------- |
| Sold on the curve                  | ~720.6 million |
| Seeded into the AMM pool           | ~167.6 million |
| Creator allocation (still vesting) | 50 million     |
| Burned at graduation               | ~61.7 million  |

| Quote                                 | Amount   |
| ------------------------------------- | -------- |
| Raised on the curve (`quote_reserve`) | 85 SOL   |
| Migration fee (creator and partner)   | 1.7 SOL  |
| Seeded into the AMM pool              | 83.3 SOL |

Nothing stays behind except the creator's unvested allocation and unclaimed fees, both of which remain claimable after graduation.

### LP and locked liquidity

The Pina AMM mints LP to the launch, which keeps none of it. With `creator_lp_share` and `partner_lp_share` both zero, every LP token is burned and the pool's liquidity is locked forever. The AMM tracks its own `lp_supply`, so burned LP is never redistributed to other liquidity providers: the locked share stays locked even as others deposit and withdraw.

## The restricted AMM tier

Graduation creates the pool at its canonical address, `[b"pool", amm_config, mint_0, mint_1]` in the AMM. If someone could create that pool first, at a bad price, graduation would fail or start from the wrong price. The configuration's AMM tier must therefore be **restricted to this program's AMM authority PDA**, `[b"amm_authority"]` under the curve program. `CreateConfig` checks that the tier is owned by the Pina AMM and that its `pool_creator_authority` is that PDA, and rejects any other tier with `InvalidAmmConfig`. No one but the curve can create pools in it.

## Sending `Graduate`

The CLI derives every account:

```sh
pina-curve graduate --base-mint <MINT>
```

Clients that build the instruction themselves need the AMM accounts. With `mint_0` and `mint_1` the two mints sorted bytewise:

| Account           | Address                                                           |
| ----------------- | ----------------------------------------------------------------- |
| `amm_authority`   | `[b"amm_authority"]` under the curve program                      |
| `amm_program`     | The Pina AMM, `pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV`       |
| `amm_config`      | The configuration's `amm_config`                                  |
| `pool`            | `[b"pool", amm_config, mint_0, mint_1]` under the AMM             |
| `lp_mint`         | `[b"pool_lp_mint", pool]` under the AMM                           |
| `pool_vault0`/`1` | `[b"pool_vault", pool, mint_0]` and `[..., mint_1]` under the AMM |
| `launch_lp_token` | The launch's associated token account for `lp_mint` (SPL Token)   |

Append `creator` and `creator_lp_token` when `creator_lp_share` is above zero, and `partner` and `partner_lp_token` when `partner_lp_share` is above zero; otherwise leave them out (the generated clients fill their slots with the program id). A missing required pair fails with `MissingLpAccount`. When the launch creator and the configuration's partner are the same address, pass only the creator pair: the two payouts target one token account, which cannot appear twice as writable, so their shares are merged into the single payment. The full account list is in [instructions.md](instructions.md#graduate).

## After graduation

Trade the token through the Pina AMM: [`pina-amm`](https://github.com/pina-rs/amm/blob/main/docs/cli.md), [`@pina-rs/amm`](https://github.com/pina-rs/amm/blob/main/docs/typescript.md), [`pina_amm`](https://github.com/pina-rs/amm/blob/main/docs/dart.md), or [`pina_amm_client`](https://github.com/pina-rs/amm/blob/main/docs/rust-client.md). The launch account keeps the pool's address in `pool`.
