//! `no_std` CPI client for the [Pina Bonding Curve](https://github.com/pina-rs/bonding_curve).
//!
//! Other on-chain programs use this crate to call the bonding curve — for
//! example to buy on behalf of a vault or to create launches for their users.
//! Build the generated instruction struct with its accounts and data, then
//! call `.invoke()` or `.invoke_signed(signers)` with the validated program
//! account. Every type is generated from the program's IDL by `pina generate`;
//! regenerate it instead of editing it.
//!
//! The account parsers in this crate check discriminators only. Before reading
//! another program's account through them, call `assert_owner` with the
//! curve's program id.
//!
//! See the repository's `docs/cpi.md` for a complete example.

#![no_std]

pub mod generated;
pub use generated::*;
