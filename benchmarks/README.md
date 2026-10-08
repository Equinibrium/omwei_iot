# OMWEI IoT Benchmark

## 1. Purpose

The benchmark tests the original OMWEI engineering hypothesis:

> A shared semantic identity can make repeated IoT exchange both interoperable and substantially cheaper to decode than text-heavy representations.

The benchmark must separate **semantic correctness** from **representation efficiency**.

## 2. Controlled semantic payload

All formats MUST represent the same observation:

```text
ambient temperature = 22.500 °C
```

The OMWEI representation uses the active registry definition for descriptor `0x0042`:

```text
descriptor = 0x0042
canonical value = 22500
```

The benchmark fixture MUST contain the equivalent semantic content for JSON and CBOR.

## 3. Representations

The initial comparison is:

| Representation | Semantic content | Transport |
|---|---|---|
| JSON | explicit field/unit/value | MQTT |
| CBOR | equivalent structured content | MQTT |
| OMWEI | descriptor + canonical value | MQTT |
| JSON | explicit field/unit/value | Zenoh |
| CBOR | equivalent structured content | Zenoh |
| OMWEI | descriptor + canonical value | Zenoh |

Transport overhead MUST be reported separately from payload/encoding overhead.

## 4. Fairness rules

The benchmark MUST:

1. use identical semantic content across formats;
2. perform equivalent canonicalization before comparison;
3. exclude unrelated application metadata;
4. use the same message count and observation sequence;
5. distinguish payload size from total wire size;
6. report cold-start and steady-state results separately where relevant;
7. report decoder configuration and registry lookup behavior;
8. avoid optimizing only one implementation through format-specific shortcuts.

The registry lookup for OMWEI MUST be measured separately when it is not already cached, because the registry is part of semantic interpretation.

## 5. Measurements

### Representation

- encoded payload bytes;
- total transmitted bytes;
- bytes per observation.

### Decode cost

- decode latency;
- CPU time;
- CPU cycles where available;
- allocations;
- allocated bytes;
- memory use;
- throughput.

### End-to-end

- publish-to-consume latency;
- sustained throughput;
- behavior under repeated observations.

### Energy

Where hardware measurement is available:

- energy per observation;
- energy per 1,000 observations;
- average power during sustained exchange.

Energy MUST be measured rather than inferred from payload size or CPU time.

## 6. Repetition

The benchmark SHOULD run at multiple scales, for example:

```text
1
1,000
100,000
1,000,000 observations
```

This is important because allocation and parsing costs that are insignificant for one message can dominate sustained sensor workloads.

## 7. Registry conditions

At minimum, test:

1. registry definition already cached;
2. registry definition loaded once before the run.

A separate cold-registry lookup test MAY be added.

The benchmark MUST NOT repeatedly parse a textual registry definition for every OMWEI observation. That would defeat the architecture being evaluated.

## 8. Expected result

The benchmark is not required to prove that OMWEI wins every metric.

It should establish:

- whether all representations preserve the same semantic meaning;
- how much representation overhead each format introduces;
- whether OMWEI reduces decode/allocation/CPU work;
- whether those reductions persist at scale;
- whether measurable energy savings occur on constrained hardware.

## 9. Historical 16-byte target

A 16-byte OMWEI encoding MAY be included as a separate benchmark variant.

It MUST NOT be treated as the definition of semantic interoperability.

The benchmark should answer two independent questions:

```text
Question A:
Do heterogeneous implementations obtain the same meaning?

Question B:
How efficiently can that meaning be represented and processed?
```

Semantic interoperability is the primary OMWEI objective. Compactness and energy efficiency are engineering outcomes to be measured.


## Executable comparison

The current Rust comparison also measures encode and decode time for 1,000,000 iterations of the same semantic observation.

Run:

```bash
cargo test
cargo run --release --bin compare
```

This is a microbenchmark, not yet an energy or transport benchmark. Results must be reported from the same machine, Rust version, optimization level, and workload. Allocation and CPU-cycle measurements remain a next step.

