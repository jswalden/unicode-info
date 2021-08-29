//! Parse the contents of `UnicodeData.txt`, the central code point registry
//! file, into queryable and iterable form.

static UNICODE_DATA_TXT: &str = include_str!("data/UnicodeData.txt");

/// Information about a particular code point.
#[derive(Copy, Clone, Debug)]
pub struct CodePointInfo {
    /// The name of the code point, e.g. CRAB or PILE OF POO or
    /// LATIN CAPITAL LETTER A.
    pub name: &'static str,

    /// The Unicode category of the code point, in its abbreviated form: for
    /// example, "Zs" rather than "Space_Separator".
    pub category: &'static str,

    /// The alias of the code point, if any.
    ///
    /// For example, U+FEFF ZERO WIDTH NO-BREAK SPACE has BYTE ORDER MARK as its
    /// alias.
    pub alias: &'static str,

    /// The code for the uppercase form of the associated code point.
    ///
    /// If the code point doesn't have an uppercase form, this will be the code
    /// point itself.
    pub uppercase: u32,

    /// The code for the lowercase form of the associated code point.
    ///
    /// If the code point doesn't have a lowercase form, this will be the code
    /// point itself.
    pub lowercase: u32,
}

/// Code point info, including its code.
#[derive(Copy, Clone, Debug)]
struct CodePoint {
    code: u32,
    info: CodePointInfo,
}

/// Code points within a range, that share all aspects except for code.
struct CodePointRange {
    range: std::ops::RangeInclusive<u32>,
    info: CodePointInfo,
}

/// A structure representing parse state during a parse of the contents of
/// `UnicodeData.txt`.
struct UnicodeDataParse {
    within_range: Option<CodePointRange>,
    lines: std::str::Lines<'static>,
}

impl UnicodeDataParse {
    /// Produce an iterator over the structured contents of `UnicodeData.txt`.
    fn parse() -> UnicodeDataParse {
        UnicodeDataParse {
            within_range: None,
            lines: UNICODE_DATA_TXT.lines(),
        }
    }
}

impl Iterator for UnicodeDataParse {
    type Item = CodePoint;

    /// Yields information about the next code point described in
    /// `UnicodeData.txt`.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // First handle any remaining iteration within a code point range.
            if let Some(ref mut within_range) = self.within_range {
                if let Some(code) = within_range.range.next() {
                    let code_point = CodePoint {
                        code,
                        info: within_range.info,
                    };
                    return Some(code_point);
                }

                // Once all code points in the range have been produced, resume
                // processing additional lines.
                self.within_range = None;
            }

            // Then loop over remaining lines in UnicodeData.txt.
            loop {
                let line = match self.lines.next() {
                    None => {
                        // There are no more lines to parse, so we're done.
                        return None;
                    }
                    Some(line) => line,
                };

                fn to_fields(line: &'static str) -> Vec<&'static str> {
                    // UnicodeData.txt consists of semicolon-delimited fields: a
                    // leading field containing the hexadecimal code value, then
                    // fourteen additional fields.  See
                    // http://www.unicode.org/reports/tr44/#UnicodeData.txt
                    // for details.
                    let fields = line.split(';').collect::<Vec<&'static str>>();
                    assert_eq!(fields.len(), 15);
                    fields
                }

                let fields = to_fields(&line);

                fn get_code(fields: &Vec<&'static str>) -> u32 {
                    u32::from_str_radix(fields[0], 16).expect("hex code")
                }

                fn decompose_fields(code: u32, fields: &Vec<&'static str>) -> CodePointInfo {
                    fn to_case(case_field: &str, code: u32) -> u32 {
                        if case_field.is_empty() {
                            code
                        } else {
                            u32::from_str_radix(case_field, 16).expect("bad hex code")
                        }
                    }

                    CodePointInfo {
                        name: fields[1],
                        category: fields[2],
                        alias: fields[10],
                        uppercase: to_case(fields[12], code),
                        lowercase: to_case(fields[13], code),
                    }
                }

                let code = get_code(&fields);
                let mut info = decompose_fields(code, &fields);

                // A consecutive code point pair may represent a range of code
                // points, for example
                //
                //   D800;<Non Private Use High Surrogate, First>;Cs;0;L;;;;;N;;;;;
                //   DB7F;<Non Private Use High Surrogate, Last>;Cs;0;L;;;;;N;;;;;
                if info.name.starts_with('<') && info.name.ends_with("First>") {
                    let range_end_line = self.lines.next().expect("second line in range");
                    let range_end_fields = to_fields(&range_end_line);

                    let last_code = get_code(&range_end_fields);

                    // Remove "<" and ", First>" to extract the general name of
                    // all code points in the range.
                    info.name = &info.name[1..info.name.len() - 8];

                    let range = CodePointRange {
                        range: code..=last_code,
                        info,
                    };

                    // Resume at start of the outer loop yielding code points
                    // within the defined range.
                    self.within_range = Some(range);
                    break;
                }

                let code_point = CodePoint { code, info };
                return Some(code_point);
            }

            // Pause examining UnicodeData.txt lines to process a code point
            // range.
        }
    }
}

type CodePointMap = std::collections::HashMap<u32, CodePointInfo>;

/// A table containing information on every code point.
///
/// Information about individual code points may be accessed by passing the code
/// point's code to the various query functions defined here.
///
/// Information about all code points may be accessed using
/// [`iter()`](CodePointTable.iter).
pub struct CodePointTable {
    map: CodePointMap,
}

/// An iterator over the code points in a `CodePointTable`.
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
    /// Return a string containing the code point's name and (if it has one) its
    /// alias.
    ///
    /// # Example
    ///
    /// ```
    /// # use unicode_info::code_point_table::CodePointTable;
    /// # use unicode_info::code_point_table::generate_code_point_table;
    /// let table: CodePointTable = generate_code_point_table();
    /// assert_eq!(table.name('A' as u32), "LATIN CAPITAL LETTER A");
    /// assert_eq!(table.name(0xFEFF),
    ///            "ZERO WIDTH NO-BREAK SPACE (BYTE ORDER MARK)");
    /// ```
    pub fn name(&self, code: u32) -> String {
        let CodePointInfo { name, alias, .. } = self.map.get(&code).expect("code point");
        let mut s = String::from(*name);
        if !alias.is_empty() {
            s.push_str(&format!(" ({alias})", alias = alias));
        }
        s
    }

    /// Return a string containing the code point's code, its name, and (if it
    /// has one) its alias.
    ///
    /// # Examples
    ///
    /// ```
    /// # use unicode_info::code_point_table::CodePointTable;
    /// # use unicode_info::code_point_table::generate_code_point_table;
    /// let table: CodePointTable = generate_code_point_table();
    /// assert_eq!(table.full_name('A' as u32),
    ///            "U+0041 LATIN CAPITAL LETTER A");
    /// assert_eq!(table.full_name(0xFEFF),
    ///            "U+FEFF ZERO WIDTH NO-BREAK SPACE (BYTE ORDER MARK)");
    /// ```
    pub fn full_name(&self, code: u32) -> String {
        format!("U+{code:04X} {name}", code = code, name = self.name(code))
    }

    /// Return an iterator over all code points and their info in this table.
    pub fn iter(&self) -> CodePointTableIter {
        CodePointTableIter {
            iter: self.map.iter(),
        }
    }
}

/// Generate a table of all code points, mapping code to characteristics.
pub fn generate_code_point_table() -> CodePointTable {
    let mut code_point_map = CodePointMap::new();

    for code_point in UnicodeDataParse::parse() {
        code_point_map.insert(code_point.code, code_point.info);
    }

    CodePointTable {
        map: code_point_map,
    }
}

#[test]
fn check_unicode_data() {
    let table = generate_code_point_table();
    assert_eq!(
        table.name('A' as u32),
        "LATIN CAPITAL LETTER A",
        "sanity check of a simple Latin-1 code point"
    );
    assert_eq!(
        table.full_name(0xFEFF),
        "U+FEFF ZERO WIDTH NO-BREAK SPACE (BYTE ORDER MARK)",
        "sanity check of a code point with an alias"
    );
    assert_eq!(
        table.full_name('💩' as u32),
        "U+1F4A9 PILE OF POO",
        "sanity check of a non-BMP code point"
    );
}
