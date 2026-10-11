#!/usr/bin/env python3
"""Exercise Rust CLI and an independent Python codec in both directions."""
from __future__ import annotations

import json
import re
import struct
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "target" / "release" / "omwei"
ATOM_VECTORS = ROOT / "tests" / "vectors" / "atom-v1.json"
DECIMAL_VECTORS = ROOT / "tests" / "vectors" / "decimal-conversion-v1.json"


def atom_bytes(identifier: int, value: int) -> bytes:
    return struct.pack(">Hi", identifier, value)


def decode_python(data: bytes) -> tuple[int, int]:
    if len(data) != 6:
        raise ValueError(f"expected six bytes, got {len(data)}")
    return struct.unpack(">Hi", data)


def rust_cli(*args: str) -> str:
    result = subprocess.run(
        [str(CLI), *args],
        cwd=ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.stdout


def rust_decoded_atom(hex_wire: str) -> tuple[int, int]:
    output = rust_cli("decode", hex_wire)
    descriptor = re.search(r"^\s*descriptor:\s+0x([0-9A-Fa-f]{4})\b", output, re.M)
    encoded = re.search(r"^\s*encoded:\s+(-?\d+)\s*$", output, re.M)
    assert descriptor and encoded, f"could not parse Rust decode output:\n{output}"
    return int(descriptor.group(1), 16), int(encoded.group(1))


def main() -> None:
    assert CLI.is_file(), f"Rust CLI binary not found at {CLI}; build it first"
    atom_set = json.loads(ATOM_VECTORS.read_text(encoding="utf-8"))
    decimal_set = json.loads(DECIMAL_VECTORS.read_text(encoding="utf-8"))
    checks = 0

    # Python-produced bytes must be accepted by Rust; fixture/Rust-produced
    # bytes must decode to the same identifier and encoded value in Python.
    for vector in atom_set["valid"]:
        identifier = int(vector["descriptor_id"], 16)
        value = vector["encoded_value"]
        python_wire = atom_bytes(identifier, value)
        fixture_wire = bytes.fromhex(vector["bytes_hex"])
        assert python_wire == fixture_wire, f"{vector['name']}: Python encode differs from fixture"
        assert rust_decoded_atom(python_wire.hex()) == (identifier, value), (
            f"{vector['name']}: Rust could not decode Python-produced bytes"
        )
        assert decode_python(fixture_wire) == (identifier, value), (
            f"{vector['name']}: Python could not decode fixture/Rust wire bytes"
        )
        checks += 2

    # Rust decimal conversion must produce bytes that the independent Python
    # decoder accepts, and Python-produced bytes must be accepted by Rust.
    for vector in decimal_set["valid"]:
        identifier = int(vector["descriptor_id"], 16)
        expected_value = vector["encoded_value"]
        output = rust_cli("convert", vector["descriptor_id"], vector["input"])
        wire_match = re.search(r"^\s*wire \(6 B\):\s+([0-9A-Fa-f]{12})\s*$", output, re.M)
        encoded_match = re.search(r"^\s*encoded:\s+(-?\d+)\s*$", output, re.M)
        assert wire_match and encoded_match, (
            f"{vector['name']}: could not parse Rust convert output:\n{output}"
        )
        rust_wire = bytes.fromhex(wire_match.group(1))
        assert int(encoded_match.group(1)) == expected_value, (
            f"{vector['name']}: Rust decimal conversion returned unexpected integer"
        )
        assert decode_python(rust_wire) == (identifier, expected_value), (
            f"{vector['name']}: Python could not decode Rust-produced bytes"
        )
        python_wire = atom_bytes(identifier, expected_value)
        assert rust_decoded_atom(python_wire.hex()) == (identifier, expected_value), (
            f"{vector['name']}: Rust could not decode Python-produced bytes"
        )
        checks += 2

    print(f"Rust/Python wire interoperability passed: {checks} directional checks.")


if __name__ == "__main__":
    main()
