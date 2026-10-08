# OMWEI IoT Benchmark

## Purpose

The benchmark tests the original OMWEI engineering hypothesis:

> A shared semantic identity can make repeated IoT exchange both interoperable and substantially cheaper to decode than text-heavy representations.

The benchmark separates **semantic correctness** from **representation efficiency**.

## Controlled semantic payload

All formats represent the same observation:

`ambient temperature = 22.500 °C`

OMWEI uses descriptor `0x0042` with canonical value `22500`.

## Results

- **`latest.csv`** is the canonical result from the latest successful GitHub Actions benchmark.
- **`history/v1.csv`** preserves the original 5-sample benchmark.
- **`history/v2.csv`** preserves the 7-sample warmup + loop-baseline benchmark.

The CI workflow updates `latest.csv` automatically after successful runs.

The timing results are microbenchmark measurements only. **They do not establish energy savings.** Energy must be measured on hardware.

## Representations

| Representation | Semantic content |
|---|---|
| JSON | explicit field/unit/value |
| CBOR | equivalent structured content |
| OMWEI | descriptor + canonical value |
| OMWEI-128 | 16-byte benchmark variant |

Transport overhead is separate from representation overhead. The current executable is a local serialization microbenchmark, not an MQTT/Zenoh transport benchmark.

## Fairness

The benchmark uses identical semantic content, the same observation counts, and multiple scales:

```text
1
1,000
100,000
1,000,000 observations
```

The registry is part of semantic interpretation and should be evaluated separately for cached and cold lookup conditions. It must not be reparsed for every observation.

## Measurements

Current executable:

- encoded payload bytes;
- encode latency;
- decode latency;
- repeated-run behavior at scale;
- raw and loop-baseline-corrected medians.

Not yet measured:

- CPU cycles;
- allocations;
- memory;
- end-to-end MQTT/Zenoh latency;
- transport bytes;
- energy.

## Running locally

```bash
cargo test --release
cargo run --release --bin benchmark
cargo run --release --bin compare
```

GitHub Actions runs the benchmark, uploads the raw artifact, and publishes the resulting `latest.csv` back into the repository.

## Interpretation

The benchmark should answer two independent questions:

```text
A: Do heterogeneous implementations obtain the same meaning?

B: How efficiently can that meaning be represented and processed?
```

Semantic interoperability is the primary OMWEI objective. Compactness and energy efficiency are engineering outcomes to be measured.
