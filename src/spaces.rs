use std::collections::HashSet;

use crate::code_point_table;
use crate::constants::{LINE_TERMINATOR, WHITE_SPACE};

pub type CodePointSet = HashSet::<u32>;

pub fn collect_spaces(code_point_table: &code_point_table::CodePointTable) -> CodePointSet {
  let mut space_set = CodePointSet::new();
  for (code, info) in code_point_table.iter() {
    if info.category == "Zs" ||
       WHITE_SPACE.contains(code) ||
       LINE_TERMINATOR.contains(code) {
      space_set.insert(*code);
    }
  }
  space_set
}

#[test]
fn space_set_contains() {
  let table = code_point_table::generate_code_point_table();
  let spaces = collect_spaces(&table);
  assert!(spaces.contains(&('\r' as u32)));
  assert!(spaces.contains(&('\n' as u32)));
}
