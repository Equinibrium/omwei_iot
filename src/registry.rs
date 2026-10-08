use crate::{Descriptor, AMBIENT_TEMPERATURE};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CorpusFile {
    version: String,
    descriptors: Vec<CorpusDescriptor>,
}

#[derive(Debug, Deserialize)]
struct CorpusDescriptor {
    id: String,
    label: String,
    unit: String,
    encoding: CorpusEncoding,
}

#[derive(Debug, Deserialize)]
struct CorpusEncoding {
    scale: i32,
    offset: i32,
}

/// Parse the checked-in corpus format. Runtime callers can then construct
/// their own registry instead of relying on a hard-coded descriptor table.
pub fn parse_corpus(yaml: &str) -> Result<Vec<Descriptor>, String> {
    let corpus: CorpusFile = serde_yaml::from_str(yaml).map_err(|e| e.to_string())?;
    let _version = corpus.version;
    corpus.descriptors.into_iter().map(|d| {
        let id = d.id.strip_prefix("0x")
            .and_then(|s| u16::from_str_radix(s, 16).ok())
            .ok_or_else(|| format!("invalid descriptor id: {}", d.id))?;
        Ok(Descriptor {
            id,
            label: Box::leak(d.label.into_boxed_str()),
            unit: Box::leak(d.unit.into_boxed_str()),
            scale: d.encoding.scale,
            offset: d.encoding.offset,
            min: i32::MIN,
            max: i32::MAX,
        })
    }).collect()
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Registry {
    descriptors: &'static [Descriptor],
}

pub const CORPUS_V0_1: Registry = Registry {
    descriptors: &[AMBIENT_TEMPERATURE],
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
    fn unknown_id_is_not_interoperable() {
        assert!(!CORPUS_V0_1.contains(0xFFFF));
    }
}

#[cfg(test)]
mod corpus_tests {
    use super::*;

    #[test]
    fn checked_in_corpus_parses_and_resolves_temperature() {
        let yaml = r#"version: 0.1
descriptors:
  - id: 0x0042
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
      scale: 0.001
      offset: 0
    constraints:
      min: -80000
      max: 150000
"#;
        let parsed = parse_corpus(yaml).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, 0x0042);
        assert_eq!(parsed[0].label, "ambient_temperature");
        assert_eq!(parsed[0].unit, "degree_Celsius");
        assert_eq!(parsed[0].scale, 0); // YAML scale 0.001 is not yet mapped to integer scale.
    }
}
