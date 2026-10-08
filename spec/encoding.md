# OMWEI IoT Encoding

## 1. Purpose

The encoding layer defines how the semantic core is represented as bytes.

It is deliberately separate from semantic meaning. A registry defines what a Descriptor ID and Canonical Value mean; an encoding defines how those fields are serialized.

## 2. Design goals

An OMWEI encoding SHOULD support:

- deterministic decoding;
- bounded or fixed-width representation where practical;
- direct field access;
- minimal parsing;
- minimal allocation;
- minimal copying;
- compact wire representation;
- implementation on constrained devices.

These goals support the primary objective of semantic interoperability by making the shared semantic representation practical for high-volume and constrained data exchange.

## 3. Base semantic model

The logical content of a Semantic Atom is:

`Descriptor ID + Canonical Value`

The byte representation MUST preserve both elements unambiguously.

The encoding MUST NOT require a receiver to infer the datatype, scale, unit, byte order, or other interpretation rules from the value itself. Those rules come from the applicable registry definition.

## 4. Fixed-width target

Earlier OMWEI implementations repeatedly targeted a 128-bit / 16-byte atom.

The current semantic core does not freeze one historical 16-byte field layout. Different historical implementations used different layouts.

A future normative encoding MAY define a 16-byte representation if it can represent the required semantic core without ambiguity.

Until such an encoding is standardized, 16 bytes remains a benchmark target rather than a normative wire size.

## 5. Variable-length encodings

Variable-length representations MAY be used where required by complex values or transport constraints.

A variable-length encoding MUST still provide deterministic boundaries and interpretation.

A variable-length representation MUST NOT introduce textual semantic names into every observation merely to recover information that is already defined by the registry.

## 6. Canonical decoding

For a given registry version and encoded Semantic Atom, decoding MUST be deterministic.

The implementation MUST reject malformed, truncated, or otherwise ambiguous encodings rather than guessing.

## 7. Encoding is not transport

Encoding defines the byte representation of OMWEI content.

Transport defines delivery.

For example:

```text
OMWEI encoding
      │
      ├── MQTT
      ├── Zenoh
      ├── CAN
      └── other transport
```

A transport binding MUST NOT alter the semantic interpretation of the encoded value.

## 8. Performance measurement

Encoding performance MUST be measured rather than inferred.

The benchmark should compare OMWEI against representative alternatives such as JSON and CBOR under equivalent semantic content.

Relevant metrics include:

- encoded payload bytes;
- total wire bytes;
- encode latency;
- decode latency;
- CPU cycles or CPU time;
- allocations;
- memory traffic where measurable;
- throughput;
- end-to-end latency;
- energy consumption where measurable.

The purpose of these measurements is to establish whether compact deterministic representation produces practical CPU, bandwidth, and energy benefits on constrained systems.
