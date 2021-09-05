//! Computation of the set of code points recognized as white space in JavaScript.

use crate::code_point_table;
#[cfg(test)]
use crate::constants::{
    CARRIAGE_RETURN, CHARACTER_TABULATION, IDEOGRAPHIC_SPACE, LINE_FEED, LINE_SEPARATOR,
    LINE_TABULATION, NO_BREAK_SPACE, PARAGRAPH_SEPARATOR, ZERO_WIDTH_NO_BREAK_SPACE,
};
use crate::constants::{LINE_TERMINATOR, MAX_BMP, WHITE_SPACE};

use crate::types::CodePointSet;

/// Compute the set of all code points that match the `WhiteSpace` or
/// `LineTerminator` productions.
///
/// Note that `WhiteSpace` includes all code points in the Unicode "Space
/// Separator", i.e. "Zs", category.
pub fn compute_white_space(code_point_table: &code_point_table::CodePointTable) -> CodePointSet {
    let mut space_set = CodePointSet::new();
    for code_point in code_point_table.iter() {
        let code = code_point.code;
        if code_point.category() == "Zs"
            || WHITE_SPACE.contains(&code)
            || LINE_TERMINATOR.contains(&code)
        {
            assert!(
                code <= MAX_BMP,
                "js::unicode::IsSpace(char32_t) depends upon non non-BMP \
                 spaces existing"
            );
            space_set.insert(code);
        }
    }
    space_set
}

#[test]
fn space_set_contains() {
    let table = code_point_table::generate_code_point_table();
    let spaces = compute_white_space(&table);
    assert!(spaces.contains(&CHARACTER_TABULATION));
    assert!(spaces.contains(&LINE_TABULATION));
    assert!(spaces.contains(&CARRIAGE_RETURN));
    assert!(spaces.contains(&LINE_FEED));
    assert!(spaces.contains(&NO_BREAK_SPACE));
    assert!(spaces.contains(&LINE_SEPARATOR));
    assert!(spaces.contains(&PARAGRAPH_SEPARATOR));
    assert!(spaces.contains(&IDEOGRAPHIC_SPACE));
    assert!(spaces.contains(&ZERO_WIDTH_NO_BREAK_SPACE));
}
