use serde::Deserialize;
use std::{collections::HashSet, env, fs, path::Path};

#[derive(Debug, Deserialize)]
struct Config {
    schema_version: String,
    descriptor: Descriptor,
    profiles: Vec<Profile>,
}

#[derive(Debug, Deserialize)]
struct Descriptor {
    id: String,
    semantic: String,
    canonical_unit: String,
    output_field: String,
    minimum: i64,
    maximum: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct Profile {
    name: String,
    field: String,
    description: String,
    addend: i64,
    multiplier: i64,
    divisor: i64,
    vectors: Vec<Vector>,
}

#[derive(Debug, Clone, Deserialize)]
struct Vector {
    raw: i64,
    expected_milli_celsius: i64,
}

fn round_half_away_from_zero(numerator: i64, divisor: i64) -> Result<i64, String> {
    if divisor <= 0 {
        return Err("divisor must be positive".into());
    }
    let abs = numerator.checked_abs().ok_or("integer overflow")?;
    let quotient = abs / divisor;
    let remainder = abs % divisor;
    let rounded = quotient + if remainder >= divisor / 2 + divisor % 2 { 1 } else { 0 };
    if numerator < 0 {
        rounded.checked_neg().ok_or_else(|| "integer overflow".into())
    } else {
        Ok(rounded)
    }
}

fn convert(profile: &Profile, raw: i64, descriptor: &Descriptor) -> Result<i64, String> {
    if profile.divisor <= 0 || profile.multiplier <= 0 {
        return Err(format!("invalid rational transform in {}", profile.name));
    }
    let shifted = raw.checked_add(profile.addend).ok_or("integer overflow")?;
    let numerator = shifted.checked_mul(profile.multiplier).ok_or("integer overflow")?;
    let value = round_half_away_from_zero(numerator, profile.divisor)?;
    if !(descriptor.minimum..=descriptor.maximum).contains(&value) {
        return Err(format!("canonical value {value} outside descriptor range"));
    }
    Ok(value)
}

fn validate(config: &Config) -> Result<(), String> {
    if config.schema_version != "1" {
        return Err("unsupported schema_version".into());
    }
    let id = config.descriptor.id.as_str();
    if id.len() != 6 || !id.starts_with("0x")
        || !id[2..].bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("descriptor id must use 0xNNNN syntax".into());
    }
    if config.descriptor.semantic.is_empty()
        || config.descriptor.canonical_unit.is_empty()
        || config.descriptor.output_field.is_empty()
        || config.descriptor.minimum > config.descriptor.maximum
        || config.profiles.is_empty()
    {
        return Err("invalid descriptor metadata or constraints".into());
    }

    let mut names = HashSet::new();
    let mut fields = HashSet::new();
    for profile in &config.profiles {
        if profile.name.is_empty() || profile.field.is_empty()
            || !names.insert(profile.name.as_str())
            || !fields.insert(profile.field.as_str())
            || profile.divisor <= 0 || profile.multiplier <= 0
        {
            return Err(format!("invalid or duplicate profile: {}", profile.name));
        }
        if profile.description.is_empty() {
            return Err(format!("missing description for profile {}", profile.name));
        }
        for vector in &profile.vectors {
            let actual = convert(profile, vector.raw, &config.descriptor)?;
            if actual != vector.expected_milli_celsius {
                return Err(format!(
                    "vector mismatch in {} for raw {}: expected {}, got {}",
                    profile.name, vector.raw, vector.expected_milli_celsius, actual
                ));
            }
        }
    }
    Ok(())
}

fn js_adapter(config: &Config) -> String {
    let mut out = String::from(r#"// Generated; do not edit.
function roundHalfAwayFromZero(n, d) {
  const sign = n < 0 ? -1 : 1;
  const a = Math.abs(n);
  return sign * (Math.floor(a / d) + ((a % d) * 2 >= d ? 1 : 0));
}

function normalizeUplink(input) {
  const payload = (input && input.data && input.data.payload) || (input && input.data) || {};
  const matches = [];
"#);
    for (index, profile) in config.profiles.iter().enumerate() {
        out.push_str(&format!(
            "  if (Object.prototype.hasOwnProperty.call(payload, {:?})) matches.push({{ index: {index}, raw: payload[{:?}] }}); // {}\n",
            profile.field, profile.field, profile.name
        ));
    }
    out.push_str(r#"  if (matches.length !== 1) return { data: {}, errors: [matches.length === 0 ? 'missing recognized source field' : 'ambiguous source fields'] };
  const match = matches[0];
  if (!Number.isSafeInteger(match.raw)) return { data: {}, errors: ['source value must be a safe integer'] };
  let numerator;
  let denominator;
  switch (match.index) {
"#);
    for (index, profile) in config.profiles.iter().enumerate() {
        out.push_str(&format!(
            "    case {index}: numerator = (match.raw + {}) * {}; denominator = {}; break;\n",
            profile.addend, profile.multiplier, profile.divisor
        ));
    }
    out.push_str(&format!(
        "    default: return {{ data: {{}}, errors: ['unknown profile'] }};\n  }}\n  if (!Number.isSafeInteger(numerator)) return {{ data: {{}}, errors: ['conversion exceeds safe integer range'] }};\n  const canonical = roundHalfAwayFromZero(numerator, denominator);\n  if (canonical < {} || canonical > {}) return {{ data: {{}}, errors: ['canonical value outside descriptor range'] }};\n  return {{ data: {{ {:?}: canonical }} }};\n}}\n",
        config.descriptor.minimum, config.descriptor.maximum, config.descriptor.output_field
    ));
    out
}

fn starlark_adapter(config: &Config) -> String {
    let mut out = String::from(r#"# Generated; do not edit. Canonical output is an integer in millidegrees Celsius.
def round_half_away_from_zero(n, d):
    sign = -1 if n < 0 else 1
    a = -n if n < 0 else n
    return sign * (a // d + (1 if (a % d) * 2 >= d else 0))

def apply(metric):
    matches = []
"#);
    out.push_str(&format!(
        "    if {:?} in metric.fields:\\n        del metric.fields[{:?}]\\n",
        config.descriptor.output_field, config.descriptor.output_field
    ));
    for (index, profile) in config.profiles.iter().enumerate() {
        out.push_str(&format!(
            "    raw_{index} = metric.fields.get({:?})\n    if raw_{index} != None:\n        matches.append(({index}, raw_{index}))\n",
            profile.field
        ));
    }
    out.push_str(r#"    if len(matches) != 1:
        metric.fields['omwei_error'] = 'missing or ambiguous source fields'
        return metric
    source_index, raw = matches[0]
    if type(raw) != 'int':
        metric.fields['omwei_error'] = 'source value must be integer'
        return metric
    canonical = None
"#);
    for (index, profile) in config.profiles.iter().enumerate() {
        out.push_str(&format!(
            "    {} source_index == {index}:\n        canonical = round_half_away_from_zero((raw + {}) * {}, {})\n",
            if index == 0 { "if" } else { "elif" },
            profile.addend, profile.multiplier, profile.divisor
        ));
    }
    out.push_str(&format!(
        "    if canonical < {} or canonical > {}:\n        metric.fields['omwei_error'] = 'canonical value outside descriptor range'\n        return metric\n    metric.fields[{:?}] = canonical\n    if 'omwei_error' in metric.fields:\n        del metric.fields['omwei_error']\n    return metric\n",
        config.descriptor.minimum, config.descriptor.maximum, config.descriptor.output_field
    ));
    out
}

fn vectors_json(config: &Config) -> String {
    let mut rows = Vec::new();
    for profile in &config.profiles {
        for vector in &profile.vectors {
            rows.push(format!(
                "    {{\"profile\": {:?}, \"raw\": {}, \"expected_milli_celsius\": {}}}",
                profile.name, vector.raw, vector.expected_milli_celsius
            ));
        }
    }
    format!(
        "{{\n  \"descriptor_id\": {:?},\n  \"semantic\": {:?},\n  \"canonical_unit\": {:?},\n  \"vectors\": [\n{}\n  ]\n}}\n",
        config.descriptor.id, config.descriptor.semantic, config.descriptor.canonical_unit,
        rows.join(",\n")
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: adapter_generator <registry.yaml> <output-dir>");
        std::process::exit(2);
    }
    let raw = fs::read_to_string(&args[1])?;
    let config: Config = serde_yaml::from_str(&raw)?;
    validate(&config).map_err(|e| format!("registry validation failed: {e}"))?;
    let output = Path::new(&args[2]);
    fs::create_dir_all(output)?;
    fs::write(output.join("tts_normalize_uplink.js"), js_adapter(&config))?;
    fs::write(output.join("telegraf_starlark.star"), starlark_adapter(&config))?;
    fs::write(output.join("test_vectors.json"), vectors_json(&config))?;
    println!("validated {} source profiles and generated 3 artifacts", config.profiles.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Config {
        serde_yaml::from_str(include_str!("../../examples/adapter-poc/registry.yaml")).unwrap()
    }

    #[test]
    fn equivalent_celsius_and_fahrenheit_inputs_share_canonical_value() {
        let c = fixture();
        validate(&c).unwrap();
        let a = c.profiles.iter().find(|p| p.name == "vendor_a_deci_celsius").unwrap();
        let b = c.profiles.iter().find(|p| p.name == "vendor_b_deci_fahrenheit").unwrap();
        assert_eq!(convert(a, 235, &c.descriptor).unwrap(), 23_500);
        assert_eq!(convert(b, 743, &c.descriptor).unwrap(), 23_500);
        assert_eq!(convert(b, 320, &c.descriptor).unwrap(), 0);
    }

    #[test]
    fn rounding_is_explicit_and_symmetric() {
        assert_eq!(round_half_away_from_zero(5, 2).unwrap(), 3);
        assert_eq!(round_half_away_from_zero(-5, 2).unwrap(), -3);
        assert_eq!(round_half_away_from_zero(4, 2).unwrap(), 2);
    }

    #[test]
    fn invalid_transform_and_range_are_rejected() {
        let c = fixture();
        let mut profile = c.profiles[0].clone();
        profile.divisor = 0;
        assert!(convert(&profile, 235, &c.descriptor).is_err());
        let fahrenheit = c.profiles.iter().find(|p| p.name == "vendor_b_deci_fahrenheit").unwrap();
        assert!(convert(fahrenheit, 100_000, &c.descriptor).is_err());
    }

    #[test]
    fn both_adapters_are_generated_from_the_same_profiles() {
        let c = fixture();
        let js = js_adapter(&c);
        let starlark = starlark_adapter(&c);
        assert!(js.contains("function normalizeUplink(input)"));
        assert!(js.contains("temp_c_dC"));
        assert!(js.contains("temp_f_dF"));
        assert!(starlark.contains("def apply(metric):"));
        assert!(starlark.contains("temp_c_dC"));
        assert!(starlark.contains("temp_f_dF"));
        assert!(vectors_json(&c).contains("\"expected_milli_celsius\": 23500"));
    }

    #[test]
    fn duplicate_fields_and_bad_vectors_are_rejected() {
        let mut c = fixture();
        c.profiles[1].field = c.profiles[0].field.clone();
        assert!(validate(&c).is_err());
        let mut c = fixture();
        c.profiles[0].vectors[0].expected_milli_celsius = 123;
        assert!(validate(&c).is_err());
    }
}
