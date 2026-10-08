# OMWEI IoT

OMWEI IoT is an open, transport-independent semantic representation for compact machine-to-machine exchange of IoT observations.

The project separates semantic meaning, compact representation, and transport.

## Semantic Core

The initial semantic core is intentionally small:

1. A **Semantic Descriptor ID** identifies the meaning and encoding of a value.
2. A **Canonical Value** carries the observation value in the encoding declared by the registry.
3. The **Semantic Registry** binds the compact ID to a machine-readable semantic definition.
4. The representation is **transport-independent**.

Example: `0x0042 | 22500` can represent ambient temperature when registry entry `0x0042` defines temperature in degree Celsius using signed int32 with scale 0.001.

## Registry

OMWEI does not attempt to replace established semantic standards. The registry may bind an OMWEI descriptor to QUDT, SOSA/SSN, industry, or vendor vocabularies.

The compact OMWEI ID is a wire-level identifier. Registry identifiers are immutable and must not be reused for another meaning.

## Transport independence

OMWEI is not a transport protocol. A semantic record may be carried by Zenoh, MQTT, CAN, or another transport.

Zenoh may be used as a reference transport, but OMWEI defines what a value means, not how it is routed.

## Historical context

Earlier OMWEI work explored fixed-width 16-byte semantic atoms and a global predicate vocabulary for industrial IoT. Those experiments motivate this repository but are not the normative specification. The historical 16-byte target is a benchmark point, not a current format constraint.

## Status

Early technical development. The first milestone is the semantic core, registry, and comparative benchmark against MQTT/JSON, MQTT/CBOR, Zenoh/JSON, and Zenoh/CBOR.
