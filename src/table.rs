//! Functions for transforming a list of integers into two lists, indexed in
//! sequence by the upper bits of the index, then by the lower bits of the
//! index.

/// From a list `t` of integers, many of which will be equal in value, compute
/// two separate lists `t1` and `t2`
pub struct TableSplit {
    t1: Vec<u32>,
    t2: Vec<u32>,
    shift: u32,
}

/// Compute the size of the smallest integer type that can represent every value
/// in `data`.
fn get_size(data: &Vec<u32>) -> usize {
    assert!(data.len() > 0);

    let max_data = data.iter().fold(0, |max, v| std::cmp::max(max, *v)) as usize;
    assert!(max_data <= usize::wrapping_shl(1usize, 32) - 1);

    data.len()
        * (if max_data <= u8::MIN as usize {
            1
        } else if max_data <= u16::MIN as usize {
            2
        } else if max_data <= u32::MIN as usize {
            4
        } else {
            panic!("unexpectedly large maximum");
        })
}

#[test]
fn test_get_size() {
    let a1 = vec![254u32];
    assert_eq!(get_size(&a1), 1, "1, max 254");

    let a3 = vec![254u32, 0, 0];
    assert_eq!(get_size(&a3), 3, "3, max 254");

    let b1 = vec![255u32];
    assert_eq!(get_size(&b1), 1, "1, max 255");

    let b3 = vec![255u32, 0, 0];
    assert_eq!(get_size(&b3), 3, "3, max 255");

    let c1 = vec![256u32];
    assert_eq!(get_size(&c1), 2, "1, max 256");

    let c3 = vec![256u32, 0, 0];
    assert_eq!(get_size(&c3), 6, "3, max 256");

    let d1 = vec![65534u32];
    assert_eq!(get_size(&d1), 2, "1, max 65534");

    let d3 = vec![65534u32, 0, 0];
    assert_eq!(get_size(&d3), 2, "3, max 65534");

    let e1 = vec![65535u32];
    assert_eq!(get_size(&e1), 2, "1, max 65535");

    let e3 = vec![65535u32, 0, 0];
    assert_eq!(get_size(&e3), 6, "1, max 65535");

    let f1 = vec![65536u32];
    assert_eq!(get_size(&f1), 4, "1, max 65536");

    let f3 = vec![65536u32, 0, 0];
    assert_eq!(get_size(&f3), 12, "1, max 65536");
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
        original_size = get_size(original_table),
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
pub fn split_table(t: &mut Vec<u32>) -> TableSplit {
    assert!(
        t.len() <= u32::MAX as usize,
        "assumed below that t2's length won't exceed u32"
    );

    let max_shift = compute_maximum_shift(&t);

    // The current best splitting discovered (i.e. the null split).
    let mut best = TableSplit {
        t1: vec![0],
        t2: t.clone(),
        shift: max_shift + 1,
    };

    // The memory consumed by the current best splitting.  (Initialized to all
    // available memory so that the first iteration of the loop below will
    // overwrite it.)
    let mut best_bytes = usize::MAX;

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

        let bytes = get_size(&t1) + get_size(&t2);
        if bytes < best_bytes {
            best = TableSplit {
                t1,
                t2,
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
