use crate::Descriptor;
use serde::Deserialize;
use serde_yaml::Value;

include!(concat!(env!("OUT_DIR"), "/registry_generated.rs"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusMetadata {
    pub registry_id: String,
    pub schema_version: String,
    pub corpus_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDescriptor {
    pub id: u16,
    pub label: String,
    pub unit: String,
    pub scale: i32,
    pub offset: i32,
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCorpus {
    pub metadata: CorpusMetadata,
    pub descriptors: Vec<ParsedDescriptor>,
}

#[derive(Debug, Deserialize)]
struct CorpusFile {
    registry_id: String,
    schema_version: String,
    corpus_version: String,
    descriptors: Vec<CorpusDescriptor>,
}

#[derive(Debug, Deserialize)]
struct CorpusDescriptor {
    id: String,
    label: String,
    unit: CorpusUnit,
    encoding: CorpusEncoding,
    constraints: CorpusConstraints,
}

#[derive(Debug, Deserialize)]
struct CorpusUnit {
    canonical: String,
}

#[derive(Debug, Deserialize)]
struct CorpusEncoding {
    scale: Value,
    offset: i32,
}

#[derive(Debug, Deserialize)]
struct CorpusConstraints {
    min: i32,
    max: i32,
}

/// Parse and validate a corpus using the same validation rules as the build.
/// The returned descriptors own their strings; parsing does not leak memory.
pub fn parse_corpus(yaml: &str) -> Result<ParsedCorpus, String> {
    crate::registry_validation::validate_corpus(yaml)?;
    let corpus: CorpusFile = serde_yaml::from_str(yaml)
        .map_err(|e| format!("validated corpus could not be decoded: {e}"))?;

    let metadata = CorpusMetadata {
        registry_id: corpus.registry_id,
        schema_version: corpus.schema_version,
        corpus_version: corpus.corpus_version,
    };

    let descriptors = corpus.descriptors
        .into_iter()
        .map(|d| {
            let id = d.id.strip_prefix("0x")
                .and_then(|s| u16::from_str_radix(s, 16).ok())
                .ok_or_else(|| format!("invalid descriptor id: {}", d.id))?;
            let scale_text = d.encoding.scale.as_str()
                .ok_or_else(|| format!("descriptor 0x{id:04X} scale must be a quoted decimal string"))?;
            let scale = crate::registry_validation::exact_scale_multiplier(&scale_text)
                .map_err(|e| format!("descriptor 0x{id:04X} scale {e}"))?;

            Ok(ParsedDescriptor {
                id,
                label: d.label,
                unit: d.unit.canonical,
                scale,
                offset: d.encoding.offset,
                min: d.constraints.min,
                max: d.constraints.max,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(ParsedCorpus { metadata, descriptors })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Registry {
    descriptors: &'static [Descriptor],
}

pub const CORPUS_V0_1: Registry = Registry {
    descriptors: DESCRIPTORS,
};

impl Registry {
    pub const fn new(descriptors: &'static [Descriptor]) -> Self {
        Self { descriptors }
    }

    pub fn resolve(&self, id: u16) -> Option<&'static Descriptor> {
        self.descriptors.iter().find(|d| d.id == id)
    }

    pub fn contains(&self, id: u16) -> bool {
        self.resolve(id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_resolves_temperature() {
        let d = CORPUS_V0_1.resolve(0x0042).unwrap();
        assert_eq!(d.label, "ambient_temperature");
        assert_eq!(d.scale, 1000);
    }

    #[test]
    fn compiled_corpus_exposes_identity_and_version() {
        assert_eq!(REGISTRY_ID, "org.omwei.core");
        assert_eq!(SCHEMA_VERSION, "1");
        assert_eq!(CORPUS_VERSION, "0.1.0");
    }

    #[test]
    fn unknown_id_is_not_interoperable() {
        assert!(!CORPUS_V0_1.contains(0xFFFF));
    }
}

#[cfg(test)]
mod corpus_tests {
    use super::*;

    const VALID: &str = r#"registry_id: org.omwei.core
schema_version: "1"
corpus_version: "0.1.0"
version: 0.1
descriptors:
  - id: "0x0042"
    namespace: core
    status: active
    version: 1
    label: ambient_temperature
    semantic:
      quantity_kind: temperature
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

    #[test]
    fn parser_returns_validated_metadata_and_descriptors() {
        let parsed = parse_corpus(VALID).unwrap();
        assert_eq!(parsed.metadata.registry_id, "org.omwei.core");
        assert_eq!(parsed.metadata.schema_version, "1");
        assert_eq!(parsed.metadata.corpus_version, "0.1.0");
        assert_eq!(parsed.descriptors.len(), 1);
        assert_eq!(parsed.descriptors[0].id, 0x0042);
        assert_eq!(parsed.descriptors[0].label, "ambient_temperature");
        assert_eq!(parsed.descriptors[0].unit, "degree_Celsius");
        assert_eq!(parsed.descriptors[0].scale, 1000);
        assert_eq!(parsed.descriptors[0].min, -80000);
        assert_eq!(parsed.descriptors[0].max, 150000);
    }

    #[test]
    fn parser_rejects_invalid_scale_instead_of_bypassing_validation() {
        let invalid = VALID.replace("scale: \"0.001\"", "scale: \"0.3333333333\"");
        assert!(parse_corpus(&invalid).unwrap_err().contains("exact positive integer reciprocal"));
    }

    #[test]
    fn parser_rejects_missing_identity() {
        let invalid = VALID.replace("registry_id: org.omwei.core\n", "");
        assert!(parse_corpus(&invalid).is_err());
    }

    #[test]
    fn parser_rejects_non_semver_corpus_version() {
        let invalid = VALID.replace("corpus_version: \"0.1.0\"", "corpus_version: \"0.1\"");
        assert!(parse_corpus(&invalid).is_err());
    }
}
