# QUDT Candidate Mapping Audit — Energy and Water (Draft)

Status: pinned RDF sources selected and audit script/workflow added; CI execution is pending. This does not finalize descriptor IDs, scales, or meter ranges.

## Pinned source release

The audit uses QUDT **v3.5.2**, resolved to immutable commit `99d41755044d5b36220d0759aaf73e62c1a3ee29` (tag object `01c17801dd651d2e206369d12695e54125ea3d6e`). All RDF files come from that same commit, rather than from live vocabulary pages.

| Artifact | SHA-256 of UTF-8 contents |
|---|---|
| `src/main/rdf/vocab/quantitykinds/VOCAB_QUDT-QUANTITY-KINDS-ALL.ttl` | `729dc6c4f429a808d4c5d6766f1be13a3dc1196fd95f5bdb417a19f251cbce8f` |
| `src/main/rdf/vocab/unit/VOCAB_QUDT-UNITS-ALL.ttl` | `372448d68405c3bf561ffc99900bb3bffc5fe9affd345eccbadacbc8043be4c7` |
| `src/main/rdf/schema/SCHEMA_QUDT.ttl` | `c29dcea035932c4cfc3882d42a0ab3665c44000d2e6152f85ea9f0d4fbba4a54` |

The machine-readable record is `docs/audits/qudt-energy-water-audit.json`. The script downloads these immutable URLs, verifies each checksum, parses Turtle with pinned RDFLib, and checks resources, supported quantity-kind/unit relations, specialization, and equal dimension vectors.

## Candidate pair findings from the pinned RDF text

| Descriptor candidate | Quantity-kind IRI | Unit IRI | RDF evidence inspected | Finding |
|---|---|---|---|---|
| cumulative active electrical energy | `http://qudt.org/vocab/quantitykind/ActiveEnergy` | `http://qudt.org/vocab/unit/KiloW-HR` | ActiveEnergy specializes ElectricEnergy; KiloW-HR has `unitForQuantityKind ElectricEnergy`; their dimension vectors match. | Supported via specialization. |
| cumulative water volume | `http://qudt.org/vocab/quantitykind/Volume` | `http://qudt.org/vocab/unit/M3` | M3 has `unitForQuantityKind Volume`; dimension vectors match. | Supported directly. |
| water volume flow rate | `http://qudt.org/vocab/quantitykind/VolumeFlowRate` | `http://qudt.org/vocab/unit/L-PER-SEC` | L-PER-SEC has `unitForQuantityKind VolumeFlowRate`; dimension vectors match. | Supported directly. |

The audit deliberately does not require reciprocal lists to be textually identical. It uses QUDT's explicit unit-for-quantity-kind relation and, for ActiveEnergy, the declared specialization path to ElectricEnergy.

## Reproduce the audit locally

From the repository root with Python 3.12:

```sh
python -m pip install -r tools/requirements-qudt-audit.txt
python tools/audit_qudt_energy_water.py
```

The GitHub Actions workflow `.github/workflows/qudt-audit.yml` runs the same script on relevant pull-request/push changes and on manual dispatch. A successful run should print three PASS lines and the final pinned commit. The script and workflow have been added, but **a CI execution has not yet been observed**; do not describe the automated audit as passed until the workflow run succeeds.

## Scope and limits

This audit establishes ontology-level compatibility, not meter suitability. It cannot establish whether target devices report kWh, m3, or L/s, nor their actual precision, valid range, reset behavior, or rollover semantics.

Still open:
- descriptor IDs `0x0048`–`0x004A) remain placeholders;
- confirm energy scale against actual meter output and lifetime requirements;
- confirm water units/resolution and select the flow-meter class/range;
- decide how unit IRIs are represented in the corpus; if schema changes, increment `schema_version` and update validation/conformance tests.

No production corpus or released wire format has been changed.
