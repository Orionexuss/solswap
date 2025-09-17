pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Bw51Xa4JoAiyhE2e8cQAmFNjB5F7pazMWjDxdwKL6Giv");

#[program]
pub mod solswap {

    use super::*;

    pub fn create_offer(ctx: Context<CreateOffer>, amount: u64) -> Result<()> {
        process_create_offer(ctx, amount)
    }

    pub fn take_offer(ctx: Context<TakeOffer>) -> Result<()> {
        process_take_offer(ctx)
    }
}
