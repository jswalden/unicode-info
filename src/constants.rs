/// The maximum code point value that can be represented in a single UTF-16 code
/// unit.
pub const MAX_BMP: u32 = 0xFFFF;

/// Code points matching the `WhiteSpace` production.
///
/// See <https://tc39.es/ecma262/#prod-WhiteSpace> for details.
pub static WHITE_SPACE: [u32; 6] = [
    '\u{0009}' as u32, // CHARACTER TABULATION
    '\u{000B}' as u32, // LINE TABULATION
    '\u{000C}' as u32, // FORM FEED
    '\u{0020}' as u32, // SPACE
    '\u{00A0}' as u32, // NO-BREAK SPACE
    '\u{FEFF}' as u32, // ZERO WIDTH NO-BREAK SPACE (also byte order mark)
];

/// Code points matching the `LineTerminator` production.
///
/// See <https://tc39.es/ecma262/#prod-LineTerminator> for details.
pub static LINE_TERMINATOR: [u32; 4] = [
    '\u{000A}' as u32, // LINE FEED
    '\u{000D}' as u32, // CARRIAGE RETURN
    '\u{2028}' as u32, // LINE SEPARATOR
    '\u{2029}' as u32, // PARAGRAPH SEPARATOR
];
