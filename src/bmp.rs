//! Various information salient to handling _only_ BMP code points.

use crate::code_point_table;
use crate::code_point_table::CodePointInfo;
use crate::constants::{COMPATIBILITY_IDENTIFIER_PART, LINE_TERMINATOR, MAX_BMP, WHITE_SPACE};
use crate::derived_core_properties;
use proc_macro2;
use quote::quote;
use std::collections::HashMap;

/// A lightweight typed wrapper around `delta = mapping - code` (with wrapping)
/// for a BMP `code -> mapping` lowercasing or uppercasing operation.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CaseDelta(pub u16);

impl quote::ToTokens for CaseDelta {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let delta = self.0;
        let code = quote! {
            ::unicode_info::bmp::CaseDelta(#delta)
        };
        tokens.extend(code);
    }
}

/// Flag indicating a code point is treated as a JavaScript spacing character.
pub const FLAG_SPACE: u8 = 1 << 0;

/// Flag indicating a code point may appear at the start of an identifier.
pub const FLAG_UNICODE_ID_START: u8 = 1 << 1;

/// Flag indicating a code point may appear in an identifier only after the
/// first code point in the identifier.
pub const FLAG_UNICODE_ID_CONTINUE_ONLY: u8 = 1 << 2;

/// For a code point `c`, store relevant information about it.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CharacterInfo {
    /// A number `lower_delta` which, when added to the code point to which this
    /// `CharacterInfo` pertains, produces the lowercased version of the code
    /// point.  (For example, because of the
    /// U+0041 LATIN CAPITAL LETTER A -> U+0061 LATIN SMALL LETTER A lowercasing
    /// relationship, for the former code point we will have
    /// `lower_delta = CaseDelta(0x61 - 0x41)`.)
    pub lower_delta: CaseDelta,

    // A number `upper_delta` that provides the same functionality as
    // `lower_delta`, for a transformation to uppercase.  (For example, because
    // of the U+0061 LATIN SMALL LETTER A -> U+0041 LATIN CAPITAL LETTER A
    // uppercasing relationship, for the former code point we will have
    // `upper_delta = CaseDelta(0x41 - 0x61)` (with wrapping).
    pub upper_delta: CaseDelta,

    /// A bitwise-or of zero or more of [`FLAG_SPACE`],
    /// [`FLAG_UNICODE_ID_START`], and [`FLAG_UNICODE_ID_CONTINUE_ONLY`].
    pub flags: u8,
}

impl CharacterInfo {
    /// `CharacterInfo` for a code point whose lowercase and uppercase forms are
    /// the code point itself, with no flags set.
    fn all_zeroes() -> CharacterInfo {
        CharacterInfo {
            lower_delta: CaseDelta(0),
            upper_delta: CaseDelta(0),
            flags: 0,
        }
    }
}

impl quote::ToTokens for CharacterInfo {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let CharacterInfo {
            lower_delta,
            upper_delta,
            flags,
        } = self;
        let code = quote! {
            ::unicode_info::bmp::CharacterInfo {
                lower_delta: #lower_delta,
                upper_delta: #upper_delta,
                flags: #flags,
            }
        };
        tokens.extend(code);
    }
}

/// Information about various categories and mappings of BMP code points.
///
/// See [`crate::non_bmp`](crate::non_bmp) for non-BMP code point information.
pub struct BMPInfo {
    /// A list of unique `CharacterInfo` values.
    pub table: Vec<CharacterInfo>,

    /// A vector, each element of which is the index in `bmp_folding_table` of
    /// that code point's `Delta`.  For example, because `CaseFolding.txt`
    /// contains
    ///
    /// ```text
    /// U+0041 LATIN CAPITAL LETTER A -> U+0061 LATIN SMALL LETTER A
    /// ```
    ///
    /// we will have `bmp_folding_table[bmp_folding_index[0x0041] as usize] == Delta(0x0061 - 0x0041)`.
    pub index: Vec<u32>,
}

/// Generate various information about code points in the base multilingual
/// plane: code points that can be represented in a single UTF-16 code unit.
pub fn generate_bmp_info(
    code_point_table: &code_point_table::CodePointTable,
    derived_properties: &derived_core_properties::DerivedCorePropertyData,
) -> BMPInfo {
    // A list of unique `CharacterInfo` that pertain to some BMP code point.
    //
    // This list must starts with `CharacterInfo::all_zeroes()` so that
    // unassigned code points will have that behavior.
    let mut table = vec![CharacterInfo::all_zeroes()];

    // `index[c]` is the index into `table` of the `delta` to be added (with wrapping) to code point `c` to compute its
    // folded code point.
    //
    // Note that because indexes are initially `0`, every code point starts out
    // as mapping to `bmp_folding_table[0]`, i.e. `Delta(0)`, i.e. folding to
    // itself.  The loop below overwrites only the indexes with non-identity
    // folds.
    let mut index = vec![0u32; (MAX_BMP + 1) as usize];

    // A hash mapping a `CharacterInfo` to its unique index in `table`.
    let mut cache = HashMap::<CharacterInfo, u32>::new();
    cache.insert(CharacterInfo::all_zeroes(), 0);

    for (code, info) in code_point_table.iter().filter_map(|pair| {
        if *pair.0 <= MAX_BMP {
            Some((*pair.0 as u16, pair.1))
        } else {
            None
        }
    }) {
        let CodePointInfo {
            category,
            uppercase,
            lowercase,
            ..
        } = info;

        let lower_delta = CaseDelta(u16::wrapping_sub(*lowercase as u16, code));
        let upper_delta = CaseDelta(u16::wrapping_sub(*uppercase as u16, code));

        let mut flags = 0;

        let code = code as u32;

        if category == &"Zs" || WHITE_SPACE.contains(&code) || LINE_TERMINATOR.contains(&code) {
            flags |= FLAG_SPACE;
        }

        if derived_properties.id_start.contains(&code) {
            flags |= FLAG_UNICODE_ID_START;
        } else if derived_properties.id_continue.contains(&code)
            || COMPATIBILITY_IDENTIFIER_PART.contains(&code)
        {
            flags |= FLAG_UNICODE_ID_CONTINUE_ONLY;
        }

        let item = CharacterInfo {
            lower_delta,
            upper_delta,
            flags,
        };

        let i = match cache.get(&item) {
            None => {
                assert!(!table.contains(&item));
                let i = table.len() as u32;
                cache.insert(item, i);
                table.push(item);
                i
            }
            Some(i) => *i,
        };
        index[code as usize] = i;
    }

    BMPInfo { table, index }
}
