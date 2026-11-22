use anchor_lang::prelude::*;

#[event]
pub struct OfferCreated {
    pub offer: Pubkey,
    pub depositor: Pubkey,
    pub token_mint_in: Pubkey,
    pub token_mint_out: Pubkey,
    pub amount: u64,
}

#[event]
pub struct OfferTaken {
    pub offer: Pubkey,
    pub taker: Pubkey,
    /// Tokens the taker received from the vault.
    pub amount_received: u64,
    /// Tokens the taker paid to the depositor.
    pub amount_paid: u64,
    /// Chainlink answer used for the conversion.
    pub price: i128,
    pub price_decimals: u8,
    pub price_timestamp: u32,
}

#[event]
pub struct OfferCancelled {
    pub offer: Pubkey,
    pub depositor: Pubkey,
    pub amount_returned: u64,
}
