//! Constant-product AMM pricing in integer token base units.
//!
//! Fees are whole basis points: 30 means 0.30%. The caller orders the
//! reserves according to the requested direction of the trade.
use num_bigint::BigUint;
use std::fmt;

const BPS_SCALE: u32 = 10_000;

/// Expected reasons a pricing request can fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteError {
    ZeroAmount,
    InsufficientLiquidity,
    InvalidFee,
    AmountOverflow,
}

impl fmt::Display for QuoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ZeroAmount => "the requested amount must be positive",
            Self::InsufficientLiquidity => "the pool has insufficient liquidity",
            Self::InvalidFee => "the fee must be less than 10,000 basis points",
            Self::AmountOverflow => "the calculated amount does not fit in u128",
        };
        f.write_str(message)
    }
}

impl std::error::Error for QuoteError {}

/// Calculate output for an exact input, rounding down.
///
/// All amounts are integer base units, not human-readable token quantities.
/// A valid request may return zero when the trade is too small; quote
/// assembly will decide whether to decline that quote.
pub fn get_amount_out(
    amount_in: u128,
    reserve_in: u128,
    reserve_out: u128,
    fee_bps: u16,
) -> Result<u128, QuoteError> {
    if amount_in == 0 {
        return Err(QuoteError::ZeroAmount);
    }
    if reserve_in == 0 || reserve_out == 0 {
        return Err(QuoteError::InsufficientLiquidity);
    }
    let fee_multiplier = fee_multiplier(fee_bps)?;
    let amount_with_fee = BigUint::from(amount_in) * fee_multiplier;
    let numerator = &amount_with_fee * BigUint::from(reserve_out);
    let denominator = BigUint::from(reserve_in) * BPS_SCALE + amount_with_fee;
    // Unsigned integer division discards the remainder: floor(n / d).
    u128::try_from(numerator / denominator).map_err(|_| QuoteError::AmountOverflow)
}

/// Calculate the input needed for an exact output, rounding up.
///
/// Rejects requests that would consume all the output reserve. The computed
/// input can exceed u128 even when every argument fits, so conversion is checked.
pub fn get_amount_in(
    amount_out: u128,
    reserve_in: u128,
    reserve_out: u128,
    fee_bps: u16,
) -> Result<u128, QuoteError> {
    if amount_out == 0 {
        return Err(QuoteError::ZeroAmount);
    }
    if reserve_in == 0 || amount_out >= reserve_out {
        return Err(QuoteError::InsufficientLiquidity);
    }
    let fee_multiplier = fee_multiplier(fee_bps)?;
    let numerator = BigUint::from(reserve_in) * BigUint::from(amount_out) * BPS_SCALE;
    // The preceding liquidity check makes this subtraction safe.
    let denominator = BigUint::from(reserve_out - amount_out) * fee_multiplier;
    // For positive integers, ceil(n / d) = (n + d - 1) / d.
    // This also handles exact division without charging an extra unit.
    let rounded_up = (numerator + &denominator - 1u32) / denominator;
    u128::try_from(rounded_up).map_err(|_| QuoteError::AmountOverflow)
}

fn fee_multiplier(fee_bps: u16) -> Result<u32, QuoteError> {
    if u32::from(fee_bps) >= BPS_SCALE {
        return Err(QuoteError::InvalidFee);
    }
    Ok(BPS_SCALE - u32::from(fee_bps))
}
