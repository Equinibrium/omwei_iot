use std::{fs, process::Command};

#[test]
fn shared_vectors_pass() {
    let output = Command::new(env!("CARGO_BIN_EXE_source-mapping-conformance"))
        .args(["examples/source-mappings-v1.json", "testdata/source-mapping-interop-v1.json"])
        .output().expect("run conformance runner");
    assert!(output.status.success(), "stdout={} stderr={}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("PASS all 18 source-mapping vectors"));
}

#[test]
fn runner_fails_closed_on_bad_wire_expectation() {
    let source = fs::read_to_string("testdata/source-mapping-interop-v1.json").unwrap();
    let mutated = source.replacen("00450000007b", "000000000000", 1);
    let path = std::env::temp_dir().join(format!("omwei-bad-vectors-{}.json", std::process::id()));
    fs::write(&path, mutated).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_source-mapping-conformance"))
        .args(["examples/source-mappings-v1.json", path.to_str().unwrap()])
        .output().expect("run conformance runner");
    let _ = fs::remove_file(path);
    assert!(!output.status.success(), "runner accepted incorrect wire expectation");
}

#[test]
fn runner_rejects_malformed_wire_hex_without_panicking() {
    let source = fs::read_to_string("testdata/source-mapping-interop-v1.json").unwrap();
    let mutated = source.replacen("00450000007b", "0045000000xz", 1);
    let path = std::env::temp_dir().join(format!("omwei-malformed-hex-{}.json", std::process::id()));
    fs::write(&path, mutated).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_source-mapping-conformance"))
        .args(["examples/source-mappings-v1.json", path.to_str().unwrap()])
        .output().expect("run conformance runner");
    let _ = fs::remove_file(path);
    assert!(!output.status.success(), "runner accepted malformed expected wire hex");
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid wire hex"));
}


#[test]
fn runner_rejects_duplicate_source_profiles() {
    let source = fs::read_to_string("examples/source-mappings-v1.json").unwrap();
    let mut document: serde_json::Value = serde_json::from_str(&source).unwrap();
    let mappings = document["mappings"].as_array_mut().unwrap();
    let duplicate = mappings[0].clone();
    mappings.push(duplicate);
    let path = std::env::temp_dir().join(format!("omwei-duplicate-profiles-{}.json", std::process::id()));
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_source-mapping-conformance"))
        .args([path.to_str().unwrap(), "testdata/source-mapping-interop-v1.json"])
        .output().expect("run conformance runner");
    let _ = fs::remove_file(path);
    assert!(!output.status.success(), "runner accepted duplicate source profiles");
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate source identity"));
}
