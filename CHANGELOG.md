# Changelog

All notable changes to `omwei-iot` are documented here.

## [0.1.2] - 2026-10-11

### Added
- Decimal conversion rounding report API: `ConversionResult { atom, rounded }` and `from_decimal_with_report()`. The `rounded` flag is true when the scaled input has a non-zero fractional remainder and quantization was required. The original `from_decimal()` API remains available for callers that do not need this report.
- Energy and water descriptor proposal documentation and QUDT mapping audit infrastructure. IDs 0x0048–0x004A are placeholders and are not allocated to the production corpus.

### Changed
- Clarified in vendor-mapping documentation that source unit normalization must account for scale before emitting atoms.
- Updated README decimal conversion section to document the new rounding report API and clarify that `rounded = true` means quantization was required, not that the original measurement was inaccurate.

### Notes
- The 6-byte atom wire format, existing descriptor IDs, and rounding rule remain unchanged.
- Energy/water descriptors remain proposals; the corpus and wire format are not changed.
- The registry contract and conformance vectors remain draft and are not a released compatibility guarantee.

## [0.1.1] - 2026-10-11

### Added
- Embedded `no_std` integration guide with Cargo setup, API examples, and registry/runtime limitations.
- CI-checked no-`std` consumer fixture and minimal Cortex-M firmware link smoke test.
- Runtime contract tests for stable wire bytes, static descriptor lookup, padded representation, input lengths, descriptor bounds, and decimal conversion rejection cases.

### Changed
- Make embedded `no_std` usage and feature configuration easier to discover from the README.
- Exclude nested CI-only consumer/firmware crates from the published package; they remain available in the Git repository and CI.

### Notes
- This release does not claim execution on physical MCU hardware.
- The registry contract and conformance vectors remain draft and are not a released compatibility guarantee.
- The six-byte atom requires compatible registry definitions at both ends; registry identity/version is not carried in each atom.
