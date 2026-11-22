use anchor_lang::prelude::*;
use chainlink_solana::v2::read_feed_v2;

use crate::error::ErrorCode;

pub struct OraclePrice {
    pub price: u128,
    pub decimals: u8,
    pub timestamp: u32,
}

/// Reads the latest Chainlink round and enforces freshness and a positive answer.
/// The SDK validates the account owner (Chainlink store program) and discriminator.
pub fn read_fresh_price(feed: &AccountInfo, max_age_secs: u32, now: i64) -> Result<OraclePrice> {
    let data = feed.try_borrow_data()?;
    let parsed = read_feed_v2(data, feed.owner.to_bytes()).map_err(|error| {
        msg!("Chainlink feed read failed: {}", error);
        ErrorCode::InvalidPriceFeed
    })?;
    let round = parsed.latest_round_data().ok_or(ErrorCode::InvalidPriceFeed)?;

    let age = now
        .checked_sub(i64::from(round.timestamp))
        .ok_or(ErrorCode::MathOverflow)?;
    require!(age <= i64::from(max_age_secs), ErrorCode::StalePrice);
    require!(round.answer > 0, ErrorCode::InvalidPrice);

    Ok(OraclePrice {
        price: round.answer as u128,
        decimals: parsed.decimals(),
        timestamp: round.timestamp,
    })
}
