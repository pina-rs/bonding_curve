//! Addresses the IDL cannot describe: the curve's data-free PDAs and the Pina
//! AMM accounts a graduation creates.
//!
//! This module is hand-written; everything under `generated` is not.

use solana_pubkey::Pubkey;

use crate::PINA_BONDING_CURVE_ID;

/// The Pina AMM program every launch graduates into. The curve rejects any
/// other AMM program at graduation.
pub const PINA_AMM_ID: Pubkey =
	Pubkey::from_str_const("pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV");

/// A launch's token account for `mint`: `[b"launch_vault", launch, mint]`.
#[must_use]
pub fn launch_vault(launch: &Pubkey, mint: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(
		&[b"launch_vault", launch.as_ref(), mint.as_ref()],
		&PINA_BONDING_CURVE_ID,
	)
	.0
}

/// The PDA that signs as a restricted Pina AMM tier's pool creator:
/// `[b"amm_authority"]`. Configurations must name a tier restricted to it.
#[must_use]
pub fn amm_authority() -> Pubkey {
	Pubkey::find_program_address(&[b"amm_authority"], &PINA_BONDING_CURVE_ID).0
}

/// The Pina AMM accounts `Graduate` creates for one launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraduationPool {
	/// The pool: `[b"pool", amm_config, mint_0, mint_1]` in the AMM.
	pub pool: Pubkey,
	/// The pool's LP mint: `[b"pool_lp_mint", pool]`. LP mints always belong
	/// to SPL Token.
	pub lp_mint: Pubkey,
	/// The smaller of the two mints, compared byte by byte.
	pub mint_0: Pubkey,
	/// The larger of the two mints.
	pub mint_1: Pubkey,
	/// The pool's vault for `mint_0`: `[b"pool_vault", pool, mint_0]`.
	pub vault_0: Pubkey,
	/// The pool's vault for `mint_1`.
	pub vault_1: Pubkey,
}

/// The pool a launch of `base_mint` raising `quote_mint` graduates into,
/// under the configuration's Pina AMM tier `amm_config`.
#[must_use]
pub fn graduation_pool(
	amm_config: &Pubkey,
	base_mint: &Pubkey,
	quote_mint: &Pubkey,
) -> GraduationPool {
	let (mint_0, mint_1) = if base_mint < quote_mint {
		(*base_mint, *quote_mint)
	} else {
		(*quote_mint, *base_mint)
	};
	let pool = Pubkey::find_program_address(
		&[
			b"pool",
			amm_config.as_ref(),
			mint_0.as_ref(),
			mint_1.as_ref(),
		],
		&PINA_AMM_ID,
	)
	.0;
	let vault = |mint: &Pubkey| {
		Pubkey::find_program_address(&[b"pool_vault", pool.as_ref(), mint.as_ref()], &PINA_AMM_ID).0
	};
	GraduationPool {
		lp_mint: Pubkey::find_program_address(&[b"pool_lp_mint", pool.as_ref()], &PINA_AMM_ID).0,
		vault_0: vault(&mint_0),
		vault_1: vault(&mint_1),
		pool,
		mint_0,
		mint_1,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn the_amm_authority_matches_the_documented_address() {
		assert_eq!(
			amm_authority(),
			Pubkey::from_str_const("ALk5JUXnbaYwVeHG4ymAVVfPKW1xaUTCrSyrvKiq7CGZ")
		);
	}

	#[test]
	fn graduation_pools_sort_their_mints() {
		let tier = Pubkey::new_unique();
		let (base, quote) = (Pubkey::new_unique(), Pubkey::new_unique());
		let pool = graduation_pool(&tier, &base, &quote);
		assert_eq!(pool, graduation_pool(&tier, &quote, &base));
		assert!(pool.mint_0 < pool.mint_1);
		assert_ne!(pool.vault_0, pool.vault_1);
	}

	#[test]
	fn vaults_differ_per_mint() {
		let launch = Pubkey::new_unique();
		assert_ne!(
			launch_vault(&launch, &Pubkey::new_unique()),
			launch_vault(&launch, &Pubkey::new_unique())
		);
	}
}
