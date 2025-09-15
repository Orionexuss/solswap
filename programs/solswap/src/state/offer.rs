use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub mint_deposit: Pubkey,
    pub mint_receive: Pubkey,
    pub amount_deposit: u64,
    pub depositor_address: Pubkey,
    pub vault: Pubkey,
    pub bump: u8,
}
