use crate::{descriptor, Descriptor, SemanticAtom, AMBIENT_TEMPERATURE_ID};

/// Resolve a semantic mapping to the descriptor defined by the shared registry.
/// Vendor field names are deliberately not part of this API.
pub fn resolve_descriptor(id: u16) -> Result<&'static Descriptor, &'static str> {
    descriptor(id).ok_or("semantic mapping references unknown descriptor")
}

/// Construct an atom only after the mapping has been resolved through the registry.
pub fn map_canonical_value(id: u16, canonical_value: i32) -> Result<SemanticAtom, &'static str> {
    let d = resolve_descriptor(id)?;
    SemanticAtom::new(d.id, canonical_value)
}

/// Reference mapping used by the interoperability example.
pub fn ambient_temperature_descriptor() -> Result<&'static Descriptor, &'static str> {
    resolve_descriptor(AMBIENT_TEMPERATURE_ID)
}
