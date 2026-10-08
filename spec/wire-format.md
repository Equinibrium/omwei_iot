# OMWEI Minimal Wire Encoding

This document defines the first executable OMWEI data-plane encoding.

It is intentionally minimal. Framing, routing, delivery, ordering, and transport concerns remain the responsibility of the underlying transport.

## 1. Logical payload

The minimal OMWEI Semantic Atom is:

```text
Descriptor ID + Canonical Value
```

For the current corpus prototype:

```text
Descriptor ID : uint16
Canonical Value: int32
Byte order: big-endian
Total: 6 bytes
```

The 6-byte encoding is the current executable baseline. It is **not yet declared a long-term frozen wire-format version**.

## 2. Byte layout

```text
byte 0       byte 1       byte 2 ........ byte 5
+------------+------------+----------------+
| Descriptor ID (uint16)  | Canonical Value|
|        big-endian       |    int32 BE    |
+------------+------------+----------------+
```

For ambient temperature:

```text
Descriptor = 0x0042
Value      = 22500
Meaning    = 22.500 °C
```

The bytes are:

```text
00 42 00 00 57 E4
```

## 3. No self-describing semantic text

The payload does not repeat:

- field name;
- unit name;
- datatype name;
- scale;
- ontology URI.

Those are resolved from the applicable OMWEI registry.

This is the mechanism that moves stable semantic description out of the repeated data plane.

## 4. Transport framing

The 6-byte atom is a payload, not a transport protocol.

MQTT, Zenoh, CAN, or another transport may add its own routing/framing metadata. Such metadata is not part of the OMWEI semantic atom.

A transport binding MUST preserve the atom bytes and MUST NOT change their semantic interpretation.

## 5. Validation

A decoder MUST reject:

- payloads with an invalid length;
- unknown Descriptor IDs;
- values outside registry constraints;
- ambiguous or otherwise invalid encodings.

A decoder MUST NOT guess the descriptor or value representation.

## 6. Historical 128-bit target

Earlier OMWEI implementations explored a 128-bit / 16-byte atom.

The current 6-byte logical encoding deliberately separates that historical target from the semantic definition. A future 16-byte optimized encoding can be benchmarked without changing the semantic model.

## 7. Benchmark question

The wire-format experiment should compare the same semantic observation using:

```text
JSON
CBOR
OMWEI 6-byte logical atom
OMWEI 16-byte historical target
```

Payload size alone is insufficient. Decode cost, allocations, CPU/cycles, latency, and energy should also be measured where practical.
