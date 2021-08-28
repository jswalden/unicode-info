use std::collections::{HashMap, HashSet};

use crate::code_point_table;
use crate::constants::MAX_BMP;
use crate::derived_core_properties;

use derived_core_properties::DerivedCorePropertyData;

pub type CaseMap = HashMap<u32, u32>;

pub type CodePointSet = HashSet<u32>;

pub struct NonBMPInfo {
    pub lowercase_map: CaseMap,
    pub uppercase_map: CaseMap,
    pub space_set: CodePointSet,
    pub id_start_set: CodePointSet,
    pub id_continue_set: CodePointSet,
}

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

    for entry in code_point_table.iter().filter(|(&code, _)| code > MAX_BMP) {
        let (code, info) = entry;
        if *code != info.lower {
            lowercase_map.insert(*code, info.lower);
        }
        if *code != info.upper {
            uppercase_map.insert(*code, info.upper);
        }
        if info.category == "Zs" {
            space_set.insert(*code);
        }
        if derived_id_start.contains(code) {
            id_start_set.insert(*code);
        }
        if derived_id_continue.contains(code) {
            id_continue_set.insert(*code);
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
fn non_bmp_space_set_is_empty() {
    let table = code_point_table::generate_code_point_table();
    let non_bmp_info = generate_non_bmp_info(&table);
    assert!(
        non_bmp_info.space_set.is_empty(),
        "js::unicode::IsSpace(char32) assumes no non-BMP space characters"
    );
}

#[test]
fn non_bmp_identifier_start() {
    let table = code_point_table::generate_code_point_table();

    const OLD_PERSIAN_SIGN_AURAMAZDAA: u32 = 0x103C8;
    assert_eq!(
        table.name(OLD_PERSIAN_SIGN_AURAMAZDAA),
        "OLD PERSIAN SIGN AURAMAZDAA"
    );

    let non_bmp_info = generate_non_bmp_info(&table);

    assert!(
        non_bmp_info
            .id_start_set
            .contains(&OLD_PERSIAN_SIGN_AURAMAZDAA),
        "OLD PERSIAN SIGN AURAMAZDAA is ID_Start"
    );
}
