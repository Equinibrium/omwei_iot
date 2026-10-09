use serde::Deserialize;
use serde_yaml::Value;
use std::{env, fs, path::PathBuf};

#[path = "src/registry_validation.rs"]
mod registry_validation;

#[derive(Deserialize)]
struct Corpus {
    registry_id: String,
    schema_version: String,
    corpus_version: String,
    descriptors: Vec<Descriptor>,
}
#[derive(Deserialize)]
struct Descriptor {
    id: String,
    label: String,
    unit: Unit,
    encoding: Encoding,
    constraints: Constraints,
}
#[derive(Deserialize)]
struct Unit { canonical: String }
#[derive(Deserialize)]
struct Encoding { scale: Value, offset: i32 }
#[derive(Deserialize)]
struct Constraints { min: i32, max: i32 }

fn main() {
    println!("cargo:rerun-if-changed=registry/corpus-v0.1.yaml");
    println!("cargo:rerun-if-changed=src/registry_validation.rs");
    let input = fs::read_to_string("registry/corpus-v0.1.yaml")
        .expect("failed to read registry corpus");
    registry_validation::validate_corpus(&input)
        .unwrap_or_else(|e| panic!("registry corpus validation failed: {e}"));
    let corpus: Corpus = serde_yaml::from_str(&input)
        .expect("validated registry corpus must deserialize for code generation");

    let mut out = format!(
        "pub const REGISTRY_ID: &str = {:?};\npub const SCHEMA_VERSION: &str = {:?};\npub const CORPUS_VERSION: &str = {:?};\n",
        corpus.registry_id, corpus.schema_version, corpus.corpus_version
    );
    out.push_str("pub static DESCRIPTORS: &[crate::Descriptor] = &[\n");
    for d in corpus.descriptors {
        let id = u16::from_str_radix(d.id.trim_start_matches("0x"), 16)
            .expect("validated descriptor ID must parse");
        let scale_text = match d.encoding.scale {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s,
            _ => panic!("validated descriptor scale must be a decimal number"),
        };
        let scale = registry_validation::exact_scale_multiplier(&scale_text)
            .expect("validated descriptor scale must have an exact integer reciprocal");
        out.push_str(&format!(
            "    crate::Descriptor {{ id: 0x{:04X}, label: {:?}, unit: {:?}, scale: {}, offset: {}, min: {}, max: {} }},\n",
            id, d.label, d.unit.canonical, scale, d.encoding.offset, d.constraints.min, d.constraints.max
        ));
    }
    out.push_str("];\n");

    let path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("registry_generated.rs");
    fs::write(path, out).expect("failed to write generated registry");
}
