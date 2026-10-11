#![no_std]

// Compile-time consumer fixture: this crate itself has no std and depends on
// OMWEI with default features disabled. It intentionally performs no I/O.

use omwei_iot::{
    decode, encode, encode_128, mapping, SemanticAtom, AMBIENT_TEMPERATURE_ID,
};

pub fn temperature_round_trip() -> Result<[u8; 6], &'static str> {
    let atom = omwei_iot::decimal_conversion::from_decimal(AMBIENT_TEMPERATURE_ID, "22.5")?;
    let wire = encode(atom);
    let decoded = decode(&wire)?;
    if decoded != atom {
        return Err("round-trip mismatch");
    }
    Ok(wire)
}

pub fn resolve_static_descriptor() -> Result<&'static str, &'static str> {
    Ok(mapping::ambient_temperature_descriptor()?.label)
}

pub fn padded_wire_atom() -> Result<[u8; 16], &'static str> {
    let atom = SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500)?;
    Ok(encode_128(atom))
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    #[test]
    fn valid_atom_has_stable_wire_bytes() {
        assert_eq!(temperature_round_trip().unwrap(), [0x00, 0x42, 0x00, 0x00, 0x57, 0xE4]);
    }

    #[test]
    fn static_registry_is_available() {
        assert_eq!(resolve_static_descriptor().unwrap(), "ambient_temperature");
    }

    #[test]
    fn padded_wire_has_zero_padding() {
        let wire = padded_wire_atom().unwrap();
        assert_eq!(&wire[..6], &[0x00, 0x42, 0x00, 0x00, 0x57, 0xE4]);
        assert!(wire[6..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn decoder_rejects_wrong_lengths() {
        for length in [0, 1, 5, 7, 16] {
            assert!(decode(&vec![0u8; length]).is_err(), "length {length} must fail");
        }
    }

    #[test]
    fn decoder_rejects_unknown_descriptor_and_out_of_range_value() {
        assert!(decode(&[0xff, 0xff, 0, 0, 0, 1]).is_err());
        assert!(decode(&[0x00, 0x42, 0x00, 0x02, 0x49, 0xF1]).is_err());
    }

    #[test]
    fn atom_constructor_enforces_bounds() {
        assert!(SemanticAtom::new(AMBIENT_TEMPERATURE_ID, -80_001).is_err());
        assert!(SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 150_001).is_err());
        assert!(SemanticAtom::new(AMBIENT_TEMPERATURE_ID, -80_000).is_ok());
        assert!(SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 150_000).is_ok());
    }

    #[test]
    fn decimal_conversion_fails_closed() {
        use omwei_iot::decimal_conversion::from_decimal;
        for input in ["", "1e2", "1.", ".5", "150.001", "-80.001"] {
            assert!(from_decimal(AMBIENT_TEMPERATURE_ID, input).is_err(), "{input:?}");
        }
    }
}
