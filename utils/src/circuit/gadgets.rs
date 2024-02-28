use super::builder::Builder;

fn half_adder(b: &mut Builder<bool>, x: usize, y: usize) -> (usize, usize) {
    let sum = b.xor(&[x, y]);
    let carry = b.and(x, y);
    (carry, sum)
}

fn full_adder(b: &mut Builder<bool>, x: usize, y: usize, carry: usize) -> (usize, usize) {
    let (carry1, sum) = half_adder(b, x, y);
    let (carry2, sum) = half_adder(b, sum, carry);
    let carry = b.or(carry1, carry2);
    (carry, sum)
}

fn ripple_adder(b: &mut Builder<bool>, xs: &[usize], ys: &[usize]) -> (usize, Vec<usize>) {
    assert_eq!(xs.len(), ys.len());
    assert!(!(xs.is_empty() | ys.is_empty()));
    let zero = b.push_const(false);
    let zero = b.const_(zero);
    let (mut carry, mut sum) = full_adder(b, xs[0], ys[0], zero);
    let mut sums = vec![sum];
    for i in 1..xs.len() {
        (carry, sum) = full_adder(b, xs[i], ys[i], carry);
        sums.push(sum)
    }
    (carry, sums)
}

#[cfg(test)]
mod tests {
    use crate::circuit::{builder::Builder, eval2, gadgets::half_adder, OP_MAX};

    use super::{full_adder, ripple_adder};

    #[test]
    fn test_full_adder() {
        let n_in = 3;
        let mut b = Builder::new(n_in);
        let (x, y, z) = (1 + OP_MAX, 2 + OP_MAX, 3 + OP_MAX);

        let (sum, carry) = full_adder(&mut b, x, y, z);
        let (gates, consts, n_gates, n_out) = b.build(&[sum, carry]);
        let wires = &mut vec![false; n_gates - n_out + n_in];
        // 0 + 0
        wires[0] = false;
        wires[0] = false;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [false, false]);

        // 0 + 1
        wires[0] = false;
        wires[1] = true;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [false, true]);

        // 1 + 0
        wires[0] = true;
        wires[1] = false;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [false, true]);

        // 1 + 1
        wires[0] = true;
        wires[1] = true;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [true, false]);

        let wires = &mut vec![false; n_gates - n_out + n_in];
        // set carry to 1
        wires[2] = true;

        // 0 + 0
        wires[0] = false;
        wires[1] = false;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [false, true]);

        // 0 + 1
        wires[0] = false;
        wires[1] = true;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);

        assert_eq!(res, [true, false]);

        // 1 + 0
        wires[0] = true;
        wires[1] = false;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [true, false]);

        // 1 + 1
        wires[0] = true;
        wires[1] = true;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [true, true]);
    }

    #[test]
    fn test_half_adder() {
        let n_in = 2;
        let mut b = Builder::new(n_in);
        let (x, y) = (1 + OP_MAX, 2 + OP_MAX);

        let (sum, carry) = half_adder(&mut b, x, y);
        let (gates, consts, n_gates, n_outs) = b.build(&[sum, carry]);
        let wires = &mut vec![false; n_gates - n_outs + n_in];
        // 0 + 0
        wires[0] = false;
        wires[0] = false;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [false, false]);

        // 0 + 1
        //         let wires = &mut vec![false; n_gates - n_outs ++ 2];
        wires[0] = false;
        wires[1] = true;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [false, true]);

        // 1 + 0
        wires[0] = true;
        wires[1] = false;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [false, true]);

        // 1 + 1
        wires[0] = true;
        wires[1] = true;
        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [true, false]);
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
        let (gates, consts, n_gates, n_outs) = b.build(&sums);
        let wires = &mut vec![false; n_gates + n_in - n_outs];
        wires[0] = false; // x0
        wires[1] = false; // x1
        wires[2] = false; // x2
        wires[3] = false; // x3
        wires[4] = false; // y0
        wires[5] = false; // y1
        wires[6] = false; // y2
        wires[7] = false; // y3

        let res = eval2(&gates, wires, n_gates, n_in, &consts);
        assert_eq!(res, [false, false, false, false, false]);

        // 0101 + 0011 = 1000 w no carry
        wires[0] = true; //  x0 1
        wires[1] = false; // x1 0
        wires[2] = true; //  x2 1
        wires[3] = false; // x3 0

        wires[4] = true; //  y0 1
        wires[5] = true; //  y1 1
        wires[6] = false; // y2 0
        wires[7] = false; // y3 0

        let res = eval2(&gates, wires, n_gates, n_in, &consts);

        //                 z0    z1    z2     z3    carry
        assert_eq!(res, [false, false, false, true, false]);

        // 1101 + 1110 = 1011 w carry
        wires[0] = true; //  x0 1
        wires[1] = false; // x1 0
        wires[2] = true; //  x2 1
        wires[3] = true; // x3 1

        wires[4] = false; //  y0 0
        wires[5] = true; //  y1 1
        wires[6] = true; // y2 1
        wires[7] = true; // y3 1

        let res = eval2(&gates, wires, n_gates, n_in, &consts);

        //                 z0    z1    z2     z3    carry
        assert_eq!(res, [true, true, false, true, true]);
    }
}
