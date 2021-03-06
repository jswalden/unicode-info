//! Various code point constants and code point sets of general interest.

/// The maximum code point value that can be represented in a single UTF-16 code
/// unit.
pub const MAX_BMP: u32 = 0xFFFF;

/// Code for U+0009 CHARACTER TABULATION.
pub const CHARACTER_TABULATION: u32 = 0x0009;
/// Code for U+000B LINE TABULATION.
pub const LINE_TABULATION: u32 = 0x000B;
/// Code for U+000C FORM FEED.
pub const FORM_FEED: u32 = 0x000C;
/// Code for U+000D CARRIAGE RETURN.
pub const CARRIAGE_RETURN: u32 = 0x000D;
/// Code for U+000A LINE FEED.
pub const LINE_FEED: u32 = 0x000A;
/// Code for U+0020 SPACE.
pub const SPACE: u32 = 0x0020;
/// Code for U+0024 DOLLAR SIGN.
pub const DOLLAR_SIGN: u32 = 0x0024;
/// Code for U+0048 LATIN CAPITAL LETTER H.
pub const LATIN_CAPITAL_LETTER_H: u32 = 0x0048;
/// Code for U+0053 LATIN CAPITAL LETTER S.
pub const LATIN_CAPITAL_LETTER_S: u32 = 0x0053;
/// Code for U+005F LOW LINE.
pub const LOW_LINE: u32 = 0x005F;
/// Code for U+0069 LATIN SMALL LETTER I.
pub const LATIN_SMALL_LETTER_I: u32 = 0x0069;
/// Code for U+00A0 NO-BREAK SPACE.
pub const NO_BREAK_SPACE: u32 = 0x00A0;
/// Code for U+00DF LATIN SMALL LETTER SHARP S.
pub const LATIN_SMALL_LETTER_SHARP_S: u32 = 0x00DF;
/// Code for U+0130 LATIN CAPITAL LETTER I WITH DOT ABOVE.
pub const LATIN_CAPITAL_LETTER_I_WITH_DOT_ABOVE: u32 = 0x0130;
/// Code for U+0301 COMBINING ACUTE ACCENT.
pub const COMBINING_ACUTE_ACCENT: u32 = 0x0301;
/// Code for U+0307 COMBINING DOT ABOVE.
pub const COMBINING_DOT_ABOVE: u32 = 0x0307;
/// Code for U+0308 COMBINING DIAERESIS.
pub const COMBINING_DIAERESIS: u32 = 0x0308;
/// Code for U+0331 COMBINING MACRON BELOW.
pub const COMBINING_MACRON_BELOW: u32 = 0x0331;
/// Code for U+0390 GREEK SMALL LETTER IOTA WITH DIALYTIKA AND TONOS.
pub const GREEK_SMALL_LETTER_IOTA_WITH_DIALYTIKA_AND_TONOS: u32 = 0x0390;
/// Code for U+0399 GREEK CAPITAL LETTER IOTA.
pub const GREEK_CAPITAL_LETTER_IOTA: u32 = 0x0399;
/// Code for U+03A3 GREEK CAPITAL LETTER SIGMA.
pub const GREEK_CAPITAL_LETTER_SIGMA: u32 = 0x03A3;
/// Code for U+03C2 GREEK SMALL LETTER FINAL SIGMA.
pub const GREEK_SMALL_LETTER_FINAL_SIGMA: u32 = 0x03C2;
/// Code for U+03C3 GREEK SMALL LETTER SIGMA.
pub const GREEK_SMALL_LETTER_SIGMA: u32 = 0x03C3;
/// Code for U+1E96 LATIN SMALL LETTER H WITH LINE BELOW.
pub const LATIN_SMALL_LETTER_H_WITH_LINE_BELOW: u32 = 0x1E96;
/// Code for U+1F50 GREEK SMALL LETTER UPSILON WITH PSILI.
pub const GREEK_SMALL_LETTER_UPSILON_WITH_PSILI: u32 = 0x1F50;
/// Code for U+200C ZERO WIDTH NON-JOINER.
pub const ZERO_WIDTH_NON_JOINER: u32 = 0x200C;
/// Code for U+200D ZERO WIDTH JOINER.
pub const ZERO_WIDTH_JOINER: u32 = 0x200D;
/// Code for U+2028 LINE SEPARATOR.
pub const LINE_SEPARATOR: u32 = 0x2028;
/// Code for U+2029 PARAGRAPH SEPARATOR.
pub const PARAGRAPH_SEPARATOR: u32 = 0x2029;
/// Code for U+3000 IDEOGRAPHIC SPACE.
pub const IDEOGRAPHIC_SPACE: u32 = 0x3000;
/// Code for U+FEFF ZERO WIDTH NO-BREAK SPACE.
pub const ZERO_WIDTH_NO_BREAK_SPACE: u32 = 0xFEFF;

/// Code points matching the `WhiteSpace` production.
///
/// See <https://tc39.es/ecma262/#prod-WhiteSpace> for details.
pub const WHITE_SPACE: [u32; 6] = [
    CHARACTER_TABULATION,
    LINE_TABULATION,
    FORM_FEED,
    SPACE,
    NO_BREAK_SPACE,
    ZERO_WIDTH_NO_BREAK_SPACE,
];

/// Code points matching the `LineTerminator` production.
///
/// See <https://tc39.es/ecma262/#prod-LineTerminator> for details.
pub const LINE_TERMINATOR: [u32; 4] = [
    LINE_FEED,
    CARRIAGE_RETURN,
    LINE_SEPARATOR,
    PARAGRAPH_SEPARATOR,
];

/// Additional code points included in the `IdentifierPart` production that are
/// not code points with the Unicode property "ID_Continue".
///
/// See <https://tc39.es/ecma262/#prod-IdentifierPart> for details.
pub const COMPATIBILITY_IDENTIFIER_PART: [u32; 2] = [ZERO_WIDTH_NON_JOINER, ZERO_WIDTH_JOINER];
