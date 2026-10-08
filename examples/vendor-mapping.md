# Vendor Mapping Example

This example shows the interoperability boundary for the same physical observation represented by different vendor models.

## Inputs

Vendor A:

```json
{"T_AMB": 22500, "unit": "milli-Celsius"}
```

Vendor B:

```json
{"ambientTemp": 72.5, "unit": "Fahrenheit"}
```

These payloads are not byte-compatible and their field names are unrelated.

## Semantic mapping

```text
T_AMB
  + milli-Celsius
  ↓
0x0042 + 22500

ambientTemp
  + Fahrenheit
  ↓ canonicalization
0x0042 + 22500
```

The OMWEI boundary therefore produces the same semantic atom:

```text
Descriptor ID = 0x0042
Canonical value = 22500
Meaning = ambient temperature
Unit = degree_Celsius
```

## Result

Once canonicalized, the two vendor observations have the same OMWEI semantic identity and deterministic value representation.

The vendor-specific JSON remains useful at the source boundary, but it does not need to travel through the OMWEI data plane.
