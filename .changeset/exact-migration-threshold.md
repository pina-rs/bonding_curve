---
bonding_curve: breaking
---

# Reject unrepresentable migration thresholds

Configuration creation rejects a migration threshold when flooring its derived price would raise less quote than requested. Curve designs previously accepted with an inexact threshold must choose an exactly representable amount. Every accepted threshold equals the quote used to derive migration supply and complete a full buy.

Track the actual quote still required through each segment instead of maintaining a second accumulated total. A partial segment must consume the exact remaining amount before the configuration is accepted.
