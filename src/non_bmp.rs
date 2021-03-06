//! Various information salient to handling non-BMP code points while ignoring
//! all BMP code points.

use crate::code_point_table;
use crate::constants::MAX_BMP;
use crate::derived_core_properties;
use crate::types::{CaseMap, CodePointSet};

use derived_core_properties::DerivedCorePropertyData;

/// Information about various categories and mappings of non-BMP code point.
///
/// See [`crate::bmp`](crate::bmp) for BMP code point information.
pub struct NonBMPInfo {
    /// A mapping of every non-BMP code point to its lowercase form, *when the
    /// lowercase form is different*.  (Identity mappings are not included.)
    pub lowercase_map: CaseMap,

    /// A mapping of every non-BMP code point to its uppercase form, *when the
    /// uppercase form is different*.  (Identity mappings are not included.)
    pub uppercase_map: CaseMap,

    /// The set of all non-BMP code points in the Zs category.
    pub space_set: CodePointSet,

    /// The set of all non-BMP code points in the ID_Start derived category,
    /// which is to say code points that may appear at the start of an
    /// identifier.
    pub id_start_set: CodePointSet,

    /// The set of all non-BMP code points in the ID_Continue derived category,
    /// which is to say code points that may appear within an identifier after
    /// its start.
    pub id_continue_set: CodePointSet,
}

/// Generate various information about code points outside the base multilingual
/// plane: code points that can't be represented in a single UTF-16 code unit.
pub fn generate_non_bmp_info(code_point_table: &code_point_table::CodePointTable) -> NonBMPInfo {
    let DerivedCorePropertyData {
        id_start: derived_id_start,
        id_continue: derived_id_continue,
    } = derived_core_properties::process_derived_core_properties();

    let mut lowercase_map = CaseMap::new();
    let mut uppercase_map = CaseMap::new();
    let mut space_set = CodePointSet::new();
    let mut id_start_set = CodePointSet::new();
    let mut id_continue_set = CodePointSet::new();

    for code_point in code_point_table
        .iter()
        .filter(|code_point| code_point.code > MAX_BMP)
    {
        let code = code_point.code;

        let lower = code_point.lowercase();
        if code != lower {
            assert!(
                lower > MAX_BMP,
                "to_lower_case_non_bmp_trail and its users assume that every
                non-BMP code point maps to a non-BMP code point"
            );
            lowercase_map.insert(code, lower);
        }

        let upper = code_point.uppercase();
        if code != upper {
            assert!(
                upper > MAX_BMP,
                "to_upper_case_non_bmp_trail and its users assume that every
                 non-BMP code point maps to a non-BMP code point"
            );
            uppercase_map.insert(code, upper);
        }
        if code_point.category() == "Zs" {
            space_set.insert(code);
        }
        if derived_id_start.contains(&code) {
            id_start_set.insert(code);
        }
        if derived_id_continue.contains(&code) {
            id_continue_set.insert(code);
        }
    }

    NonBMPInfo {
        lowercase_map,
        uppercase_map,
        space_set,
        id_start_set,
        id_continue_set,
    }
}

#[test]
fn non_bmp_lowercase_map() {
    let table = code_point_table::generate_code_point_table();
    let non_bmp_info = generate_non_bmp_info(&table);

    assert_eq!(
        table.full_name(0x10403),
        "U+10403 DESERET CAPITAL LETTER LONG AH"
    );
    assert_eq!(
        table.full_name(0x1042B),
        "U+1042B DESERET SMALL LETTER LONG AH"
    );
    assert_eq!(non_bmp_info.lowercase_map.get(&0x10403), Some(&0x1042B));
}

#[cfg(test)]
const DESERET_SMALL_LETTER_LONG_AH: u32 = 0x1042B;
#[cfg(test)]
const DESERET_CAPITAL_LETTER_LONG_AH: u32 = 0x10403;
#[cfg(test)]
const OLD_PERSIAN_SIGN_AURAMAZDAA: u32 = 0x103C8;

#[test]
fn check_constant_code_point_values() {
    let table = code_point_table::generate_code_point_table();
    assert_eq!(
        table.full_name(DESERET_SMALL_LETTER_LONG_AH),
        "U+1042B DESERET SMALL LETTER LONG AH"
    );
    assert_eq!(
        table.full_name(DESERET_CAPITAL_LETTER_LONG_AH),
        "U+10403 DESERET CAPITAL LETTER LONG AH"
    );
    assert_eq!(
        table.name(OLD_PERSIAN_SIGN_AURAMAZDAA),
        "OLD PERSIAN SIGN AURAMAZDAA"
    );
}

#[test]
fn non_bmp_uppercase_map() {
    let table = code_point_table::generate_code_point_table();
    let non_bmp_info = generate_non_bmp_info(&table);

    assert_eq!(
        non_bmp_info
            .uppercase_map
            .get(&DESERET_SMALL_LETTER_LONG_AH),
        Some(&DESERET_CAPITAL_LETTER_LONG_AH)
    );
}

#[test]
fn non_bmp_space_set_is_empty() {
    let table = code_point_table::generate_code_point_table();
    let non_bmp_info = generate_non_bmp_info(&table);
    assert!(
        non_bmp_info.space_set.is_empty(),
        "js::unicode::IsSpace(char32) is defined assuming there are no non-BMP \
         space characters"
    );
}

#[test]
fn non_bmp_identifier_start() {
    let table = code_point_table::generate_code_point_table();

    let non_bmp_info = generate_non_bmp_info(&table);

    assert!(
        non_bmp_info
            .id_start_set
            .contains(&OLD_PERSIAN_SIGN_AURAMAZDAA),
        "OLD PERSIAN SIGN AURAMAZDAA is ID_Start"
    );
}

#[test]
fn non_bmp_identifier_continue() {
    let table = code_point_table::generate_code_point_table();

    assert_eq!(
        table.name(OLD_PERSIAN_SIGN_AURAMAZDAA),
        "OLD PERSIAN SIGN AURAMAZDAA"
    );

    let non_bmp_info = generate_non_bmp_info(&table);

    assert!(
        non_bmp_info
            .id_continue_set
            .contains(&OLD_PERSIAN_SIGN_AURAMAZDAA),
        "OLD PERSIAN SIGN AURAMAZDAA is ID_Start"
    );
}
