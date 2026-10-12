//! SBF entrypoint.
//!
//! [`dispatch_entrypoint!`] reads the instruction discriminator before any
//! account, routes to the matching `*Accounts` struct, and serves the reserved
//! `Migrate` route for every account type listed on
//! [`crate::CurveInstruction`]. The module also embeds the program's
//! `security.txt`, so only the deployed binary carries it.

use pina::*;

use crate::*;

dispatch_entrypoint!(CurveInstruction);

// Contact details for security researchers, embedded in the deployed binary.
// Everything that may change lives behind the linked pages, so updating it
// never needs a program upgrade.
solana_security_txt::security_txt! {
	name: "Pina Bonding Curve",
	project_url: "https://github.com/pina-rs/bonding_curve",
	contacts: "link:https://github.com/pina-rs/bonding_curve/security/advisories/new",
	policy: "https://github.com/pina-rs/bonding_curve/blob/main/SECURITY.md",
	preferred_languages: "en",
	source_code: "https://github.com/pina-rs/bonding_curve"
}
