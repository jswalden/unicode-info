use std::collections::HashMap;

static UNICODE_DATA_TXT: &str = include_str!("data/UnicodeData.txt");

#[derive(Copy, Clone, Debug)]
struct CodePointDetails {
  name: &'static str,
  category: &'static str,
  alias: &'static str,
  uppercase: u32,
  lowercase: u32,
}

#[derive(Copy, Clone, Debug)]
struct CodePoint {
  code: u32,
  details: CodePointDetails,
}

struct CodePointRange {
  range: std::ops::RangeInclusive::<u32>,
  details: CodePointDetails,
}

struct UnicodeData {
  within_range: Option::<CodePointRange>,
  lines: std::str::Lines<'static>,
}

impl UnicodeData {
  fn read() -> UnicodeData {
    UnicodeData {
      within_range: None,
      lines: UNICODE_DATA_TXT.lines()
    }
  }
}

fn to_case(case_field: &str, code: u32) -> u32 {
  if case_field.is_empty() {
    code
  } else {
    u32::from_str_radix(case_field, 16).expect("bad hex code")
  }
}

impl Iterator for UnicodeData {
  type Item = CodePoint;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(ref mut within_range) = self.within_range {
        if let Some(code) = within_range.range.next() {
          let code_point = CodePoint {
            code,
            details: within_range.details,
          };
          return Some(code_point);
        }
        
        self.within_range = None;
      }

      loop {
        let line = self.lines.next()?;

        fn to_fields(line: &'static str) -> Vec<&'static str> {
          let fields = line.split(';').collect::<Vec<&'static str>>();
          assert_eq!(fields.len(), 15,
                     concat!("1 hexadecimal code field, 14 fields listed in ",
                             "http://www.unicode.org/reports/tr44/#UnicodeData.txt"));
          fields
        }

        let fields = to_fields(&line);

        fn decompose_fields(code: u32, fields: &Vec<&'static str>) -> CodePointDetails {
          CodePointDetails {
            name: fields[1],
            category: fields[2],
            alias: fields[10],
            uppercase: to_case(fields[12], code),
            lowercase: to_case(fields[13], code),
          }          
        }

        fn get_code(fields: &Vec<&'static str>) -> u32 {
          u32::from_str_radix(fields[0], 16).expect("hex code")
        }

        let code = get_code(&fields);
        let mut details = decompose_fields(code, &fields);

        if details.name.starts_with('<') && details.name.ends_with("First>") {
          let range_end_line = self.lines.next().expect("second line in range");
          let range_end_fields = to_fields(&range_end_line);

          let last_code = get_code(&range_end_fields);

          details.name = &details.name[1..details.name.len() - 8];

          let range = CodePointRange {
            range: code..=last_code,
            details,
          };

          self.within_range = Some(range);
          break;
        }

        let code_point = CodePoint {
          code,
          details,
        };

        return Some(code_point);
      }
    }
  }
}

pub struct CodePointInfo {
  pub name: &'static str,
  pub alias: &'static str,
  pub category: &'static str,
  pub upper: u32,
  pub lower: u32,
}

type CodePointMap = HashMap::<u32, CodePointInfo>;

pub struct CodePointTable {
  map: CodePointMap,
}

pub struct CodePointTableIter<'a> {
  iter: std::collections::hash_map::Iter<'a, u32, CodePointInfo>,
}

impl<'a> Iterator for CodePointTableIter<'a> {
  type Item = (&'a u32, &'a CodePointInfo);

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}

impl CodePointTable {
  pub fn name(&self, code: u32) -> String {
    let CodePointInfo { name, alias, .. } = self.map.get(&code).expect("code point");
    format!("{name}{alias}",
            name = name,
            alias = if alias.is_empty() {
              String::new()
            } else {
              format!(" ({alias})", alias = alias)
            })
  }

  pub fn full_name(&self, code: u32) -> String {
    format!("U+{code:04X} {name}", code = code, name = self.name(code))
  }

  pub fn iter(&self) -> CodePointTableIter {
    CodePointTableIter { iter: self.map.iter() }
  }
}

pub fn generate_code_point_table() -> CodePointTable {
  let mut code_point_map = CodePointMap::new();

  for code_point in UnicodeData::read() {
    let info = CodePointInfo {
      upper: code_point.details.uppercase,
      lower: code_point.details.lowercase,
      name: code_point.details.name,
      alias: code_point.details.alias,
      category: code_point.details.category,
    };
    code_point_map.insert(code_point.code, info);
  }

  CodePointTable { map: code_point_map }
}

#[test]
fn check_unicode_data() {
  let table = generate_code_point_table();
  assert_eq!(table.name('A' as u32), "LATIN CAPITAL LETTER A",
             "sanity check on ASCII capital A");
}
