//! OMWEI integrator CLI — list, explain, convert, decode.
//!
//! Offline helper for IoT integrators working with the compiled corpus.
//! No network, no floating-point conversion path in the critical rules.

use omwei_iot::{
    decimal_conversion::from_decimal,
    decode, encode, descriptor,
    registry::{CORPUS_V0_1, CORPUS_VERSION, DESCRIPTORS, REGISTRY_ID, SCHEMA_VERSION},
    Descriptor, SemanticAtom,
};
use std::env;
use std::process;

fn main() {
    let mut args = env::args().skip(1);
    let cmd = match args.next() {
        Some(c) => c,
        None => {
            print_usage();
            process::exit(2);
        }
    };

    let result = match cmd.as_str() {
        "list" => cmd_list(),
        "explain" => {
            let key = args.next().unwrap_or_else(|| {
                eprintln!("usage: omwei explain <id|label>");
                process::exit(2);
            });
            cmd_explain(&key)
        }
        "convert" => {
            let key = args.next().unwrap_or_else(|| {
                eprintln!("usage: omwei convert <id|label> <decimal>");
                process::exit(2);
            });
            let decimal = args.next().unwrap_or_else(|| {
                eprintln!("usage: omwei convert <id|label> <decimal>");
                process::exit(2);
            });
            cmd_convert(&key, &decimal)
        }
        "decode" => {
            let hex = args.next().unwrap_or_else(|| {
                eprintln!("usage: omwei decode <12-hex-chars>");
                process::exit(2);
            });
            cmd_decode(&hex)
        }
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        }
        other => {
            eprintln!("unknown command: {other}");
            print_usage();
            process::exit(2);
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn print_usage() {
    eprintln!(
        "omwei — OMWEI integrator CLI (corpus {CORPUS_VERSION})\n\
         \n\
         Commands:\n\
           list                         List all descriptors in the compiled corpus\n\
           explain <id|label>           Show descriptor metadata and constraints\n\
           convert <id|label> <decimal> Physical decimal → atom + wire hex\n\
           decode  <12-hex-chars>       6-byte atom hex → physical value\n\
         \n\
         Examples:\n\
           omwei list\n\
           omwei explain 0x0042\n\
           omwei explain ambient_temperature\n\
           omwei convert 0x0042 22.5\n\
           omwei decode 0042000057E4\n\
         \n\
         Registry: {REGISTRY_ID}  schema={SCHEMA_VERSION}"
    );
}

fn cmd_list() -> Result<(), String> {
    println!(
        "OMWEI corpus {CORPUS_VERSION}  ({REGISTRY_ID}, schema {SCHEMA_VERSION})\n"
    );
    println!(
        "{:<8} {:<28} {:<28} {:>8} {:>8} {:>10} {:>10}",
        "ID", "label", "unit", "scale", "offset", "min", "max"
    );
    println!("{}", "-".repeat(110));
    for d in DESCRIPTORS.iter() {
        println!(
            "0x{:04X}   {:<28} {:<28} {:>8} {:>8} {:>10} {:>10}",
            d.id, d.label, d.unit, d.scale, d.offset, d.min, d.max
        );
    }
    println!("\n{} descriptors", DESCRIPTORS.len());
    Ok(())
}

fn cmd_explain(key: &str) -> Result<(), String> {
    let d = resolve_descriptor(key)?;
    println!("Descriptor 0x{:04X}", d.id);
    println!("  label:   {}", d.label);
    println!("  unit:    {}", d.unit);
    println!("  scale:   {}  (physical = encoded / scale + offset)", d.scale);
    println!("  offset:  {}", d.offset);
    println!("  range:   {} ..= {}  (encoded domain)", d.min, d.max);
    println!("  physical range (approx):");
    println!(
        "    {} ..= {} {}",
        format_physical(d.min, d),
        format_physical(d.max, d),
        d.unit
    );
    println!("  corpus:  {REGISTRY_ID} {CORPUS_VERSION}");
    Ok(())
}

fn cmd_convert(key: &str, decimal: &str) -> Result<(), String> {
    let d = resolve_descriptor(key)?;
    let atom = from_decimal(d.id, decimal).map_err(|e| e.to_string())?;
    let bytes = encode(atom);
    let hex: String = bytes.iter().map(|b| format!("{b:02X}")).collect();

    println!("convert  {decimal} {}  →  atom", d.unit);
    println!("  descriptor:  0x{:04X} ({})", atom.descriptor_id, d.label);
    println!("  encoded:     {}", atom.canonical_value);
    println!("  physical:    {} {}", format_physical(atom.canonical_value, d), d.unit);
    println!("  wire (6 B):  {hex}");
    println!("  bytes:       {:?}", bytes);
    Ok(())
}

fn cmd_decode(hex_input: &str) -> Result<(), String> {
    let hex = hex_input.trim().trim_start_matches("0x").trim_start_matches("0X");
    if hex.len() != 12 {
        return Err(format!(
            "expected 12 hex characters (6 bytes), got {} chars", 
            hex.len()
        ));
    }
    let mut bytes = [0u8; 6];
    for i in 0..6 {
        let byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| format!("invalid hex at position {}", i * 2))?;
        bytes[i] = byte;
    }

    let atom = decode(&bytes).map_err(|e| e.to_string())?;
    let d = descriptor(atom.descriptor_id).ok_or("unknown descriptor after decode")?;

    println!("decode  {hex}");
    println!("  descriptor:  0x{:04X} ({})", atom.descriptor_id, d.label);
    println!("  encoded:     {}", atom.canonical_value);
    println!("  physical:    {} {}", format_physical(atom.canonical_value, d), d.unit);
    println!("  unit:        {}", d.unit);
    println!("  scale:       {}  offset: {}", d.scale, d.offset);
    Ok(())
}

fn resolve_descriptor(key: &str) -> Result<&'static Descriptor, String> {
    let key = key.trim();
    // Try hex ID first (0x0042 or 0042 or 42)
    if let Some(id) = parse_id(key) {
        return CORPUS_V0_1
            .resolve(id)
            .ok_or_else(|| format!("unknown descriptor id 0x{id:04X}"));
    }
    // Fall back to label (case-sensitive, exact match)
    DESCRIPTORS
        .iter()
        .find(|d| d.label == key)
        .ok_or_else(|| format!("unknown descriptor label or id: {key:?}"))
}

fn parse_id(s: &str) -> Option<u16> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        return u16::from_str_radix(rest, 16).ok();
    }
    // Bare hex without 0x if it looks like hex and has only hex digits
    if s.bytes().all(|b| b.is_ascii_hexdigit()) && (s.len() <= 4) {
        return u16::from_str_radix(s, 16).ok();
    }
    // Decimal integer
    s.parse::<u16>().ok()
}

/// Format encoded value as physical decimal using exact integer arithmetic.
/// physical = encoded / scale + offset
fn format_physical(encoded: i32, d: &Descriptor) -> String {
    let scale = d.scale as i64;
    let offset = d.offset as i64;
    let enc = encoded as i64;

    // physical = enc / scale + offset
    // Represent as mixed number or fixed decimal.
    if scale == 1 {
        return format!("{}", enc + offset);
    }

    // Integer division + remainder for exact display
    let total_numer = enc + offset * scale; // in units of 1/scale
    let whole = total_numer / scale;
    let mut rem = total_numer % scale;
    if rem < 0 {
        // normalize remainder positive for display of negative numbers
        // e.g. -1.5 with scale 1000 → whole=-2, rem=500? better keep sign on whole
    }

    if rem == 0 {
        return format!("{whole}");
    }

    // Decimal digits: pad to log10(scale) places when scale is power of 10
    let digits = scale_digits(scale);
    if digits > 0 {
        let abs_rem = rem.unsigned_abs() as u64;
        let frac = format!("{:0width$}", abs_rem, width = digits as usize);
        // trim trailing zeros for readability
        let frac_trim = frac.trim_end_matches('0');
        if frac_trim.is_empty() {
            format!("{whole}")
        } else if total_numer < 0 && whole == 0 {
            format!("-0.{frac_trim}")
        } else {
            format!("{whole}.{frac_trim}")
        }
    } else {
        // Non-power-of-10 scale: show as fraction
        format!("{whole} + {rem}/{scale}")
    }
}

fn scale_digits(scale: i64) -> u32 {
    if scale <= 0 {
        return 0;
    }
    let mut s = scale as u64;
    let mut digits = 0u32;
    while s % 10 == 0 {
        s /= 10;
        digits += 1;
    }
    if s == 1 {
        digits
    } else {
        0 // not a pure power of 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_id_accepts_hex_and_decimal() {
        assert_eq!(parse_id("0x0042"), Some(0x0042));
        assert_eq!(parse_id("0X42"), Some(0x42));
        assert_eq!(parse_id("42"), Some(0x42)); // decimal 42 == 0x002A? wait
        // "42" as decimal is 42, as hex would be 0x42 if we preferred hex.
        // Our rule: bare digits try decimal first only if not pure hex path.
        // Actually current: if all hex digits and len<=4, treat as hex.
        // "42" is hex digits → 0x42. Good for short IDs.
        assert_eq!(parse_id("42"), Some(0x0042));
        assert_eq!(parse_id("0042"), Some(0x0042));
    }

    #[test]
    fn format_physical_temperature() {
        let d = descriptor(0x0042).unwrap();
        assert_eq!(format_physical(22_500, d), "22.5");
        assert_eq!(format_physical(0, d), "0");
        assert_eq!(format_physical(-80_000, d), "-80");
        assert_eq!(format_physical(1, d), "0.001");
        assert_eq!(format_physical(-1, d), "-0.001");
    }

    #[test]
    fn resolve_by_label_and_id() {
        assert_eq!(resolve_descriptor("ambient_temperature").unwrap().id, 0x0042);
        assert_eq!(resolve_descriptor("0x0042").unwrap().label, "ambient_temperature");
    }
}
