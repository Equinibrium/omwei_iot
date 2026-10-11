# Energy and Water Descriptor Proposal (Draft)

Status: proposal only. IDs below are placeholders, not allocations. This document does not change the released corpus or six-byte wire format.

## Proposed meanings

| Placeholder ID | Label | Meaning | Quantity-kind IRI | Canonical unit | Unit IRI | Scale |
|---|---|---|---|---|---|---|
| 0x0048 | cumulative_active_electrical_energy | Cumulative active electrical energy counter; not power and not generic thermal/gas energy | http://qudt.org/vocab/quantitykind/ActiveEnergy | kWh | http://qudt.org/vocab/unit/KiloW-HR | 0.01 kWh/count (provisional resolution) |
| 0x0049 | cumulative_water_volume | Cumulative water volume counter | http://qudt.org/vocab/quantitykind/Volume | m3 | http://qudt.org/vocab/unit/M3 | 0.001 m3/count (1 litre/count) |
| 0x004A | water_volume_flow_rate | Instantaneous water volume flow rate | http://qudt.org/vocab/quantitykind/VolumeFlowRate | L/s (pending meter review) | http://qudt.org/vocab/unit/L-PER-SEC | 0.001 L/s/count (pending meter review) |

All encodings remain signed int32, two's-complement big-endian, with a 16-bit unsigned big-endian descriptor ID. The QUDT quantity-kind and unit IRIs are separate semantic fields; the current corpus schema has no explicit typed unit-IRI field. If the schema is changed to add one, schema_version must be incremented and compatibility/validation tests updated.

## Energy resolution trade-off

At 0.01 kWh/count, a 15-minute interval at 200 W consumes 0.05 kWh and therefore equals 5 counts. The maximum nonnegative int32 reading is 21,474,836.47 kWh, about 5.86 years at a continuous 418 kW. At 0.1 kWh/count the same interval is half a count and cannot be distinguished from zero by counter differencing; maximum is 214,748,364.7 kWh, about 58.65 years at 418 kW. Select final resolution from actual meter capabilities and deployment requirements, not this draft alone.

## Provisional test vectors

IDs are placeholders, so wire bytes are illustrative only and must not be frozen until allocation is approved. Values are canonical physical units; encoded = physical / scale.

| Label | Input | Encoded integer | Provisional six-byte atom |
|---|---:|---:|---|
| cumulative_active_electrical_energy | 0.05 kWh | 5 | 004800000005 |
| cumulative_active_electrical_energy | 418 kWh | 41800 | 00480000A348 |
| cumulative_water_volume | 1.234 m3 | 1234 | 0049000004D2 |
| water_volume_flow_rate | 12.345 L/s | 12345 | 004A00003039 |

Required additional tests: zero; one scale step; exact half-step rounding according to the contract; encoded minimum and maximum; one value below/above each valid range; negative values for monotonic counters; unknown ID; exact round-trip bytes. For each descriptor, the test must assert both encoded integer and six wire bytes.

## Counter lifecycle proposal

A counter reset, rollover, meter replacement, or correction must not silently look like a negative consumption delta. Consumers should detect a lower reading and mark the interval as discontinuous/invalid for ordinary delta calculation unless a separately defined source event explains the transition. The descriptor itself remains unchanged. The Semantic Record or source lifecycle metadata should identify reset/replacement/correction and, where available, counter epoch or meter identity. Do not infer rollover by arithmetic alone unless the meter's modulus and rollover behavior are explicitly known. Delta computation across an unexplained decrease must fail closed or be flagged, never silently wrap.

## Open project decisions

1. Approve allocation policy and final IDs; 0x0048–0x004A are placeholders only.
2. Confirm energy resolution against real meter output (0.01 kWh is the current candidate, not a final decision).
3. Confirm canonical units and reported resolutions from target water meters; m3 and L/s are provisional pending that review.
4. Select the flow device class and valid encoded range; do not automatically treat all int32 values as valid.
5. Decide how unit IRIs are represented. If this changes the corpus document schema, increment schema_version.
6. Define source/counter epoch metadata and conformance tests for resets, rollovers, replacement, and correction.

## QUDT references for review

- http://qudt.org/vocab/quantitykind/ActiveEnergy
- http://qudt.org/vocab/unit/KiloW-HR
- http://qudt.org/vocab/quantitykind/Volume
- http://qudt.org/vocab/unit/M3
- http://qudt.org/vocab/quantitykind/VolumeFlowRate
- http://qudt.org/vocab/unit/L-PER-SEC
