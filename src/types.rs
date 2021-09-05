//! Common types used across this crate, with meaning not defined within a
//! specific module.

use quote::quote;
use std::collections::{HashMap, HashSet};

/// A set of code point values.
pub type CodePointSet = HashSet<u32>;

/// A mapping from code points to their case-mapped form (uppercase or lowercase
/// as stated in context).
pub type CaseMap = HashMap<u32, u32>;

/// An enum denoting a Rust numeric type.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum NumericType {
    U8,
    U16,
    U32,
}

/// The lowercase, uppercase
pub struct MappedCodePoint {
    pub lower: u32,
    pub upper: u32,
    pub flags: Flags,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Flags(pub u8);

/// Flag indicating a code point is treated as a JavaScript spacing character.
const FLAG_SPACE: u8 = 1 << 0;

/// Flag indicating a code point may appear at the start of an identifier.
pub const FLAG_UNICODE_ID_START: u8 = 1 << 1;

/// Flag indicating a code point may appear in an identifier only after the
/// first code point in the identifier.
pub const FLAG_UNICODE_ID_CONTINUE_ONLY: u8 = 1 << 2;

impl Flags {
    pub fn is_space(&self) -> bool {
        self.0 & FLAG_SPACE != 0
    }

    pub fn is_unicode_id_start(&self) -> bool {
        self.0 & FLAG_UNICODE_ID_START != 0
    }

    pub fn is_unicode_id_continue_only(&self) -> bool {
        self.0 & FLAG_UNICODE_ID_CONTINUE_ONLY != 0
    }

    pub fn set_space(&mut self) {
        self.0 |= FLAG_SPACE;
    }

    pub fn set_unicode_id_start(&mut self) {
        self.0 |= FLAG_UNICODE_ID_START;
    }

    pub fn set_unicode_id_continue_only(&mut self) {
        self.0 |= FLAG_UNICODE_ID_CONTINUE_ONLY;
    }
}

impl quote::ToTokens for Flags {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let flags = self.0;
        let code = quote! {
            ::unicode_info::types::Flags(#flags)
        };
        tokens.extend(code);
    }
}
