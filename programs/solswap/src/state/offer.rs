use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Offer {
    /// Client-chosen id; lets one depositor keep several offers open at once.
    pub id: u64,
    pub depositor: Pubkey,
    /// Mint escrowed in the vault (what the taker receives).
    pub token_mint_in: Pubkey,
    /// Mint the depositor wants back (what the taker pays).
    pub token_mint_out: Pubkey,
    pub deposited_amount: u64,
    pub created_at: i64,
    pub bump: u8,
}
