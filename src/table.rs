//! Functions for transforming a list of integers into two lists, indexed in
//! sequence by the upper bits of the index, then by the lower bits of the
//! index.

use crate::types::NumericType;

/// From a list `t` of integers, many of which will be equal in value, compute
/// two separate lists `index1` and `index2` with given element types, plus the
/// shift necessary to make the smallest possible two-level page table for `t`
/// using them.
pub struct TableSplit {
    pub index1: Vec<u32>,
    pub index1_elem_type: NumericType,
    pub index2: Vec<u32>,
    pub index2_elem_type: NumericType,
    pub shift: u32,
}

/// Compute the type of the smallest integer type that can represent every value
/// in `data`.
fn get_element_type(data: &Vec<u32>) -> NumericType {
    assert!(data.len() > 0);

    let max_data = data.iter().fold(0, |max, v| std::cmp::max(max, *v)) as usize;
    assert!(max_data <= usize::wrapping_shl(1usize, 32) - 1);

    if max_data <= u8::MAX as usize {
        NumericType::U8
    } else if max_data <= u16::MAX as usize {
        NumericType::U16
    } else if max_data <= u32::MAX as usize {
        NumericType::U32
    } else {
        panic!(
            "unexpectedly large maximum: {max_data}",
            max_data = max_data
        );
    }
}

fn get_size(t: NumericType) -> usize {
    match t {
        NumericType::U8 => 1,
        NumericType::U16 => 2,
        NumericType::U32 => 4,
    }
}

/// Compute the size of the smallest integer type in bytes that can represent
/// every value in `data`.
fn get_element_size(data: &Vec<u32>) -> usize {
    get_size(get_element_type(&data))
}

#[test]
fn test_get_element_type() {
    let a = vec![254u32, 0, 0];
    assert_eq!(get_element_type(&a), NumericType::U8);

    let b = vec![255u32, 0, 0];
    assert_eq!(get_element_type(&b), NumericType::U8);

    let c = vec![256u32, 0, 0];
    assert_eq!(get_element_type(&c), NumericType::U16);

    let d = vec![65534u32, 0, 0];
    assert_eq!(get_element_type(&d), NumericType::U16);

    let e = vec![65535u32, 0, 0];
    assert_eq!(get_element_type(&e), NumericType::U16);

    let f = vec![65536u32, 0, 0];
    assert_eq!(get_element_type(&f), NumericType::U32);
}

/// Print diagnostic information about the optimal table splitting.
fn dump_best_split(s: &TableSplit, original_table: &Vec<u32>, bytes: usize) {
    eprintln!(
        "Best: {index1_len}+{index2_len} bins at shift {shift}; {bytes} bytes",
        index1_len = s.index1.len(),
        index2_len = s.index2.len(),
        shift = s.shift,
        bytes = bytes
    );
    eprintln!(
        "Size of original table: {original_size} bytes",
        original_size = get_element_size(&original_table) * original_table.len(),
    );
}

/// Compute the maximum possible `shift` such that `(t.len() - 1) >> shift` is
/// still nonzero.
fn compute_maximum_shift(t: &Vec<u32>) -> u32 {
    t.len().next_power_of_two().trailing_zeros() - 1
}

#[test]
fn test_maximum_shift() {
    assert_eq!(compute_maximum_shift(&vec![0; 2]), 0);
    assert_eq!(compute_maximum_shift(&vec![0; 3]), 1);
    assert_eq!(compute_maximum_shift(&vec![0; 4]), 1);
    assert_eq!(compute_maximum_shift(&vec![0; 5]), 2);
    assert_eq!(compute_maximum_shift(&vec![0; 6]), 2);
    assert_eq!(compute_maximum_shift(&vec![0; 7]), 2);
    assert_eq!(compute_maximum_shift(&vec![0; 8]), 2);
    assert_eq!(compute_maximum_shift(&vec![0; 9]), 3);
    assert_eq!(compute_maximum_shift(&vec![0; 10]), 3);
}

/// Given a (large) table `t` of values, return the best possible splitting of
/// that table into a two-level page table.
///
/// Various code point information is conceptually determined by looking up a
/// value in an array, by code point.  Even given that we can limit this to only
/// BMP code points, that's still a 64K-element array: a lot of memory!  And for
/// our use cases, most of those elements have identical values.
///
/// The first possible optimization is to store
///
/// 1. A `table` array of every unique value in `t`.
/// 1. An `index` array whose indexes are code points, whose elements are appropriate
/// indexes into the first array.
///
/// This saves some memory, _if_ `t` elements occupy more memory than the
/// elements of `index`.  But `index` is still just as long as `t`, so it
/// doesn't save much.
///
/// The second optimization is to split that second array into two, giving us
///
/// 1. A `table` array of every unique value in `t`.
/// 1. An `index1` array, indexed by the upper `(32 - N)` bits of a code point,
/// that stores partial index values.
/// 1. An `index2` array, indexed by an `index1` element shifted `N` bits upward
/// plus the remaining lower `N` bits of a code point, whose elements are
/// indexes into `table`.
///
/// Or to say it in pseudocode,
///
/// ```text
/// let mask = (1 << N) - 1;
/// for i in 0..t.len() {
///     let index1_entry = index1[i >> N];
///     let index1_index_component = index1_entry << N;
///     let mask_component = i & mask;
///     assert_eq!(t[i], index2[index1_index_component + mask_component]);
/// }
/// ```
/// The idea is, we determine which `2**N`-element bucket size produces few
/// enough unique elements in `table, and few enough elements (of small enough
/// size) in `index1` and `index2`, to minimize space overall.
///
/// (When `N = 0`, you can think of `index1` as an array of elements whose
/// values are their indexes and `index2` as identical to `index` in the first
/// optimization scheme.)
pub fn split_table(t: &Vec<u32>) -> TableSplit {
    assert!(
        t.len() <= u32::MAX as usize,
        "assumed below that t2's length won't exceed u32"
    );

    // The memory consumed by the current best splitting.  (Initialized to all
    // available memory so that the first iteration of the loop below
    // overwrites it.)
    let mut best_bytes = usize::MAX;

    // The current best splitting -- immediately overwritten.
    let mut best = TableSplit {
        index1: vec![],
        index1_elem_type: NumericType::U8,
        index2: vec![],
        index2_elem_type: NumericType::U8,
        shift: 0,
    };

    // The maximum possible downshift of a valid index of `t` that will produce
    // _some_ nonzero value.
    let max_shift = compute_maximum_shift(&t);

    // For every possible shift that leaves some index into `t` nonzero...
    for candidate_shift in 0..=max_shift {
        //  Let `t` be split into chunks of the corresponding size.
        let size = 1 << candidate_shift;

        // Let `index1` and `index2` be empty arrays.
        let mut index1 = vec![];
        let mut index2 = vec![];

        // Start a cache of chunks -> an index stored in `index1`.
        let mut bincache = std::collections::HashMap::<&[u32], u32>::new();

        // For every chunk,
        for i in (0..t.len()).step_by(size) {
            let bin = &t[i..i + size];

            let index = match bincache.get(&bin) {
                None => {
                    // If the chunk isn't cached, append chunk to `index2`,
                    // then use the chunk start as index.
                    let index = index2.len() as u32;
                    bincache.insert(bin, index);
                    index2.extend_from_slice(bin);
                    index
                }
                Some(index) => {
                    // If the chunk's index is cached, use that index.
                    *index
                }
            };

            // Add the index, shifted, to the end of `index1`.
            index1.push(index >> candidate_shift);
        }

        let index1_elem_type = get_element_type(&index1);
        let index2_elem_type = get_element_type(&index2);

        let index1_size = get_size(index1_elem_type) * index1.len();
        let index2_size = get_size(index2_elem_type) * index2.len();

        // If the total size of `index1` and `index2` beats the previous best,
        // update with the new best result.
        let bytes = index1_size + index2_size;
        if bytes < best_bytes {
            best = TableSplit {
                index1,
                index1_elem_type,
                index2,
                index2_elem_type,
                shift: candidate_shift,
            };
            best_bytes = bytes;
        }
    }

    dump_best_split(&best, &t, best_bytes);

    #[cfg(test)]
    {
        // Exhaustively verify that the decomposition is correct.
        let shift = best.shift;
        let index1 = &best.index1;
        let index2 = &best.index2;
        for i in 0..t.len() {
            let mask = (1 << shift) - 1;
            let index1_entry = index1[i >> shift];
            let index1_index_component = index1_entry << shift;
            let mask_component = i & mask;
            assert_eq!(
                t[i],
                index2[index1_index_component as usize + mask_component]
            );
        }
    }

    best
}
