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
