use omwei_iot::{decode, encode, SemanticAtom, AMBIENT_TEMPERATURE_ID};

#[derive(Debug)]
struct VendorA { t_amb_milli_c: i32 }

#[derive(Debug)]
struct VendorB { ambient_temp_c: f64 }

#[derive(Debug)]
struct VendorC { air_temperature_f: f64 }

fn vendor_a_to_omwei(v: VendorA) -> SemanticAtom {
    SemanticAtom::new(AMBIENT_TEMPERATURE_ID, v.t_amb_milli_c).unwrap()
}

fn vendor_b_to_omwei(v: VendorB) -> SemanticAtom {
    let milli_c = (v.ambient_temp_c * 1000.0).round() as i32;
    SemanticAtom::new(AMBIENT_TEMPERATURE_ID, milli_c).unwrap()
}

fn vendor_c_to_omwei(v: VendorC) -> SemanticAtom {
    let milli_c = (((v.air_temperature_f - 32.0) * 5.0 / 9.0) * 1000.0).round() as i32;
    SemanticAtom::new(AMBIENT_TEMPERATURE_ID, milli_c).unwrap()
}

#[test]
fn heterogeneous_vendor_models_produce_same_semantic_atom() {
    let a = vendor_a_to_omwei(VendorA { t_amb_milli_c: 22_500 });
    let b = vendor_b_to_omwei(VendorB { ambient_temp_c: 22.5 });
    let c = vendor_c_to_omwei(VendorC { air_temperature_f: 72.5 });

    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(encode(a), encode(b));
    assert_eq!(encode(b), encode(c));
}

#[test]
fn canonical_omwei_value_has_registry_defined_meaning() {
    let atom = vendor_c_to_omwei(VendorC { air_temperature_f: 72.5 });

    assert_eq!(atom.descriptor_id, AMBIENT_TEMPERATURE_ID);
    assert_eq!(atom.canonical_value, 22_500);

    let decoded = decode(&encode(atom)).unwrap();
    let descriptor = omwei_iot::descriptor(decoded.descriptor_id).unwrap();

    assert_eq!(descriptor.label, "ambient_temperature");
    assert_eq!(descriptor.unit, "degree_Celsius");
    assert_eq!(decoded.canonical_value as f64 * 0.001, 22.5);
}

#[test]
fn vendor_field_names_do_not_enter_the_omwei_data_plane() {
    let a = vendor_a_to_omwei(VendorA { t_amb_milli_c: 22_500 });
    let b = vendor_b_to_omwei(VendorB { ambient_temp_c: 22.5 });

    let wire_a = encode(a);
    let wire_b = encode(b);

    assert_eq!(wire_a, [0x00, 0x42, 0x00, 0x00, 0x57, 0xE4]);
    assert_eq!(wire_a, wire_b);
    assert!(!wire_a.windows(4).any(|w| w == b"T_AMB"));
    assert!(!wire_a.windows(11).any(|w| w == b"ambientTemp"));
}
