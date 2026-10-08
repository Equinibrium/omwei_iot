use omwei_iot::{decode, encode, SemanticAtom, AMBIENT_TEMPERATURE_ID};

fn atom() -> SemanticAtom {
    SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500).unwrap()
}

// Minimal transport envelopes. Their metadata is deliberately outside
// the OMWEI semantic atom.
fn mqtt_payload(atom: SemanticAtom) -> Vec<u8> {
    let mut frame = b"topic=sensors/temp;".to_vec();
    frame.extend_from_slice(&encode(atom));
    frame
}

fn zenoh_payload(atom: SemanticAtom) -> Vec<u8> {
    let mut frame = b"key=sensors/temp;".to_vec();
    frame.extend_from_slice(&encode(atom));
    frame
}

fn can_payload(atom: SemanticAtom) -> Vec<u8> {
    let mut frame = vec![0x18, 0xFF, 0x50, 0xE5]; // simulated CAN identifier
    frame.extend_from_slice(&encode(atom));
    frame
}

fn extract_atom(frame: &[u8], prefix_len: usize) -> SemanticAtom {
    decode(&frame[prefix_len..]).unwrap()
}

#[test]
fn same_atom_survives_mqtt_zenoh_and_can_bindings() {
    let expected = atom();

    let mqtt = mqtt_payload(expected);
    let zenoh = zenoh_payload(expected);
    let can = can_payload(expected);

    assert_eq!(extract_atom(&mqtt, b"topic=sensors/temp;".len()), expected);
    assert_eq!(extract_atom(&zenoh, b"key=sensors/temp;".len()), expected);
    assert_eq!(extract_atom(&can, 4), expected);
}

#[test]
fn transport_metadata_is_not_part_of_semantic_atom() {
    let bytes = encode(atom());

    let mqtt = mqtt_payload(atom());
    let zenoh = zenoh_payload(atom());
    let can = can_payload(atom());

    assert_eq!(&mqtt[mqtt.len() - bytes.len()..], &bytes);
    assert_eq!(&zenoh[zenoh.len() - bytes.len()..], &bytes);
    assert_eq!(&can[can.len() - bytes.len()..], &bytes);

    assert_ne!(&mqtt[..mqtt.len() - bytes.len()], &zenoh[..zenoh.len() - bytes.len()]);
    assert_ne!(&zenoh[..zenoh.len() - bytes.len()], &can[..can.len() - bytes.len()]);
}

#[test]
fn changing_atom_bytes_changes_decoded_semantics() {
    let original = atom();

    let mut mqtt = mqtt_payload(original);
    let last = mqtt.len() - 1;
    mqtt[last] ^= 0x01;

    let changed = decode(&mqtt[mqtt.len() - 6..]).unwrap();
    assert_ne!(changed, original);
}
