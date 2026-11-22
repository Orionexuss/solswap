use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Amount must be greater than zero")]
    AmountZero,

    #[msg("Token in must be WSOL or the configured USDC mint")]
    InvalidTokenIn,

    #[msg("Token out must be WSOL or the configured USDC mint")]
    InvalidTokenOut,

    #[msg("Token in and token out cannot be the same")]
    SameToken,

    #[msg("Only the program upgrade authority can initialize the config")]
    Unauthorized,

    #[msg("Price feed account does not match the configured Chainlink feed")]
    InvalidPriceFeed,

    #[msg("Oracle price is older than the allowed maximum age")]
    StalePrice,

    #[msg("Oracle price must be positive")]
    InvalidPrice,

    #[msg("Maximum price age must be between 1 and 3600 seconds")]
    InvalidMaxPriceAge,

    #[msg("Required payment exceeds the taker's maximum")]
    SlippageExceeded,

    #[msg("Arithmetic overflow")]
    MathOverflow,
}
