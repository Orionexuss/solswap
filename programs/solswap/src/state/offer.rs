use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub token_mint_in: Pubkey,
    pub token_mint_out: Pubkey,
    pub amount_deposit: u64,
    pub depositor: Pubkey,
    pub vault: Pubkey,
    pub bump: u8,
}
