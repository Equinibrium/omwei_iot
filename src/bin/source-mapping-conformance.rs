use omwei_iot::{descriptor, encode, SemanticAtom};
use serde::Deserialize;
use std::{collections::HashSet, env, fs, process};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileDocument { schema_version: String, mappings: Vec<SourceMapping> }
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceIdentity { protocol: String, device: String, point: String }
impl PartialEq for SourceIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.protocol == other.protocol && self.device == other.device && self.point == other.point
    }
}
impl Eq for SourceIdentity {}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceMapping {
    source: SourceIdentity,
    descriptor_id: u16,
    source_unit: String,
    canonical_unit: String,
    scale_numerator: i64,
    scale_denominator: i64,
    offset_numerator: i64,
    offset_denominator: i64,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorDocument { schema_version: String, description: String, cases: Vec<VectorCase> }
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorCase {
    name: String,
    sample: SourceSample,
    expected_wire_hex: Option<String>,
    expected_error: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceSample { source: SourceIdentity, value: String, unit: String }

fn invalid(message: impl Into<String>) -> String { format!("invalid_decimal:{}", message.into()) }
fn parse_decimal(value: &str) -> Result<(i128, i128), String> {
    if value.is_empty() || value.trim() != value { return Err(invalid("empty or whitespace")); }
    let (negative, body) = match value.as_bytes()[0] {
        b'-' => (true, &value[1..]),
        b'+' => (false, &value[1..]),
        _ => (false, value),
    };
    if body.is_empty() { return Err(invalid("missing digits")); }
    let mut split = body.split('.');
    let whole = split.next().unwrap_or("");
    let fraction = split.next().unwrap_or("");
    if split.next().is_some() || whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit()) ||
       (body.contains('.') && (fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()))) ||
       fraction.len() > 18 {
        return Err(invalid("expected plain base-10 decimal with at most 18 fractional digits"));
    }
    let digits = format!("{}{}", whole, fraction);
    let mut numerator = digits.parse::<i128>().map_err(|_| "out_of_range: decimal integer overflow".to_string())?;
    if negative { numerator = numerator.checked_neg().ok_or_else(|| "out_of_range: decimal negation overflow".to_string())?; }
    let denominator = 10_i128.checked_pow(fraction.len() as u32).ok_or_else(|| "out_of_range: decimal denominator overflow".to_string())?;
    Ok((numerator, denominator))
}
fn mul(a: i128, b: i128) -> Result<i128, String> { a.checked_mul(b).ok_or_else(|| "out_of_range: rational multiplication overflow".into()) }
fn add(a: i128, b: i128) -> Result<i128, String> { a.checked_add(b).ok_or_else(|| "out_of_range: rational addition overflow".into()) }
fn round_away(n: i128, d: i128) -> Result<i128, String> {
    if d <= 0 { return Err("invalid_profile: denominator must be positive".into()); }
    let q = n.checked_abs().ok_or_else(|| "out_of_range: absolute value overflow".to_string())? / d;
    let r = n.checked_abs().ok_or_else(|| "out_of_range: absolute value overflow".to_string())? % d;
    let rounded = if r.checked_mul(2).ok_or_else(|| "out_of_range: rounding overflow".to_string())? >= d { q + 1 } else { q };
    Ok(if n < 0 { -rounded } else { rounded })
}
fn map(mapping: &SourceMapping, sample: &SourceSample) -> Result<[u8; 6], String> {
    if mapping.source != sample.source { return Err("source_identity_mismatch".into()); }
    if mapping.source_unit != sample.unit { return Err("observed_source_unit_mismatch".into()); }
    if mapping.scale_numerator == 0 || mapping.scale_denominator <= 0 || mapping.offset_denominator <= 0 {
        return Err("invalid_profile: invalid rational scale/offset".into());
    }
    let d = descriptor(mapping.descriptor_id).ok_or_else(|| "invalid_profile: unknown descriptor".to_string())?;
    if mapping.canonical_unit != d.unit { return Err("invalid_profile: canonical unit mismatch".into()); }
    let (raw_n, raw_d) = parse_decimal(&sample.value)?;
    let scale_n = mapping.scale_numerator as i128;
    let scale_d = mapping.scale_denominator as i128;
    let offset_n = mapping.offset_numerator as i128;
    let offset_d = mapping.offset_denominator as i128;
    let physical_n = add(mul(mul(raw_n, scale_n)?, offset_d)?, mul(mul(offset_n, raw_d)?, scale_d)?)?;
    let physical_d = mul(mul(raw_d, scale_d)?, offset_d)?;
    let shifted_n = add(physical_n, -mul(d.offset as i128, physical_d)?)?;
    let encoded_n = mul(shifted_n, d.scale as i128)?;
    let encoded = round_away(encoded_n, physical_d)?;
    if encoded < i32::MIN as i128 || encoded > i32::MAX as i128 { return Err("out_of_range".into()); }
    let value = encoded as i32;
    if value < d.min || value > d.max { return Err("out_of_range".into()); }
    let atom = SemanticAtom::new(d.id, value).map_err(|_| "out_of_range".to_string())?;
    Ok(encode(atom))
}
fn validate_profiles(doc: &ProfileDocument) -> Result<(), String> {
    if doc.schema_version != "1" || doc.mappings.is_empty() { return Err("invalid_profile: expected schema_version 1 and non-empty mappings".into()); }
    let mut identities = HashSet::new();
    for m in &doc.mappings {
        if m.source.protocol.trim().is_empty() || m.source.device.trim().is_empty() || m.source.point.trim().is_empty() ||
           m.source_unit.trim().is_empty() || m.canonical_unit.trim().is_empty() ||
           m.scale_numerator == 0 || m.scale_denominator <= 0 || m.offset_denominator <= 0 {
            return Err("invalid_profile: incomplete identity, units, or rational parameters".into());
        }
        let d = descriptor(m.descriptor_id).ok_or_else(|| format!("invalid_profile: unknown descriptor {}", m.descriptor_id))?;
        if m.canonical_unit != d.unit { return Err(format!("invalid_profile: canonical unit {:?} does not match {:?}", m.canonical_unit, d.unit)); }
        let key = format!("{}\0{}\0{}", m.source.protocol, m.source.device, m.source.point);
        if !identities.insert(key) { return Err("invalid_profile: duplicate source identity".into()); }
    }
    Ok(())
}
fn run(profile_path: &str, vector_path: &str) -> Result<(), String> {
    let profile_data = fs::read_to_string(profile_path).map_err(|e| format!("read profiles: {e}"))?;
    let profiles: ProfileDocument = serde_json::from_str(&profile_data).map_err(|e| format!("parse profiles: {e}"))?;
    validate_profiles(&profiles)?;
    let vector_data = fs::read_to_string(vector_path).map_err(|e| format!("read vectors: {e}"))?;
    let vectors: VectorDocument = serde_json::from_str(&vector_data).map_err(|e| format!("parse vectors: {e}"))?;
    if vectors.schema_version != "1" || vectors.description.trim().is_empty() || vectors.cases.is_empty() {
        return Err("invalid_vectors: expected schema_version 1, description, and non-empty cases".into());
    }
    let mut names = HashSet::new();
    for case in &vectors.cases {
        if case.name.trim().is_empty() || !names.insert(case.name.clone()) { return Err("invalid_vectors: empty or duplicate case name".into()); }
        if case.expected_wire_hex.is_some() == case.expected_error.is_some() { return Err(format!("{}: invalid_vectors: exactly one expected outcome required", case.name)); }
        let mut selected = profiles.mappings.iter().find(|m| m.source == case.sample.source);
        if selected.is_none() && case.expected_error.as_deref() == Some("source_identity_mismatch") {
            selected = profiles.mappings.iter().find(|m| m.source.protocol == case.sample.source.protocol && m.source.device == case.sample.source.device);
        }
        let mapping = selected.ok_or_else(|| format!("{}: no matching profile for vector", case.name))?;
        let result = map(mapping, &case.sample);
        if let Some(expected_error) = &case.expected_error {
            let got = result.as_ref().map(|_| String::new()).unwrap_or_else(|e| e.split(':').next().unwrap_or(e).to_string());
            if got != *expected_error { return Err(format!("{}: expected error {}, got {}", case.name, expected_error, got)); }
            println!("PASS {}: {}", case.name, got);
        } else {
            let result = result.map_err(|e| format!("{}: unexpected error: {}", case.name, e))?;
            let expected = decode_hex(case.expected_wire_hex.as_deref().unwrap())?;
            if result != expected { return Err(format!("{}: wire mismatch: expected {}, got {}", case.name, hex(&expected), hex(&result))); }
            println!("PASS {}: {}", case.name, hex(&result));
        }
    }
    println!("PASS all {} source-mapping vectors", vectors.cases.len());
    Ok(())
}
fn decode_hex(s: &str) -> Result<[u8; 6], String> {
    if s.len() != 12 { return Err("invalid_vectors: expected_wire_hex must contain 12 hex chars".into()); }
    let mut out = [0u8; 6];
    for i in 0..6 {
        out[i] = u8::from_str_radix(&s[i*2..i*2+2], 16).map_err(|_| "invalid_vectors: invalid wire hex".to_string())?;
    }
    Ok(out)
}
fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn main() {
    let mut args = env::args().skip(1);
    let profiles = args.next().unwrap_or_else(|| "examples/source-mappings-v1.json".into());
    let vectors = args.next().unwrap_or_else(|| "testdata/source-mapping-interop-v1.json".into());
    if args.next().is_some() { eprintln!("usage: source-mapping-conformance [profiles.json] [vectors.json]"); process::exit(2); }
    if let Err(err) = run(&profiles, &vectors) { eprintln!("FAIL: {err}"); process::exit(1); }
}
