use anchor_lang::prelude::*;

#[derive(Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, InitSpace)]
pub enum Status {
    Active,
    Completed,
    Closed,
}

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub mint_deposit: Pubkey,
    pub mint_receive: Pubkey,
    pub amount_deposit: u64,
    pub depositor_address: Pubkey,
    pub vault: Pubkey,
    pub bump: u8,
    pub status: Status,
}
