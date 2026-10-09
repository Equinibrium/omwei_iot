//! Deterministic decimal-string conversion for OMWEI's encoded integer domain.
//!
//! No binary floating-point arithmetic is used. Decimal input must use plain
//! base-10 notation (optional leading sign, optional decimal point; no exponent).
//! Values are rounded to nearest integer after applying the descriptor's integer
//! scale multiplier. Exact halfway cases round away from zero.

use crate::{descriptor, SemanticAtom};

fn parse_decimal(input: &str) -> Result<(i128, i128), &'static str> {
    if input.is_empty() {
        return Err("empty decimal value");
    }

    let (negative, unsigned) = match input.as_bytes()[0] {
        b'-' => (true, &input[1..]),
        b'+' => (false, &input[1..]),
        _ => (false, input),
    };
    if unsigned.is_empty() {
        return Err("decimal value has no digits");
    }

    let mut parts = unsigned.split('.');
    let whole = parts.next().ok_or("invalid decimal value")?;
    let fraction = parts.next();
    if parts.next().is_some() || whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit()) {
        return Err("decimal must use plain base-10 notation");
    }
    let fraction = fraction.unwrap_or("");
    if fraction.contains(|c: char| !c.is_ascii_digit()) {
        return Err("decimal must use plain base-10 notation");
    }
    if unsigned.contains('.') && fraction.is_empty() {
        return Err("decimal point must be followed by digits");
    }
    // Bound the textual precision and prevent unbounded work / denominator growth.
    if fraction.len() > 18 {
        return Err("decimal precision exceeds 18 fractional digits");
    }

    let whole_value: i128 = whole.parse().map_err(|_| "decimal value is too large")?;
    let denominator = 10_i128
        .checked_pow(fraction.len() as u32)
        .ok_or("decimal precision overflow")?;
    let fraction_value: i128 = if fraction.is_empty() {
        0
    } else {
        fraction.parse().map_err(|_| "decimal value is too large")?
    };
    let mut numerator = whole_value
        .checked_mul(denominator)
        .and_then(|v| v.checked_add(fraction_value))
        .ok_or("decimal value is too large")?;
    if negative {
        numerator = numerator.checked_neg().ok_or("decimal value is too large")?;
    }
    Ok((numerator, denominator))
}

/// Convert a plain decimal string in the descriptor's canonical physical unit
/// into a validated atom. Scale is interpreted as an integer multiplier and
/// offset as a physical-unit offset: encoded = round((value - offset) * scale).
///
/// Rounding is nearest, with exact ties away from zero. Out-of-range results,
/// unknown descriptors, malformed decimals, and arithmetic overflow are rejected.
pub fn from_decimal(descriptor_id: u16, input: &str) -> Result<SemanticAtom, &'static str> {
    let d = descriptor(descriptor_id).ok_or("unknown descriptor")?;
    if d.scale <= 0 {
        return Err("descriptor scale multiplier must be positive");
    }

    let (numerator, denominator) = parse_decimal(input)?;
    let offset_numerator = (d.offset as i128)
        .checked_mul(denominator)
        .ok_or("decimal arithmetic overflow")?;
    let shifted = numerator
        .checked_sub(offset_numerator)
        .ok_or("decimal arithmetic overflow")?;
    let scaled = shifted
        .checked_mul(d.scale as i128)
        .ok_or("decimal arithmetic overflow")?;

    let magnitude = scaled.checked_abs().ok_or("decimal arithmetic overflow")?;
    let mut rounded = magnitude / denominator;
    let remainder = magnitude % denominator;
    if remainder.checked_mul(2).ok_or("decimal arithmetic overflow")? >= denominator {
        rounded = rounded.checked_add(1).ok_or("decimal arithmetic overflow")?;
    }
    let signed = if scaled < 0 {
        rounded.checked_neg().ok_or("decimal arithmetic overflow")?
    } else {
        rounded
    };
    let encoded = i32::try_from(signed).map_err(|_| "encoded value overflows int32")?;
    SemanticAtom::new(descriptor_id, encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AMBIENT_TEMPERATURE_ID;

    #[test]
    fn decimal_conversion_uses_exact_base_ten() {
        assert_eq!(from_decimal(AMBIENT_TEMPERATURE_ID, "22.500").unwrap().canonical_value, 22_500);
        assert_eq!(from_decimal(AMBIENT_TEMPERATURE_ID, "22.5").unwrap().canonical_value, 22_500);
    }

    #[test]
    fn ties_round_away_from_zero() {
        assert_eq!(from_decimal(AMBIENT_TEMPERATURE_ID, "0.0005").unwrap().canonical_value, 1);
        assert_eq!(from_decimal(AMBIENT_TEMPERATURE_ID, "-0.0005").unwrap().canonical_value, -1);
        assert_eq!(from_decimal(AMBIENT_TEMPERATURE_ID, "22.5005").unwrap().canonical_value, 22_501);
        assert_eq!(from_decimal(AMBIENT_TEMPERATURE_ID, "-22.5005").unwrap().canonical_value, -22_501);
    }

    #[test]
    fn encoded_range_is_checked_after_rounding() {
        assert_eq!(from_decimal(AMBIENT_TEMPERATURE_ID, "-80.000").unwrap().canonical_value, -80_000);
        assert_eq!(from_decimal(AMBIENT_TEMPERATURE_ID, "150.000").unwrap().canonical_value, 150_000);
        assert!(from_decimal(AMBIENT_TEMPERATURE_ID, "-80.0005").is_err());
        assert!(from_decimal(AMBIENT_TEMPERATURE_ID, "150.0005").is_err());
    }

    #[test]
    fn malformed_and_excess_precision_values_fail_closed() {
        for value in ["", "+", "-", ".5", "1.", "1e2", "1.2.3", "1.0000000000000000001"] {
            assert!(from_decimal(AMBIENT_TEMPERATURE_ID, value).is_err(), "{value:?}");
        }
        assert!(from_decimal(0xffff, "1").is_err());
    }
}
