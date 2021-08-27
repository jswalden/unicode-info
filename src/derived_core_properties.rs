use std::collections::HashSet;

static DERIVED_CORE_TXT: &str = include_str!("data/DerivedCoreProperties.txt");

struct CodePointAndProperty {
  code_point: u32,
  property: &'static str,
}

struct CodePointAndPropertyIter {
  range: std::ops::RangeInclusive::<u32>,
  property: &'static str,
}

impl Iterator for CodePointAndPropertyIter {
  type Item = CodePointAndProperty;
  
  fn next(&mut self) -> Option<Self::Item> {
    match self.range.next() {
      None => None,
      Some(code_point) => {
        Some(CodePointAndProperty { code_point, property: self.property })
      }
    }
  }
}

struct DerivedCoreProperties {
  within_range: Option::<CodePointAndPropertyIter>,
  lines: std::str::Lines<'static>,
}

impl DerivedCoreProperties {
  fn read() -> DerivedCoreProperties {
    DerivedCoreProperties {
      within_range: None,
      lines: DERIVED_CORE_TXT.lines(),
    }
  }
}

impl Iterator for DerivedCoreProperties {
  type Item = CodePointAndProperty;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(ref mut range) = self.within_range {
        if let Some(within_range) = range.next() {
          return Some(within_range);
        }

        self.within_range = None;
      }

      loop {
        let line_with_comment = self.lines.next()?;
        let line = line_with_comment.split('#').nth(0).expect("splitting returns at least one string");
        if line.is_empty() {
          continue;
        }

        let mut fields = line.split(';');

        let range = fields.next().expect("single code point or range").trim();
        let property = fields.next().expect("property").trim();
        
        if range.contains("..") {
          let mut nums = range.split("..");
          let start = u32::from_str_radix(nums.next().expect("start"), 16).expect("hex start");
          let end = u32::from_str_radix(nums.next().expect("end"), 16).expect("hex end");
          let iter = CodePointAndPropertyIter {
            range: start..=end,
            property
          };
          self.within_range = Some(iter);
          break;
        }

        let code_point = u32::from_str_radix(range, 16).expect("hex code point");
        return Some(CodePointAndProperty { code_point, property });
      }
    }
  }
}

pub struct DerivedCorePropertyData {
  pub id_start: HashSet::<u32>,
  pub id_continue: HashSet::<u32>,
}

pub fn process_derived_core_properties() -> DerivedCorePropertyData {
  let mut id_start = HashSet::<u32>::new();
  let mut id_continue = HashSet::<u32>::new();

  for CodePointAndProperty { code_point, property } in DerivedCoreProperties::read() {
    let s = match property {
      "ID_Start" => &mut id_start,
      "ID_Continue" => &mut id_continue,
      _ => {
        continue;
      }
    };

    s.insert(code_point);
  }

  DerivedCorePropertyData { id_start, id_continue }
}

#[test]
fn check_derived_core_properties() {
  let dcp = process_derived_core_properties();

  let starts = dcp.id_start;
  let starts_count = starts.len();

  let continues = dcp.id_continue;
  let continues_count = continues.len();

  // These constants were derived not from messing around and finding out, but
  // from comments after respective sections in DerivedCoreProperties.txt.
  assert_eq!(starts_count, 131_482);
  assert_eq!(continues_count, 134_434);
}
