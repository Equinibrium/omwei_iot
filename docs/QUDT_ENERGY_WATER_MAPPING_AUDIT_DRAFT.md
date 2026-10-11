# QUDT Candidate Mapping Audit — Energy and Water (Draft)

Status: preliminary source-page audit, not a release-pinned ontology validation. This report does not change the released corpus or allocate descriptor IDs.

## Scope and method

Candidate pairs are the three descriptor proposals in `docs/ENERGY_WATER_DESCRIPTOR_PROPOSAL.md`. Source pages were opened from the official QUDT vocabulary on 2026-10-11. The page inspection is evidence for review, but it is not yet a reproducible RDF-level validation against one pinned QUDT release: the vocabulary pages can be regenerated, and this repository does not yet vendor a specific QUDT release or store a digest of the ontology files.

Do not treat a missing inverse relation on a unit page as a contradiction by itself. In QUDT, `qudt:applicableUnit` and quantity-kind relations have different semantics; validation must follow the ontology properties, not compare rendered page lists for equality.

## Candidate pair findings

| Descriptor candidate | Quantity-kind IRI | Unit IRI | Preliminary finding | Decision |
|---|---|---|---|---|
| cumulative active electrical energy | `http://qudt.org/vocab/quantitykind/ActiveEnergy` | `http://qudt.org/vocab/unit/KiloW-HR` | The official ActiveEnergy page lists KiloW-HR as an applicable unit and identifies ActiveEnergy as a specialization of ElectricEnergy. The KiloW-HR page lists Energy and ElectricEnergy, with a compatible energy dimension vector. This supports the pair through specialization. | Plausible; confirm using pinned RDF and dimensional checks. |
| cumulative water volume | `http://qudt.org/vocab/quantitykind/Volume` | `http://qudt.org/vocab/unit/M3` | The official Volume page lists M3 as an applicable unit; the M3 page lists Volume as a quantity kind and reports the volume dimension vector. This is positive page-level evidence, but the pages report different QUDT versions. | Pending RDF-level check; do not mark as fully audited. |
| water volume flow rate | `http://qudt.org/vocab/quantitykind/VolumeFlowRate` | `http://qudt.org/vocab/unit/L-PER-SEC` | The official VolumeFlowRate page lists L-PER-SEC as an applicable unit; the L-PER-SEC page associates it with VolumeFlowRate and VolumePerTime and reports a compatible volume-per-time dimension vector. This is positive page-level evidence, but the pages report different QUDT versions. | Strong page-level support; confirm against pinned RDF. |

## Proposed machine validation procedure

For a chosen QUDT release:

1. Store the release identifier, retrieval date, source URL(s), and SHA-256 digest for the exact RDF/OWL files used.
2. Parse the RDF rather than scrape the HTML page presentation.
3. Resolve each quantity-kind and unit IRI; fail if either IRI is absent.
4. Check the unit-to-quantity-kind applicability according to the release's declared predicates, including `qudt:applicableUnit` where present. Do not demand that inverse lists be textually identical.
5. Check dimensional compatibility using QUDT's quantity-kind/unit dimension-vector metadata when available. Dimensional compatibility is necessary but may not distinguish semantically different quantities with the same dimensions.
6. Record the actual supporting triples and any derived inference separately. If the ontology does not directly or inferentially support a pair, report it for review rather than infer it from the human-readable label.
7. Keep meter-specific suitability separate from ontology validity: QUDT compatibility cannot establish the meter's output unit, resolution, range, or counter behavior.

A machine-readable evidence snapshot is also stored at `docs/audits/qudt-energy-water-audit.json`; it records the observed page-level claims, dimension vectors, reported versions, and decision boundaries.

## Reproducibility gap

All three pairs have positive official-page evidence in the current snapshot. However, the pages expose mixed QUDT versions, so this report does not claim a completed pinned-release RDF audit. Before this is called reproducible, add an audit script/test that runs against a checked-in or CI-downloaded, version-pinned QUDT RDF artifact and asserts the three pairs above, with a recorded digest. Do not claim those assertions pass until that test has actually run.

## Open decisions unaffected by this audit

- Descriptor IDs 0x0048–0x004A remain placeholders.
- Confirm energy scale against real meter capabilities.
- Confirm water meter units/resolution and flow device class/range.
- Decide how typed unit IRIs are represented in the corpus; if the schema changes, increment `schema_version` and update validation/conformance tests.
