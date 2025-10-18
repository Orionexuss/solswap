pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("3c9wj6bDT9opsUWPAPdGjdddv1GKF8R7yDpR9ZH7VpvX");

#[program]
pub mod solswap {

    use super::*;

    pub fn init_config(ctx: Context<InitConfig>, usdc_mint: Pubkey) -> Result<()> {
        crate::instructions::process_init_config(ctx, usdc_mint)
    }

    pub fn create_offer(ctx: Context<CreateOffer>, amount: u64) -> Result<()> {
        crate::instructions::process_create_offer(ctx, amount)
    }

    pub fn take_offer(ctx: Context<TakeOffer>) -> Result<()> {
        crate::instructions::process_take_offer(ctx)
    }
}
