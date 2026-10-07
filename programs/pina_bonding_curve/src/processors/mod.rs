//! Account lists and handlers for every instruction.
//!
//! Each `*Accounts` struct is the ordered account list its instruction expects;
//! the generated clients build the same list. Handlers validate every account
//! before reading or moving value, commit state before any outgoing transfer
//! or CPI, and emit an event when they succeed.

mod claims;
mod common;
mod config;
mod graduate;
mod launch;
mod trade;

pub use claims::*;
pub use config::*;
pub use graduate::*;
pub use launch::*;
pub use trade::*;
