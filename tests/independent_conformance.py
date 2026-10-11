#!/usr/bin/env python3
"""Independent, dependency-free OMWEI v1 conformance runner.

This deliberately does not import the Rust crate or use its encoder/decoder.
The checked-in JSON vectors are the shared test contract.
"""
from __future__ import annotations

import json
import re
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ATOM_VECTORS = ROOT / "tests" / "vectors" / "atom-v1.json"
DECIMAL_VECTORS = ROOT / "tests" / "vectors" / "decimal-conversion-v1.json"

# Independently represented subset of the registry needed to validate values.
# Keep this table aligned with registry/corpus-v0.1.yaml; vector metadata and
# expected wire bytes are checked separately below.
DESCRIPTORS = {
    0x0042: ("ambient_temperature", "degree_Celsius", 1000, 0, -80_000, 150_000),
    0x0043: ("relative_humidity", "percent", 1000, 0, 0, 100_000),
    0x0044: ("atmospheric_pressure", "pascal", 1, 0, 0, 200_000),
    0x0045: ("active_power", "watt", 1, 0, 0, 100_000_000),
    0x0046: ("vibration_rms", "millimeter_per_second", 1000, 0, 0, 1_000_000),
    0x0047: ("methane_concentration", "part_per_million", 1, 0, 0, 1_000_000),
}
DECIMAL_RE = re.compile(r"[+-]?[0-9]+(?:\.[0-9]+)?\Z")


def descriptor_id(text: str) -> int:
    return int(text, 16)


def encode_atom(identifier: int, value: int) -> bytes:
    if identifier not in DESCRIPTORS:
        raise ValueError("unknown_descriptor")
    _, _, _, _, minimum, maximum = DESCRIPTORS[identifier]
    if not minimum <= value <= maximum:
        raise ValueError("value_out_of_range")
    if not -(2**31) <= value < 2**31:
        raise ValueError("value_out_of_range")
    return struct.pack(">Hi", identifier, value)


def decode_atom(data: bytes) -> tuple[int, int]:
    if len(data) != 6:
        raise ValueError("invalid_atom_length")
    identifier, value = struct.unpack(">Hi", data)
    if identifier not in DESCRIPTORS:
        raise ValueError("unknown_descriptor")
    _, _, _, _, minimum, maximum = DESCRIPTORS[identifier]
    if not minimum <= value <= maximum:
        raise ValueError("value_out_of_range")
    return identifier, value


def convert_decimal(text: str, identifier: int) -> int:
    if identifier not in DESCRIPTORS:
        raise ValueError("unknown_descriptor")
    if not isinstance(text, str) or not DECIMAL_RE.fullmatch(text):
        raise ValueError("invalid_decimal")

    unsigned = text[1:] if text[:1] in ("+", "-") else text
    whole, dot, fraction = unsigned.partition(".")
    if len(fraction) > 18:
        raise ValueError("invalid_decimal")

    denominator = 10 ** len(fraction)
    numerator = int(whole) * denominator + (int(fraction) if fraction else 0)
    if text.startswith("-"):
        numerator = -numerator

    _, _, scale, offset, minimum, maximum = DESCRIPTORS[identifier]
    scaled = (numerator - offset * denominator) * scale
    magnitude = abs(scaled)
    rounded = magnitude // denominator
    if (magnitude % denominator) * 2 >= denominator:
        rounded += 1
    if scaled < 0:
        rounded = -rounded

    if not -(2**31) <= rounded < 2**31 or not minimum <= rounded <= maximum:
        raise ValueError("value_out_of_range")
    return rounded


def expect_error(fn, reason: str, label: str) -> None:
    try:
        fn()
    except ValueError as error:
        actual = str(error)
        if actual != reason:
            raise AssertionError(f"{label}: expected {reason}, got {actual}") from error
        return
    raise AssertionError(f"{label}: expected rejection ({reason})")


def main() -> None:
    atom_set = json.loads(ATOM_VECTORS.read_text(encoding="utf-8"))
    assert atom_set["vector_set"] == "omwei-iot-atom-v1"
    assert atom_set["encoding"]["atom_bytes"] == 6

    for vector in atom_set["valid"]:
        name = vector["name"]
        identifier = descriptor_id(vector["descriptor_id"])
        assert identifier in DESCRIPTORS, f"{name}: unknown descriptor in reference table"
        label, unit, _, _, minimum, maximum = DESCRIPTORS[identifier]
        assert vector["label"] == label, f"{name}: descriptor label mismatch"
        assert vector["unit"] == unit, f"{name}: canonical unit mismatch"
        value = vector["encoded_value"]
        assert minimum <= value <= maximum, f"{name}: vector violates descriptor bounds"
        expected = bytes.fromhex(vector["bytes_hex"])
        actual = encode_atom(identifier, value)
        assert actual == expected, (
            f"{name}: encoded bytes {actual.hex()} != expected {expected.hex()}"
        )
        assert decode_atom(expected) == (identifier, value), f"{name}: decode mismatch"

    for vector in atom_set["invalid"]:
        identifier = descriptor_id(vector["descriptor_id"])
        expect_error(
            lambda identifier=identifier, value=vector["encoded_value"]:
                encode_atom(identifier, value),
            vector["reason"],
            vector["name"],
        )

    for malformed in (b"", b"\x00\x42\x00\x00\x57", b"\x00\x42\x00\x00\x57\xe4\x00"):
        expect_error(lambda malformed=malformed: decode_atom(malformed),
                     "invalid_atom_length", f"malformed length {len(malformed)}")

    decimal_set = json.loads(DECIMAL_VECTORS.read_text(encoding="utf-8"))
    assert decimal_set["vector_set"] == "omwei-iot-decimal-conversion-v1"
    for vector in decimal_set["valid"]:
        actual = convert_decimal(vector["input"], descriptor_id(vector["descriptor_id"]))
        assert actual == vector["encoded_value"], (
            f"{vector['name']}: encoded {actual}, expected {vector['encoded_value']}"
        )

    for vector in decimal_set["invalid"]:
        identifier = descriptor_id(vector["descriptor_id"])
        expect_error(
            lambda vector=vector, identifier=identifier:
                convert_decimal(vector["input"], identifier),
            vector["reason"],
            vector["name"],
        )

    # Explicit offset case: (10.05 - 10) * 10 = 0.5 -> +1.
    # This catches the common error of scaling before subtracting the offset.
    original = DESCRIPTORS[0x0042]
    try:
        DESCRIPTORS[0x0042] = (original[0], original[1], 10, 10, -100, 100)
        assert convert_decimal("10.05", 0x0042) == 1
        assert convert_decimal("0", 0x0042) == -100
    finally:
        DESCRIPTORS[0x0042] = original

    print(
        "Independent OMWEI conformance passed: "
        f"{len(atom_set['valid'])} valid atom vectors, "
        f"{len(atom_set['invalid'])} invalid atom vectors, "
        f"{len(decimal_set['valid'])} valid decimal vectors, "
        f"{len(decimal_set['invalid'])} invalid decimal vectors, "
        "plus malformed-length and nonzero-offset checks."
    )


if __name__ == "__main__":
    main()
