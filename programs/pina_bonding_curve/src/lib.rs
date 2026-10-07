//! # Pina Bonding Curve
//!
//! A launchpad program for Solana, built with [Pina](https://github.com/pina-rs/pina),
//! that sells a new token along a configurable price curve and then graduates
//! it into the [Pina AMM](https://github.com/pina-rs/amm).
//!
//! The design follows Meteora's Dynamic Bonding Curve:
//!
//! - **Launch configurations** ([`LaunchConfig`]) belong to a partner — a
//!   launchpad, app, or community — who chooses the curve, the quote token,
//!   the fees, how fees split with creators, the creator's vested allocation,
//!   and where the liquidity goes at graduation. Anyone can create one.
//! - **Curves** are up to sixteen segments of concentrated liquidity between
//!   square-root prices ([`curve`]), so a configuration can describe anything
//!   from a single constant-product curve to a hand-shaped price schedule.
//! - **Launches** ([`Launch`]) mint a creator's fixed supply into a vault the
//!   program controls, revoke the mint authority, and trade along the curve
//!   until the configured amount of quote has been raised.
//! - **Migration** is permissionless: anyone can move a completed launch into a
//!   Pina AMM pool at exactly the curve's final price. The AMM tier is
//!   restricted to this program, so nobody can create that pool first.
//!
//! ## Module map
//!
//! | Module | Contents |
//! | --- | --- |
//! | [`state`] | Account layouts, seeds, and limits |
//! | [`instructions`] | Instruction data layouts and discriminators |
//! | [`processors`] | Account lists and handlers for every instruction |
//! | [`curve`] | Segment math, buys, sells, and configuration quantities |
//! | [`math`] | 256-bit helpers, fees, the fee schedule, and vesting |
//! | [`events`] | Events emitted to the transaction log |
//! | [`errors`] | Program error codes |

#![no_std]
#![allow(clippy::inline_always)]
// `AccountView` is a copyable handle, but borrowing it keeps account access and
// mutability explicit at helper boundaries.
#![allow(clippy::trivially_copy_pass_by_ref)]

#[cfg(all(
	not(any(target_os = "solana", target_arch = "bpf")),
	not(feature = "bpf-entrypoint"),
	not(test)
))]
extern crate std;

/// On-chain entrypoint that routes every instruction to its processor.
#[cfg(feature = "bpf-entrypoint")]
pub mod entrypoint;

pub mod curve;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod processors;
pub mod state;

mod token;

pub use errors::*;
pub use events::*;
pub use instructions::*;
use pina::*;
pub use processors::*;
pub use state::*;

declare_id!("CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9");
