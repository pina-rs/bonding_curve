//! Rust client for the [Pina Bonding Curve](https://github.com/pina-rs/bonding_curve).
//!
//! Everything except `addresses` is generated from the program's IDL by
//! `pina generate`; regenerate it instead of editing it. The crate gives
//! off-chain Rust code:
//!
//! - instruction builders that take each instruction's ordered accounts and
//!   data (`instructions::Buy`, `instructions::CreateLaunch`, ...);
//! - account decoders for `LaunchConfig` and `Launch` (`accounts`);
//! - event decoders for every log record the program emits (`events`);
//! - the program's error codes (`errors`) and address (`PINA_BONDING_CURVE_ID`);
//! - hand-written helpers for the launch vaults, the AMM authority, and the
//!   Pina AMM pool a launch graduates into (`addresses`).
//!
//! Decoders check discriminators and schema versions only. Before trusting a
//! decoded account, compare the account's owner with `PINA_BONDING_CURVE_ID`
//! or fetch it from a derived PDA address.
//!
//! See the repository's `docs/rust-client.md` for complete examples.

pub mod addresses;
pub mod generated;
pub use generated::programs::*;
pub use generated::*;
