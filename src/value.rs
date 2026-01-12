use crate::statics;
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Represents a number that can preserve distinction between I64, U64, and F64 for round-tripping.
/// Terra Invicta saves are sensitive to integer vs float formatting in some fields.
#[derive(Debug, Clone, PartialEq)]
pub enum TiNumber {
    I64(i64),
    U64(u64),
    F64(f64),
}

impl TiNumber {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            TiNumber::I64(v) => Some(*v),
            TiNumber::U64(v) => i64::try_from(*v).ok(),
            TiNumber::F64(_) => None,
        }
    }
}

impl Serialize for TiNumber {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TiNumber::I64(v) => serializer.serialize_i64(*v),
            TiNumber::U64(v) => serializer.serialize_u64(*v),
            TiNumber::F64(v) => serializer.serialize_f64(*v),
        }
    }
}

impl<'de> Deserialize<'de> for TiNumber {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct NumberVisitor;

        impl<'de> de::Visitor<'de> for NumberVisitor {
            type Value = TiNumber;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a JSON5 number")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(TiNumber::I64(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(TiNumber::U64(v))
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(TiNumber::F64(v))
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}

/// Represents a value in the Terra Invicta save format (JSON5 subset).
/// Supports specific serialization rules (e.g. empty objects spanning lines) to match game output.
#[derive(Debug, Clone, PartialEq)]
pub enum TiValue {
    Null,
    Bool(bool),
    Number(TiNumber),
    String(String),
    Array(Vec<TiValue>),
    Object(IndexMap<String, TiValue>),
}

impl TiValue {
    pub fn as_object(&self) -> Option<&IndexMap<String, TiValue>> {
        match self {
            TiValue::Object(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut IndexMap<String, TiValue>> {
        match self {
            TiValue::Object(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[TiValue]> {
        match self {
            TiValue::Array(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<TiValue>> {
        match self {
            TiValue::Array(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            TiValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&TiValue> {
        self.as_object().and_then(|m| m.get(key))
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut TiValue> {
        self.as_object_mut().and_then(|m| m.get_mut(key))
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            TiValue::Null => "null",
            TiValue::Bool(_) => "bool",
            TiValue::Number(_) => "number",
            TiValue::String(_) => "string",
            TiValue::Array(_) => "array",
            TiValue::Object(_) => "object",
        }
    }

    pub fn parse_json5(text: &str) -> anyhow::Result<TiValue> {
        Ok(json5::from_str::<TiValue>(text)?)
    }

    pub fn to_json5_pretty(&self) -> String {
        let mut out = String::new();
        self.write_json5(&mut out, 0, true);
        out.push('\n');
        out
    }

    /// Serialize in a Terra Invicta-like style:
    /// - 4-space indentation
    /// - keys always quoted
    /// - non-ASCII escaped (ensure_ascii)
    /// - empty objects rendered across multiple lines ("{}" quirk)
    pub fn to_ti_save_pretty(&self) -> String {
        self.to_ti_save_pretty_with_newline(statics::NL_LF)
    }

    pub fn to_ti_save_pretty_with_newline(&self, newline: &str) -> String {
        let mut out = String::new();
        self.write_ti_save(&mut out, 0, newline);
        out
    }

    pub fn to_json5_compact(&self) -> String {
        let mut out = String::new();
        self.write_json5(&mut out, 0, false);
        out
    }

    fn write_json5(&self, out: &mut String, indent: usize, pretty: bool) {
        match self {
            TiValue::Null => out.push_str("null"),
            TiValue::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
            TiValue::Number(n) => n.write_json5(out),
            TiValue::String(s) => write_escaped_string(out, s),
            TiValue::Array(values) => {
                out.push('[');
                if pretty && !values.is_empty() {
                    out.push('\n');
                }
                for (i, v) in values.iter().enumerate() {
                    if pretty {
                        out.push_str(&" ".repeat(indent + 4));
                    } else if i > 0 {
                        out.push(' ');
                    }
                    v.write_json5(out, indent + 4, pretty);
                    if i + 1 != values.len() {
                        out.push(',');
                    }
                    if pretty {
                        out.push('\n');
                    }
                }
                if pretty && !values.is_empty() {
                    out.push_str(&" ".repeat(indent));
                }
                out.push(']');
            }
            TiValue::Object(map) => {
                out.push('{');
                if pretty && !map.is_empty() {
                    out.push('\n');
                }
                for (i, (k, v)) in map.iter().enumerate() {
                    if pretty {
                        out.push_str(&" ".repeat(indent + 4));
                    } else if i > 0 {
                        out.push(' ');
                    }
                    write_escaped_string(out, k);
                    out.push(':');
                    if pretty {
                        out.push(' ');
                    }
                    v.write_json5(out, indent + 4, pretty);
                    if i + 1 != map.len() {
                        out.push(',');
                    }
                    if pretty {
                        out.push('\n');
                    }
                }
                if pretty && !map.is_empty() {
                    out.push_str(&" ".repeat(indent));
                }
                out.push('}');
            }
        }
    }

    fn write_ti_save(&self, out: &mut String, indent: usize, newline: &str) {
        match self {
            TiValue::Null => out.push_str("null"),
            TiValue::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
            TiValue::Number(n) => n.write_ti_save(out),
            TiValue::String(s) => write_escaped_string_ascii(out, s),
            TiValue::Array(values) => {
                out.push('[');
                if !values.is_empty() {
                    out.push_str(newline);
                    for (i, v) in values.iter().enumerate() {
                        out.push_str(&" ".repeat(indent + 4));
                        v.write_ti_save(out, indent + 4, newline);
                        if i + 1 != values.len() {
                            out.push(',');
                        }
                        out.push_str(newline);
                    }
                    out.push_str(&" ".repeat(indent));
                }
                out.push(']');
            }
            TiValue::Object(map) => {
                out.push('{');
                if map.is_empty() {
                    // Match the game's odd formatting for empty objects.
                    out.push_str(newline);
                    out.push_str(newline);
                    out.push_str(&" ".repeat(indent));
                    out.push('}');
                    return;
                }

                out.push_str(newline);
                for (i, (k, v)) in map.iter().enumerate() {
                    out.push_str(&" ".repeat(indent + 4));
                    write_escaped_string_ascii(out, k);
                    out.push_str(": ");
                    v.write_ti_save(out, indent + 4, newline);
                    if i + 1 != map.len() {
                        out.push(',');
                    }
                    out.push_str(newline);
                }
                out.push_str(&" ".repeat(indent));
                out.push('}');
            }
        }
    }

    pub fn is_relational_ref(&self) -> Option<i64> {
        // Matches {"value": <int>} with optional "$type".
        let obj = self.as_object()?;
        let value = obj.get(statics::TI_REF_FIELD_VALUE)?;
        match value {
            TiValue::Number(n) => n.as_i64(),
            _ => None,
        }
    }
}

impl TiNumber {
    fn write_json5(&self, out: &mut String) {
        match self {
            TiNumber::I64(v) => out.push_str(&v.to_string()),
            TiNumber::U64(v) => out.push_str(&v.to_string()),
            TiNumber::F64(v) => {
                if v.is_nan() {
                    out.push_str("NaN");
                } else if v.is_infinite() {
                    if v.is_sign_negative() {
                        out.push_str("-Infinity");
                    } else {
                        out.push_str("Infinity");
                    }
                } else {
                    let mut buf = ryu::Buffer::new();
                    let s = buf.format(*v);
                    // Match the original game's use of uppercase exponent.
                    if s.contains('e') {
                        out.push_str(&s.replace('e', "E"));
                    } else {
                        out.push_str(s);
                    }
                }
            }
        }
    }

    fn write_ti_save(&self, out: &mut String) {
        match self {
            TiNumber::I64(_) | TiNumber::U64(_) => self.write_json5(out),
            TiNumber::F64(v) => {
                if v.is_nan() {
                    out.push_str("NaN");
                    return;
                }
                if v.is_infinite() {
                    if v.is_sign_negative() {
                        out.push_str("-Infinity");
                    } else {
                        out.push_str("Infinity");
                    }
                    return;
                }

                // Terra Invicta tends to use scientific notation for very small magnitudes
                // (e.g. 2E-05, 1E-07) rather than expanding to a long decimal.
                let abs = v.abs();
                if *v != 0.0 && abs < 1e-4 {
                    // Force scientific notation, uppercase E, and pad exponent to 2 digits.
                    // Rust's exp formatting produces minimal exponent digits (e.g. e-7),
                    // so we normalize it to TI's e-07 style.
                    let s = format!("{:e}", v);
                    if let Some((mantissa, exp)) = s.split_once('e') {
                        out.push_str(mantissa);
                        out.push('E');

                        let (sign, digits) = match exp.as_bytes().first().copied() {
                            Some(b'+') => ('+', &exp[1..]),
                            Some(b'-') => ('-', &exp[1..]),
                            _ => ('+', exp),
                        };

                        // Preserve '+' only if it was present originally (TI samples mostly show '-').
                        let had_plus = exp.starts_with('+');
                        if sign == '-' {
                            out.push('-');
                        } else if had_plus {
                            out.push('+');
                        }

                        let Ok(exp_num) = digits.parse::<u32>() else {
                            out.push_str(digits);
                            return;
                        };

                        if exp_num < 10 {
                            out.push('0');
                        }
                        out.push_str(&exp_num.to_string());
                    } else {
                        // Fallback: still enforce uppercase E if something odd occurs.
                        out.push_str(&s.replace('e', "E"));
                    }
                    return;
                }

                // Default: reuse the JSON5 float formatting (keeps exponent uppercase when used).
                self.write_json5(out);
            }
        }
    }
}

fn write_escaped_string(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                use std::fmt::Write as _;
                write!(out, "\\u{:04X}", c as u32).ok();
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_escaped_string_ascii(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                use std::fmt::Write as _;
                write!(out, "\\u{:04x}", c as u32).ok();
            }
            c if (c as u32) > 0x7F => {
                let cp = c as u32;
                if cp <= 0xFFFF {
                    use std::fmt::Write as _;
                    write!(out, "\\u{:04x}", cp).ok();
                } else {
                    // Encode as UTF-16 surrogate pair.
                    let u = cp - 0x1_0000;
                    let high = 0xD800 + ((u >> 10) & 0x3FF);
                    let low = 0xDC00 + (u & 0x3FF);
                    use std::fmt::Write as _;
                    write!(out, "\\u{:04x}\\u{:04x}", high, low).ok();
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

impl Serialize for TiValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TiValue::Null => serializer.serialize_unit(),
            TiValue::Bool(v) => serializer.serialize_bool(*v),
            TiValue::Number(n) => n.serialize(serializer),
            TiValue::String(s) => serializer.serialize_str(s),
            TiValue::Array(values) => values.serialize(serializer),
            TiValue::Object(map) => map.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for TiValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> de::Visitor<'de> for ValueVisitor {
            type Value = TiValue;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a JSON5 value")
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(TiValue::Null)
            }

            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(TiValue::Null)
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(TiValue::Bool(v))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(TiValue::Number(TiNumber::I64(v)))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(TiValue::Number(TiNumber::U64(v)))
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(TiValue::Number(TiNumber::F64(v)))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(TiValue::String(v.to_owned()))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(TiValue::String(v))
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element::<TiValue>()? {
                    values.push(value);
                }
                Ok(TiValue::Array(values))
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = IndexMap::new();
                while let Some((key, value)) = map.next_entry::<String, TiValue>()? {
                    values.insert(key, value);
                }
                Ok(TiValue::Object(values))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{TiNumber, TiValue};
    use crate::statics;
    use indexmap::IndexMap;

    #[test]
    fn parse_json5_supports_infinity_and_nan() {
        let v = TiValue::parse_json5("{ a: Infinity, b: -Infinity, c: NaN }").unwrap();
        let obj = v.as_object().unwrap();

        let a = obj.get("a").unwrap();
        let b = obj.get("b").unwrap();
        let c = obj.get("c").unwrap();

        match a {
            TiValue::Number(TiNumber::F64(x)) => assert!(x.is_infinite() && x.is_sign_positive()),
            _ => panic!("expected Infinity"),
        }
        match b {
            TiValue::Number(TiNumber::F64(x)) => assert!(x.is_infinite() && x.is_sign_negative()),
            _ => panic!("expected -Infinity"),
        }
        match c {
            TiValue::Number(TiNumber::F64(x)) => assert!(x.is_nan()),
            _ => panic!("expected NaN"),
        }
    }

    #[test]
    fn to_ti_save_pretty_empty_object_has_double_newline() {
        let v = TiValue::Object(IndexMap::new());
        assert_eq!(v.to_ti_save_pretty(), "{\n\n}");
    }

    #[test]
    fn to_ti_save_pretty_escapes_non_ascii_as_u16() {
        let v = TiValue::String("caf\u{00E9}".to_string());
        assert_eq!(v.to_ti_save_pretty(), "\"caf\\u00e9\"");
    }

    #[test]
    fn to_ti_save_pretty_escapes_astral_plane_as_surrogate_pair() {
        let v = TiValue::String("😀".to_string());
        assert_eq!(v.to_ti_save_pretty(), "\"\\ud83d\\ude00\"");
    }

    #[test]
    fn to_ti_save_pretty_uses_lowercase_hex() {
        let v = TiValue::String("Ä".to_string());
        let s = v.to_ti_save_pretty_with_newline(statics::NL_LF);
        assert_eq!(s, "\"\\u00c4\"");
    }

    #[test]
    fn numbers_use_uppercase_exponent() {
        let v = TiValue::Number(TiNumber::F64(1e-6));
        let s = v.to_json5_compact();
        assert!(s.contains('E'));
        assert!(!s.contains('e'));
    }

    #[test]
    fn ti_save_uses_scientific_for_small_values_with_padded_exponent() {
        let v = TiValue::Number(TiNumber::F64(2e-5));
        assert_eq!(v.to_ti_save_pretty(), "2E-05");

        let v = TiValue::Number(TiNumber::F64(1e-7));
        assert_eq!(v.to_ti_save_pretty(), "1E-07");
    }

    #[test]
    fn is_relational_ref_requires_integer_value_field() {
        let v = TiValue::parse_json5("{ value: 42 }").unwrap();
        assert_eq!(v.is_relational_ref(), Some(42));

        // Optional $type is ignored by the detector, but should still parse.
        let v = TiValue::parse_json5("{ $type: 'X', value: 7 }").unwrap();
        assert_eq!(v.is_relational_ref(), Some(7));

        // Floating-point values are not treated as relational refs.
        let v = TiValue::parse_json5("{ value: 1.25 }").unwrap();
        assert_eq!(v.is_relational_ref(), None);

        // Validate we are using the shared constant.
        assert_eq!(statics::TI_REF_FIELD_VALUE, "value");
    }
}
