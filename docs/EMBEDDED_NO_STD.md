# Using OMWEI in an embedded `no_std` Rust project

This guide describes the current embedded integration surface of `omwei-iot` v0.1.0. The no-`std` configuration is CI-checked against `thumbv7em-none-eabihf`, including a separate consumer crate. The registry contract is still a draft; treat the current descriptor corpus and wire format as development-stage interfaces rather than a released compatibility guarantee.

## 1. Add OMWEI without default features

In your firmware or embedded library's `Cargo.toml`:

```toml
[dependencies]
omwei-iot = { version = "0.1", default-features = false }
```

For a workspace checkout, a path dependency is also supported:

```toml
[dependencies]
omwei-iot = { path = "../omwei_iot", default-features = false }
```

Disabling default features turns off the crate's `std` feature and its optional runtime Serde/JSON/CBOR/YAML dependencies. The crate's build script still runs on the build host and validates the YAML corpus to generate the static descriptor table; it does not mean YAML parsing is included in the embedded runtime.

Install the target used by the CI check if needed:

```sh
rustup target add thumbv7em-none-eabihf
cargo check --no-default-features --lib --target thumbv7em-none-eabihf
```

Use the target appropriate for your board and toolchain. This command checks the library for a target; a firmware executable may additionally need a target-specific linker configuration, entry point, panic handler, and board support.

## 2. Encode and decode an atom

The compact atom is six bytes: a 16-bit descriptor ID followed by a signed 32-bit canonical value, both big-endian. The receiver must use the same descriptor registry and interpretation rules.

```rust
#![no_std]

use omwei_iot::{
    decode, encode, SemanticAtom, AMBIENT_TEMPERATURE_ID,
};

pub fn encode_temperature(canonical_millidegrees_c: i32) -> Result<[u8; 6], &'static str> {
    let atom = SemanticAtom::new(
        AMBIENT_TEMPERATURE_ID,
        canonical_millidegrees_c,
    )?;
    Ok(encode(atom))
}

pub fn decode_temperature(bytes: &[u8]) -> Result<i32, &'static str> {
    let atom = decode(bytes)?;
    if atom.descriptor_id != AMBIENT_TEMPERATURE_ID {
        return Err("unexpected descriptor");
    }
    Ok(atom.canonical_value)
}
```

For descriptor `0x0042` (`ambient_temperature`), canonical value `22_500` represents `22.500 °C` in the current corpus. The constructor and decoder reject unknown descriptor IDs and values outside the descriptor's declared encoded range.

The decimal-string helper is also available without `std` and does not use floating-point arithmetic:

```rust
use omwei_iot::{decimal_conversion::from_decimal, AMBIENT_TEMPERATURE_ID};

let atom = from_decimal(AMBIENT_TEMPERATURE_ID, "22.5")?;
assert_eq!(atom.canonical_value, 22_500);
# Ok::<(), &'static str>(())
```

The conversion accepts plain base-10 notation, supports up to 18 fractional digits, rounds exact halfway cases away from zero, and returns an error for malformed, unknown, overflowing, or out-of-range values. If the device already stores the canonical integer, construct the atom directly and avoid converting through a string.

## 3. Resolve the static registry

The built-in corpus is compiled into a static descriptor table. You can resolve a descriptor without loading YAML at runtime:

```rust
use omwei_iot::mapping::ambient_temperature_descriptor;

pub fn temperature_unit() -> Result<&'static str, &'static str> {
    Ok(ambient_temperature_descriptor()?.unit)
}
```

The registry's IDs, units, scale, offset, and constraints are part of the semantic contract, not merely implementation details. Vendor field names should be mapped to OMWEI IDs at the device or gateway boundary; they are not encoded into each atom.

## 4. Optional 16-byte representation

If an integration specifically requires the historical 16-byte container, `encode_128` returns a 16-byte array containing the six-byte atom followed by ten zero bytes:

```rust
use omwei_iot::{encode_128, SemanticAtom, AMBIENT_TEMPERATURE_ID};

let atom = SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500)?;
let wire: [u8; 16] = encode_128(atom);
assert_eq!(&wire[..6], &omwei_iot::encode(atom));
assert!(wire[6..].iter().all(|byte| *byte == 0));
# Ok::<(), &'static str>(())
```

This is padding around the current six-byte representation, not a separate semantic encoding or a place to store registry identity.

## 5. Runtime boundary and limitations

With `default-features = false`, the intended runtime surface is:

- `SemanticAtom` construction and validation;
- static descriptor lookup and mapping helpers;
- six-byte `encode` / `decode` and the 16-byte padded helper;
- exact decimal-string conversion.

The following remain host-side or `std`-only in the current crate:

- the `omwei` command-line tool;
- dynamic YAML corpus parsing and corpus-validation API;
- JSON observation helper and optional Serde JSON/CBOR/YAML integrations.

The wire atom does **not** carry registry ID, registry version, or corpus version. Both ends must therefore be provisioned with compatible descriptor definitions through firmware, gateway configuration, or another control-plane mechanism. Do not assume that matching bytes alone prove semantic interoperability.

OMWEI does not provide a transport, radio driver, framing protocol, message authentication, replay protection, or device lifecycle management. Those are responsibilities of the surrounding system. Apply the appropriate transport framing and security controls for your deployment.

## 6. Verify the no-`std` configuration

From the repository root, the CI-equivalent checks are:

```sh
cargo check --no-default-features --lib --target thumbv7em-none-eabihf
cargo check --manifest-path tests/no_std_smoke/Cargo.toml --target thumbv7em-none-eabihf
cargo test --no-default-features --lib
cargo test
```

The standalone fixture in `tests/no_std_smoke/` is a minimal `#![no_std]` consumer that exercises the public API. Use it as a starting point for an application-specific compile check, not as a complete firmware image.
