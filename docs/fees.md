# Fees

A launch charges three kinds of fee over its life. All of them are set by the configuration and can never change for a live launch.

| Fee         | When                        | Paid in      | Goes to                                             |
| ----------- | --------------------------- | ------------ | --------------------------------------------------- |
| Trading fee | Every buy and sell          | Quote        | Split between creator and partner                   |
| Migration   | Once, at graduation         | Quote        | Split between creator and partner                   |
| Pool fees   | Every swap after graduation | Either token | The Pina AMM tier's LPs, protocol, and pool creator |

Rates are parts per million (`10_000` is 1%).

## The trading fee

### Rate over time

The rate starts at `start_fee_rate` when the launch activates and falls in a straight line to `end_fee_rate` over `fee_decay_duration` seconds:

```text
elapsed = now - activation_time
rate    = end                                             if decay = 0 or elapsed >= decay
        = start - (start - end) * elapsed / decay         otherwise (integer division)
```

```text
rate
 50% ┤●
     │  ●
     │    ●
     │      ●
     │        ●
  1% ┤          ●──────────────────────────
     └──────────┬───────────────────────── time
     activation 60 s
```

The decay exists to tax sniping. A bot buying in the launch block with a 50% opening fee gives up half its purchase; a person buying a minute later pays the end rate. The rate is computed from the cluster clock on every trade, so there is nothing to crank.

### How a trade is charged

The fee is always taken from the quote side and always rounded up, in the curve's favour.

**Buy.** `quote_amount_in` includes the fee:

```text
fee = ceil(quote_amount_in * rate / 1_000_000)
net = quote_amount_in - fee            spent on the curve
```

If the buy reaches the migration price before using all of `net`, the launch completes and the buyer pays only for what the curve used: the program grosses the used quote back up (`ceil(used * 1_000_000 / (1_000_000 - rate))`), charges that, and leaves the rest in the buyer's account.

**Sell.** The fee comes out of the proceeds:

```text
quote_out = curve's quote for base_amount_in
fee       = ceil(quote_out * rate / 1_000_000)
received  = quote_out - fee
```

`minimum_quote_out` is compared with `received`, after the fee.

### The split

```text
creator_fee = floor(fee * creator_fee_share / 1_000_000)
partner_fee = fee - creator_fee
```

Both shares accrue on the `Launch` account (`creator_fees`, `partner_fees`) and stay in the launch's quote vault, separate from `quote_reserve`, until claimed. Fees never back the curve: the reserve that prices trades excludes them.

## The migration fee

At graduation, the program takes `floor(quote_reserve * migration_fee_rate / 1_000_000)` from the raised quote before seeding the pool, and splits it between creator and partner with the same `creator_fee_share`. Rounding dust that cannot be paired with base at the final price joins the migration fee. See [graduation.md](graduation.md).

## Claiming

| Instruction              | Signer                      | Pays                                                 |
| ------------------------ | --------------------------- | ---------------------------------------------------- |
| `ClaimPartnerFees`       | The configuration authority | All accrued partner fees, to any quote token account |
| `ClaimCreatorFees`       | The launch creator          | All accrued creator fees, to any quote token account |
| `ClaimCreatorAllocation` | The launch creator          | Base vested so far and not yet claimed               |

Claims work at every stage, before and after graduation, and fail with `NothingToClaim` when nothing has accrued. `SetLaunchCreator` hands the creator's fee and allocation rights to a new address, for example a multisig or a revenue-sharing program.

```sh
pina-curve claim partner-fees --base-mint <MINT>   # signed by the partner
pina-curve claim creator-fees --base-mint <MINT>   # signed by the creator
pina-curve claim allocation   --base-mint <MINT>   # signed by the creator
```

## Fees after graduation

A graduated launch trades on the Pina AMM, in the tier named by the configuration's `amm_config`. The pool snapshots that tier's rates when it is created:

- the **trade fee** goes to liquidity providers, minus the tier's **protocol share**;
- the **creator fee** is paid to the pool's creator in the **quote token**. The configuration's `pool_creator_mode` makes that either the launch creator (`0`) or the partner (`1`).

Collect pool creator fees with the AMM's `CollectCreatorFees` (`pina-amm fees collect-creator --pool <POOL>`). See the [AMM's fee guide](https://github.com/pina-rs/amm/blob/main/docs/fees.md).

## Worked example

A configuration with a 50% → 1% decay over 60 seconds, a 50% creator share, and a 2% migration fee:

| Event                             | Fee       | Creator    | Partner    |
| --------------------------------- | --------- | ---------- | ---------- |
| Buy 1 SOL at activation           | 0.5 SOL   | 0.25 SOL   | 0.25 SOL   |
| Buy 1 SOL 30 s later              | 0.255 SOL | 0.1275 SOL | 0.1275 SOL |
| Buy 1 SOL after a minute          | 0.01 SOL  | 0.005 SOL  | 0.005 SOL  |
| Graduation with an 85 SOL reserve | 1.7 SOL   | 0.85 SOL   | 0.85 SOL   |
