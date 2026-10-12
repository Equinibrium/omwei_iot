#!/usr/bin/env python3
"""Reproducible QUDT energy/water mapping audit pinned to QUDT v3.5.2.

Downloads immutable RDF source files, verifies SHA-256 before parsing, and checks
unit/quantity-kind compatibility using explicit unitForQuantityKind relations,
specialization paths, and equal dimension vectors. It intentionally does not
scrape HTML or require inverse page lists to be textually identical.
"""
from __future__ import annotations

import hashlib
import sys
from urllib.request import Request, urlopen

from rdflib import Graph, Namespace, URIRef
from rdflib.namespace import RDF

QUDT_COMMIT = "99d41755044d5b36220d0759aaf73e62c1a3ee29"
BASE = f"https://raw.githubusercontent.com/qudt/qudt-public-repo/{QUDT_COMMIT}/"
FILES = {
    "quantity_kinds": {
        "path": "src/main/rdf/vocab/quantitykinds/VOCAB_QUDT-QUANTITY-KINDS-ALL.ttl",
        "sha256": "729dc6c4f429a808d4c5d6766f1be13a3dc1196fd95f5bdb417a19f251cbce8f",
    },
    "units": {
        "path": "src/main/rdf/vocab/unit/VOCAB_QUDT-UNITS-ALL.ttl",
        "sha256": "372448d68405c3bf561ffc99900bb3bffc5fe9affd345eccbadacbc8043be4c7",
    },
    "schema": {
        "path": "src/main/rdf/schema/SCHEMA_QUDT.ttl",
        "sha256": "c29dcea035932c4cfc3882d42a0ab3665c44000d2e6152f85ea9f0d4fbba4a54",
    },
}

QK = Namespace("http://qudt.org/vocab/quantitykind/")
UNIT = Namespace("http://qudt.org/vocab/unit/")
QUDT = Namespace("http://qudt.org/schema/qudt/")

PAIRS = (
    ("ActiveEnergy", "KiloW-HR", "ElectricEnergy"),
    ("Volume", "M3", None),
    ("VolumeFlowRate", "L-PER-SEC", None),
)


def fetch_verified(name: str) -> bytes:
    metadata = FILES[name]
    url = BASE + metadata["path"]
    request = Request(url, headers={"User-Agent": "omwei-iot-qudt-audit/1.0"})
    with urlopen(request, timeout=60) as response:
        data = response.read()
    digest = hashlib.sha256(data).hexdigest()
    if digest != metadata["sha256"]:
        raise RuntimeError(
            f"{name}: SHA-256 mismatch for {url}: expected "
            f"{metadata['sha256']}, got {digest}"
        )
    print(f"OK SHA-256 {name}: {digest}")
    return data


def parse_graph(data: bytes, name: str) -> Graph:
    graph = Graph()
    graph.parse(data=data, format="turtle", publicID=BASE + FILES[name]["path"])
    return graph


def specialization_reaches(graph: Graph, child: URIRef, ancestor: URIRef) -> bool:
    """Walk qudt:specializationOf from a quantity kind to its ancestors."""
    pending = [child]
    visited: set[URIRef] = set()
    while pending:
        current = pending.pop()
        if current == ancestor:
            return True
        if current in visited:
            continue
        visited.add(current)
        pending.extend(graph.objects(current, QUDT.specializationOf))
    return False


def main() -> int:
    try:
        qk_graph = parse_graph(fetch_verified("quantity_kinds"), "quantity_kinds")
        unit_graph = parse_graph(fetch_verified("units"), "units")
        schema_graph = parse_graph(fetch_verified("schema"), "schema")
    except Exception as exc:
        print(f"FAIL: could not fetch, checksum, or parse pinned QUDT RDF: {exc}", file=sys.stderr)
        return 1

    # Verify that the pinned schema actually defines the predicates used below.
    required_predicates = (
        QUDT.unitForQuantityKind,
        QUDT.specializationOf,
        QUDT.hasDimensionVector,
    )
    for predicate in required_predicates:
        if not any(schema_graph.triples((predicate, RDF.type, None))):
            print(f"FAIL: pinned schema does not declare {predicate}", file=sys.stderr)
            return 1

    failed = False
    for qk_name, unit_name, inherited_kind in PAIRS:
        quantity_kind = QK[qk_name]
        unit = UNIT[unit_name]
        if not any(qk_graph.triples((quantity_kind, RDF.type, QUDT.QuantityKind))):
            print(f"FAIL: missing quantity kind {quantity_kind}")
            failed = True
            continue
        if not any(unit_graph.triples((unit, RDF.type, QUDT.Unit))):
            print(f"FAIL: missing unit {unit}")
            failed = True
            continue

        qk_dimensions = set(qk_graph.objects(quantity_kind, QUDT.hasDimensionVector))
        unit_dimensions = set(unit_graph.objects(unit, QUDT.hasDimensionVector))
        if not qk_dimensions or not unit_dimensions or not (qk_dimensions & unit_dimensions):
            print(
                f"FAIL: dimension mismatch or missing dimension vector: "
                f"{quantity_kind}={sorted(map(str, qk_dimensions))}, "
                f"{unit}={sorted(map(str, unit_dimensions))}"
            )
            failed = True
            continue

        direct_pair = (unit, QUDT.unitForQuantityKind, quantity_kind) in unit_graph
        if inherited_kind is None:
            supported = direct_pair
            route = "direct unitForQuantityKind"
        else:
            parent = QK[inherited_kind]
            supported = (
                (unit, QUDT.unitForQuantityKind, parent) in unit_graph
                and specialization_reaches(qk_graph, quantity_kind, parent)
            )
            route = f"unitForQuantityKind({inherited_kind}) + specialization path"
        if not supported:
            print(
                f"FAIL: unsupported quantity-kind/unit pair {quantity_kind} / {unit}; "
                f"expected route: {route}"
            )
            failed = True
            continue

        print(
            f"PASS: {quantity_kind} / {unit}; route={route}; "
            f"dimension={next(iter(qk_dimensions & unit_dimensions))}"
        )

    if failed:
        print("QUDT energy/water audit FAILED.", file=sys.stderr)
        return 1
    print(f"QUDT energy/water audit PASSED against pinned commit {QUDT_COMMIT}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
