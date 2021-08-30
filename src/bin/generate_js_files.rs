extern crate unicode_info;

use std::fmt;
use std::result::Result;
use unicode_info::case_folding;
use unicode_info::code_point_table;

enum Error {
    #[allow(unused)]
    Unknown,
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::Unknown => {
                write!(fmt, "unknown error")
            }
        }
    }
}

fn main() -> Result<(), Error> {
    let _table = code_point_table::generate_code_point_table();
    let _case_folding = case_folding::process_case_folding();
    Ok(())
}
