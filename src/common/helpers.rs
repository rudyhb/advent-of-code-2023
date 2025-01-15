pub fn least_common_multiple(left: u64, right: u64) -> u64 {
    if left == 0 && right == 0 {
        return 0;
    }
    left * right / greatest_common_denominator(left, right)
}

pub fn least_common_multiple_for(numbers: &[u64]) -> u64 {
    assert!(numbers.len() >= 2);
    let mut current = least_common_multiple(numbers[0], numbers[1]);
    for &next in numbers.iter().skip(2) {
        current = least_common_multiple(current, next);
    }
    current
}

pub fn greatest_common_denominator(left: u64, right: u64) -> u64 {
    let min = left.min(right);
    let max = left.max(right);
    if min == 0 {
        return max;
    }
    greatest_common_denominator(min, max % min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(6, greatest_common_denominator(48, 18));
        assert_eq!(1, greatest_common_denominator(17, 13));
        assert_eq!(12, greatest_common_denominator(60, 48));
        assert_eq!(25, greatest_common_denominator(75, 100));
        assert_eq!(1, greatest_common_denominator(101, 10));
        assert_eq!(14, greatest_common_denominator(98, 42));
        assert_eq!(4, greatest_common_denominator(28, 20));
        assert_eq!(9, greatest_common_denominator(27, 9));
        assert_eq!(7, greatest_common_denominator(7, 49));
        assert_eq!(27, greatest_common_denominator(81, 27));
        assert_eq!(10, greatest_common_denominator(100, 90));
        assert_eq!(16, greatest_common_denominator(256, 144));
        assert_eq!(1, greatest_common_denominator(29, 16));
        assert_eq!(17, greatest_common_denominator(17, 17));
    }

    #[test]
    fn test_lcm() {
        assert_eq!(36, least_common_multiple(12, 18));
        assert_eq!(221, least_common_multiple(13, 17));
        assert_eq!(60, least_common_multiple(15, 20));
        assert_eq!(105, least_common_multiple(21, 15));
        assert_eq!(120, least_common_multiple(24, 40));
        assert_eq!(56, least_common_multiple(8, 14));
        assert_eq!(72, least_common_multiple(9, 8));
        assert_eq!(49, least_common_multiple(7, 49));
        assert_eq!(90, least_common_multiple(9, 10));
        assert_eq!(252, least_common_multiple(36, 42));
        assert_eq!(48, least_common_multiple(12, 48));
        assert_eq!(17, least_common_multiple(17, 17));
        assert_eq!(1, least_common_multiple(1, 1));
        assert_eq!(0, least_common_multiple(0, 5));
        assert_eq!(0, least_common_multiple(0, 0));
    }
}
