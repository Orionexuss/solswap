use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Amount must be greater than zero")]
    AmountZero,

    #[msg("Token account mint does not match expected mint")]
    InvalidTokenIn,

    #[msg("Token account mint does not match expected mint")]
    InvalidTokenOut,

    #[msg("Token in and token out cannot be the same")]
    SameToken,
}
