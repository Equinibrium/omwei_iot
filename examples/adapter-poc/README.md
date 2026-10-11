# Registry-driven adapter generator PoC

Example / proof-of-concept under `examples/adapter-poc/`. It does **not** change the production registry contract or wire format.

Source profiles here are **input encodings** (vendor field → canonical value), not new OMWEI semantic descriptors.

## Claim under test

Can one declarative registry fixture generate two different platform adapter artifacts that normalize different source representations to the same canonical OMWEI value?

- Descriptor: `0x0042`, ambient temperature.
- Vendor A field: `temp_c_dC`; raw `235` means 23.5 °C.
- Vendor B field: `temp_f_dF`; raw `743` means 74.3 °F.
- Canonical output field: `ambient_temperature_mC`; both must produce integer `23500`, with a scale of 0.001 °C.

## Run from repository root

```sh
cargo test --bin adapter_generator
cargo run --bin adapter_generator -- examples/adapter-poc/registry.yaml examples/adapter-poc/generated
```

The generator writes a The Things Stack `normalizeUplink(input)` JavaScript adapter, a Telegraf Starlark processor, and test-vector JSON. It expects the source fields to have already been decoded into numeric values. It is not a raw LoRaWAN byte decoder.

## Deterministic conversion

For each source profile:

`canonical_mC = round_half_away_from_zero(((raw + addend) × multiplier) / divisor)`

- Celsius tenths: `raw × 100`
- Fahrenheit tenths: `(raw − 320) × 500 / 9`

Both generated adapters reject missing/ambiguous source fields, non-integer input, and values outside the descriptor range. They do not clamp or silently choose a default. The JavaScript adapter also rejects conversion intermediates outside JavaScript's safe-integer range. Equivalent overflow behavior in the Telegraf Starlark runtime has not yet been established, so this PoC does not claim parity for that edge case.

## CI evidence and limits

CI generates the JavaScript adapter, Starlark processor, and test-vector JSON from the same registry fixture. It executes every registry vector against the JavaScript adapter and builds a Telegraf smoke configuration from those same vectors, then compares the emitted canonical-value counts and checks fail-closed handling of invalid fixtures.

This is a container-level Telegraf test, not a live The Things Stack integration test. Passing CI demonstrates the fixture validator/conversion, generated JavaScript behavior, and Telegraf processor behavior for the tested vectors and invalid cases. It does not establish production interoperability, nor does it establish that the current production corpus schema already supports source profiles; this fixture deliberately tests a proposed extension separately.
