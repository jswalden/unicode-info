//! Parse the contents of `CaseFolding.txt`, the central code point registry
//! file.

use crate::constants::MAX_BMP;

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

struct CaseFoldingInfo {
    code: u32,
    mapping: u32,
}

impl Iterator for CaseFoldingParse {
    type Item = CaseFoldingInfo;

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
                return Some(CaseFoldingInfo { code, mapping });
            }

            assert!(
                ["F", "T"].contains(&row[1]),
                "should expect to see only (C)ommon, (S)imple, (F)ull, and (T)urkish foldings"
            );
        }
    }
}

/// A type storing a code point and all (non-identical) code points that are
/// equivalent to it after case folding.
pub type CodeAndEquivalents = (u32, Vec<u32>);

/// Data resulting from processing `CaseFolding.txt`.
pub struct CaseFoldingData {
    all_codes_with_equivalents: Vec<CodeAndEquivalents>,
}

type SortedMap<K, V> = std::collections::BTreeMap<K, V>;
type SortedSet<T> = std::collections::BTreeSet<T>;

/// Generate common and simple case-folding information from `CaseFolding.txt`.
///
/// Case folding is the process of converting code point sequences to a
/// canonical, folded form.  JavaScript depends upon the case folding process to
/// implement Unicode case-insensitive `/foo/iu` regular expressions.
/// (`Intl.Collator` also depends upon case folding, but such dependence is
/// internal to ICU and therefore doesn't have to be addressed here.)
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
    // (An example of a code point folded to by multiple code points is
    // U+03C3 GREEK SMALL LETTER SIGMA: both U+03A3 GREEK CAPITAL LETTER SIGMA
    // and U+03C2 GREEK SMALL LETTER FINAL SIGMA fold to it.)
    let mut reverse_folding_map = SortedMap::<u32, Vec<u32>>::new();

    // Compute both of the above maps from the full set of one-way mappings.
    for CaseFoldingInfo { code, mapping } in CaseFoldingParse::simple_and_common_foldings() {
        folding_map.insert(code, mapping);

        reverse_folding_map
            .entry(mapping)
            .or_insert(vec![])
            .push(code);
    }

    // Build a (sorted) set of all code points participating in non-identity
    // case folding.
    let mut all_folding_codes = SortedSet::<u32>::new();
    all_folding_codes.extend(folding_map.keys());
    all_folding_codes.extend(reverse_folding_map.keys());

    let mut all_codes_with_equivalents = Vec::<CodeAndEquivalents>::new();

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

    for _ in all_folding_codes.iter().filter(|code| **code <= MAX_BMP) {}

    CaseFoldingData {
        all_codes_with_equivalents,
    }
}

#[test]
fn check_case_folding() {
    let CaseFoldingData {
        all_codes_with_equivalents,
    } = process_case_folding();

    assert!(all_codes_with_equivalents.contains(&(0x0399, vec![0x03B9, 0x0345, 0x1FBE])),
            "GREEK CAPITAL LETTER IOTA, GREEK SMALL LETTER IOTA, COMBINING GREEK YPOGEGRAMMENI (GREEK NON-SPACING IOTA BELOW), GREEK PROSGEGRAMMENI");
}
