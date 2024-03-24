use super::builder::Builder;
use super::builder::Res as Circuit;
use super::ARG0;

/// A circuit that computes x * y - z
pub fn mul_eq() -> Circuit<u64> {
    let n_in = 3;
    let x = ARG0;
    let y = ARG0 + 1;
    let z = ARG0 + 2;
    let mut b = Builder::new(n_in);
    let mul = b.mul(x, y);
    let sub = b.sub(mul, z);
    b.build(&[sub])
}

/// A circuit that computes x * 42 - y
pub fn mul_const() -> Circuit<u64> {
    let n_in = 2;
    let x = ARG0;
    let y = ARG0 + 1;
    let mut b = Builder::new(n_in);
    let c = b.push_const(42);
    let mul = b.mul_const(x, c);
    let sub = b.sub(mul, y);
    b.build(&[sub])
}

/// A circuit that computes x.pow(y) - z,
/// Assumes 0 <= y < 4
pub fn pow() -> Circuit<u64> {
    let n_in = 3;
    let x = ARG0;
    let y = ARG0 + 1;
    let z = ARG0 + 2;
    let mut b = Builder::new(n_in);
    let one = b.push_const(1);
    let one = b.const_(one);
    let xpow0 = one;
    let xpow1 = x;
    let xpow2 = b.mul(xpow1, x);
    let xpow3 = b.mul(xpow2, x);
    // let xpow4 = b.mul(xpow3, x);
    // let xpow5 = b.mul(xpow4, x);
    let mul = b.select(y, &[xpow0, xpow1, xpow2, xpow3]);
    let sub = b.sub(mul, z);
    b.build(&[sub])
}

/// A circuit that computes x + y - 42
pub fn add_eq_42() -> Circuit<u64> {
    let n_in = 2;
    let x = ARG0;
    let y = ARG0 + 1;
    let mut b = Builder::new(n_in);
    let add = b.add(&[x, y]);
    let c42 = b.push_const(42);
    let c42 = b.const_(c42);
    let sub = b.sub(add, c42);
    b.build(&[sub])
}

/// A circuit that computes x + y - z
pub fn add_eq() -> Circuit<u64> {
    let n_in = 3;
    let x = ARG0;
    let y = ARG0 + 1;
    let z = ARG0 + 2;
    let mut b = Builder::new(n_in);
    let add = b.add(&[x, y]);
    let sub = b.sub(add, z);
    b.build(&[sub])
}

/// A circuit that computes a * b * c - d
pub fn mul_mul_eq() -> Circuit<u64> {
    let n_in = 4;
    let x = ARG0;
    let y = ARG0 + 1;
    let z = ARG0 + 2;
    let a = ARG0 + 3;
    let mut b = Builder::new(n_in);
    let mul = b.mul(x, y);
    let mul = b.mul(mul, z);
    let sub = b.sub(mul, a);
    b.build(&[sub])
}

/// A circuit that computes select(x, [y, z])
pub fn select_eq() -> Circuit<u64> {
    let n_in = 3;
    let x = ARG0;
    let y = ARG0 + 1;
    let z = ARG0 + 2;
    let mut b = Builder::new(n_in);
    let select = b.select(x, &[y, z]);
    b.build(&[select])
}

/// A circuit that computes select(i, [a, b, c, d])
pub fn select_eq2() -> Circuit<u64> {
    let n_in = 5;
    let i = ARG0;
    let a = ARG0 + 1;
    let b_ = ARG0 + 2;
    let c = ARG0 + 3;
    let d = ARG0 + 4;
    let mut b = Builder::new(n_in);
    let select = b.select(i, &[a, b_, c, d]);
    b.build(&[select])
}

/// A circuit that computes select_const(i, cs)
pub fn select_const(c1: u64, c2: u64) -> Circuit<u64> {
    let n_in = 1;
    let x = ARG0;
    let mut b = Builder::new(n_in);
    let c1 = b.push_const(c1);
    let c2 = b.push_const(c2);
    let select = b.select_const_range(x, c1, c2 + 1, 1);
    b.build(&[select])
}

/// A circuit that computes select_const(i, cs)
pub fn select_const_vec(cs: &[u64]) -> Circuit<u64> {
    let n_in = 1;
    let x = ARG0;
    let mut b = Builder::new(n_in);
    let mut ids = vec![];
    for c in cs {
        ids.push(b.push_const(*c));
    }
    let select = b.select_const_range(x, ids[0], ids[ids.len() - 1] + 1, 1);
    b.build(&[select])
}

/// A circuit that computes encode_4(a, b, c, d) - c
pub fn encode4(c: u64) -> Circuit<u64> {
    let n_in = 4;
    let x0 = ARG0;
    let mut b = Builder::new(n_in);
    let y = b.encode4(x0);
    let c = b.push_const(c);
    let c = b.const_(c);
    let z = b.sub(y, c);
    b.build(&[z])
}

/// A circuit that computes decode32(x),
/// which outputs 00...0 only if x is 0.
pub fn decode32() -> Circuit<u64> {
    let n_in = 1;
    let x = ARG0;
    let mut b = Builder::new(n_in);
    let x0 = b.decode32(x);
    b.build(&(x0..x0 + 32).collect::<Vec<_>>())
}

/// A circuit that computes decode64(x),
/// which outputs 00...0 only if x is 0.
pub fn decode64() -> Circuit<u64> {
    let n_in = 1;
    let x = ARG0;
    let mut b = Builder::new(n_in);
    let x0 = b.decode64(x);
    b.build(&(x0..x0 + 64).collect::<Vec<_>>())
}

pub fn decode64_2(i: usize) -> Circuit<u64> {
    let n_in = 1;
    let x = ARG0;
    let mut b = Builder::new(n_in);
    let one = b.push_const(1);
    let one = b.const_(one);
    let x0 = b.decode64(x);
    let xi = x0 + i;
    let sub = b.sub(xi, one);
    b.build(&[sub])
}

/// A circuit that, on inputs i, x0, y0, x1, y1, asserts
/// that x(1-i) = y(1-i)

pub fn check_all_eq_but_one() -> Circuit<u64> {
    let n_in = 5;
    let x = ARG0;
    let mut b = Builder::new(n_in);
    let xs = &[(ARG0 + 1, ARG0 + 2), (ARG0 + 3, ARG0 + 4)];
    let _ = b.check_all_eq_but_one(x, xs);
    b.build(&[])
}

// /// A circuit that computes decode32(x)[i] - xi
// /// where
// pub fn decode32(mut x: u64) -> Circuit<u64> {
//     let n_in = 32;
//     let xid = ARG0;
//     let mut b = Builder::new(n_in);
//     // push x in binary as consts
//     let mut ids = vec![];
//     for _ in 0..32 {
//         xi = u64::from(x.trailing_ones() > 0);
//         let id = b.push_const(xi);
//         let id = b.const_(id);
//         ids.push(id);
//         x >>= 1;
//     }
//     let x0 = b.decode32(xid);
//     if
//     let c = b.push_const(c);
//     let c = b.const_(c);
//     let z = b.sub(y, c);
//     b.build(&[z])
// }

#[cfg(test)]
mod test {
    use super::*;
    use crate::circuit::eval64;

    #[test]
    fn test_mul_eq() {
        let c = &mul_eq();
        let w = vec![2, 2, 4];
        assert_eq!(*eval64(c, w).last().unwrap(), 0)
    }
}
