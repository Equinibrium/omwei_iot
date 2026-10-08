use omwei_iot::mapping;

#[test]
fn unknown_mapping_is_not_interoperable() {
    assert!(mapping::map_canonical_value(0xFFFF, 123).is_err());
}

#[test]
fn mapping_resolves_semantics_from_shared_registry() {
    let descriptor = mapping::resolve_descriptor(0x0042).unwrap();

    assert_eq!(descriptor.label, "ambient_temperature");
    assert_eq!(descriptor.unit, "degree_Celsius");
    assert_eq!(descriptor.scale, 1000);
}
