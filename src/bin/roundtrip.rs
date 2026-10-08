use omwei_iot::{decode, encode, SemanticAtom, AMBIENT_TEMPERATURE_ID};

fn main() {
    let source = SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500).unwrap();
    let payload = encode(source);
    let received = decode(&payload).unwrap();

    println!("OMWEI wire round-trip");
    println!("  payload bytes: {:02X?}", payload);
    println!("  payload size:  {} bytes", payload.len());
    println!("  decoded ID:    0x{:04X}", received.descriptor_id);
    println!("  decoded value: {}", received.canonical_value);
    println!("  round-trip:    {}", source == received);

    assert_eq!(source, received);
}
