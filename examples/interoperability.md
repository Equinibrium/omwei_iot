# OMWEI IoT Interoperability Example

This example shows the primary purpose of OMWEI IoT: semantic interoperability between heterogeneous devices and systems.

## 1. Different vendor models

Assume two vendors measure the same physical quantity.

Vendor A calls the field `T_AMB`.
Vendor B calls it `ambientTemp`.

Their internal representations are unrelated.

```text
Vendor A                         Vendor B
T_AMB = 22.5 °C                  ambientTemp = 22.5 °C
     │                                  │
     └──────────────┐      ┌─────────────┘
                    ▼      ▼
                  OMWEI
                 0x0042
                    │
                    ▼
            ambient temperature
```

The interoperability boundary is the OMWEI semantic identifier, not the vendor field name.

## 2. Registry definition

Suppose the registry defines:

```yaml
id: 0x0042
label: ambient_temperature

semantic:
  quantity_kind: temperature

unit:
  canonical: degree_Celsius

encoding:
  datatype: int32
  signed: true
  byte_order: big_endian
  scale: 0.001
  offset: 0
```

The data-plane value can therefore be represented as:

```text
0x0042 | 22500
```

The receiver resolves the identifier and deterministically interprets the value:

```text
0x0042
  → ambient temperature
22500
  → int32 × 0.001
  → 22.500 °C
```

The textual semantic description does not have to be repeated in every observation.

## 3. Canonicalization

Different source units may be normalized before entering the OMWEI data plane.

```text
Vendor A
72.5 °F
   │
   ▼
canonicalization
   │
   ▼
22.5 °C
   │
   ▼
0x0042 | 22500
   │
   ▼
Vendor B
22.500 °C
```

The two vendors may continue using Fahrenheit and Celsius internally. OMWEI defines the canonical representation at the interoperability boundary.

## 4. What this demonstrates

### Semantic interoperability

Different vendor concepts can map to one shared semantic identity.

### Deterministic interpretation

The registry specifies how the compact value is decoded. A receiver does not guess the datatype, unit, scale, or byte order.

### Compact data-plane representation

Stable semantic information is resolved through the registry rather than repeatedly transmitted as textual field names, units, and type descriptions.

## 5. What it does not demonstrate

This example does not require:

- a new transport protocol;
- a new ontology;
- a specific messaging system;
- authorization;
- execution authority;
- hardware enforcement.

The same OMWEI representation can be carried over MQTT, Zenoh, CAN, or another suitable transport.

## 6. The interoperability test

A minimal implementation test should verify:

1. Vendor A's `T_AMB` maps to OMWEI descriptor `0x0042`.
2. Vendor B's `ambientTemp` maps to the same descriptor.
3. Both implementations resolve `0x0042` to the same registry definition.
4. `22500` is decoded as `22.500 °C`.
5. No vendor-specific field name is required in the OMWEI data-plane representation.
6. An equivalent source value such as `72.5 °F` canonicalizes to the same OMWEI value.

If any of these conditions fail, the systems are not interoperable at the OMWEI semantic boundary.
