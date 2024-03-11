use super::builder::Builder;

pub fn half_adder<T>(b: &mut Builder<T>, x: usize, y: usize) -> (usize, usize) {
    let sum = b.xor(&[x, y]);
    let carry = b.and(x, y);
    (carry, sum)
}

pub fn full_adder<T>(b: &mut Builder<T>, x: usize, y: usize, carry: usize) -> (usize, usize) {
    let (carry1, sum) = half_adder(b, x, y);
    let (carry2, sum) = half_adder(b, sum, carry);
    let carry = b.or(carry1, carry2);
    (carry, sum)
}

pub fn ripple_adder<T>(b: &mut Builder<T>, xs: &[usize], ys: &[usize]) -> (usize, Vec<usize>)
where
    T: Default,
{
    assert_eq!(xs.len(), ys.len());
    assert!(!(xs.is_empty() | ys.is_empty()));
    let zero = b.push_const(T::default());
    let zero = b.const_(zero);
    let (mut carry, mut sum) = full_adder(b, xs[0], ys[0], zero);
    let mut sums = vec![sum];
    for i in 1..xs.len() {
        (carry, sum) = full_adder(b, xs[i], ys[i], carry);
        sums.push(sum)
    }
    (carry, sums)
}

/// Decodes instr as encoded by frontend::miniram::encode::encode_instr_u64
///
/// Input: i, the (index of the) constant holding the 64 bit
/// instruction.
///
/// Returns op, dst, arg0, arg1, arg1 as a word
pub fn decode_instr64(b: &mut Builder<u64>, i: usize) -> (usize, usize, usize, usize, usize) {
    // Destruct instruction into its bit-decomposition
    let i1 = b.decode64(i);

    let op = b.encode8(i1 + 56);
    let dst = b.encode4(i1 + 48);
    let arg0 = b.encode4(i1 + 40);
    let arg1 = b.encode4(i1);
    let arg1_word = b.encode32(i1);
    (op, dst, arg0, arg1, arg1_word)
}

#[cfg(test)]
mod tests {
    use crate::circuit::builder::Builder;
    use crate::circuit::eval64;
    use crate::circuit::ARG0;

    use super::{full_adder, half_adder, ripple_adder};

    #[test]
    fn test_full_adder() {
        let n_in = 3;
        let mut b = Builder::new(n_in);
        let (x, y, z) = (ARG0, 1 + ARG0, 2 + ARG0);

        let (sum, carry) = full_adder(&mut b, x, y, z);
        let c = &b.build(&[sum, carry]);

        // 0 + 0
        let wires = vec![0, 0, 0];
        let res = eval64(c, wires);
        assert_eq!(res, [0, 0]);

        // 0 + 1
        let wires = vec![0, 1, 0];
        let res = eval64(c, wires);
        assert_eq!(res, [0, 1]);

        // 1 + 0
        let wires = vec![1, 0, 0];
        let res = eval64(c, wires);
        assert_eq!(res, [0, 1]);

        // 1 + 1
        let wires = vec![1, 1, 0];
        let res = eval64(c, wires);
        assert_eq!(res, [1, 0]);

        // set carry to 1

        // 0 + 0
        let wires = vec![0, 0, 1];
        let res = eval64(c, wires);
        assert_eq!(res, [0, 1]);

        // 0 + 1
        let wires = vec![0, 1, 1];
        let res = eval64(c, wires);
        assert_eq!(res, [1, 0]);

        // 1 + 0
        let wires = vec![1, 0, 1];
        let res = eval64(c, wires);
        assert_eq!(res, [1, 0]);

        // 1 + 1
        let wires = vec![1, 1, 1];
        let res = eval64(c, wires);
        assert_eq!(res, [1, 1]);
    }

    #[test]
    fn test_half_adder() {
        let n_in = 2;
        let mut b: Builder<u64> = Builder::new(n_in);
        let (x, y) = (ARG0, 1 + ARG0);

        let (sum, carry) = half_adder(&mut b, x, y);
        let c = &b.build(&[sum, carry]);
        // 0 + 0
        let wires = vec![0, 0];
        let res = eval64(c, wires);
        assert_eq!(res, [0, 0]);

        // 0 + 1
        let wires = vec![0, 1];
        let res = eval64(c, wires);
        assert_eq!(res, [0, 1]);

        // 1 + 0
        let wires = vec![1, 0];
        let res = eval64(c, wires);
        assert_eq!(res, [0, 1]);

        // 1 + 1
        let wires = vec![1, 1];
        let res = eval64(c, wires);
        assert_eq!(res, [1, 0]);
    }

    #[test]
    fn test_ripple_adder() {
        let n_in = 8;
        let mut b = Builder::new(n_in);
        b.validate();

        let xs: Vec<usize> = (0..8).map(|x| x + ARG0).collect();
        let ys = &xs[4..];
        let xs = &xs[0..4];

        let (carry, mut sums) = ripple_adder(&mut b, xs, ys);
        assert_eq!(sums.len(), 4);
        sums.push(carry);
        let c = &b.build(&sums);
        let mut wires = vec![0; n_in];
        wires[0] = 0; // x0
        wires[1] = 0; // x1
        wires[2] = 0; // x2
        wires[3] = 0; // x3
        wires[4] = 0; // y0
        wires[5] = 0; // y1
        wires[6] = 0; // y2
        wires[7] = 0; // y3

        let res = eval64(c, wires);
        assert_eq!(res, [0, 0, 0, 0, 0]);

        let mut wires = vec![0; n_in];
        // 0101 + 0011 = 1000 w no carry
        wires[0] = 1; //  x0 1
        wires[1] = 0; // x1 0
        wires[2] = 1; //  x2 1
        wires[3] = 0; // x3 0

        wires[4] = 1; //  y0 1
        wires[5] = 1; //  y1 1
        wires[6] = 0; // y2 0
        wires[7] = 0; // y3 0

        let res = eval64(c, wires);

        //                 z0    z1    z2     z3    carry
        assert_eq!(res, [0, 0, 0, 1, 0]);

        let mut wires = vec![0; n_in];
        // 1101 + 1110 = 1011 w carry
        wires[0] = 1; //  x0 1
        wires[1] = 0; // x1 0
        wires[2] = 1; //  x2 1
        wires[3] = 1; // x3 1

        wires[4] = 0; //  y0 0
        wires[5] = 1; //  y1 1
        wires[6] = 1; // y2 1
        wires[7] = 1; // y3 1

        let res = eval64(c, wires);

        //                 z0    z1    z2     z3    carry
        assert_eq!(res, [1, 1, 0, 1, 1]);
    }
}
