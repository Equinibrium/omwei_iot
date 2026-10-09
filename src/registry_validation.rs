use serde_yaml::Value;
use std::collections::HashSet;

/// Validate an OMWEI corpus deterministically without network access.
pub fn validate_corpus(yaml: &str) -> Result<(), String> {
    let root: Value = serde_yaml::from_str(yaml).map_err(|e| format!("invalid YAML: {e}"))?;
    let map = root.as_mapping().ok_or("corpus root must be a mapping")?;
    nonempty(map, "registry_id")?;
    let schema = nonempty(map, "schema_version")?;
    if schema != "1" { return Err(format!("unsupported schema_version: {schema}")); }
    nonempty(map, "corpus_version")?;
    let descriptors = map.get(&Value::String("descriptors".into())).and_then(Value::as_sequence)
        .ok_or("descriptors must be a sequence")?;
    if descriptors.is_empty() { return Err("descriptors must not be empty".into()); }
    let mut ids = HashSet::new();
    for (i, item) in descriptors.iter().enumerate() {
        let p = format!("descriptors[{i}]");
        let d = item.as_mapping().ok_or_else(|| format!("{p} must be a mapping"))?;
        let idv = d.get(&Value::String("id".into())).ok_or_else(|| format!("{p}.id is required"))?;
        let idtext = match idv { Value::String(s) => s.clone(), Value::Number(n) => n.to_string(), _ => return Err(format!("{p}.id must be a string or integer")) };
        let idtext = idtext.strip_prefix("0x").unwrap_or(&idtext);
        let id = u16::from_str_radix(idtext, 16).map_err(|_| format!("{p}.id must be a 16-bit hexadecimal ID"))?;
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
        let scale = enc.get(&Value::String("scale".into())).and_then(Value::as_f64).ok_or_else(|| format!("{p}.encoding.scale must be numeric"))?;
        if !scale.is_finite() || scale <= 0.0 { return Err(format!("{p}.encoding.scale must be finite and greater than zero")); }
        // The current generated Descriptor stores the reciprocal scale as i32.
        // Reject values that cannot be represented exactly by that implementation.
        let reciprocal = 1.0 / scale;
        if !reciprocal.is_finite()
            || reciprocal < 1.0
            || reciprocal > i32::MAX as f64
            || (reciprocal - reciprocal.round()).abs() > 1e-9
        {
            return Err(format!("{p}.encoding.scale must have an integer reciprocal representable as i32"));
        }
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
      scale: 0.001
      offset: 0
    constraints:
      min: -80000
      max: 150000
"#;
    #[test] fn accepts_valid_corpus() { assert!(validate_corpus(VALID).is_ok()); }
    #[test] fn rejects_duplicate_ids() {
        let (header, descriptor) = VALID.split_once("descriptors:\\n").unwrap();
        let invalid = format!("{header}descriptors:\\n{descriptor}{descriptor}");
        assert!(validate_corpus(&invalid).unwrap_err().contains("duplicate descriptor ID"));
    }
    #[test] fn rejects_unknown_status() {
        assert!(validate_corpus(&VALID.replace("status: active", "status: maybe")).unwrap_err().contains("status is not supported"));
    }
    #[test] fn rejects_invalid_range() {
        assert!(validate_corpus(&VALID.replace("min: -80000", "min: 200000")).unwrap_err().contains("min must be <= max"));
    }
    #[test] fn rejects_zero_scale() {
        assert!(validate_corpus(&VALID.replace("scale: 0.001", "scale: 0")).unwrap_err().contains("scale must be finite"));
    }
    #[test] fn rejects_unrepresentable_scale() {
        assert!(validate_corpus(&VALID.replace("scale: 0.001", "scale: 0.003")).unwrap_err().contains("integer reciprocal"));
    }
    #[test] fn rejects_unknown_schema() {
        assert!(validate_corpus(&VALID.replace("schema_version: \"1\"", "schema_version: \"2\"")).unwrap_err().contains("unsupported schema_version"));
    }
}
