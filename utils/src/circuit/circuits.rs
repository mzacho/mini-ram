use super::builder::Builder;
use super::builder::Res as Circuit;

/// Construct a circuit that computes x * y - z
pub fn mul_eq() -> Circuit<u64> {
    let n_in = 3;
    let x = 0;
    let y = 1;
    let z = 2;
    let mut b = Builder::new(n_in);
    let mul = b.mul(x, y);
    let sub = b.sub(mul, z);
    b.build(&[sub])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::circuit::eval64;

    fn test_mul_eq() {
        let c = &mul_eq();
        let w = vec![2, 2, 4];
        assert_eq!(*eval64(c, w).last().unwrap(), 0)
    }
}
