# Registry-driven adapter generator PoC

This is an isolated experiment on branch `experiment/adapter-generator-poc`. It does not change the production registry contract or wire format.

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

Both generated adapters reject missing/ambiguous source fields, non-integer input, conversion overflow, and values outside the descriptor range. They do not clamp or silently choose a default.

## Evidence boundary

Passing Rust tests demonstrates the fixture validator/conversion and generator output shape. It does not, by itself, prove that the emitted JavaScript and Starlark execute correctly inside live The Things Stack and Telegraf deployments. That requires a platform-level smoke test. It also does not establish that the current production corpus schema already supports source profiles; this fixture deliberately tests a proposed extension separately.
