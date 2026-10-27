---
bonding_curve: breaking
---

# Assign canonical mint roles before trading

Launch creation requires the base mint to sort after the quote mint by address bytes. Since a base mint has one launch PDA, each accepted launch now uniquely identifies the AMM's unordered pair and tier; reciprocal launches fail before minting supply or accepting trades. The CLI generates compatible mint keys automatically and checks existing mint addresses.
