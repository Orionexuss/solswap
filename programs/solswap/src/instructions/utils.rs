use anchor_lang::prelude::*;

use crate::error::ErrorCode;

fn pow10(exponent: u32) -> Result<u128> {
    10u128.checked_pow(exponent).ok_or(ErrorCode::MathOverflow.into())
}

/// `ceil(numerator / denominator)` in u128, returned as u64.
fn div_ceil_u64(numerator: u128, denominator: u128) -> Result<u64> {
    let quotient = numerator.div_ceil(denominator);
    u64::try_from(quotient).map_err(|_| ErrorCode::MathOverflow.into())
}

/// Lamports the taker must pay for `usdc_amount` base units.
/// `price` is USD per SOL with `price_decimals` decimals. Rounds up, in the depositor's favor.
pub fn usdc_to_lamports(
    usdc_amount: u64,
    price: u128,
    price_decimals: u8,
    usdc_decimals: u8,
    sol_decimals: u8,
) -> Result<u64> {
    require!(price > 0, ErrorCode::InvalidPrice);
    let numerator = (usdc_amount as u128)
        .checked_mul(pow10(u32::from(sol_decimals) + u32::from(price_decimals))?)
        .ok_or(ErrorCode::MathOverflow)?;
    let denominator = price
        .checked_mul(pow10(u32::from(usdc_decimals))?)
        .ok_or(ErrorCode::MathOverflow)?;
    div_ceil_u64(numerator, denominator)
}

/// USDC base units the taker must pay for `lamports`. Rounds up, in the depositor's favor.
pub fn lamports_to_usdc(
    lamports: u64,
    price: u128,
    price_decimals: u8,
    usdc_decimals: u8,
    sol_decimals: u8,
) -> Result<u64> {
    require!(price > 0, ErrorCode::InvalidPrice);
    let numerator = (lamports as u128)
        .checked_mul(price)
        .and_then(|value| value.checked_mul(pow10(u32::from(usdc_decimals)).ok()?))
        .ok_or(ErrorCode::MathOverflow)?;
    let denominator = pow10(u32::from(sol_decimals) + u32::from(price_decimals))?;
    div_ceil_u64(numerator, denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 70.00000000 USD per SOL, 8 decimals (Chainlink SOL / USD).
    const PRICE: u128 = 7_000_000_000;

    #[test]
    fn one_sol_costs_seventy_usdc() {
        assert_eq!(lamports_to_usdc(1_000_000_000, PRICE, 8, 6, 9).unwrap(), 70_000_000);
    }

    #[test]
    fn seventy_usdc_costs_one_sol() {
        assert_eq!(usdc_to_lamports(70_000_000, PRICE, 8, 6, 9).unwrap(), 1_000_000_000);
    }

    #[test]
    fn rounding_favors_the_depositor() {
        // 1 lamport is worth 0.00000007 USDC: the taker still pays 1 base unit.
        assert_eq!(lamports_to_usdc(1, PRICE, 8, 6, 9).unwrap(), 1);
        // 1 USDC base unit is worth 14.28 lamports: the taker pays 15.
        assert_eq!(usdc_to_lamports(1, PRICE, 8, 6, 9).unwrap(), 15);
    }

    #[test]
    fn rejects_non_positive_prices() {
        assert!(lamports_to_usdc(1, 0, 8, 6, 9).is_err());
        assert!(usdc_to_lamports(1, 0, 8, 6, 9).is_err());
    }

    #[test]
    fn large_amounts_do_not_overflow() {
        assert!(lamports_to_usdc(u64::MAX / 2, PRICE, 8, 6, 9).is_ok());
    }
}
