//! Processes `SpecialCasing.txt` to extract all special casing information.

use crate::bmp;
use crate::constants::MAX_BMP;
use crate::types::MappedCodePoint;
#[cfg(test)]
use crate::{
    code_point_table,
    constants::{
        COMBINING_DOT_ABOVE, GREEK_CAPITAL_LETTER_SIGMA, GREEK_SMALL_LETTER_FINAL_SIGMA,
        GREEK_SMALL_LETTER_SIGMA, LATIN_CAPITAL_LETTER_I_WITH_DOT_ABOVE, LATIN_CAPITAL_LETTER_S,
        LATIN_SMALL_LETTER_I, LATIN_SMALL_LETTER_SHARP_S,
    },
    derived_core_properties,
};
use std::collections::BTreeMap;
#[cfg(test)]
use std::{collections::HashSet, iter::FromIterator};

static SPECIAL_CASING_TXT: &str = include_str!("data/SpecialCasing.txt");

pub struct SpecialCase {
    code: u32,
    lower: Vec<u32>,
    upper: Vec<u32>,
    languages: Vec<&'static str>,
    contexts: Vec<&'static str>,
}

struct SpecialCasing {
    lines: std::str::Lines<'static>,
}

impl SpecialCasing {
    fn read() -> SpecialCasing {
        SpecialCasing {
            lines: SPECIAL_CASING_TXT.lines(),
        }
    }
}

impl Iterator for SpecialCasing {
    type Item = SpecialCase;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Format:
            // <code>; <lower>; <title>; <upper>; (<condition_list>;)? # <comment>
            let line_with_comment = self.lines.next()?;
            let line = line_with_comment
                .split('#')
                .nth(0)
                .expect("splitting returns at least one string");
            if line.is_empty() {
                continue;
            }

            let mut fields = line.split(';');

            let code =
                u32::from_str_radix(fields.next().expect("code field").trim(), 16).expect("code");

            let mut parse_next_codes = || {
                let field = fields.next().expect("codes field").trim();
                if field.is_empty() {
                    vec![] // a code point can be replaced with nothing
                } else {
                    field
                        .split(' ')
                        .map(|code| u16::from_str_radix(code, 16).expect("bad code in list") as u32)
                        .collect::<Vec<u32>>()
                }
            };

            let lower = parse_next_codes();
            let _title = parse_next_codes();
            let upper = parse_next_codes();

            let mut languages = vec![];
            let mut contexts = vec![];
            let conditions = fields.next().expect("condition_list").trim();
            if !conditions.is_empty() {
                for cond in conditions.split(' ') {
                    if cond.chars().nth(0).expect("condition").is_lowercase() {
                        languages.push(cond);
                    } else {
                        contexts.push(cond);
                    }
                }
            }

            return Some(SpecialCase {
                code,
                lower,
                upper,
                languages,
                contexts,
            });
        }
    }
}

// We use `BTreeMap` for mappings so that keys are conveniently exposed in
// consistent, sorted order.

/// A mapping from code point to replacement code point sequence, independent of
/// language or context within a string.
pub type UnconditionalMapping = BTreeMap<u32, Vec<u32>>; // BTreeMap offers sorted keys

/// A mapping from code point to replacement code point sequence and the context
/// in which the mapping should be applied.  (It is possible for there to be no
/// context, if language-dependency is implicit in where this type appears.)
pub type ContextualMapping<Context> = BTreeMap<u32, (Vec<u32>, Context)>;

/// Casing mappings computed from `SpecialCasing.txt`.
///
/// Some case mappings are applied only in contexts where ICU will handle things
/// for us, or where we can simply inline the relevant handling and not depend
/// on a fully generalized system.  If this changes, the `#[cfg(test)]`
/// attributes on some of these fields may have to be removed and the fields
/// made public.  Assertions in a test function far below hopefully will be able
/// to detect this.
pub struct SpecialCasingData {
    /// Unconditional mappings, performed for all languages and contexts, when
    /// lowercasing.
    pub unconditional_tolower: UnconditionalMapping,

    /// Unconditional mappings, performed for all languages and contexts, when
    /// uppercasing.
    pub unconditional_toupper: UnconditionalMapping,

    /// Lowercasings that apply in particular contexts but independent of
    /// language.
    #[cfg(test)]
    conditional_tolower: ContextualMapping<&'static str>,

    /// Uppercasings that apply in particular contexts but independent of
    /// language.
    #[cfg(test)]
    conditional_toupper: ContextualMapping<&'static str>,

    /// Language-dependent lowercasings, that potentially only apply in a
    /// particular context.
    #[cfg(test)]
    lang_conditional_tolower: BTreeMap<&'static str, ContextualMapping<Option<&'static str>>>,

    /// Language-dependent uppercasings, that potentially only apply in a
    /// particular context.
    #[cfg(test)]
    lang_conditional_toupper: BTreeMap<&'static str, ContextualMapping<Option<&'static str>>>,
}

/// Generate sets containing code points within salient categories.
pub fn process_special_casing(bmp: &bmp::BMPInfo) -> SpecialCasingData {
    // Use BTreeMap for all these maps for naturally sorted keys ordering.

    // Unconditional special casing.
    let mut unconditional_tolower = UnconditionalMapping::new();
    let mut unconditional_toupper = UnconditionalMapping::new();

    // Conditional special casing: applicable in context yet
    // language-independent.
    let mut conditional_tolower = ContextualMapping::<&'static str>::new();
    let mut conditional_toupper = ContextualMapping::<&'static str>::new();

    // Conditional special casing: language-dependent, possibly only applicable
    // in context.
    type LangToMapping = BTreeMap<&'static str, ContextualMapping<Option<&'static str>>>;
    let mut lang_conditional_tolower = LangToMapping::new();
    let mut lang_conditional_toupper = LangToMapping::new();

    let case_info = |code: u32| bmp.table[bmp.index[code as usize] as usize].apply(code);

    for SpecialCase {
        code,
        upper,
        lower,
        languages,
        contexts,
    } in SpecialCasing::read()
    {
        assert!(code <= MAX_BMP, "non-BMP special not handled yet");
        assert!(languages.len() <= 1, "only 0/1 languages handled");
        assert!(contexts.len() <= 1, "only 0/1 casing contexts handled");

        let MappedCodePoint {
            lower: default_lower,
            upper: default_upper,
            ..
        } = case_info(code);

        let has_special_lower = lower.len() != 1 || lower[0] != default_lower;
        let has_special_upper = upper.len() != 1 || upper[0] != default_upper;

        // Invariant: If |code| has casing per UnicodeData.txt, then it also has
        // casing rules in SpecialCasing.txt.
        assert!(code == default_lower || lower.len() != 1 || code != lower[0]);
        assert!(code == default_upper || upper.len() != 1 || code != upper[0]);

        let language: Option<&'static str> = match languages.get(0) {
            Some(language) => Some(*language),
            None => None,
        };
        let context = match contexts.get(0) {
            Some(context) => Some(*context),
            None => None,
        };

        match (language, context) {
            (None, None) => {
                if has_special_lower {
                    unconditional_tolower.insert(code, lower);
                }
                if has_special_upper {
                    unconditional_toupper.insert(code, upper);
                }
            }
            (None, Some(context)) => {
                if has_special_lower {
                    conditional_tolower.insert(code, (lower, context));
                }
                if has_special_upper {
                    conditional_toupper.insert(code, (upper, context));
                }
            }
            (Some(language), context) => {
                if has_special_lower {
                    lang_conditional_tolower
                        .entry(language)
                        .or_insert_with(|| ContextualMapping::new())
                        .insert(code, (lower, context));
                }
                if has_special_upper {
                    lang_conditional_toupper
                        .entry(language)
                        .or_insert_with(|| ContextualMapping::new())
                        .insert(code, (upper, context));
                }
            }
        };
    }

    SpecialCasingData {
        unconditional_tolower,
        unconditional_toupper,
        #[cfg(test)]
        conditional_tolower,
        #[cfg(test)]
        conditional_toupper,
        #[cfg(test)]
        lang_conditional_tolower,
        #[cfg(test)]
        lang_conditional_toupper,
    }
}

#[test]
fn check_special_casing() {
    let cpt = code_point_table::generate_code_point_table();
    let dcp = derived_core_properties::process_derived_core_properties();
    let bmp = bmp::generate_bmp_info(&cpt, &dcp);

    let case_info = |code: u32| bmp.table[bmp.index[code as usize] as usize].apply(code);

    let SpecialCasingData {
        unconditional_tolower,
        unconditional_toupper,
        conditional_tolower,
        conditional_toupper,
        lang_conditional_tolower,
        lang_conditional_toupper,
    } = process_special_casing(&bmp);

    let lower_case = |code| case_info(code).lower;
    let upper_case = |code| case_info(code).upper;

    fn accept_ascii(code: &&u32) -> bool {
        **code <= 0x7F
    }
    fn accept_latin1(code: &&u32) -> bool {
        **code <= 0xFF
    }

    fn is_empty<I>(mut iter: I) -> bool
    where
        I: Iterator,
    {
        iter.next().is_none()
    }

    // Ensure no ASCII code points have special case mappings.
    assert!(is_empty(unconditional_tolower.keys().filter(accept_ascii)));
    assert!(is_empty(unconditional_toupper.keys().filter(accept_ascii)));
    assert!(is_empty(conditional_tolower.keys().filter(accept_ascii)));
    assert!(is_empty(conditional_toupper.keys().filter(accept_ascii)));

    // Ensure no Latin-1 code points have special lower case mappings.
    assert!(is_empty(unconditional_tolower.keys().filter(accept_latin1)));
    assert!(is_empty(conditional_tolower.keys().filter(accept_latin1)));

    // Ensure no Latin-1 code points have conditional special upper case
    // mappings.
    assert!(is_empty(conditional_toupper.keys().filter(accept_latin1)));

    // Ensure U+00DF LATIN SMALL LETTER SHARP S is the only Latin-1 code point
    // with a special upper case mapping.
    assert!([LATIN_SMALL_LETTER_SHARP_S]
        .iter()
        .eq(unconditional_toupper.keys().filter(accept_latin1)));

    // Ensure U+0130 LATIN CAPITAL LETTER I WITH DOT ABOVE is the only code
    // point with a special lower case mapping.
    assert!([LATIN_CAPITAL_LETTER_I_WITH_DOT_ABOVE]
        .iter()
        .eq(unconditional_tolower.keys()));

    // Ensure no code points have language-independent conditional upper case
    // mappings.
    assert!(is_empty(conditional_toupper.iter()));

    // Ensure U+03A3 GREEK CAPITAL LETTER SIGMA is the only code point with
    // language-independent conditional lower case mapping.
    assert!([GREEK_CAPITAL_LETTER_SIGMA]
        .iter()
        .eq(conditional_tolower.keys()));

    // Verify U+0130 LATIN CAPITAL LETTER I WITH DOT ABOVE and
    // U+03A3 GREEK CAPITAL LETTER SIGMA have simple, non-identity lower
    // case mappings.
    assert!([
        LATIN_CAPITAL_LETTER_I_WITH_DOT_ABOVE,
        GREEK_CAPITAL_LETTER_SIGMA
    ]
    .iter()
    .all(|ch| *ch != lower_case(*ch)));

    // Ensure Azeri, Lithuanian, and Turkish are the only languages with
    // conditional case mappings.
    assert_eq!(
        vec![&"az", &"lt", &"tr"],
        lang_conditional_tolower
            .keys()
            .collect::<Vec<&&'static str>>()
    );
    assert_eq!(
        vec![&"az", &"lt", &"tr"],
        lang_conditional_toupper
            .keys()
            .collect::<Vec<&&'static str>>()
    );

    // Verify that the maximum case-mapping length is three characters.
    // (Do we depend/rely on this in specific places?  It would be trivial to
    // return this maximum from this code for a code-based dependency...)
    assert!(
        unconditional_tolower
            .values()
            .chain(unconditional_toupper.values())
            .chain(
                conditional_tolower
                    .values()
                    .map(|(replacements, _)| replacements),
            )
            .chain(
                conditional_toupper
                    .values()
                    .map(|(replacements, _)| replacements),
            )
            .map(|replacements| replacements.len())
            .max()
            .expect("replacement list is nonempty")
            <= 3,
        "the maximum replacement-sequence length is three code points"
    );

    // Ensure all case mapping contexts are known (see Unicode 9.0,
    // ??3.13 Default Case Algorithms).
    assert!(HashSet::<&'static str>::from_iter([
        "After_I",
        "After_Soft_Dotted",
        "Final_Sigma",
        "More_Above",
        "Not_Before_Dot",
    ])
    .is_superset(
        &(conditional_tolower.values().map(|(_, context)| *context))
            .chain(conditional_toupper.values().map(|(_, context)| *context))
            .chain(
                lang_conditional_tolower
                    .values()
                    .flat_map(|dict| dict.values())
                    .filter_map(|(_, context)| match *context {
                        Some(context) => Some(context),
                        None => None,
                    }),
            )
            .chain(
                lang_conditional_toupper
                    .values()
                    .flat_map(|dict| dict.values())
                    .filter_map(|(_, context)| match *context {
                        Some(context) => Some(context),
                        None => None,
                    }),
            )
            .collect::<HashSet<&'static str>>()
    ));

    // Special casing for U+00DF LATIN SMALL LETTER SHARP S.
    assert_eq!(
        upper_case(LATIN_SMALL_LETTER_SHARP_S),
        LATIN_SMALL_LETTER_SHARP_S
    );
    assert_eq!(
        unconditional_toupper[&LATIN_SMALL_LETTER_SHARP_S],
        [LATIN_CAPITAL_LETTER_S, LATIN_CAPITAL_LETTER_S]
    );

    // Special casing for U+0130 LATIN CAPITAL LETTER I WITH DOT ABOVE.
    assert_eq!(
        unconditional_tolower[&LATIN_CAPITAL_LETTER_I_WITH_DOT_ABOVE],
        [LATIN_SMALL_LETTER_I, COMBINING_DOT_ABOVE]
    );

    // Special casing for U+03A3 GREEK CAPITAL LETTER SIGMA.
    assert_eq!(
        lower_case(GREEK_CAPITAL_LETTER_SIGMA),
        GREEK_SMALL_LETTER_SIGMA
    );
    assert_eq!(
        conditional_tolower[&GREEK_CAPITAL_LETTER_SIGMA],
        (vec![GREEK_SMALL_LETTER_FINAL_SIGMA], "Final_Sigma")
    );
}
