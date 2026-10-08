use omwei_iot::{decode, encode, descriptor, SemanticAtom};

#[test]
fn corpus_defines_multiple_shared_semantics() {
    let cases = [
        (0x0042, "ambient_temperature", "degree_Celsius", 22_500i32),
        (0x0043, "relative_humidity", "percent", 55_500i32),
        (0x0045, "active_power", "watt", 1_250i32),
    ];

    for (id, label, unit, value) in cases {
        let d = descriptor(id).expect("corpus descriptor must exist");
        assert_eq!(d.id, id);
        assert_eq!(d.label, label);
        assert_eq!(d.unit, unit);

        let atom = SemanticAtom::new(id, value).unwrap();
        assert_eq!(decode(&encode(atom)).unwrap(), atom);
    }
}

#[test]
fn different_vendor_models_can_converge_on_each_corpus_semantic() {
    // Vendor A uses canonical integer representations.
    let temperature_a = SemanticAtom::new(0x0042, 22_500).unwrap();
    let humidity_a = SemanticAtom::new(0x0043, 55_500).unwrap();
    let power_a = SemanticAtom::new(0x0045, 1_250).unwrap();

    // Vendor B uses ordinary engineering-unit values.
    let temperature_b = SemanticAtom::new(0x0042, (22.5_f64 * 1000.0) as i32).unwrap();
    let humidity_b = SemanticAtom::new(0x0043, (55.5_f64 * 1000.0) as i32).unwrap();
    let power_b = SemanticAtom::new(0x0045, 1250).unwrap();

    assert_eq!(temperature_a, temperature_b);
    assert_eq!(humidity_a, humidity_b);
    assert_eq!(power_a, power_b);

    assert_eq!(encode(temperature_a), encode(temperature_b));
    assert_eq!(encode(humidity_a), encode(humidity_b));
    assert_eq!(encode(power_a), encode(power_b));
}

#[test]
fn unknown_descriptor_cannot_enter_interoperable_data_plane() {
    assert!(SemanticAtom::new(0xFFFF, 123).is_err());
    assert!(descriptor(0xFFFF).is_none());
}
