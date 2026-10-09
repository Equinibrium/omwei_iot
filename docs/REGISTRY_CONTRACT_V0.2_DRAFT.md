# OMWEI IoT Registry Contract v0.2 — Draft

**Status:** design draft for review  
**Scope:** registry identity, descriptor lifecycle, compatibility, numeric interpretation, and distribution  
**Non-goal:** defining a transport protocol or a rich industrial information model

This document defines the contract that independent OMWEI IoT implementations must follow. It is a draft: do not treat it as a released compatibility guarantee until the format and conformance tests are implemented and versioned together.

## 1. Registry and corpus identity

A registry is an authority and namespace for a defined set of semantic descriptors. A corpus is a versioned, distributable snapshot of that registry.

Every corpus MUST declare:

- a stable `registry_id` (for example, `org.omwei.core`);
- a `schema_version` identifying the registry document format;
- a `corpus_version` identifying the released corpus snapshot;
- the descriptor entries and their lifecycle status.

The document schema version and corpus release version are different concepts. A schema change does not automatically mean that descriptor meanings changed, and a corpus release can add descriptors without changing the schema.

A consumer MUST be able to identify the corpus snapshot it was built or configured against. An embedded consumer may compile the corpus into its firmware; it does not need a runtime registry service.

## 2. Descriptor identity and namespaces

The wire representation currently carries a 16-bit descriptor ID. The namespace is registry metadata; it is **not** implicitly encoded in those 16 bits.

- A descriptor ID MUST identify one meaning within the applicable globally shared allocation.
- An assigned ID MUST NOT be reused, even after deprecation or removal from the current core set.
- A descriptor's label may be corrected without changing identity only when the defined meaning remains unchanged.
- A change to the meaning, quantity kind, canonical unit, value interpretation, or constraints in a way that changes valid interpretation MUST receive a new ID.
- Two descriptors MUST NOT share an ID in the same distributable corpus.

Namespaces describe stewardship and intended scope, not permission to collide:

- **core** — small, stable, generally reusable concepts;
- **industry** — concepts maintained for a defined industry;
- **vendor** — vendor-defined concepts;
- **private/plant** — locally controlled concepts.

Because the current atom carries no namespace field, IDs from different namespaces still need non-overlapping assignments whenever atoms may cross those boundaries. A private ID MUST NOT be treated as globally interoperable until its allocation and definition are shared with every relevant participant. The initial implementation should use explicit registry allocation rules rather than assuming that a namespace string makes a 16-bit ID unique.

## 3. Descriptor lifecycle

Each descriptor MUST have a lifecycle status, at minimum:

- `active` — valid for new data;
- `deprecated` — retained and resolvable, but not recommended for new deployments;
- `retired` — no longer recommended for new use; historical meaning and decoding information remain available.

Unknown status values MUST fail validation. A descriptor ID and its historical definition MUST remain resolvable in published corpus history. Removal from a current distribution MUST NOT imply that the ID can be reassigned.

Lifecycle status does not itself alter the meaning of an atom. A consumer may apply local policy to deprecated or retired descriptors, but must not reinterpret them.

## 4. Versioning and compatibility

Use semantic versioning for corpus releases: `MAJOR.MINOR.PATCH`.

- **PATCH:** corrections to non-semantic metadata or documentation that do not change interpretation.
- **MINOR:** additive descriptors or additive optional metadata that preserve existing descriptor interpretations.
- **MAJOR:** changes that can cause an existing corpus consumer to interpret data differently, changes to mandatory schema semantics, or incompatible allocation/encoding changes.

A release MUST include a compatibility note describing the change. A minor release MUST preserve the IDs and interpretation of all descriptors already published in earlier compatible releases. A descriptor's own version is not a substitute for the corpus version.

Consumers SHOULD reject an unsupported major schema version. They MAY accept a newer compatible corpus release if the descriptors they use retain their exact definitions. For constrained devices, the supported corpus version may be fixed at build time and reported in device metadata or deployment configuration.

## 5. Canonical value and numeric interpretation

The current atom layout is six bytes:

| Field | Width | Encoding |
|---|---:|---|
| Descriptor ID | 16 bits | unsigned, big-endian |
| Encoded value | 32 bits | signed two's-complement integer, big-endian |

For the current encoding model, a descriptor's decimal `scale` and `offset` define the physical value as:

`physical_value = encoded_integer × scale + offset`

The encoded integer is the value carried in the atom. For example, descriptor `0x0042` uses scale `0.001`; integer `22500` therefore represents `22.500 degree_Celsius`.

The corpus MUST define the unit, scale, offset, valid encoded-integer range, and rounding/conversion rule for every descriptor. The current prototype's integer `min` and `max` constraints apply to the encoded integer unless a future schema explicitly distinguishes encoded-domain and physical-domain constraints.

Implementations MUST NOT rely on binary floating-point behavior to decide wire bytes. Conversion from external values to encoded integers must specify rounding and overflow behavior; the implementation must reject out-of-range values rather than silently wrap. The exact decimal representation and rounding rule are to be fixed by the conformance implementation before v0.2 is declared stable.

An unknown descriptor ID MUST be rejected or surfaced as unknown. A consumer MUST NOT guess a unit or infer meaning from the numeric ID or label alone.

## 6. Corpus validation requirements

A corpus validator MUST reject at least:

- malformed or unsupported schema versions;
- missing required fields;
- malformed IDs or IDs outside the supported 16-bit range;
- duplicate IDs;
- unknown lifecycle statuses;
- empty registry identity or corpus version;
- invalid ranges where `min > max`;
- zero or invalid scales;
- descriptors whose required semantic identity or canonical unit is missing;
- malformed external vocabulary mappings;
- changes that violate the declared compatibility policy.

Validation MUST be deterministic and must not depend on network access. External vocabulary links may be checked by a separate optional audit, but network availability MUST NOT determine whether a valid, self-contained corpus can be used offline.

## 7. Distribution and deployment

The registry is a data contract, not necessarily a service.

Supported distribution modes may include:

1. generated static tables compiled into firmware or applications;
2. a checked-in corpus file distributed with an application;
3. deployment/configuration delivery to gateways or edge nodes;
4. an optional registry service for discovery and update workflows.

Every deployment MUST ensure that producers and consumers use compatible definitions for the IDs they exchange. A registry service is optional; a shared and identifiable corpus is not.

## 8. Conformance

An implementation conforms only to the requirements it tests and declares. The initial conformance suite should verify:

1. identical descriptor ID resolves to identical semantic metadata in two independently loaded registry instances;
2. known encoded values decode to the same unit and physical value;
3. external-unit conversion uses the specified deterministic rounding and range checks;
4. the same six atom bytes retain the same meaning across transport adapters;
5. unknown IDs, duplicate IDs, malformed corpora, and incompatible definitions fail closed;
6. additive corpus releases preserve all previously published descriptor interpretations.

The test suite should use fixed test vectors checked into the repository. Property-based tests may supplement them but must not replace stable vectors.

## 9. Deliberate limits

This contract does not define MQTT, Zenoh, CAN, device discovery, authorization, execution authority, runtime governance, hardware enforcement, or a mandatory 16-byte atom. It defines the smallest shared semantic contract required to make exchanged machine data interpretable consistently.
