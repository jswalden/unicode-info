//! Common types used across this crate, with meaning not defined within a
//! specific module.

use std::collections::{HashMap, HashSet};

/// A set of code point values.
pub type CodePointSet = HashSet<u32>;

/// A mapping from code points to their case-mapped form (uppercase or lowercase
/// as stated in context).
pub type CaseMap = HashMap<u32, u32>;
