use serde_yaml::Value;
use std::collections::HashSet;

/// Validate an OMWEI corpus deterministically without network access.
pub fn validate_corpus(yaml: &str) -> Result<(), String> {
    let root: Value = serde_yaml::from_str(yaml).map_err(|e| format!("invalid YAML: {e}"))?;
    let map = root.as_mapping().ok_or("corpus root must be a mapping")?;
    nonempty(map, "registry_id")?;
    let schema = nonempty(map, "schema_version")?;
    if schema != "1" { return Err(format!("unsupported schema_version: {schema}")); }
    let corpus_version = nonempty(map, "corpus_version")?;
    if !is_semver(&corpus_version) {
        return Err(format!("corpus_version must be MAJOR.MINOR.PATCH: {corpus_version}"));
    }
    let descriptors = map.get(&Value::String("descriptors".into())).and_then(Value::as_sequence)
        .ok_or("descriptors must be a sequence")?;
    if descriptors.is_empty() { return Err("descriptors must not be empty".into()); }
    let mut ids = HashSet::new();
    for (i, item) in descriptors.iter().enumerate() {
        let p = format!("descriptors[{i}]");
        let d = item.as_mapping().ok_or_else(|| format!("{p} must be a mapping"))?;
        let idv = d.get(&Value::String("id".into())).ok_or_else(|| format!("{p}.id is required"))?;
        let idtext = idv.as_str()
            .ok_or_else(|| format!("{p}.id must be a quoted hexadecimal string such as \"0x0042\""))?;
        let hex = idtext.strip_prefix("0x").or_else(|| idtext.strip_prefix("0X")).unwrap_or(idtext);
        let id = u16::from_str_radix(hex, 16)
            .map_err(|_| format!("{p}.id must be a 16-bit hexadecimal ID"))?;
        if !ids.insert(id) { return Err(format!("duplicate descriptor ID: 0x{id:04X}")); }

        let namespace = desc_string(d, "namespace", &p)?;
        if !["core", "industry", "vendor", "private"].contains(&namespace.as_str()) {
            return Err(format!("{p}.namespace is not supported: {namespace}"));
        }
        let status = desc_string(d, "status", &p)?;
        if !["active", "deprecated", "retired"].contains(&status.as_str()) {
            return Err(format!("{p}.status is not supported: {status}"));
        }
        desc_string(d, "label", &p)?;
        let semantic = child_map(d, "semantic", &p)?;
        nonempty(semantic, "quantity_kind").map_err(|e| format!("{p}.{e}"))?;
        if let Some(mappings) = semantic.get(&Value::String("mappings".into())) {
            let mappings = mappings.as_sequence().ok_or_else(|| format!("{p}.semantic.mappings must be a sequence"))?;
            for (j, m) in mappings.iter().enumerate() {
                let m = m.as_mapping().ok_or_else(|| format!("{p}.semantic.mappings[{j}] must be a mapping"))?;
                for key in ["vocabulary", "concept"] {
                    m.get(&Value::String(key.into())).and_then(Value::as_str).filter(|s| !s.trim().is_empty())
                        .ok_or_else(|| format!("{p}.semantic.mappings[{j}].{key} is required"))?;
                }
            }
        }
        let unit = child_map(d, "unit", &p)?;
        nonempty(unit, "system").map_err(|e| format!("{p}.{e}"))?;
        nonempty(unit, "canonical").map_err(|e| format!("{p}.{e}"))?;
        let enc = child_map(d, "encoding", &p)?;
        for (key, expected) in [("datatype", "int32"), ("byte_order", "big_endian")] {
            let actual = enc.get(&Value::String(key.into())).and_then(Value::as_str).ok_or_else(|| format!("{p}.encoding.{key} is required"))?;
            if actual != expected { return Err(format!("{p}.encoding.{key} must be {expected}")); }
        }
        if enc.get(&Value::String("signed".into())).and_then(Value::as_bool) != Some(true) {
            return Err(format!("{p}.encoding.signed must be true for int32"));
        }
        let scale_text = enc.get(&Value::String("scale".into())).and_then(Value::as_str)
            .ok_or_else(|| format!("{p}.encoding.scale must be a quoted base-10 decimal string"))?
            .to_owned();
        exact_scale_multiplier(&scale_text)
            .map_err(|e| format!("{p}.encoding.scale {e}"))?;
        enc.get(&Value::String("offset".into())).and_then(Value::as_i64).filter(|v| i32::try_from(*v).is_ok())
            .ok_or_else(|| format!("{p}.encoding.offset must fit int32"))?;
        let c = child_map(d, "constraints", &p)?;
        let min = int32(c, "min", &p)?;
        let max = int32(c, "max", &p)?;
        if min > max { return Err(format!("{p}.constraints.min must be <= max")); }
        d.get(&Value::String("version".into())).and_then(Value::as_u64).filter(|v| *v > 0)
            .ok_or_else(|| format!("{p}.version must be a positive integer"))?;
    }
    Ok(())
}

/// Return the exact positive integer R for a decimal scale where 1 / scale = R.
/// No floating-point conversion or tolerance is used. Exponents are deliberately
/// rejected so the corpus has one simple, auditable decimal representation.
pub fn exact_scale_multiplier(scale: &str) -> Result<i32, String> {
    if scale.is_empty() || scale.starts_with('-') || scale.starts_with('+') {
        return Err("must be a positive plain decimal with an integer reciprocal".into());
    }
    let mut parts = scale.split('.');
    let whole = parts.next().unwrap_or("");
    let fraction = parts.next().unwrap_or("");
    if parts.next().is_some()
        || whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || (scale.contains('.') && fraction.is_empty())
    {
        return Err("must be a positive plain decimal with an integer reciprocal".into());
    }
    let digits = format!("{whole}{fraction}");
    let numerator: u128 = digits.parse().map_err(|_| "is too large for exact decimal validation".to_string())?;
    let denominator = 10_u128.checked_pow(fraction.len() as u32)
        .ok_or_else(|| "has excessive decimal precision".to_string())?;
    if numerator == 0 || denominator % numerator != 0 {
        return Err("must have an exact positive integer reciprocal".into());
    }
    let reciprocal = denominator / numerator;
    if reciprocal == 0 || reciprocal > i32::MAX as u128 {
        return Err("integer reciprocal must fit a positive int32".into());
    }
    Ok(reciprocal as i32)
}

fn is_semver(version: &str) -> bool {
    // SemVer 2.0.0: MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD].
    let (core_and_pre, build) = match version.split_once('+') {
        Some((left, right)) if !right.is_empty() && !right.contains('+') => (left, Some(right)),
        Some(_) => return false,
        None => (version, None),
    };
    let (core, prerelease) = match core_and_pre.split_once('-') {
        Some((left, right)) if !right.is_empty() => (left, Some(right)),
        Some(_) => return false,
        None => (core_and_pre, None),
    };
    let core_parts: Vec<&str> = core.split('.').collect();
    if core_parts.len() != 3 || !core_parts.iter().all(|part| {
        !part.is_empty()
            && part.bytes().all(|b| b.is_ascii_digit())
            && (*part == "0" || !part.starts_with('0'))
    }) {
        return false;
    }
    let valid_identifiers = |value: &str, prerelease_mode: bool| {
        value.split('.').all(|identifier| {
            !identifier.is_empty()
                && identifier.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && !(prerelease_mode
                    && identifier.bytes().all(|b| b.is_ascii_digit())
                    && identifier.len() > 1
                    && identifier.starts_with('0'))
        })
    };
    prerelease.map_or(true, |v| valid_identifiers(v, true))
        && build.map_or(true, |v| valid_identifiers(v, false))
}

fn nonempty(m: &serde_yaml::Mapping, key: &str) -> Result<String, String> {
    m.get(&Value::String(key.into())).and_then(Value::as_str).filter(|s| !s.trim().is_empty())
        .map(str::to_owned).ok_or_else(|| format!("{key} must be a non-empty string"))
}
fn desc_string(m: &serde_yaml::Mapping, key: &str, p: &str) -> Result<String, String> {
    nonempty(m, key).map_err(|e| format!("{p}.{e}"))
}
fn child_map<'a>(m: &'a serde_yaml::Mapping, key: &str, p: &str) -> Result<&'a serde_yaml::Mapping, String> {
    m.get(&Value::String(key.into())).and_then(Value::as_mapping).ok_or_else(|| format!("{p}.{key} is required"))
}
fn int32(m: &serde_yaml::Mapping, key: &str, p: &str) -> Result<i32, String> {
    m.get(&Value::String(key.into())).and_then(Value::as_i64).filter(|v| i32::try_from(*v).is_ok()).map(|v| v as i32)
        .ok_or_else(|| format!("{p}.constraints.{key} must fit int32"))
}

#[cfg(test)]
mod tests {
    use super::*;
    const VALID: &str = r#"
registry_id: org.omwei.core
schema_version: "1"
corpus_version: "0.1.0"
descriptors:
  - id: "0x0042"
    namespace: core
    status: active
    version: 1
    label: ambient_temperature
    semantic:
      quantity_kind: temperature
      mappings:
        - vocabulary: QUDT
          concept: Temperature
    unit:
      system: SI
      canonical: degree_Celsius
    encoding:
      datatype: int32
      signed: true
      byte_order: big_endian
      scale: "0.001"
      offset: 0
    constraints:
      min: -80000
      max: 150000
"#;
    #[test] fn accepts_valid_corpus() { assert!(validate_corpus(VALID).is_ok()); }
    #[test] fn rejects_unquoted_descriptor_ids() {
        let corpus = VALID.replace("id: \"0x0042\"", "id: 0x0042");
        assert!(validate_corpus(&corpus).unwrap_err().contains("quoted hexadecimal string"));
    }
    #[test] fn rejects_duplicate_ids() {
        let (header, descriptor) = VALID.split_once("descriptors:\n").unwrap();
        let invalid = format!("{header}descriptors:\n{descriptor}{descriptor}");
        assert!(validate_corpus(&invalid).unwrap_err().contains("duplicate descriptor ID"));
    }
    #[test] fn rejects_unknown_status() {
        assert!(validate_corpus(&VALID.replace("status: active", "status: maybe")).unwrap_err().contains("status is not supported"));
    }
    #[test] fn rejects_invalid_range() {
        assert!(validate_corpus(&VALID.replace("min: -80000", "min: 200000")).unwrap_err().contains("min must be <= max"));
    }
    #[test] fn rejects_zero_scale() {
        assert!(validate_corpus(&VALID.replace("scale: "0".001", "scale: 0")).unwrap_err().contains("integer reciprocal"));
    }
    #[test] fn rejects_approximate_reciprocal_scale() {
        assert!(validate_corpus(&VALID.replace("scale: 0.001", "scale: "0.3333333333"")).unwrap_err().contains("exact positive integer reciprocal"));
    }
    #[test] fn rejects_non_semver_corpus_version() {
        for version in ["0.1", "v0.1.0", "0.01.0", "0.1.0-", "0.1.0+"] {
            assert!(validate_corpus(&VALID.replace("corpus_version: \"0.1.0\"", &format!("corpus_version: \"{version}\""))).is_err());
        }
        assert!(validate_corpus(&VALID.replace("corpus_version: \"0.1.0\"", "corpus_version: \"0.1.0-beta.1+build.7\"")).is_ok());
    }
    #[test] fn rejects_unknown_schema() {
        assert!(validate_corpus(&VALID.replace("schema_version: \"1\"", "schema_version: \"2\"")).unwrap_err().contains("unsupported schema_version"));
    }
    #[test] fn exact_scale_examples() {
        assert_eq!(exact_scale_multiplier("0.001").unwrap(), 1000);
        assert_eq!(exact_scale_multiplier("0.25").unwrap(), 4);
        assert!(exact_scale_multiplier("0.3333333333").is_err());
        assert!(exact_scale_multiplier("1e-3").is_err());
    }
}
