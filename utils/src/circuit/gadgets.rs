use super::builder::Builder;

fn half_adder<T>(b: &mut Builder<T>, x: usize, y: usize) -> (usize, usize) {
    let sum = b.xor(&[x, y]);
    let carry = b.and(x, y);
    (carry, sum)
}

fn full_adder<T>(b: &mut Builder<T>, x: usize, y: usize, carry: usize) -> (usize, usize) {
    let (carry1, sum) = half_adder(b, x, y);
    let (carry2, sum) = half_adder(b, sum, carry);
    let carry = b.or(carry1, carry2);
    (carry, sum)
}

fn ripple_adder<T>(b: &mut Builder<T>, xs: &[usize], ys: &[usize]) -> (usize, Vec<usize>)
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

fn u32_to_bits<T>(b: &mut Builder<T>, x: usize) -> [usize; 32] {
    todo!() // maybe not needed?
}

#[cfg(test)]
mod tests {
    use crate::circuit::{builder::Builder, builder::Res, eval64, gadgets::half_adder, OP_MAX};

    use super::{full_adder, ripple_adder};

    #[test]
    fn test_full_adder() {
        let n_in = 3;
        let mut b = Builder::new(n_in);
        let (x, y, z) = (1 + OP_MAX, 2 + OP_MAX, 3 + OP_MAX);

        let (sum, carry) = full_adder(&mut b, x, y, z);
        let Res {
            gates,
            consts,
            n_gates,
            n_out,
        } = b.build(&[sum, carry]);
        let wires = &mut vec![0; n_gates - n_out + n_in];
        // 0 + 0
        wires[0] = 0;
        wires[0] = 0;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [0, 0]);

        // 0 + 1
        wires[0] = 0;
        wires[1] = 1;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [0, 1]);

        // 1 + 0
        wires[0] = 1;
        wires[1] = 0;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [0, 1]);

        // 1 + 1
        wires[0] = 1;
        wires[1] = 1;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [1, 0]);

        let wires = &mut vec![0; n_gates - n_out + n_in];
        // set carry to 1
        wires[2] = 1;

        // 0 + 0
        wires[0] = 0;
        wires[1] = 0;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [0, 1]);

        // 0 + 1
        wires[0] = 0;
        wires[1] = 1;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);

        assert_eq!(res, [1, 0]);

        // 1 + 0
        wires[0] = 1;
        wires[1] = 0;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [1, 0]);

        // 1 + 1
        wires[0] = 1;
        wires[1] = 1;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [1, 1]);
    }

    #[test]
    fn test_half_adder() {
        let n_in = 2;
        let mut b: Builder<u64> = Builder::new(n_in);
        let (x, y) = (1 + OP_MAX, 2 + OP_MAX);

        let (sum, carry) = half_adder(&mut b, x, y);
        let Res {
            gates,
            consts,
            n_gates,
            n_out,
        } = b.build(&[sum, carry]);
        let wires = &mut vec![0; n_gates - n_out + n_in];
        // 0 + 0
        wires[0] = 0;
        wires[0] = 0;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [0, 0]);

        // 0 + 1
        //         let wires = &mut vec![0; n_gates - n_outs ++ 2];
        wires[0] = 0;
        wires[1] = 1;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [0, 1]);

        // 1 + 0
        wires[0] = 1;
        wires[1] = 0;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [0, 1]);

        // 1 + 1
        wires[0] = 1;
        wires[1] = 1;
        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [1, 0]);
    }

    #[test]
    fn test_ripple_adder() {
        let n_in = 8;
        let mut b = Builder::new(n_in);
        b.validate();

        let xs: Vec<usize> = (1..9).map(|x| x + OP_MAX).collect();
        let ys = &xs[4..];
        let xs = &xs[0..4];

        let (carry, mut sums) = ripple_adder(&mut b, xs, ys);
        assert_eq!(sums.len(), 4);
        sums.push(carry);
        let Res {
            gates,
            consts,
            n_gates,
            n_out,
        } = b.build(&sums);
        let wires = &mut vec![0; n_gates + n_in - n_out];
        wires[0] = 0; // x0
        wires[1] = 0; // x1
        wires[2] = 0; // x2
        wires[3] = 0; // x3
        wires[4] = 0; // y0
        wires[5] = 0; // y1
        wires[6] = 0; // y2
        wires[7] = 0; // y3

        let res = eval64(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [0, 0, 0, 0, 0]);

        // 0101 + 0011 = 1000 w no carry
        wires[0] = 1; //  x0 1
        wires[1] = 0; // x1 0
        wires[2] = 1; //  x2 1
        wires[3] = 0; // x3 0

        wires[4] = 1; //  y0 1
        wires[5] = 1; //  y1 1
        wires[6] = 0; // y2 0
        wires[7] = 0; // y3 0

        let res = eval64(&gates, wires, n_gates, n_in, &consts);

        //                 z0    z1    z2     z3    carry
        assert_eq!(res, [0, 0, 0, 1, 0]);

        // 1101 + 1110 = 1011 w carry
        wires[0] = 1; //  x0 1
        wires[1] = 0; // x1 0
        wires[2] = 1; //  x2 1
        wires[3] = 1; // x3 1

        wires[4] = 0; //  y0 0
        wires[5] = 1; //  y1 1
        wires[6] = 1; // y2 1
        wires[7] = 1; // y3 1

        let res = eval64(&gates, wires, n_gates, n_in, &consts);

        //                 z0    z1    z2     z3    carry
        assert_eq!(res, [1, 1, 0, 1, 1]);
    }
}
