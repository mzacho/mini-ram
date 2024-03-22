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
    let x = ARG0 + 0;
    let y = ARG0 + 1;
    let z = ARG0 + 2;
    let mut b = Builder::new(n_in);
    let select = b.select(x, &[y, z]);
    b.build(&[select])
}

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
