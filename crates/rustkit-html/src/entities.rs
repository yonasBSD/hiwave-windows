//! HTML entity decoding.

use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref ENTITIES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Common named entities
        m.insert("lt", "<");
        m.insert("gt", ">");
        m.insert("amp", "&");
        m.insert("quot", "\"");
        m.insert("apos", "'");
        m.insert("nbsp", "\u{00A0}");
        m.insert("copy", "©");
        m.insert("reg", "®");
        m.insert("trade", "™");
        m.insert("hellip", "…");
        m.insert("mdash", "—");
        m.insert("ndash", "–");
        m.insert("ldquo", "\u{201C}");
        m.insert("rdquo", "\u{201D}");
        m.insert("lsquo", "\u{2018}");
        m.insert("rsquo", "\u{2019}");
        m.insert("bull", "•");
        m.insert("middot", "·");
        m.insert("times", "×");
        m.insert("divide", "÷");
        m.insert("euro", "€");
        m.insert("pound", "£");
        m.insert("yen", "¥");
        m.insert("cent", "¢");
        m.insert("deg", "°");
        m.insert("plusmn", "±");
        m.insert("micro", "µ");
        m.insert("para", "¶");
        m.insert("sect", "§");
        m.insert("frac14", "¼");
        m.insert("frac12", "½");
        m.insert("frac34", "¾");
        m.insert("sup1", "¹");
        m.insert("sup2", "²");
        m.insert("sup3", "³");
        m.insert("alpha", "α");
        m.insert("beta", "β");
        m.insert("gamma", "γ");
        m.insert("delta", "δ");
        m.insert("epsilon", "ε");
        m.insert("pi", "π");
        m.insert("sigma", "σ");
        m.insert("omega", "ω");
        m
    };
}

/// Decode HTML entities in a string.
pub fn decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            // Try to parse an entity
            let mut entity = String::new();
            let mut found_end = false;

            // Look ahead for entity content
            while let Some(&next_ch) = chars.peek() {
                if next_ch == ';' {
                    chars.next(); // consume ';'
                    found_end = true;
                    break;
                } else if next_ch.is_alphanumeric() || next_ch == '#' {
                    entity.push(next_ch);
                    chars.next();
                } else {
                    // Invalid entity character
                    break;
                }

                // Limit entity length to prevent DoS
                if entity.len() > 32 {
                    break;
                }
            }

            if found_end && !entity.is_empty() {
                if entity.starts_with('#') {
                    // Numeric entity
                    if let Some(decoded) = decode_numeric(&entity[1..]) {
                        result.push_str(&decoded);
                        continue;
                    }
                } else {
                    // Named entity
                    if let Some(&decoded) = ENTITIES.get(entity.as_str()) {
                        result.push_str(decoded);
                        continue;
                    }
                }
            }

            // If we couldn't decode, output the original
            result.push('&');
            result.push_str(&entity);
            if found_end {
                result.push(';');
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn decode_numeric(num_str: &str) -> Option<String> {
    let (radix, digits) = if num_str.starts_with('x') || num_str.starts_with('X') {
        (16, &num_str[1..])
    } else {
        (10, num_str)
    };

    let code_point = u32::from_str_radix(digits, radix).ok()?;
    char::from_u32(code_point).map(|c| c.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_basic_entities() {
        assert_eq!(decode("&lt;"), "<");
        assert_eq!(decode("&gt;"), ">");
        assert_eq!(decode("&amp;"), "&");
        assert_eq!(decode("&quot;"), "\"");
        assert_eq!(decode("&apos;"), "'");
    }

    #[test]
    fn test_decode_numeric_entities() {
        assert_eq!(decode("&#65;"), "A");
        assert_eq!(decode("&#x41;"), "A");
        assert_eq!(decode("&#x1F600;"), "😀");
    }

    #[test]
    fn test_decode_multiple_entities() {
        assert_eq!(decode("&lt;div&gt;"), "<div>");
        assert_eq!(decode("Tom &amp; Jerry"), "Tom & Jerry");
    }

    #[test]
    fn test_decode_unknown_entity() {
        assert_eq!(decode("&unknown;"), "&unknown;");
    }

    #[test]
    fn test_decode_no_entities() {
        assert_eq!(decode("hello world"), "hello world");
    }

    #[test]
    fn test_decode_incomplete_entity() {
        assert_eq!(decode("&lt"), "&lt");
        assert_eq!(decode("&"), "&");
    }

    #[test]
    fn test_decode_nbsp() {
        assert_eq!(decode("&nbsp;"), "\u{00A0}");
    }

    #[test]
    fn test_decode_copyright() {
        assert_eq!(decode("&copy;"), "©");
    }
}

