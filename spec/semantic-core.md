# OMWEI Semantic Core

## 1. Purpose

The OMWEI Semantic Core defines the minimum information required for a compact, interoperable representation of an IoT observation.

It deliberately does not define a transport, security protocol, ontology, authorization model, or execution-enforcement mechanism.

## 2. Core model

An OMWEI semantic value consists of:

`Semantic Descriptor ID + Canonical Value`

The descriptor identifies how the value is to be interpreted.

The registry definition associated with the descriptor supplies semantic identity, quantity kind or event meaning, canonical unit where applicable, datatype and width, scale and offset where applicable, validity constraints, version, lifecycle status, and mappings to external semantic standards.

## 3. Semantic Descriptor ID

A Semantic Descriptor ID is a compact numeric identifier assigned by the OMWEI registry.

Identifiers are unique within the registry, never reused for a different meaning, and remain resolvable after deprecation. A replacement descriptor receives a new identifier.

The descriptor is a compact wire identifier, not a globally universal ontology identifier.

## 4. Canonical Value

The value is encoded according to the descriptor definition.

The registry MUST specify, where relevant: signedness, integer width, byte order, scale, offset, rounding, overflow behavior, invalid/unavailable representation, and valid range.

Fixed-width integer or fixed-point representations are preferred for deterministic compact encoding where practical.

## 5. Registry binding

A descriptor MUST resolve to an immutable semantic definition.

The registry SHOULD map to established vocabularies where an appropriate concept exists, including QUDT and SOSA/SSN.

The OMWEI registry is a binding and lookup layer, not a replacement ontology.

## 6. Observation context

Source identity, location, timestamp, quality, sequence information, and similar context are not required inside every Semantic Atom.

They MAY be carried by the transport or a Semantic Record envelope. For example, a Zenoh resource path may identify a source while the OMWEI payload carries only descriptor and value.

## 7. Semantic Atom vs Semantic Record

A **Semantic Atom** is the minimal compact semantic value: descriptor plus value.

A **Semantic Record** may additionally contain source, timestamp, sequence number, quality/status, integrity/authentication metadata, and other context.

## 8. Complex values

Vectors, GPS coordinates, waveforms, images, strings, and structured events MAY use extension representations. Extensions MUST preserve an explicit semantic descriptor and unambiguous value encoding.

## 9. Semantic equivalence

Interoperability is not defined solely by byte equality. A conforming implementation MUST decode an OMWEI value into the meaning declared by its registry entry.

Where canonicalization is defined, equivalent source representations SHOULD resolve to the same canonical semantic value. For example, 72.5 °F may resolve to 22.5 °C when Celsius is the registry canonical unit.

## 10. Transport independence

OMWEI does not depend on a particular transport. Reference transports may include Zenoh, MQTT, CAN, or other byte- or message-oriented transports.

A transport binding MUST NOT change the semantic meaning of an OMWEI record.

## 11. Non-goals

The Semantic Core does not define authorization, policy evaluation, execution authority, hardware enforcement, trusted execution environments, cryptographic signature algorithms, consensus, or global governance.

## 12. Initial implementation target

The first implementation should compare MQTT + JSON, MQTT + CBOR, Zenoh + JSON, Zenoh + CBOR, and Zenoh + OMWEI.

Metrics should include payload and total wire size, encode/decode latency, CPU cost, allocations, memory use, throughput, end-to-end latency, and suitability for constrained devices.

The historical 16-byte atom is a benchmark target rather than a mandatory current format.
