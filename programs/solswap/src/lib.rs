pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("4RGdWYYuAVYWUH23MQxZomSRUYpstb5zTVbrSGocHYBn");

/// Oracle-priced escrow swaps between WSOL and USDC.
///
/// A depositor escrows one asset in a PDA-owned vault. Any taker can fill the whole offer by
/// paying the other asset at the current Chainlink SOL / USD price.
#[program]
pub mod solswap {
    use super::*;

    pub fn init_config(
        ctx: Context<InitConfig>,
        usdc_mint: Pubkey,
        price_feed: Pubkey,
        max_price_age_secs: u32,
    ) -> Result<()> {
        process_init_config(ctx, usdc_mint, price_feed, max_price_age_secs)
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        price_feed: Pubkey,
        max_price_age_secs: u32,
    ) -> Result<()> {
        process_update_config(ctx, price_feed, max_price_age_secs)
    }

    pub fn create_offer(ctx: Context<CreateOffer>, offer_id: u64, amount: u64) -> Result<()> {
        process_create_offer(ctx, offer_id, amount)
    }

    pub fn take_offer(ctx: Context<TakeOffer>, max_amount_out: u64) -> Result<()> {
        process_take_offer(ctx, max_amount_out)
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        process_cancel_offer(ctx)
    }
}
