---
bonding_curve: user
---

# Cheaper trades on a smaller program

## User impact

Every buy and sell now costs about 440 and 375 fewer compute units, and the deployed program is about 13 KB (10%) smaller, which lowers its rent. Trades no longer revalidate the price curve stored in an immutable configuration, copy only the pricing fields they need out of it, and share one compiled copy of the wide division and signer-seed routines instead of a copy per call site. Nothing about the math, fees, or account layout changed.
