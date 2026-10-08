use omwei_iot::{encode, mapping, SemanticAtom, AMBIENT_TEMPERATURE_ID};

#[derive(Debug)]
struct VendorA {
    t_amb_milli_c: i32,
}

#[derive(Debug)]
struct VendorB {
    ambient_temp_c: f64,
}

fn vendor_a_to_omwei(v: VendorA) -> SemanticAtom {
    mapping::map_canonical_value(AMBIENT_TEMPERATURE_ID, v.t_amb_milli_c).unwrap()
}

fn vendor_b_to_omwei(v: VendorB) -> SemanticAtom {
    let milli_c = (v.ambient_temp_c * 1000.0).round() as i32;
    mapping::map_canonical_value(AMBIENT_TEMPERATURE_ID, milli_c).unwrap()
}

fn canonicalize_fahrenheit_to_celsius(f: f64) -> i32 {
    (((f - 32.0) * 5.0 / 9.0) * 1000.0).round() as i32
}

fn main() {
    let a = VendorA { t_amb_milli_c: 22_500 };
    let b = VendorB { ambient_temp_c: 22.5 };
    let b_fahrenheit = canonicalize_fahrenheit_to_celsius(72.5);

    let atom_a = vendor_a_to_omwei(a);
    let atom_b = vendor_b_to_omwei(b);

    println!("Vendor A semantic input: T_AMB = 22500 milli-Celsius");
    println!("Vendor B semantic input: ambientTemp = 22.5 Celsius");
    println!();
    println!("Vendor B normalized from 72.5 F: {} milli-Celsius", b_fahrenheit);
    println!();
    println!("OMWEI A: {:?}", atom_a);
    println!("OMWEI B: {:?}", atom_b);
    println!("Semantic equality: {}", atom_a == atom_b);
    println!("Wire equality:     {}", encode(atom_a) == encode(atom_b));

    assert_eq!(atom_a, atom_b);
    assert_eq!(encode(atom_a), encode(atom_b));
    assert_eq!(b_fahrenheit, 22_500);
}
