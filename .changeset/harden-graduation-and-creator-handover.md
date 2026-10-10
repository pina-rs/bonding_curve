---
bonding_curve: major
---

# Harden graduation payouts and creator handover

- A partner who launches under their own configuration no longer bricks graduation: when the launch creator and the partner are the same address, the creator and partner LP shares are paid as one transfer to that address's LP token account instead of naming the same writable account twice, which the runtime rejects.
- `SetLaunchCreator` rejects the default address, which could otherwise hold creator rights that can never sign, leaving fees and the vested allocation unclaimable forever.
- `Graduate` asserts the launch's price equals the configuration's migration price before seeding the pool, turning any future state drift into a failed transaction instead of a mispriced pool.
