# OMWEI Semantic Core

## 1. Purpose

The OMWEI Semantic Core defines the minimum information required for **semantic interoperability** between heterogeneous IoT devices and systems.

Its purpose is to allow different implementations to exchange an observation or event using a shared semantic identity, without repeatedly carrying vendor-specific textual descriptions of that meaning in the data plane.

The core deliberately separates:

- semantic meaning;
- compact deterministic representation;
- observation context;
- transport.

It does not define a transport, security protocol, ontology, authorization model, or execution-enforcement mechanism.

## 2. Core model

An OMWEI **Semantic Atom** consists of:

`Semantic Descriptor ID + Canonical Value`

The descriptor identifies the semantic meaning and the rules required to interpret the value.

The registry definition associated with the descriptor supplies, as applicable:

- semantic identity;
- quantity kind or event meaning;
- canonical unit;
- datatype and width;
- scale and offset;
- validity constraints;
- version and lifecycle status;
- mappings to external semantic standards.

The two fields are intentionally small because the semantic description is resolved through the registry rather than repeatedly transmitted with every observation.

## 3. Semantic Descriptor ID

A Semantic Descriptor ID is a compact numeric identifier assigned by the OMWEI registry.

For interoperability, communicating systems MUST use the same registry definition for a descriptor. An identifier is unique within that registry namespace, MUST NOT be reused for a different meaning, and MUST remain resolvable after deprecation.

A replacement semantic definition receives a new identifier.

The descriptor is a compact wire identifier. It is not itself a complete ontology identifier; the registry provides the semantic definition and may bind it to established external vocabularies.

## 4. Canonical Value

The value is encoded according to the descriptor definition.

The registry MUST specify, where relevant:

- signedness;
- integer or floating-point representation;
- width;
- byte order;
- scale;
- offset;
- rounding;
- overflow behavior;
- invalid/unavailable representation;
- valid range.

Fixed-width integer or fixed-point representations are preferred where practical because they support deterministic decoding and compact implementations.

A receiver MUST be able to decode the value without guessing its representation.

## 5. Registry binding

The registry is part of the interoperability mechanism. It is not merely documentation.

A descriptor MUST resolve to an immutable semantic definition for the applicable registry version.

The registry SHOULD map OMWEI descriptors to established semantic vocabularies where appropriate, including QUDT, SOSA/SSN, industry vocabularies, and vendor vocabularies.

OMWEI therefore provides a compact binding layer between wire representation and established semantic meaning; it does not attempt to replace those vocabularies with a new ontology.

## 6. Observation context

Source identity, location, timestamp, quality, sequence information, and similar context are not required inside every Semantic Atom.

This is intentional. Repeating context in every compact value would increase the data-plane representation and undermine the minimal-atom design.

Context MAY instead be carried by:

- a Semantic Record envelope;
- a transport/session context;
- a resource or topic identity;
- another explicitly defined context mechanism.

For example, a Zenoh resource path may identify a source while the OMWEI payload carries only descriptor and value.

Context MUST NOT silently change the semantic definition of the descriptor.

## 7. Semantic Atom vs Semantic Record

A **Semantic Atom** is the minimum interoperable data-plane unit:

`Descriptor ID + Canonical Value`

A **Semantic Record** may additionally contain:

- source;
- timestamp;
- sequence number;
- location;
- quality/status;
- integrity or authentication metadata;
- other observation context.

This separation allows the repeated semantic meaning and repeated context to remain outside the smallest data-plane representation where appropriate.

## 8. Complex values

Vectors, GPS coordinates, waveforms, images, strings, and structured events MAY use extension representations.

An extension MUST preserve:

1. an explicit semantic descriptor;
2. an unambiguous value encoding;
3. deterministic interpretation rules.

An extension MUST NOT make the base semantic identity ambiguous.

## 9. Semantic equivalence

OMWEI interoperability is semantic, not merely byte-level.

Two implementations are interoperable when they resolve the same descriptor to the same registry-defined meaning and apply the same canonical interpretation rules to the value.

Equivalent source representations MAY be normalized to the same canonical value. For example, if Celsius is canonical, 72.5 °F may be converted to 22.5 °C before being represented as the canonical OMWEI value.

Byte equality alone is insufficient to establish semantic interoperability.

## 10. Compactness and parsing

Compactness is an implementation property supporting the interoperability goal.

By moving stable semantic descriptions into the registry, the repeated data-plane representation can avoid carrying textual field names, units, and type descriptions.

This can reduce:

- textual parsing;
- repeated semantic interpretation;
- dynamic allocation;
- memory copying;
- wire size;
- CPU work.

A fixed-width deterministic representation can additionally support direct decoding and low-overhead embedded implementations.

Reduced computation, memory activity, and transmission size may reduce energy consumption on constrained devices.

## 11. Transport independence

OMWEI does not depend on a particular transport.

Reference transports may include Zenoh, MQTT, CAN, or other byte- or message-oriented transports.

A transport binding MUST NOT change the semantic meaning of an OMWEI record.

## 12. Non-goals

The Semantic Core does not define:

- a transport protocol;
- authorization;
- policy evaluation;
- execution authority;
- hardware enforcement;
- trusted execution environments;
- cryptographic signature algorithms;
- consensus;
- global governance.

These concerns may exist in systems using OMWEI, but they are outside the IoT semantic core.

## 13. Initial implementation target

The first implementation should compare:

- MQTT + JSON;
- MQTT + CBOR;
- Zenoh + JSON;
- Zenoh + CBOR;
- Zenoh + OMWEI.

The comparison should measure:

- payload size;
- total wire size;
- encode/decode latency;
- CPU cost;
- allocations;
- memory use;
- throughput;
- end-to-end latency;
- suitability for constrained devices;
- energy consumption where measurable.

The historical 128-bit / 16-byte atom is a benchmark target rather than a mandatory current format.
