use rand::{thread_rng, Rng};

// 2^64 - 2^32 + 1
pub const P64: u64 = 0xffff_ffff_0000_0001;
pub const P128: u128 = 0xffff_ffff_0000_0001;

pub fn add(x: u64, y: u64) -> u64 {
    let sum: u128 = x as u128 + y as u128;
    (sum % P128) as u64
}

pub fn add_fast(x: u64, y: u64) -> u64 {
    let mut sum: u128 = x as u128 + y as u128;
    if sum > P128 {
        sum -= P128;
    }
    sum as u64
}

pub fn add_winterfell(x: u64, y: u64) -> u64 {
    // a + b = a - (p - b)
    let (x1, c1) = x.overflowing_sub(P64 - y);
    let adj = 0u32.wrapping_sub(c1 as u32);
    x1.wrapping_sub(adj as u64)
}

pub fn mul(x: u64, y: u64) -> u64 {
    let product: u128 = x as u128 * y as u128;
    (product % P128) as u64
}

pub fn mul_reduce159(x: u64, y: u64) -> u64 {
    let product: u128 = x as u128 * y as u128;
    reduce159(product)
}

pub fn mul_reduce_montgomery(x: u64, y: u64) -> u64 {
    let product: u128 = x as u128 * y as u128;
    reduce_montgomery(product)
}

/// Assume that x consists of four 32-bit values: a, b, c, d:
///
/// - a contains 32 least significant bits,
/// - d contains 32 most significant bits.
///
/// x is broken into corresponding values as shown below
const LOWER_MASK: u64 = 0xffff_ffff;

#[inline(always)]
fn reduce159(x: u128) -> u64 {
    let ab = x as u64;
    let cd = (x >> 64) as u64;
    let c = (cd as u32) as u64;
    let d = cd >> 32;

    // compute ab - d; because d may be greater than ab, handle potential underflow
    let (tmp0, is_under) = ab.overflowing_sub(d);
    let tmp1 = tmp0.wrapping_sub(LOWER_MASK * (is_under as u64));

    // compute c * 2^32 - c; this is guaranteed not to underflow
    let tmp2 = (c << 32) - c;

    // add temp values and return the result; because each of the temp may be up to 64 bits,
    // handle potential overflow
    let (result, is_over) = tmp1.overflowing_add(tmp2);
    result.wrapping_add(LOWER_MASK * (is_over as u64))
}

#[inline(always)]
const fn reduce_montgomery(x: u128) -> u64 {
    // See reference above for a description of the following implementation.
    let xl = x as u64;
    let xh = (x >> 64) as u64;
    let (a, e) = xl.overflowing_add(xl << 32);

    let b = a.wrapping_sub(a >> 32).wrapping_sub(e as u64);

    let (r, c) = xh.overflowing_sub(b);
    r.wrapping_sub(0u32.wrapping_sub(c as u32) as u64)
}

#[inline(always)]
pub fn montgomery_equals(lhs: u64, rhs: u64) -> bool {
    let t = lhs ^ rhs;
    0xffffffffffffffff == !((((t | t.wrapping_neg()) as i64) >> 63) as u64)
}

pub fn random_elements(n: usize) -> Vec<u64> {
    (0..n + 1)
        .map(|_| thread_rng().gen_range(0..P64))
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn add_equivalence() {
        let n_operations = 1_000;
        let operands = random_elements(n_operations + 1);
        for (&x, &y) in operands.iter().tuple_windows() {
            assert_eq!(add(x, y), add_fast(x, y));
            assert_eq!(add(x, y), add_winterfell(x, y));
        }
    }

    #[test]
    fn mul_equivalence() {
        let n_operations = 1_000;
        let operands = random_elements(n_operations + 1);
        for (&x, &y) in operands.iter().tuple_windows() {
            let expected_product = mul(x, y);
            assert_eq!(expected_product, mul_reduce159(x, y));

            let expected_montgomery_product = reduce_montgomery(expected_product as u128);
            let actual_montgomery_product = mul_reduce_montgomery(x, y);
            assert!(montgomery_equals(
                expected_montgomery_product,
                actual_montgomery_product
            ));
        }
    }
}
