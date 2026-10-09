use serde::{Deserialize, Serialize};

pub mod mapping;
pub mod registry;
pub mod registry_validation;

pub const AMBIENT_TEMPERATURE_ID: u16 = 0x0042;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Descriptor {
    pub id: u16,
    pub label: &'static str,
    pub unit: &'static str,
    pub scale: i32,
    pub offset: i32,
    pub min: i32,
    pub max: i32,
}

pub const AMBIENT_TEMPERATURE: Descriptor = Descriptor {
    id: AMBIENT_TEMPERATURE_ID,
    label: "ambient_temperature",
    unit: "degree_Celsius",
    scale: 1000,
    offset: 0,
    min: -80_000,
    max: 150_000,
};

pub fn descriptor(id: u16) -> Option<&'static Descriptor> {
    registry::CORPUS_V0_1.resolve(id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticAtom {
    pub descriptor_id: u16,
    pub canonical_value: i32,
}

impl SemanticAtom {
    pub fn new(descriptor_id: u16, canonical_value: i32) -> Result<Self, &'static str> {
        let d = descriptor(descriptor_id).ok_or("unknown descriptor")?;
        if !(d.min..=d.max).contains(&canonical_value) {
            return Err("value outside descriptor constraints");
        }
        Ok(Self { descriptor_id, canonical_value })
    }
}

pub fn encode(atom: SemanticAtom) -> [u8; 6] {
    let mut out = [0u8; 6];
    out[..2].copy_from_slice(&atom.descriptor_id.to_be_bytes());
    out[2..].copy_from_slice(&atom.canonical_value.to_be_bytes());
    out
}

pub fn decode(bytes: &[u8]) -> Result<SemanticAtom, &'static str> {
    if bytes.len() != 6 { return Err("invalid OMWEI atom length"); }
    let descriptor_id = u16::from_be_bytes([bytes[0], bytes[1]]);
    let canonical_value = i32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
    SemanticAtom::new(descriptor_id, canonical_value)
}

pub fn encode_128(atom: SemanticAtom) -> [u8; 16] {
    let mut out = [0u8; 16];
    out[..6].copy_from_slice(&encode(atom));
    out
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JsonObservation {
    pub semantic: String,
    pub value: f64,
    pub unit: String,
}

pub fn json_observation() -> JsonObservation {
    JsonObservation {
        semantic: "ambient_temperature".into(),
        value: 22.5,
        unit: "degree_Celsius".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_resolves_temperature() {
        let d = descriptor(AMBIENT_TEMPERATURE_ID).unwrap();
        assert_eq!(d.label, "ambient_temperature");
        assert_eq!(d.unit, "degree_Celsius");
    }
    #[test]
    fn canonical_temperature_round_trip() {
        let atom = SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500).unwrap();
        assert_eq!(decode(&encode(atom)).unwrap(), atom);
    }
    #[test]
    fn deterministic_big_endian_encoding() {
        let atom = SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500).unwrap();
        assert_eq!(encode(atom), [0x00, 0x42, 0x00, 0x00, 0x57, 0xE4]);
    }
    #[test]
    fn historical_128_bit_target_is_16_bytes() {
        let atom = SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500).unwrap();
        assert_eq!(encode_128(atom).len(), 16);
        assert_eq!(&encode_128(atom)[..6], &encode(atom));
    }
    #[test]
    fn unknown_descriptor_is_rejected() {
        assert!(SemanticAtom::new(0xFFFF, 1).is_err());
    }
    #[test]
    fn json_fixture_has_same_semantic_value() {
        let j = json_observation();
        assert_eq!(j.semantic, "ambient_temperature");
        assert_eq!(j.value, 22.5);
        assert_eq!(j.unit, "degree_Celsius");
    }
}
