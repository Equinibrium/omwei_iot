use omwei_iot::{decode, descriptor, encode, SemanticAtom};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct VectorSet {
    vector_set: String,
    status: String,
    valid: Vec<ValidVector>,
    invalid: Vec<InvalidVector>,
}

#[derive(Debug, Deserialize)]
struct ValidVector {
    name: String,
    descriptor_id: String,
    label: String,
    unit: String,
    encoded_value: i32,
    bytes_hex: String,
}

#[derive(Debug, Deserialize)]
struct InvalidVector {
    name: String,
    descriptor_id: String,
    encoded_value: i32,
    reason: String,
}

fn parse_id(text: &str) -> u16 {
    u16::from_str_radix(text.strip_prefix("0x").unwrap_or(text), 16)
        .expect("vector descriptor ID must be hexadecimal")
}

fn parse_hex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0, "hex byte string must have even length");
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("valid hex byte"))
        .collect()
}

fn vectors() -> VectorSet {
    serde_json::from_str(include_str!("vectors/atom-v1.json"))
        .expect("conformance vector file must be valid JSON")
}

#[test]
fn fixed_vectors_match_registry_and_wire_bytes() {
    let set = vectors();
    assert_eq!(set.vector_set, "omwei-iot-atom-v1");
    assert_eq!(set.status, "draft");
    assert_eq!(set.valid.len(), 6);

    for vector in set.valid {
        let id = parse_id(&vector.descriptor_id);
        let descriptor = descriptor(id).unwrap_or_else(|| panic!("{}: descriptor missing", vector.name));
        assert_eq!(descriptor.label, vector.label, "{}: label", vector.name);
        assert_eq!(descriptor.unit, vector.unit, "{}: unit", vector.name);

        let atom = SemanticAtom::new(id, vector.encoded_value)
            .unwrap_or_else(|e| panic!("{}: {e}", vector.name));
        let expected = parse_hex(&vector.bytes_hex);
        assert_eq!(expected.len(), 6, "{}: vector must contain six bytes", vector.name);
        assert_eq!(encode(atom).as_slice(), expected.as_slice(), "{}: encoded bytes", vector.name);
        assert_eq!(decode(&expected).unwrap(), atom, "{}: decode round trip", vector.name);
    }
}

#[test]
fn invalid_vectors_are_rejected() {
    let set = vectors();
    assert_eq!(set.invalid.len(), 5);

    for vector in set.invalid {
        let id = parse_id(&vector.descriptor_id);
        let result = SemanticAtom::new(id, vector.encoded_value);
        assert!(result.is_err(), "{} must be rejected", vector.name);
        match vector.reason.as_str() {
            "unknown_descriptor" => assert!(descriptor(id).is_none(), "{}: expected unknown ID", vector.name),
            "value_out_of_range" => assert!(descriptor(id).is_some(), "{}: expected known ID", vector.name),
            other => panic!("{}: unrecognized expected failure reason {other}", vector.name),
        }
    }
}

#[test]
fn malformed_atom_lengths_are_rejected() {
    assert!(decode(&[]).is_err());
    assert!(decode(&[0x00, 0x42, 0x00, 0x00, 0x57]).is_err());
    assert!(decode(&[0x00, 0x42, 0x00, 0x00, 0x57, 0xE4, 0x00]).is_err());
}
