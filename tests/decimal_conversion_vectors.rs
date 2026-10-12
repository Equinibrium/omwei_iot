use omwei_iot::decimal_conversion::{from_decimal, from_decimal_with_report};
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
    input: String,
    encoded_value: i32,
}

#[derive(Debug, Deserialize)]
struct InvalidVector {
    name: String,
    descriptor_id: String,
    input: String,
    reason: String,
}

fn parse_id(text: &str) -> u16 {
    u16::from_str_radix(text.strip_prefix("0x").unwrap_or(text), 16)
        .expect("vector descriptor ID must be hexadecimal")
}

fn vectors() -> VectorSet {
    serde_json::from_str(include_str!("vectors/decimal-conversion-v1.json"))
        .expect("decimal conversion vector file must be valid JSON")
}

#[test]
fn decimal_vectors_produce_exact_encoded_integers() {
    let set = vectors();
    assert_eq!(set.vector_set, "omwei-iot-decimal-conversion-v1");
    assert_eq!(set.status, "draft");
    assert_eq!(set.valid.len(), 13);

    for vector in set.valid {
        let atom = from_decimal(parse_id(&vector.descriptor_id), &vector.input)
            .unwrap_or_else(|e| panic!("{}: {e}", vector.name));
        assert_eq!(atom.canonical_value, vector.encoded_value, "{}", vector.name);
    }
}

#[test]
fn invalid_decimal_vectors_fail_closed() {
    let set = vectors();
    assert_eq!(set.invalid.len(), 16);

    for vector in set.invalid {
        let result = from_decimal(parse_id(&vector.descriptor_id), &vector.input);
        assert!(result.is_err(), "{} must be rejected ({})", vector.name, vector.reason);
    }
}


#[test]
fn rounding_report_matches_quantization_remainder() {
    let exact = from_decimal_with_report(0x0042, "22.5").unwrap();
    assert_eq!(exact.atom.canonical_value, 22_500);
    assert!(!exact.rounded);

    // A non-zero remainder is reported even when rounding leaves the integer unchanged.
    let rounded_down = from_decimal_with_report(0x0042, "22.5004").unwrap();
    assert_eq!(rounded_down.atom.canonical_value, 22_500);
    assert!(rounded_down.rounded);

    let rounded_up = from_decimal_with_report(0x0042, "22.5005").unwrap();
    assert_eq!(rounded_up.atom.canonical_value, 22_501);
    assert!(rounded_up.rounded);

    let negative = from_decimal_with_report(0x0042, "-22.5004").unwrap();
    assert_eq!(negative.atom.canonical_value, -22_500);
    assert!(negative.rounded);

    // Existing API remains a compatibility convenience and yields the same atom.
    assert_eq!(from_decimal(0x0042, "22.5004").unwrap(), rounded_down.atom);
}
