# OMWEI IoT Interoperability Requirements

## 1. Scope

This document defines the minimum conditions for claiming semantic interoperability between two OMWEI IoT implementations.

Interoperability is established at the semantic boundary. It does not require identical vendor software, internal data models, or transports.

## 2. Shared semantic identity

Two implementations exchanging an OMWEI Semantic Atom MUST resolve its Descriptor ID against the same applicable registry definition.

A local alias such as `T_AMB` or `ambientTemp` is not itself an OMWEI semantic identity.

## 3. Deterministic interpretation

For every active descriptor, the applicable registry definition MUST provide enough information to decode the value without guessing.

Where applicable this includes:

- datatype;
- width;
- signedness;
- byte order;
- scale;
- offset;
- rounding;
- overflow behavior;
- invalid/unavailable representation;
- valid range.

Two conforming implementations using the same descriptor and canonical value MUST obtain the same semantic result.

## 4. Canonicalization

An implementation MAY convert an external representation into the registry-defined canonical representation before transmission.

For example:

```text
72.5 °F
   ↓
22.5 °C
   ↓
descriptor 0x0042 + canonical value 22500
```

Canonicalization rules MUST be deterministic. Equivalent source observations SHOULD resolve to the same canonical semantic value when the registry defines the required conversion.

## 5. Context

Context such as source, time, location, sequence, and quality MAY be carried outside the Semantic Atom.

Moving context outside the atom MUST NOT make the descriptor meaning ambiguous.

A receiver MUST be able to determine which context applies to an atom.

## 6. Transport

Interoperability MUST NOT depend on a particular transport.

The same semantic atom may be carried through MQTT, Zenoh, CAN, or another transport without changing its semantic interpretation.

Transport-specific metadata MUST NOT silently redefine an OMWEI descriptor.

## 7. Minimal interoperability test

An implementation pair passes the basic interoperability test if:

1. both resolve the same Descriptor ID to the same semantic definition;
2. both decode the canonical value identically;
3. both obtain the same semantic meaning;
4. equivalent external representations can be canonicalized consistently where conversion is defined;
5. transport changes do not change the semantic result.

Byte equality is neither necessary nor sufficient as the sole interoperability criterion.

## 8. Compactness is separate

An implementation can be semantically interoperable without being compact.

Compactness is an optimization enabled by stable semantic identity and deterministic encoding.

The OMWEI design goal is therefore:

```text
semantic interoperability
        ↓
stable registry identity
        ↓
deterministic canonical representation
        ↓
compact repeated data-plane exchange
        ↓
lower parsing / CPU / bandwidth / energy cost
```

Performance claims MUST be established by measurement rather than inferred from byte count alone.