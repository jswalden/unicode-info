//! Parse the contents of `CaseFolding.txt`, the central code point registry
//! file.

use std::collections::HashMap;

static CASE_FOLDING_TXT: &str = include_str!("data/CaseFolding.txt");

type CodePointMap = HashMap<u32, u32>;

struct CaseFoldingParse {
    lines: std::str::Lines<'static>,
}

impl CaseFoldingParse {
    fn parse() -> CaseFoldingParse {
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
            // File format is:
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

            if ["F", "T"].contains(&row[1]) {
                continue;
            }

            assert!(
                ["C", "S"].contains(&row[1]),
                "expect either (C)ommon or (S)imple case foldings"
            );

            let code = u32::from_str_radix(row[0], 16).expect("hex code");
            let mapping = u32::from_str_radix(row[2], 16).expect("hex mapping");

            return Some(CaseFoldingInfo { code, mapping });
        }
    }
}

pub struct CaseFoldingData {}

pub fn process_case_folding() -> CaseFoldingData {
    let mut folding_map = CodePointMap::new();

    for CaseFoldingInfo { code, mapping } in CaseFoldingParse::parse() {
        folding_map.insert(code, mapping);
    }
    CaseFoldingData {}
}

#[test]
fn check_case_folding() {}
