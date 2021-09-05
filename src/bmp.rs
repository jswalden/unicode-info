//! Various information salient to handling _only_ BMP code points.

use crate::code_point_table;
use crate::constants::{COMPATIBILITY_IDENTIFIER_PART, LINE_TERMINATOR, MAX_BMP, WHITE_SPACE};
use crate::derived_core_properties;
use crate::types::{Flags, MappedCodePoint};
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

/// For a code point `c`, store relevant information about it.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CharacterInfo {
    /// A number `upper_delta` which, when added (with wrapping) to the code
    /// point to which this `CharacterInfo` pertains, produces the uppercase
    /// version of the code point.  (For example, because of the
    /// `U+0061 LATIN SMALL LETTER A -> U+0041 LATIN CAPITAL LETTER A`
    /// uppercasing relationship, for the former code point we will have
    /// `upper_delta = CaseDelta(u16::wrapping_sub(0x41, 0x61))`.)
    pub upper_delta: CaseDelta,

    // A number `lower_delta` that provides the same functionality as
    // `upper_delta`, for a transformation to lowercase.  (For example, because
    // of the `U+0041 LATIN CAPITAL LETTER A -> U+0061 LATIN SMALL LETTER A`
    // lowercasing relationship, for the former code point we will have
    // `lower_delta = CaseDelta(0x61 - 0x41)`.
    pub lower_delta: CaseDelta,

    /// Flags pertaining to the associated code point.
    pub flags: Flags,
}

impl CharacterInfo {
    /// `CharacterInfo` for a code point whose lowercase and uppercase forms are
    /// the code point itself, with no flags set.
    fn all_zeroes() -> CharacterInfo {
        CharacterInfo {
            lower_delta: CaseDelta(0),
            upper_delta: CaseDelta(0),
            flags: Flags(0),
        }
    }

    pub fn apply(&self, code: u32) -> MappedCodePoint {
        assert!(
            code <= MAX_BMP,
            "case info only tracked for BMP code points"
        );

        let upper = u16::wrapping_add(code as u16, self.upper_delta.0) as u32;
        let lower = u16::wrapping_add(code as u16, self.lower_delta.0) as u32;
        MappedCodePoint {
            upper,
            lower,
            flags: self.flags,
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

    for code_point in code_point_table
        .iter()
        .filter(|code_point| code_point.code <= MAX_BMP)
    {
        let code = code_point.code;
        let category = code_point.category();
        let uppercase = code_point.uppercase();
        let lowercase = code_point.lowercase();

        assert!(uppercase <= MAX_BMP);
        assert!(lowercase <= MAX_BMP);

        let lower_delta = CaseDelta(u16::wrapping_sub(lowercase as u16, code as u16));
        let upper_delta = CaseDelta(u16::wrapping_sub(uppercase as u16, code as u16));

        let mut flags = Flags(0);

        if category == "Zs" || WHITE_SPACE.contains(&code) || LINE_TERMINATOR.contains(&code) {
            flags.set_space();
        }

        if derived_properties.id_start.contains(&code) {
            flags.set_unicode_id_start();
        } else if derived_properties.id_continue.contains(&code)
            || COMPATIBILITY_IDENTIFIER_PART.contains(&code)
        {
            flags.set_unicode_id_continue_only();
        }

        let item = CharacterInfo {
            upper_delta,
            lower_delta,
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
