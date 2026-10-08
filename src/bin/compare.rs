use omwei_iot::{encode, encode_128, json_observation, SemanticAtom, AMBIENT_TEMPERATURE_ID};
use std::time::Instant;

fn main() {
    let atom = SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500).unwrap();
    let json = json_observation();

    let json_bytes = serde_json::to_vec(&json).unwrap();
    let cbor_bytes = serde_cbor::to_vec(&json).unwrap();
    let omwei_bytes = encode(atom);
    let omwei_128 = encode_128(atom);

    println!("Semantic payload: ambient temperature = 22.500 °C");
    println!();
    println!("Representation       Bytes");
    println!("JSON                 {}", json_bytes.len());
    println!("CBOR                 {}", cbor_bytes.len());
    println!("OMWEI logical        {}", omwei_bytes.len());
    println!("OMWEI 128-bit        {}", omwei_128.len());

    const N: usize = 1_000_000;

    // Encoding benchmark: construct the same semantic observation repeatedly.
    let start = Instant::now();
    let mut json_encoded = 0usize;
    for _ in 0..N { json_encoded += serde_json::to_vec(&json).unwrap().len(); }
    let json_encode_elapsed = start.elapsed();

    let start = Instant::now();
    let mut cbor_encoded = 0usize;
    for _ in 0..N { cbor_encoded += serde_cbor::to_vec(&json).unwrap().len(); }
    let cbor_encode_elapsed = start.elapsed();

    let start = Instant::now();
    let mut omwei_encoded = 0usize;
    for _ in 0..N { omwei_encoded += encode(atom).len(); }
    let omwei_encode_elapsed = start.elapsed();

    let start = Instant::now();
    let mut json_sum = 0usize;
    for _ in 0..N {
        let v: omwei_iot::JsonObservation = serde_json::from_slice(&json_bytes).unwrap();
        json_sum += v.value as usize;
    }
    let json_elapsed = start.elapsed();

    let start = Instant::now();
    let mut cbor_sum = 0usize;
    for _ in 0..N {
        let v: omwei_iot::JsonObservation = serde_cbor::from_slice(&cbor_bytes).unwrap();
        cbor_sum += v.value as usize;
    }
    let cbor_elapsed = start.elapsed();

    let start = Instant::now();
    let mut omwei_sum = 0i64;
    for _ in 0..N {
        let v = omwei_iot::decode(&omwei_bytes).unwrap();
        omwei_sum += v.canonical_value as i64;
    }
    let omwei_elapsed = start.elapsed();

    println!();
    println!("Decode benchmark: {} iterations", N);
    println!("JSON       {:?}", json_elapsed);
    println!("CBOR       {:?}", cbor_elapsed);
    println!("OMWEI      {:?}", omwei_elapsed);
    println!("Checksums  {} / {} / {}", json_sum, cbor_sum, omwei_sum);
    println!();
    println!("Encode benchmark: {} iterations", N);
    println!("JSON       {:?} ({} bytes total)", json_encode_elapsed, json_encoded);
    println!("CBOR       {:?} ({} bytes total)", cbor_encode_elapsed, cbor_encoded);
    println!("OMWEI      {:?} ({} bytes total)", omwei_encode_elapsed, omwei_encoded);
}
