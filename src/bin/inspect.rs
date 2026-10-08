use omwei_iot::{descriptor, encode, encode_128, json_observation, SemanticAtom, AMBIENT_TEMPERATURE_ID};

fn main() {
    let atom = SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500).unwrap();
    let d = descriptor(atom.descriptor_id).unwrap();

    println!("OMWEI semantic atom");
    println!("  descriptor: 0x{:04X} ({})", atom.descriptor_id, d.label);
    println!("  value:      {:.3} {}", atom.canonical_value as f64 / d.scale as f64, d.unit);
    println!("  logical:    {} bytes {:?}", encode(atom).len(), encode(atom));
    println!("  128-bit:    {} bytes", encode_128(atom).len());
    println!("  JSON:       {}", serde_json::to_string(&json_observation()).unwrap());
}
