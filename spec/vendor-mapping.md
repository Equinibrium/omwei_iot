# Vendor Mapping

OMWEI semantic interoperability begins at the boundary between a local/vendor data model and the shared OMWEI semantic registry.

A vendor field name is **not** an OMWEI semantic identifier.

For example:

| Vendor | Local field | OMWEI descriptor |
|---|---|---:|
| Vendor A | `T_AMB` | `0x0042` |
| Vendor B | `ambientTemp` | `0x0042` |
| Vendor C | `air_temperature` | `0x0042` |
| Vendor D | `temp_air` | `0x0042` |

The mapping is semantic, not a string substitution.

## Mapping requirements

A mapping MUST establish:

1. the source/vendor vocabulary;
2. the source field or concept;
3. the source unit and representation;
4. the OMWEI Descriptor ID;
5. the canonical unit;
6. the conversion/canonicalization rule;
7. any assumptions or loss of precision.

A mapping MUST NOT silently equate fields whose meanings differ.

## Example

Vendor A:

```text
T_AMB = 22500
unit = milli-Celsius
```

Vendor B:

```text
ambientTemp = 72.5
unit = Fahrenheit
```

Both can map to:

```text
Descriptor: 0x0042
Canonical unit: degree_Celsius
Canonical value: 22500
```

The result is interoperable because both source observations resolve to the same registry-defined meaning and canonical value.

## Boundary model

```text
Vendor data model
      |
      | semantic mapping
      v
OMWEI Descriptor + Canonical Value
      |
      | compact encoding
      v
Transport payload
```

The vendor mapping layer is outside the smallest OMWEI atom. The atom carries the result of canonicalization, not the vendor-specific schema.

## What this does not claim

OMWEI does not automatically discover that two arbitrary vendor fields have the same meaning.

Semantic equivalence must be established by a declared mapping, registry definition, or other controlled semantic process.

This distinction is essential: compact encoding solves representation efficiency; the registry and mappings establish interoperability.
