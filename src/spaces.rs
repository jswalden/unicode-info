use crate::code_point_table;
use crate::constants::{LINE_TERMINATOR, WHITE_SPACE};

use crate::types::CodePointSet;

/// Compute the set of all code points that match the `WhiteSpace` or
/// `LineTerminator` productions.
///
/// Note that `WhiteSpace` includes all code points in the Unicode "Space
/// Separator", i.e. "Zs", category.
pub fn compute_white_space(code_point_table: &code_point_table::CodePointTable) -> CodePointSet {
    let mut space_set = CodePointSet::new();
    for (code, info) in code_point_table.iter() {
        if info.category == "Zs" || WHITE_SPACE.contains(code) || LINE_TERMINATOR.contains(code) {
            space_set.insert(*code);
        }
    }
    space_set
}

#[test]
fn space_set_contains() {
    let table = code_point_table::generate_code_point_table();
    let spaces = compute_white_space(&table);
    assert!(spaces.contains(&0x0009));
    assert!(spaces.contains(&0x000B));
    assert!(spaces.contains(&0x000D));
    assert!(spaces.contains(&0x000A));
    assert!(spaces.contains(&0x00A0));
    assert!(spaces.contains(&0x2028));
    assert!(spaces.contains(&0x2029));
    assert!(spaces.contains(&0x3000));
    assert!(spaces.contains(&0xFEFF));
}
