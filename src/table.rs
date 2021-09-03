//! Functions for transforming a list of integers into two lists, indexed in
//! sequence by the upper bits of the index, then by the lower bits of the
//! index.

use crate::types::NumericType;

/// From a list `t` of integers, many of which will be equal in value, compute
/// two separate lists `t1` and `t2`
pub struct TableSplit {
    pub t1: Vec<u32>,
    pub t1_elem_type: NumericType,
    pub t2: Vec<u32>,
    pub t2_elem_type: NumericType,
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
        "Best: {t1_len}+{t2_len} bins at shift {shift}; {bytes} bytes",
        t2_len = s.t2.len(),
        t1_len = s.t1.len(),
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

/// Given a (large) table `t` of values, many of which are equal, split `t` into
/// two tables and a `shift` such that
///
/// ```text
/// // assuming `i` is a valid index into table `t`
/// for i in 0..t.len() {
///     let mask = (1 << shift) - 1;
///     let t1_entry = t1[i >> shift];
///     let t1_index_component = t1_entry << shift;
///     let mask_component = i & mask;
///     assert!(t[i], t2[t1_index_component + mask_component]);
/// }
/// ```
///
/// where `shift` is chosen to minimize the memory consumed by `t1` and `t2`,
/// assuming each is represented using the smallest possible integer type.
pub fn split_table(t: &Vec<u32>) -> TableSplit {
    assert!(
        t.len() <= u32::MAX as usize,
        "assumed below that t2's length won't exceed u32"
    );

    // The memory consumed by the current best splitting.  (Initialized to all
    // available memory so that the first iteration of the loop below will
    // overwrite it.)
    let mut best_bytes = usize::MAX;

    // The current best splitting -- immediately overwritten.
    let mut best = TableSplit {
        t1: vec![],
        t1_elem_type: NumericType::U8,
        t2: vec![],
        t2_elem_type: NumericType::U8,
        shift: 0,
    };

    let max_shift = compute_maximum_shift(&t);
    for candidate_shift in 0..=max_shift {
        let mut t1 = vec![];
        let mut t2 = vec![];
        let size = 1 << candidate_shift;

        let mut bincache = std::collections::HashMap::<&[u32], u32>::new();

        for i in (0..t.len()).step_by(size) {
            let bin = &t[i..i + size];

            let index = match bincache.get(&bin) {
                None => {
                    let index = t2.len() as u32;
                    bincache.insert(bin, index);
                    t2.extend_from_slice(bin);
                    index
                }
                Some(index) => *index,
            };
            t1.push(index >> candidate_shift);
        }

        let t1_elem_type = get_element_type(&t1);
        let t2_elem_type = get_element_type(&t2);

        let bytes = get_size(t1_elem_type) * t1.len() + get_size(t2_elem_type) * t2.len();
        if bytes < best_bytes {
            best = TableSplit {
                t1,
                t1_elem_type,
                t2,
                t2_elem_type,
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
        let t1 = &best.t1;
        let t2 = &best.t2;
        for i in 0..t.len() {
            let mask = (1 << shift) - 1;
            let t1_entry = t1[i >> shift];
            let t1_index_component = t1_entry << shift;
            let mask_component = i & mask;
            assert_eq!(t[i], t2[t1_index_component as usize + mask_component]);
        }
    }

    best
}
