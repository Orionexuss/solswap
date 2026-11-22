use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    /// Can update the config. Set to the program upgrade authority at initialization.
    pub admin: Pubkey,
    /// The only non-WSOL mint offers may use.
    pub usdc_mint: Pubkey,
    /// Chainlink SOL / USD data feed account.
    pub price_feed: Pubkey,
    /// Oracle rounds older than this are rejected by `take_offer`.
    pub max_price_age_secs: u32,
    pub bump: u8,
}
