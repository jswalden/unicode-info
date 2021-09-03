//! Parse the contents of `CaseFolding.txt`, the central code point registry
//! file.

extern crate proc_macro2;

use crate::constants::MAX_BMP;
use quote::quote;
use std::collections::HashMap;
use std::convert::TryFrom;

static CASE_FOLDING_TXT: &str = include_str!("data/CaseFolding.txt");

struct CaseFoldingParse {
    lines: std::str::Lines<'static>,
}

impl CaseFoldingParse {
    fn simple_and_common_foldings() -> CaseFoldingParse {
        CaseFoldingParse {
            lines: CASE_FOLDING_TXT.lines(),
        }
    }
}

impl Iterator for CaseFoldingParse {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // The entries in this file are in the following machine-readable
            // format:
            //
            // <code>; <status>; <mapping>; # <name>
            let line = self
                .lines
                .next()?
                .split('#')
                .nth(0)
                .expect("splitting returns at least one string");
            if line == "" {
                continue;
            }

            let row = line.split("; ").collect::<Vec<&'static str>>();
            assert!(row.len() == 4);

            // Unicode regular expression support depends only on common/simple
            // foldings.
            if ["C", "S"].contains(&row[1]) {
                let code = u32::from_str_radix(row[0], 16).expect("hex code");
                let mapping = u32::from_str_radix(row[2], 16).expect("hex mapping");
                return Some((code, mapping));
            }

            assert!(
                ["F", "T"].contains(&row[1]),
                "should see (C)ommon, (S)imple, (F)ull, and (T)urkish foldings"
            );
        }
    }
}

/// A type storing a code point and all (non-identical) code points that are
/// equivalent to it after case folding.
pub type CodeWithEquivalents = (u32, Vec<u32>);

/// `delta` in the `code + delta == mapping` identity used to convert from a
/// BMP code point to its folded code point.
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Delta(pub u16);

impl quote::ToTokens for Delta {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let num = self.0;
        let code = quote! {
            ::unicode_info::case_folding::Delta(#num)
        };
        tokens.extend(code);
    }
}

/// Data resulting from processing `CaseFolding.txt`.
pub struct CaseFoldingData {
    /// A list of `(code, [equivalents])` for every code that participates in
    /// non-identity case folding, ordered by code.  For example, if we had
    /// these mappings:
    ///
    /// ```text
    /// A -> C
    /// B -> C
    /// ```
    ///
    /// this vector would be
    ///
    /// ```text
    /// [(A, [B, C]), (B, [A, C]), (C, [A, B])]
    /// ```
    ///
    /// Arrays of equivalents are in sorted order.
    ///
    /// The included codes span the full BMP and non-BMP gamut.
    pub all_codes_with_equivalents: Vec<CodeWithEquivalents>,

    /// A list of unique `Delta` values.
    pub bmp_folding_table: Vec<Delta>,

    /// A vector, each element of which is the index in
    /// [`CaseFoldingData::bmp_folding_table`](CaseFoldingData::bmp_folding_table)
    /// of that code point's `Delta`.  For example, because `CaseFolding.txt`
    /// contains
    ///
    /// ```text
    /// U+0041 LATIN CAPITAL LETTER A -> U+0061 LATIN SMALL LETTER A
    /// ```
    ///
    /// we will have `bmp_folding_table[bmp_folding_index[0x0041] as usize] == Delta(0x0061 - 0x0041)`.
    pub bmp_folding_index: Vec<u32>,
}

type SortedMap<K, V> = std::collections::BTreeMap<K, V>;
type SortedSet<T> = std::collections::BTreeSet<T>;

/// Generate common and simple case-folding information from `CaseFolding.txt`.
///
/// Case folding is the process of converting code point sequences to a
/// canonical, folded form.  JavaScript depends upon the case folding process to
/// implement 1) case-insensitive comparisons for `/foo/iu` regular expressions
/// and 2) aspects of `Intl.Collator`.  (v8 handles the former for non-BMP code
/// points, and ICU handles the latter.  We record case folding information for
/// both BMP and non-BMP code points in
/// [`CaseFoldingData::all_codes_with_equivalents`](CaseFoldingData::all_codes_with_equivalents),
/// for testing purposes, but our generated folding tables only have to handle
/// BMP code points.)
///
/// Conceptually you can think of case folding, applied to the ASCII subset of
/// strings, as mapping them to lowercase.  Across the entirety of Unicode, for
/// historical reasons, the canonical folding isn't consistently lowercase or
/// consistently uppercase.  See
/// [Unicode §5.18 Case Mappings](https://www.unicode.org/versions/latest/ch05.pdf)
/// for full details.  In part for this reason, JavaScript only indirectly
/// exposes the actual case folding algorithm.
///
/// The entries in `CaseFolding.txt` map from code point to folded
/// code point, in four different and potentially overlapping ways.  Because
/// Unicode regular expressions
/// [depend](https://tc39.es/ecma262/#sec-runtime-semantics-canonicalize-ch)
/// upon only  "simple" and "common" foldings, we discard "Turkish" and "Full"
/// foldings during processing.
pub fn process_case_folding() -> CaseFoldingData {
    // Basic map of code -> folded for all Common/Simple mappings.
    let mut folding_map = SortedMap::<u32, u32>::new();

    // The inverse of `folding_map`: a map of folded -> vec![one or more codes].
    // (An example of a code point folded to by multiple code points: is
    // both U+03A3 GREEK CAPITAL LETTER SIGMA and U+03C2 GREEK SMALL LETTER
    // FINAL SIGMA fold to U+03C3 GREEK SMALL LETTER SIGMA.)
    let mut reverse_folding_map = SortedMap::<u32, Vec<u32>>::new();

    // Compute both of the above maps from the full set of one-way mappings.
    for (code, mapping) in CaseFoldingParse::simple_and_common_foldings() {
        folding_map.insert(code, mapping);

        reverse_folding_map
            .entry(mapping)
            .or_insert_with(|| vec![])
            .push(code);
    }

    // Build a (sorted) set of all code points participating in non-identity
    // case folding.
    let mut all_folding_codes = SortedSet::<u32>::new();
    all_folding_codes.extend(folding_map.keys());
    all_folding_codes.extend(reverse_folding_map.keys());

    // A list of `(code, [equivalents])` for every code that participates in
    // non-identity case folding, ordered by code.  (Thus if a code has two
    // equivalents, each of _those_ codes will have an entry whose equivalents
    // will be the other equivalent and this code.)
    //
    // By construction each `[equivalents]` is in sorted order.
    let mut all_codes_with_equivalents = Vec::<CodeWithEquivalents>::new();

    // Build a list of every "interesting" code and all the codes that map to
    // it.
    for code in all_folding_codes.iter() {
        let equivs: Vec<u32> = match folding_map.get(code) {
            Some(mapping) => {
                let mut equivs = vec![*mapping];
                equivs.extend(
                    reverse_folding_map
                        .get(mapping)
                        .expect("inverse mapping")
                        .iter()
                        .filter_map(|c| if *c != *code { Some(*c) } else { None }),
                );
                equivs
            }
            None => reverse_folding_map
                .get(code)
                .expect("must map either forward or backward")
                .clone(),
        };

        all_codes_with_equivalents.push((*code, equivs));
    }

    // A list of unique deltas from a code point to its folded code point.
    // This list starts with `Delta(0)` because entries in `bmp_folding_index`
    // are `0` unless `CaseFolding.txt` entries modify that.
    let mut bmp_folding_table: Vec<Delta> = vec![Delta(0)];

    // A hash mapping a `Delta` to its unique index in `bmp_folding_table`.
    let mut bmp_folding_cache = HashMap::<Delta, u32>::new();

    // `bmp_folding_index[c]` is the index into `bmp_folding_table` of the
    // `delta` to be added (with wrapping) to code point `c` to compute its
    // folded code point.
    //
    // Note that because indexes are initially `0`, every code point starts out
    // as mapping to `bmp_folding_table[0]`, i.e. `Delta(0)`, i.e. folding to
    // itself.  The loop below overwrites only the indexes with non-identity
    // folds.
    let mut bmp_folding_index = vec![0u32; (MAX_BMP + 1) as usize];

    for (code, mapping) in folding_map.iter().filter(|(code, _)| **code <= MAX_BMP) {
        let code = u16::try_from(*code).expect("valid because BMP");
        let mapping = u16::try_from(*mapping).expect("valid because BMP");

        // BMP case folding `code -> mapping` is implemented as successive table
        // lookups, that together produce `delta` from the identity
        // `code + delta == mapping`.
        let delta = Delta(u16::wrapping_sub(mapping, code));

        let index = match bmp_folding_cache.get(&delta) {
            None => {
                assert!(!bmp_folding_table.contains(&delta));
                let index = bmp_folding_table.len() as u32;
                bmp_folding_cache.insert(delta, index);
                bmp_folding_table.push(delta);
                index
            }
            Some(index) => *index,
        };

        bmp_folding_index[code as usize] = index;
    }

    CaseFoldingData {
        all_codes_with_equivalents,
        bmp_folding_table,
        bmp_folding_index,
    }
}

#[test]
fn check_case_folding() {
    let CaseFoldingData {
        all_codes_with_equivalents,
        bmp_folding_index,
        bmp_folding_table,
    } = process_case_folding();

    assert!(all_codes_with_equivalents.contains(&(0x0399, vec![0x03B9, 0x0345, 0x1FBE])),
            "GREEK CAPITAL LETTER IOTA, GREEK SMALL LETTER IOTA, COMBINING GREEK YPOGEGRAMMENI (GREEK NON-SPACING IOTA BELOW), GREEK PROSGEGRAMMENI");

    assert_eq!(
        bmp_folding_table[bmp_folding_index[0x0041] as usize],
        Delta(0x0061 - 0x0041),
        "verify 'A' -> 'a' correspondence noted in `CaseFoldingData` docs"
    );

    for (code, table_index) in bmp_folding_index.iter().enumerate() {
        let _computed_delta = bmp_folding_table[*table_index as usize];

        let _idx = code;
    }
}
