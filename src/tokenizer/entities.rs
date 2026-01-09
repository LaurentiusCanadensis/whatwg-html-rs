//! HTML5 named character reference decoding.
//!
//! Implements HTML5 character reference (entity) decoding per WHATWG spec.
//! Supports both named entities (&amp;, &nbsp;) and numeric references (&#60;, &#x3C;).

use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

/// Decode a named HTML entity (without the & and optional ;).
///
/// Returns the decoded character(s) if the entity is valid.
pub fn decode_entity(name: &str) -> Option<&'static str> {
    NAMED_ENTITIES.get(name).copied()
}

/// Check if an entity is a legacy entity that can be used without semicolon.
pub fn is_legacy_entity(name: &str) -> bool {
    LEGACY_ENTITIES.contains(name)
}

/// Decode a numeric character reference.
///
/// Returns the decoded character if valid, or the replacement character for invalid codepoints.
pub fn decode_numeric(codepoint: u32) -> char {
    // Check for replacements first (Windows-1252 compatibility)
    if let Some(&replacement) = NUMERIC_REPLACEMENTS.get(&codepoint) {
        return replacement;
    }

    // Check for invalid ranges
    if codepoint > 0x10FFFF {
        return '\u{FFFD}';
    }

    // Check for surrogate pairs (invalid in UTF-8)
    if (0xD800..=0xDFFF).contains(&codepoint) {
        return '\u{FFFD}';
    }

    // Check for noncharacters
    if is_noncharacter(codepoint) {
        // Return the character but it's technically an error
        return char::from_u32(codepoint).unwrap_or('\u{FFFD}');
    }

    // Check for control characters (except allowed ones)
    if is_control_character(codepoint) && !is_allowed_control(codepoint) {
        return char::from_u32(codepoint).unwrap_or('\u{FFFD}');
    }

    char::from_u32(codepoint).unwrap_or('\u{FFFD}')
}

/// Check if a codepoint is a noncharacter.
fn is_noncharacter(cp: u32) -> bool {
    // Noncharacters: U+FDD0-U+FDEF and U+nFFFE-U+nFFFF for each plane
    (0xFDD0..=0xFDEF).contains(&cp)
        || (cp & 0xFFFE) == 0xFFFE && cp <= 0x10FFFF
}

/// Check if a codepoint is a control character.
fn is_control_character(cp: u32) -> bool {
    (0x0001..=0x001F).contains(&cp) || (0x007F..=0x009F).contains(&cp)
}

/// Check if a control character is allowed.
fn is_allowed_control(cp: u32) -> bool {
    // TAB, LF, FF, CR, and SPACE are allowed
    matches!(cp, 0x09 | 0x0A | 0x0C | 0x0D | 0x20)
}

/// HTML5 numeric character reference replacements (for Windows-1252 compatibility).
static NUMERIC_REPLACEMENTS: Lazy<HashMap<u32, char>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(0x00, '\u{FFFD}'); // NULL -> REPLACEMENT CHARACTER
    m.insert(0x80, '\u{20AC}'); // EURO SIGN
    m.insert(0x82, '\u{201A}'); // SINGLE LOW-9 QUOTATION MARK
    m.insert(0x83, '\u{0192}'); // LATIN SMALL LETTER F WITH HOOK
    m.insert(0x84, '\u{201E}'); // DOUBLE LOW-9 QUOTATION MARK
    m.insert(0x85, '\u{2026}'); // HORIZONTAL ELLIPSIS
    m.insert(0x86, '\u{2020}'); // DAGGER
    m.insert(0x87, '\u{2021}'); // DOUBLE DAGGER
    m.insert(0x88, '\u{02C6}'); // MODIFIER LETTER CIRCUMFLEX ACCENT
    m.insert(0x89, '\u{2030}'); // PER MILLE SIGN
    m.insert(0x8A, '\u{0160}'); // LATIN CAPITAL LETTER S WITH CARON
    m.insert(0x8B, '\u{2039}'); // SINGLE LEFT-POINTING ANGLE QUOTATION MARK
    m.insert(0x8C, '\u{0152}'); // LATIN CAPITAL LIGATURE OE
    m.insert(0x8E, '\u{017D}'); // LATIN CAPITAL LETTER Z WITH CARON
    m.insert(0x91, '\u{2018}'); // LEFT SINGLE QUOTATION MARK
    m.insert(0x92, '\u{2019}'); // RIGHT SINGLE QUOTATION MARK
    m.insert(0x93, '\u{201C}'); // LEFT DOUBLE QUOTATION MARK
    m.insert(0x94, '\u{201D}'); // RIGHT DOUBLE QUOTATION MARK
    m.insert(0x95, '\u{2022}'); // BULLET
    m.insert(0x96, '\u{2013}'); // EN DASH
    m.insert(0x97, '\u{2014}'); // EM DASH
    m.insert(0x98, '\u{02DC}'); // SMALL TILDE
    m.insert(0x99, '\u{2122}'); // TRADE MARK SIGN
    m.insert(0x9A, '\u{0161}'); // LATIN SMALL LETTER S WITH CARON
    m.insert(0x9B, '\u{203A}'); // SINGLE RIGHT-POINTING ANGLE QUOTATION MARK
    m.insert(0x9C, '\u{0153}'); // LATIN SMALL LIGATURE OE
    m.insert(0x9E, '\u{017E}'); // LATIN SMALL LETTER Z WITH CARON
    m.insert(0x9F, '\u{0178}'); // LATIN CAPITAL LETTER Y WITH DIAERESIS
    m
});

/// Legacy named entities that can be used without a trailing semicolon.
static LEGACY_ENTITIES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "gt", "lt", "amp", "quot", "nbsp", "AMP", "QUOT", "GT", "LT", "COPY", "REG", "AElig",
        "Aacute", "Acirc", "Agrave", "Aring", "Atilde", "Auml", "Ccedil", "ETH", "Eacute", "Ecirc",
        "Egrave", "Euml", "Iacute", "Icirc", "Igrave", "Iuml", "Ntilde", "Oacute", "Ocirc",
        "Ograve", "Oslash", "Otilde", "Ouml", "THORN", "Uacute", "Ucirc", "Ugrave", "Uuml",
        "Yacute", "aacute", "acirc", "acute", "aelig", "agrave", "aring", "atilde", "auml",
        "brvbar", "ccedil", "cedil", "cent", "copy", "curren", "deg", "divide", "eacute", "ecirc",
        "egrave", "eth", "euml", "frac12", "frac14", "frac34", "iacute", "icirc", "iexcl", "igrave",
        "iquest", "iuml", "laquo", "macr", "micro", "middot", "not", "ntilde", "oacute", "ocirc",
        "ograve", "ordf", "ordm", "oslash", "otilde", "ouml", "para", "plusmn", "pound", "raquo",
        "reg", "sect", "shy", "sup1", "sup2", "sup3", "szlig", "thorn", "times", "uacute", "ucirc",
        "ugrave", "uml", "uuml", "yacute", "yen", "yuml",
    ]
    .into_iter()
    .collect()
});

/// HTML5 named character references.
/// This is a subset of the most common entities. The full list has 2231 entries.
static NAMED_ENTITIES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::with_capacity(300);

    // Core entities
    m.insert("amp", "&");
    m.insert("lt", "<");
    m.insert("gt", ">");
    m.insert("quot", "\"");
    m.insert("apos", "'");
    m.insert("nbsp", "\u{00A0}");

    // Uppercase variants
    m.insert("AMP", "&");
    m.insert("LT", "<");
    m.insert("GT", ">");
    m.insert("QUOT", "\"");

    // Latin-1 supplement
    m.insert("iexcl", "\u{00A1}");
    m.insert("cent", "\u{00A2}");
    m.insert("pound", "\u{00A3}");
    m.insert("curren", "\u{00A4}");
    m.insert("yen", "\u{00A5}");
    m.insert("brvbar", "\u{00A6}");
    m.insert("sect", "\u{00A7}");
    m.insert("uml", "\u{00A8}");
    m.insert("copy", "\u{00A9}");
    m.insert("COPY", "\u{00A9}");
    m.insert("ordf", "\u{00AA}");
    m.insert("laquo", "\u{00AB}");
    m.insert("not", "\u{00AC}");
    m.insert("shy", "\u{00AD}");
    m.insert("reg", "\u{00AE}");
    m.insert("REG", "\u{00AE}");
    m.insert("macr", "\u{00AF}");
    m.insert("deg", "\u{00B0}");
    m.insert("plusmn", "\u{00B1}");
    m.insert("sup2", "\u{00B2}");
    m.insert("sup3", "\u{00B3}");
    m.insert("acute", "\u{00B4}");
    m.insert("micro", "\u{00B5}");
    m.insert("para", "\u{00B6}");
    m.insert("middot", "\u{00B7}");
    m.insert("cedil", "\u{00B8}");
    m.insert("sup1", "\u{00B9}");
    m.insert("ordm", "\u{00BA}");
    m.insert("raquo", "\u{00BB}");
    m.insert("frac14", "\u{00BC}");
    m.insert("frac12", "\u{00BD}");
    m.insert("frac34", "\u{00BE}");
    m.insert("iquest", "\u{00BF}");

    // Latin letters with diacritics
    m.insert("Agrave", "\u{00C0}");
    m.insert("Aacute", "\u{00C1}");
    m.insert("Acirc", "\u{00C2}");
    m.insert("Atilde", "\u{00C3}");
    m.insert("Auml", "\u{00C4}");
    m.insert("Aring", "\u{00C5}");
    m.insert("AElig", "\u{00C6}");
    m.insert("Ccedil", "\u{00C7}");
    m.insert("Egrave", "\u{00C8}");
    m.insert("Eacute", "\u{00C9}");
    m.insert("Ecirc", "\u{00CA}");
    m.insert("Euml", "\u{00CB}");
    m.insert("Igrave", "\u{00CC}");
    m.insert("Iacute", "\u{00CD}");
    m.insert("Icirc", "\u{00CE}");
    m.insert("Iuml", "\u{00CF}");
    m.insert("ETH", "\u{00D0}");
    m.insert("Ntilde", "\u{00D1}");
    m.insert("Ograve", "\u{00D2}");
    m.insert("Oacute", "\u{00D3}");
    m.insert("Ocirc", "\u{00D4}");
    m.insert("Otilde", "\u{00D5}");
    m.insert("Ouml", "\u{00D6}");
    m.insert("times", "\u{00D7}");
    m.insert("Oslash", "\u{00D8}");
    m.insert("Ugrave", "\u{00D9}");
    m.insert("Uacute", "\u{00DA}");
    m.insert("Ucirc", "\u{00DB}");
    m.insert("Uuml", "\u{00DC}");
    m.insert("Yacute", "\u{00DD}");
    m.insert("THORN", "\u{00DE}");
    m.insert("szlig", "\u{00DF}");
    m.insert("agrave", "\u{00E0}");
    m.insert("aacute", "\u{00E1}");
    m.insert("acirc", "\u{00E2}");
    m.insert("atilde", "\u{00E3}");
    m.insert("auml", "\u{00E4}");
    m.insert("aring", "\u{00E5}");
    m.insert("aelig", "\u{00E6}");
    m.insert("ccedil", "\u{00E7}");
    m.insert("egrave", "\u{00E8}");
    m.insert("eacute", "\u{00E9}");
    m.insert("ecirc", "\u{00EA}");
    m.insert("euml", "\u{00EB}");
    m.insert("igrave", "\u{00EC}");
    m.insert("iacute", "\u{00ED}");
    m.insert("icirc", "\u{00EE}");
    m.insert("iuml", "\u{00EF}");
    m.insert("eth", "\u{00F0}");
    m.insert("ntilde", "\u{00F1}");
    m.insert("ograve", "\u{00F2}");
    m.insert("oacute", "\u{00F3}");
    m.insert("ocirc", "\u{00F4}");
    m.insert("otilde", "\u{00F5}");
    m.insert("ouml", "\u{00F6}");
    m.insert("divide", "\u{00F7}");
    m.insert("oslash", "\u{00F8}");
    m.insert("ugrave", "\u{00F9}");
    m.insert("uacute", "\u{00FA}");
    m.insert("ucirc", "\u{00FB}");
    m.insert("uuml", "\u{00FC}");
    m.insert("yacute", "\u{00FD}");
    m.insert("thorn", "\u{00FE}");
    m.insert("yuml", "\u{00FF}");

    // Greek letters
    m.insert("Alpha", "\u{0391}");
    m.insert("Beta", "\u{0392}");
    m.insert("Gamma", "\u{0393}");
    m.insert("Delta", "\u{0394}");
    m.insert("Epsilon", "\u{0395}");
    m.insert("Zeta", "\u{0396}");
    m.insert("Eta", "\u{0397}");
    m.insert("Theta", "\u{0398}");
    m.insert("Iota", "\u{0399}");
    m.insert("Kappa", "\u{039A}");
    m.insert("Lambda", "\u{039B}");
    m.insert("Mu", "\u{039C}");
    m.insert("Nu", "\u{039D}");
    m.insert("Xi", "\u{039E}");
    m.insert("Omicron", "\u{039F}");
    m.insert("Pi", "\u{03A0}");
    m.insert("Rho", "\u{03A1}");
    m.insert("Sigma", "\u{03A3}");
    m.insert("Tau", "\u{03A4}");
    m.insert("Upsilon", "\u{03A5}");
    m.insert("Phi", "\u{03A6}");
    m.insert("Chi", "\u{03A7}");
    m.insert("Psi", "\u{03A8}");
    m.insert("Omega", "\u{03A9}");
    m.insert("alpha", "\u{03B1}");
    m.insert("beta", "\u{03B2}");
    m.insert("gamma", "\u{03B3}");
    m.insert("delta", "\u{03B4}");
    m.insert("epsilon", "\u{03B5}");
    m.insert("zeta", "\u{03B6}");
    m.insert("eta", "\u{03B7}");
    m.insert("theta", "\u{03B8}");
    m.insert("iota", "\u{03B9}");
    m.insert("kappa", "\u{03BA}");
    m.insert("lambda", "\u{03BB}");
    m.insert("mu", "\u{03BC}");
    m.insert("nu", "\u{03BD}");
    m.insert("xi", "\u{03BE}");
    m.insert("omicron", "\u{03BF}");
    m.insert("pi", "\u{03C0}");
    m.insert("rho", "\u{03C1}");
    m.insert("sigmaf", "\u{03C2}");
    m.insert("sigma", "\u{03C3}");
    m.insert("tau", "\u{03C4}");
    m.insert("upsilon", "\u{03C5}");
    m.insert("phi", "\u{03C6}");
    m.insert("chi", "\u{03C7}");
    m.insert("psi", "\u{03C8}");
    m.insert("omega", "\u{03C9}");

    // Mathematical operators
    m.insert("forall", "\u{2200}");
    m.insert("part", "\u{2202}");
    m.insert("exist", "\u{2203}");
    m.insert("empty", "\u{2205}");
    m.insert("nabla", "\u{2207}");
    m.insert("isin", "\u{2208}");
    m.insert("notin", "\u{2209}");
    m.insert("ni", "\u{220B}");
    m.insert("prod", "\u{220F}");
    m.insert("sum", "\u{2211}");
    m.insert("minus", "\u{2212}");
    m.insert("lowast", "\u{2217}");
    m.insert("radic", "\u{221A}");
    m.insert("prop", "\u{221D}");
    m.insert("infin", "\u{221E}");
    m.insert("ang", "\u{2220}");
    m.insert("and", "\u{2227}");
    m.insert("or", "\u{2228}");
    m.insert("cap", "\u{2229}");
    m.insert("cup", "\u{222A}");
    m.insert("int", "\u{222B}");
    m.insert("there4", "\u{2234}");
    m.insert("sim", "\u{223C}");
    m.insert("cong", "\u{2245}");
    m.insert("asymp", "\u{2248}");
    m.insert("ne", "\u{2260}");
    m.insert("equiv", "\u{2261}");
    m.insert("le", "\u{2264}");
    m.insert("ge", "\u{2265}");
    m.insert("sub", "\u{2282}");
    m.insert("sup", "\u{2283}");
    m.insert("nsub", "\u{2284}");
    m.insert("sube", "\u{2286}");
    m.insert("supe", "\u{2287}");
    m.insert("oplus", "\u{2295}");
    m.insert("otimes", "\u{2297}");
    m.insert("perp", "\u{22A5}");
    m.insert("sdot", "\u{22C5}");

    // Arrows
    m.insert("larr", "\u{2190}");
    m.insert("uarr", "\u{2191}");
    m.insert("rarr", "\u{2192}");
    m.insert("darr", "\u{2193}");
    m.insert("harr", "\u{2194}");
    m.insert("crarr", "\u{21B5}");
    m.insert("lArr", "\u{21D0}");
    m.insert("uArr", "\u{21D1}");
    m.insert("rArr", "\u{21D2}");
    m.insert("dArr", "\u{21D3}");
    m.insert("hArr", "\u{21D4}");

    // General punctuation and symbols
    m.insert("bull", "\u{2022}");
    m.insert("hellip", "\u{2026}");
    m.insert("prime", "\u{2032}");
    m.insert("Prime", "\u{2033}");
    m.insert("oline", "\u{203E}");
    m.insert("frasl", "\u{2044}");
    m.insert("weierp", "\u{2118}");
    m.insert("image", "\u{2111}");
    m.insert("real", "\u{211C}");
    m.insert("trade", "\u{2122}");
    m.insert("alefsym", "\u{2135}");
    m.insert("euro", "\u{20AC}");

    // Spacing and formatting
    m.insert("ensp", "\u{2002}");
    m.insert("emsp", "\u{2003}");
    m.insert("thinsp", "\u{2009}");
    m.insert("zwnj", "\u{200C}");
    m.insert("zwj", "\u{200D}");
    m.insert("lrm", "\u{200E}");
    m.insert("rlm", "\u{200F}");

    // Quotation marks
    m.insert("ndash", "\u{2013}");
    m.insert("mdash", "\u{2014}");
    m.insert("lsquo", "\u{2018}");
    m.insert("rsquo", "\u{2019}");
    m.insert("sbquo", "\u{201A}");
    m.insert("ldquo", "\u{201C}");
    m.insert("rdquo", "\u{201D}");
    m.insert("bdquo", "\u{201E}");
    m.insert("dagger", "\u{2020}");
    m.insert("Dagger", "\u{2021}");
    m.insert("permil", "\u{2030}");
    m.insert("lsaquo", "\u{2039}");
    m.insert("rsaquo", "\u{203A}");

    // Card suits
    m.insert("spades", "\u{2660}");
    m.insert("clubs", "\u{2663}");
    m.insert("hearts", "\u{2665}");
    m.insert("diams", "\u{2666}");

    // Miscellaneous technical
    m.insert("lceil", "\u{2308}");
    m.insert("rceil", "\u{2309}");
    m.insert("lfloor", "\u{230A}");
    m.insert("rfloor", "\u{230B}");
    m.insert("lang", "\u{27E8}");
    m.insert("rang", "\u{27E9}");
    m.insert("loz", "\u{25CA}");

    // Additional common entities
    m.insert("OElig", "\u{0152}");
    m.insert("oelig", "\u{0153}");
    m.insert("Scaron", "\u{0160}");
    m.insert("scaron", "\u{0161}");
    m.insert("Yuml", "\u{0178}");
    m.insert("fnof", "\u{0192}");
    m.insert("circ", "\u{02C6}");
    m.insert("tilde", "\u{02DC}");

    m
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_entities() {
        assert_eq!(decode_entity("amp"), Some("&"));
        assert_eq!(decode_entity("lt"), Some("<"));
        assert_eq!(decode_entity("gt"), Some(">"));
        assert_eq!(decode_entity("quot"), Some("\""));
        assert_eq!(decode_entity("nbsp"), Some("\u{00A0}"));
    }

    #[test]
    fn test_uppercase_entities() {
        assert_eq!(decode_entity("AMP"), Some("&"));
        assert_eq!(decode_entity("LT"), Some("<"));
        assert_eq!(decode_entity("GT"), Some(">"));
    }

    #[test]
    fn test_unknown_entity() {
        assert_eq!(decode_entity("unknown"), None);
        assert_eq!(decode_entity(""), None);
    }

    #[test]
    fn test_legacy_entities() {
        assert!(is_legacy_entity("amp"));
        assert!(is_legacy_entity("nbsp"));
        assert!(is_legacy_entity("copy"));
        assert!(!is_legacy_entity("euro")); // Not a legacy entity
    }

    #[test]
    fn test_numeric_decoding() {
        assert_eq!(decode_numeric(60), '<');
        assert_eq!(decode_numeric(0x3C), '<');
        assert_eq!(decode_numeric(0x20AC), '\u{20AC}'); // Euro sign

        // Windows-1252 compatibility
        assert_eq!(decode_numeric(0x80), '\u{20AC}'); // Maps to Euro sign
        assert_eq!(decode_numeric(0x00), '\u{FFFD}'); // NULL -> replacement

        // Invalid codepoints
        assert_eq!(decode_numeric(0xD800), '\u{FFFD}'); // Surrogate
        assert_eq!(decode_numeric(0x110000), '\u{FFFD}'); // Out of range
    }
}
