use omwei_iot::{decode, encode, encode_128, json_observation, SemanticAtom, AMBIENT_TEMPERATURE_ID};
use std::fs;
use std::hint::black_box;
use std::time::{Duration, Instant};

const SIZES: &[usize] = &[1, 1_000, 100_000, 1_000_000];
const SAMPLES: usize = 7;
const WARMUP: usize = 10_000;

struct Row {
    op: &'static str,
    format: &'static str,
    n: usize,
    min: f64,
    median: f64,
    max: f64,
    baseline_median: f64,
    corrected_median: f64,
    bytes: usize,
}

fn ns_per_op(elapsed: Duration, n: usize) -> f64 {
    elapsed.as_secs_f64() * 1e9 / n as f64
}

fn measure<F>(n: usize, mut f: F) -> f64
where
    F: FnMut() -> usize,
{
    let start = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..n {
        checksum = checksum.wrapping_add(black_box(f()));
    }
    black_box(checksum);
    ns_per_op(start.elapsed(), n)
}

fn warmup<F>(mut f: F)
where
    F: FnMut() -> usize,
{
    for _ in 0..WARMUP {
        black_box(f());
    }
}

fn samples<F>(n: usize, mut f: F) -> Vec<f64>
where
    F: FnMut() -> usize,
{
    warmup(&mut f);
    (0..SAMPLES).map(|_| measure(n, &mut f)).collect()
}

fn baseline_samples(n: usize) -> Vec<f64> {
    samples(n, || 0)
}

fn summarize(
    rows: &mut Vec<Row>,
    op: &'static str,
    format: &'static str,
    n: usize,
    bytes: usize,
    mut values: Vec<f64>,
    baseline: &[f64],
) {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let corrected: Vec<f64> = values
        .iter()
        .zip(baseline.iter())
        .map(|(value, base)| value - base)
        .collect();
    let mut corrected_sorted = corrected;
    corrected_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    rows.push(Row {
        op,
        format,
        n,
        min: values[0],
        median: values[values.len() / 2],
        max: *values.last().unwrap(),
        baseline_median: {
            let mut b = baseline.to_vec();
            b.sort_by(|a, b| a.partial_cmp(b).unwrap());
            b[b.len() / 2]
        },
        corrected_median: corrected_sorted[corrected_sorted.len() / 2].max(0.0),
        bytes,
    });
}

fn main() {
    let atom = SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500).unwrap();
    let json = json_observation();
    let json_bytes = serde_json::to_vec(&json).unwrap();
    let cbor_bytes = serde_cbor::to_vec(&json).unwrap();
    let omwei_bytes = encode(atom);
    let omwei_128 = encode_128(atom);

    println!("OMWEI IoT benchmark");
    println!("Semantic payload: ambient temperature = 22.500 °C");
    println!();
    println!("Representation       Payload bytes");
    println!("JSON                 {}", json_bytes.len());
    println!("CBOR                 {}", cbor_bytes.len());
    println!("OMWEI logical        {}", omwei_bytes.len());
    println!("OMWEI 128-bit        {}", omwei_128.len());
    println!();
    println!("{} samples per batch, {} warm-up operations; raw and baseline-corrected median are reported.", SAMPLES, WARMUP);
    println!("Timings are measured directly; no energy claim is inferred.");
    println!();

    let mut rows = Vec::new();

    for &n in SIZES {
        let baseline = baseline_samples(n);
        summarize(&mut rows, "encode", "JSON", n, json_bytes.len(), samples(n, || {
            let bytes = serde_json::to_vec(black_box(&json)).unwrap();
            black_box(bytes.len())
        }), &baseline);
        summarize(&mut rows, "encode", "CBOR", n, cbor_bytes.len(), samples(n, || {
            let bytes = serde_cbor::to_vec(black_box(&json)).unwrap();
            black_box(bytes.len())
        }), &baseline);
        summarize(&mut rows, "encode", "OMWEI", n, omwei_bytes.len(), samples(n, || {
            let bytes = encode(black_box(atom));
            black_box(bytes[0] as usize)
        }), &baseline);
        summarize(&mut rows, "encode", "OMWEI-128", n, omwei_128.len(), samples(n, || {
            let bytes = encode_128(black_box(atom));
            black_box(bytes[0] as usize)
        }), &baseline);

        summarize(&mut rows, "decode", "JSON", n, json_bytes.len(), samples(n, || {
            let value: omwei_iot::JsonObservation =
                serde_json::from_slice(black_box(&json_bytes)).unwrap();
            black_box(value.value.to_bits() as usize)
        }), &baseline);
        summarize(&mut rows, "decode", "CBOR", n, cbor_bytes.len(), samples(n, || {
            let value: omwei_iot::JsonObservation =
                serde_cbor::from_slice(black_box(&cbor_bytes)).unwrap();
            black_box(value.value.to_bits() as usize)
        }), &baseline);
        summarize(&mut rows, "decode", "OMWEI", n, omwei_bytes.len(), samples(n, || {
            let value = decode(black_box(&omwei_bytes)).unwrap();
            black_box(value.canonical_value as usize)
        }), &baseline);
    }

    println!(
        "{:<7} {:<10} {:>10} {:>12} {:>14} {:>14} {:>14}",
        "Op", "Format", "N", "Base med.", "Raw med.", "Corrected med.", "Max ns/op"
    );
    println!("{}", "-".repeat(96));
    for row in &rows {
        println!(
            "{:<7} {:<10} {:>10} {:>12.2} {:>14.2} {:>14.2} {:>14.2}",
            row.op, row.format, row.n, row.baseline_median, row.median, row.corrected_median, row.max
        );
    }

    fs::create_dir_all("benchmark-results").unwrap();
    let mut csv = String::from(
        "operation,representation,observations,samples,min_ns_per_op,median_ns_per_op,max_ns_per_op,baseline_median_ns_per_op,corrected_median_ns_per_op,payload_bytes\n",
    );
    for row in &rows {
        csv.push_str(&format!(
            "{},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{}\n",
            row.op, row.format, row.n, SAMPLES, row.min, row.median, row.max, row.baseline_median, row.corrected_median, row.bytes
        ));
    }
    fs::write("benchmark-results/benchmark.csv", csv).unwrap();
    println!("Wrote benchmark-results/benchmark.csv");
}
