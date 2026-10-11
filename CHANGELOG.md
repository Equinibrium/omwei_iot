# Changelog

All notable changes to `omwei-iot` are documented here.

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
