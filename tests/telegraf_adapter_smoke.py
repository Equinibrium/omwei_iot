#!/usr/bin/env python3
"""Build and assert a Telegraf smoke test from the registry-generated vectors."""

from collections import Counter
import json
from pathlib import Path
import re
import sys


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def input_block(payload):
    payload_json = json.dumps(payload, separators=(",", ":"))
    command = "printf '%s' '" + payload_json + "'"
    lines = [
        "[[inputs.exec]]",
        f"  commands = [{json.dumps(command)}]",
        '  timeout = "5s"',
        '  data_format = "json_v2"',
    ]
    for field, value in payload.items():
        field_type = "int" if isinstance(value, int) and not isinstance(value, bool) else "string"
        lines.extend([
            "  [[inputs.exec.json_v2]]",
            "    [[inputs.exec.json_v2.field]]",
            f"      path = {json.dumps(field)}",
            f'      type = "{field_type}"',
        ])
    return "\n".join(lines)


def generate(vectors_path, starlark_path, output_path):
    vectors = read_json(vectors_path)["vectors"]
    if not vectors:
        raise ValueError("test_vectors.json must contain at least one vector")

    payloads = [{vector["field"]: vector["raw"]} for vector in vectors]
    payloads.extend([
        {"temp_c_dC": 235, "temp_f_dF": 743, "ambient_temperature_mC": 999},
        {"temp_c_dC": 1_000_000},
        {"temp_c_dC": "235"},
        {"ambient_temperature_mC": 999},
        {"temp_c_dC": 9_007_199_254_740_991},
    ])

    blocks = [input_block(payload) for payload in payloads]
    starlark = Path(starlark_path).read_text(encoding="utf-8").rstrip()
    blocks.append("[[processors.starlark]]\n  source = '''\n" + starlark + "\n'''")
    Path(output_path).write_text("\n\n".join(blocks) + "\n", encoding="utf-8")
    print(f"generated Telegraf smoke config for {len(vectors)} registry vectors and {len(payloads) - len(vectors)} invalid fixtures")


def assert_output(vectors_path, output_path):
    vectors = read_json(vectors_path)["vectors"]
    output = Path(output_path).read_text(encoding="utf-8").splitlines()

    actual_values = Counter()
    actual_errors = Counter()
    for line in output:
        value_match = re.search(r"(?:^|[ ,])ambient_temperature_mC=(-?\d+)i(?:,| )", line)
        if value_match:
            actual_values[int(value_match.group(1))] += 1
        error_match = re.search(r'omwei_error="([^"]+)"', line)
        if error_match:
            actual_errors[error_match.group(1)] += 1

    expected_values = Counter(vector["expected_milli_celsius"] for vector in vectors)
    expected_errors = Counter({
        "missing or ambiguous source fields": 2,
        "source value must be integer": 1,
        "canonical value outside descriptor range": 2,
    })

    if actual_values != expected_values:
        raise AssertionError(f"canonical outputs differ: expected {expected_values}, got {actual_values}")
    if actual_errors != expected_errors:
        raise AssertionError(f"error classifications differ: expected {expected_errors}, got {actual_errors}")
    if any(value == 999 for value in actual_values):
        raise AssertionError("caller-supplied canonical value 999 was accepted")
    print("Telegraf adapter smoke test passed: registry vectors match canonical outputs; all invalid fixtures fail closed")


def main():
    if len(sys.argv) == 5 and sys.argv[1] == "generate":
        generate(sys.argv[2], sys.argv[3], sys.argv[4])
    elif len(sys.argv) == 4 and sys.argv[1] == "assert":
        assert_output(sys.argv[2], sys.argv[3])
    else:
        raise SystemExit(
            "usage: telegraf_adapter_smoke.py generate <vectors.json> <processor.star> <config.out>\n"
            "   or: telegraf_adapter_smoke.py assert <vectors.json> <telegraf-test-output>"
        )


if __name__ == "__main__":
    main()
