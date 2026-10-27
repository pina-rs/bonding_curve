---
bonding_curve: breaking
---

# Reject unrepresentable migration thresholds

Configuration creation rejects a migration threshold when flooring its derived price would raise less quote than requested. Curve designs previously accepted with an inexact threshold must choose an exactly representable amount. Every accepted threshold equals the quote used to derive migration supply and complete a full buy.
