use {
    num_traits::PrimInt,
    std::ops::{BitAndAssign, BitOrAssign},
};

pub fn nth_bit_set<N: PrimInt>(number: N, n: usize) -> bool {
    (number & (N::one() << n)) != N::zero()
}

pub fn set_nth_bit<N: PrimInt + BitOrAssign + BitAndAssign>(number: &mut N, n: usize, set: bool) {
    let mask = N::one() << n;
    if set {
        *number |= mask;
    } else {
        *number &= !mask;
    }
}

#[test]
#[expect(clippy::bool_assert_comparison)]
fn test_nth_bit_set() {
    let number: u8 = 0b0100_0100;
    assert_eq!(nth_bit_set(number, 0), false);
    assert_eq!(nth_bit_set(number, 1), false);
    assert_eq!(nth_bit_set(number, 2), true);
    assert_eq!(nth_bit_set(number, 3), false);
    assert_eq!(nth_bit_set(number, 4), false);
    assert_eq!(nth_bit_set(number, 5), false);
    assert_eq!(nth_bit_set(number, 6), true);
    assert_eq!(nth_bit_set(number, 7), false);
    assert_eq!(nth_bit_set(0u64, 0), false);
    assert_eq!(nth_bit_set(u64::MAX, 63), true);
}

#[test]
#[expect(clippy::bool_assert_comparison)]
fn test_set_nth_bit() {
    let mut number: u8 = 0b0000_0000;
    set_nth_bit(&mut number, 0, true);
    assert_eq!(number, 0b0000_0001);
    set_nth_bit(&mut number, 1, true);
    assert_eq!(number, 0b0000_0011);
    set_nth_bit(&mut number, 2, true);
    assert_eq!(number, 0b0000_0111);
    set_nth_bit(&mut number, 0, false);
    assert_eq!(number, 0b0000_0110);

    let mut all_bits_set: u64 = 0;
    for i in 0..64 {
        set_nth_bit(&mut all_bits_set, i, true);
        assert_eq!(nth_bit_set(all_bits_set, i), true);
    }

    let mut no_bits_set: u64 = u64::MAX;
    for i in 0..64 {
        set_nth_bit(&mut no_bits_set, i, false);
        assert_eq!(nth_bit_set(no_bits_set, i), false);
    }
}
