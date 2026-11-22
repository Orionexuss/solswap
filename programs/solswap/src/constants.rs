use anchor_lang::prelude::*;

/// Seed of the singleton [`crate::Config`] account.
#[constant]
pub const CONFIG_SEED: &[u8] = b"swap_config";

/// Seed prefix of every [`crate::Offer`] account: `["offer", depositor, offer_id_le]`.
#[constant]
pub const OFFER_SEED: &[u8] = b"offer";

/// Default freshness window for oracle prices, in seconds.
#[constant]
pub const DEFAULT_MAX_PRICE_AGE_SECS: u32 = 60;

/// Upper bound accepted by `init_config` / `update_config`, so a typo cannot disable the check.
pub const MAX_ALLOWED_PRICE_AGE_SECS: u32 = 3_600;

pub const WSOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
