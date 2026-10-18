# Source mapping conformance runner (experimental)

This runner is an independent Rust implementation of the source-mapping v1
contract used by `omwei-org/omwei-iot-go`. It consumes the shared profile/vector
document shapes and compares exact six-byte OMWEI atoms or stable rejection
categories.

## Run

From the repository root:

```sh
cargo test --all-targets
cargo run --bin source-mapping-conformance
```

Optional positional arguments select other profile and vector documents:

```sh
cargo run --bin source-mapping-conformance -- profiles.json vectors.json
```

The fixtures in `examples/source-mappings-v1.json` and
`testdata/source-mapping-interop-v1.json` mirror the Go repository's fixtures.
Keep them synchronized when the shared contract changes.

## Scope and limits

- Implements exact rational source scale/offset using checked integer arithmetic.
- Validates profile identity uniqueness, descriptor existence, and canonical unit.
- Rejects unknown JSON fields, duplicate vector names, and ambiguous outcomes.
- Compares the full six-byte wire result and expected error identifiers.
- Includes a negative integration test proving an incorrect expected wire value
  causes a non-zero runner exit.

The current fixtures are synthetic. The SunSpec vector starts after the adapter
has resolved the model point and `W_SF`; it does not perform Modbus I/O or verify
vendor register addresses. Passing these vectors is not evidence of a live OT
Fabric integration, device identity provisioning, freshness, or authorization.
