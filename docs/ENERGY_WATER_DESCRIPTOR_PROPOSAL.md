# Energy and Water Descriptor Proposal (Draft)

Status: proposal only. IDs below are placeholders, not allocations. This document does not change the released corpus or six-byte wire format.

## Proposed meanings

| Placeholder ID | Label | Meaning | Quantity-kind IRI | Canonical unit identifier | Unit IRI | Scale |
|---|---|---|---|---|---|---|
| 0x0048 | cumulative_active_electrical_energy | Cumulative active electrical energy counter; not power and not generic thermal/gas energy | http://qudt.org/vocab/quantitykind/ActiveEnergy | kilowatt_hour | http://qudt.org/vocab/unit/KiloW-HR | 0.01 kWh/count (provisional resolution) |
| 0x0049 | cumulative_water_volume | Cumulative water volume counter | http://qudt.org/vocab/quantitykind/Volume | cubic_meter | http://qudt.org/vocab/unit/M3 | 0.001 m³/count (1 litre/count) |
| 0x004A | water_volume_flow_rate | Instantaneous water volume flow rate | http://qudt.org/vocab/quantitykind/VolumeFlowRate | liter_per_second | http://qudt.org/vocab/unit/L-PER-SEC | 0.001 L/s/count (pending meter review) |

The identifiers above follow the spelling style of existing corpus canonical-unit names. They are registry identifiers, not claims that every listed unit is an SI unit. The current validator requires `unit.system` to be a non-empty string but does not restrict it to an enumeration; no production corpus change is proposed here.

All encodings remain signed int32, two's-complement big-endian, with a 16-bit unsigned big-endian descriptor ID. The QUDT quantity-kind and unit IRIs are separate semantic fields; the current corpus schema has no explicit typed unit-IRI field. If the schema is changed to add one, `schema_version` must be incremented and compatibility/validation tests updated.

## Proposed encoded constraints versus device limits

For the two cumulative counters, a provisional *representation-domain* constraint is encoded integer `min = 0`, `max = 2147483647`. With the proposed scales, that corresponds to:

- Energy: 0 to 21,474,836.47 kWh.
- Water volume: 0 to 2,147,483.647 m³.

These are the nonnegative limits of the selected int32 encoding at the proposed scales, not verified meter limits, service-life guarantees, or claims about realistic installation sizes. A device may have a lower native counter limit, a different reset/rollover behavior, or finer/coarser resolution. The instantaneous flow descriptor has no proposed valid min/max yet because a target meter class has not been selected; the int32 type limit alone must not be mistaken for a physically valid flow range.

## Energy resolution trade-off

At 0.01 kWh/count, a 15-minute interval at 200 W consumes 0.05 kWh and therefore equals 5 counts. The maximum nonnegative int32 reading is 21,474,836.47 kWh, about 5.86 years at a continuous 418 kW. At 0.1 kWh/count the same 15-minute interval consumes half a count. For a quantized cumulative counter, the observed difference between two readings can be 0 or 0.1 kWh depending on the readings' position relative to quantization boundaries. Thus a real 0.05 kWh interval can have a quantization error as large as 0.05 kWh (100% relative error); it is inaccurate to say the interval can never be distinguished from zero. Select final resolution from actual meter capabilities and deployment requirements, not this draft alone.

## Provisional test vectors

IDs are placeholders, so wire bytes are illustrative only and must not be frozen until allocation is approved. Values are expressed in the canonical physical unit; encoded = physical / scale.

| Label | Input | Encoded integer | Provisional six-byte atom |
|---|---:|---:|---|
| cumulative_active_electrical_energy | 0.05 kWh | 5 | 004800000005 |
| cumulative_active_electrical_energy | 418 kWh | 41800 | 00480000A348 |
| cumulative_water_volume | 1.234 m³ | 1234 | 0049000004D2 |
| water_volume_flow_rate | 12.345 L/s | 12345 | 004A00003039 |

The companion JSON file `docs/vectors/energy-water-proposal-v1.json` contains expanded illustrative vectors and rejection/lifecycle scenarios. All vectors must be updated if IDs, scales, units, or valid ranges change. They are proposal fixtures, not released conformance guarantees.

## Counter lifecycle proposal

A counter reset, rollover, meter replacement, or correction must not silently look like a negative consumption delta. Consumers should detect a lower reading and mark the interval as discontinuous/invalid for ordinary delta calculation unless a separately defined source event explains the transition. The descriptor itself remains unchanged. The Semantic Record or source lifecycle metadata should identify reset/replacement/correction and, where available, counter epoch or meter identity. Do not infer rollover by arithmetic alone unless the meter's modulus and rollover behavior are explicitly known. Delta computation across an unexplained decrease must fail closed or be flagged, never silently wrap.

## QUDT mapping and validation policy (proposal)

Do not require the quantity-kind page and unit page to display identical reciprocal lists as a condition of validity. QUDT distinguishes properties that enumerate all applicable unit/quantity-kind pairs from narrower inverse relationships: in particular, the schema documentation says `qudt:isQuantityKindOf` is the strict inverse of `qudt:hasQuantityKind`, and does not enumerate every allowed unit/quantity-kind pair; `qudt:applicableUnit` is the property intended to express those applicable pairs.

For this proposal, an automated QUDT audit should:
1. resolve both the quantity-kind IRI and unit IRI independently;
2. use the quantity kind's `applicableUnit` relation, when present in the selected QUDT release, as the primary positive pairing check;
3. check dimensional compatibility and known quantity-kind specialization relationships;
4. report missing or contradictory metadata for manual review rather than infer meaning from labels or require simplistic bidirectional page-list equality;
5. record the QUDT release/version used by the audit so results are reproducible offline.

This is an OMWEI validation policy proposal, not a claim that QUDT pages alone settle meter-specific suitability. The water units and device ranges still require confirmation against actual target meters.

## Open project decisions

1. Approve allocation policy and final IDs; 0x0048–0x004A are placeholders only.
2. Confirm energy resolution against real meter output (0.01 kWh is the current candidate, not a final decision).
3. Confirm canonical units and reported resolutions from target water meters; cubic meter and liter per second are provisional pending that review.
4. Select the flow device class and valid encoded range; do not automatically treat all int32 values as valid.
5. Decide how unit IRIs are represented. If this changes the corpus document schema, increment `schema_version`.
6. Define source/counter epoch metadata and conformance tests for resets, rollovers, replacement, and correction.
7. Resolve the int32 range/resolution trade-off against actual deployment requirements before finalizing the energy scale or changing the wire format.

## QUDT references for review

- http://qudt.org/vocab/quantitykind/ActiveEnergy
- http://qudt.org/vocab/unit/KiloW-HR
- http://qudt.org/vocab/quantitykind/Volume
- http://qudt.org/vocab/unit/M3
- http://qudt.org/vocab/quantitykind/VolumeFlowRate
- http://qudt.org/vocab/unit/L-PER-SEC
- QUDT schema documentation: https://www.qudt.org/doc/2026/01/DOC_SCHEMA-QUDT.html
