# OMWEI IoT

OMWEI IoT is a semantic interoperability layer for heterogeneous machines.

Its purpose is to let devices and systems exchange observations and events using a shared, stable semantic identity rather than repeatedly exchanging and parsing vendor-specific textual descriptions of meaning.

The project separates **semantic meaning**, **compact representation**, and **transport**.

## Why OMWEI IoT

The primary problem is **semantic interoperability** between heterogeneous IoT devices and systems.

A device may use a vendor-specific internal model such as `T_AMB`, while another system may call the same observation `ambient_temperature`. OMWEI provides a stable semantic identifier that both can resolve through a shared registry.

The key idea is:

```text
Shared semantic meaning
        ↓
Stable OMWEI Descriptor ID
        ↓
Compact deterministic representation
        ↓
Transport
```

The compact representation is an implementation mechanism, not the primary purpose. Because semantic descriptions do not need to be repeatedly carried and parsed in the data plane, OMWEI can also reduce parsing, interpretation, allocation, copying, computation, and wire overhead. These reductions can lower CPU, bandwidth, and energy consumption.

## Semantic Core

The initial semantic core is intentionally small:

1. A **Semantic Descriptor ID** identifies a stable registry-defined meaning.
2. A **Canonical Value** carries the observation value according to the registry-defined encoding.
3. The **Semantic Registry** defines the meaning and deterministic interpretation of the descriptor and value.
4. The representation is **transport-independent**.

For example:

```text
0x0042 | 22500
```

may represent:

```text
0x0042  → ambient temperature
22500   → 22.500 °C
```

The receiver does not need a textual field name and unit in every observation. It resolves `0x0042` through the registry and applies the declared datatype, scale, offset, and constraints.

## Interoperability

OMWEI interoperability is **semantic**, not merely byte-level.

Two systems are interoperable when they can resolve the same OMWEI Descriptor ID to the same defined meaning and interpret its value according to the same canonical rules.

For example:

```text
Vendor A                    Vendor B
"T_AMB"                     "ambientTemp"
   │                              │
   └──────────┐        ┌──────────┘
              ▼        ▼
             OMWEI Descriptor
                  0x0042
                     │
                     ▼
             ambient temperature
                  22.500 °C
```

The vendors may retain completely different internal data models. The shared semantic identity exists at the interoperability boundary.

## Registry

The registry is part of the semantic interoperability mechanism; it is not merely documentation.

A registry entry defines, as applicable:

- semantic identity and status;
- quantity kind or event meaning;
- canonical unit;
- datatype and width;
- signedness and byte order;
- scale and offset;
- rounding and overflow behavior;
- invalid or unavailable representation;
- validity constraints;
- version and lifecycle;
- mappings to established semantic vocabularies.

OMWEI does not attempt to replace established semantic standards. A registry entry may map an OMWEI descriptor to QUDT, SOSA/SSN, industry vocabularies, or vendor vocabularies.

Descriptor IDs are immutable and must not be reused for a different meaning. Deprecated IDs remain resolvable.

## Compact Representation

The data-plane representation is intentionally compact and deterministic.

The design aims to minimize:

- textual parsing;
- repeated semantic interpretation;
- dynamic allocation;
- unnecessary copying;
- wire overhead.

A deterministic fixed-width representation also makes direct decoding and low-overhead implementations possible, including embedded and hardware-oriented implementations.

## Transport Independence

OMWEI is **not a transport protocol** and does not replace MQTT, Zenoh, CAN, or other transports.

A transport carries the OMWEI representation:

```text
              OMWEI semantic representation
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
            MQTT       Zenoh        CAN
```

OMWEI defines what the exchanged value means and how it is deterministically represented; the underlying transport defines how it is delivered.

## Historical Context

Earlier OMWEI work repeatedly explored a fixed **128-bit / 16-byte semantic atom** and compact predicate identifiers for industrial IoT.

The historical record includes different 16-byte layouts, so the exact historical layout is not frozen here. The stable design invariant is the combination of:

- compact semantic identity;
- deterministic value representation;
- fixed-width / bounded representation;
- registry-defined meaning;
- transport independence.

The historical 16-byte target is therefore retained as a design and performance benchmark, not as a normative current format constraint.

## Non-Goals

OMWEI IoT does not define:

- a transport protocol;
- authorization or execution authority;
- runtime governance;
- hardware enforcement;
- a TEE requirement;
- a cryptographic algorithm;
- consensus or global governance.

These concerns may exist in systems that use OMWEI, but they are outside the IoT semantic core.

## Status

Early technical development.

The initial milestone is to establish the semantic core and registry, then demonstrate interoperability and measure the cost of alternative representations:

- MQTT/JSON
- MQTT/CBOR
- Zenoh/JSON
- Zenoh/CBOR
- Zenoh/OMWEI

The benchmark should measure not only bytes on the wire, but also parsing/decoding work, allocations, CPU cost, latency, and energy where measurable.
