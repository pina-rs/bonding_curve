---
bonding_curve: fix
---

# Reject unrepresentable migration thresholds

Configuration creation rejects a migration threshold when flooring its derived price would raise less quote than requested. Every accepted threshold equals the quote used to derive migration supply and complete a full buy.
