use super::builder::Builder;

pub fn bit_comparator<T>(
    b: &mut Builder<T>,
    x: usize,
    y: usize,
    one: usize,
) -> (usize, usize, usize) {
    let x_neg = b.xor_bits(&[x, one]);
    let y_neg = b.xor_bits(&[y, one]);
    let x_lt_y = b.and_bits(x_neg, y);
    let x_gt_y = b.and_bits(y_neg, x);
    let eq = b.xor_bits(&[x_lt_y, x_gt_y, one]);
    (x_lt_y, eq, x_gt_y)
}

/// Inputs: x0, .., xn
///         y0, .., yn
///
/// Outputs: x < y, x = y
///          where x = sum{i=0,..n} xi*2^i
///                y = sum{i=0,..n} yi*2^i
pub fn word_comparator<T>(
    b: &mut Builder<T>,
    xs: &[usize],
    ys: &[usize],
    one: usize,
) -> (usize, usize) {
    assert_eq!(xs.len(), ys.len());
    assert!(!(xs.is_empty() | ys.is_empty()));
    let (lt, eq, _) = bit_comparator(b, xs[0], ys[0], one);
    let mut prev_lt = lt;
    let mut prev_eq = eq;
    for i in 1..xs.len() {
        let (lt, eq, _) = bit_comparator(b, xs[i], ys[i], one);
        let tmp = b.and_bits(eq, prev_lt);
        prev_lt = b.or_bits(lt, tmp);
        prev_eq = b.and_bits(eq, prev_eq);
    }
    (prev_lt, prev_eq)
}

pub fn half_adder<T>(b: &mut Builder<T>, x: usize, y: usize) -> (usize, usize) {
    let sum = b.xor_bits(&[x, y]);
    let carry = b.and_bits(x, y);
    (carry, sum)
}

pub fn full_adder<T>(b: &mut Builder<T>, x: usize, y: usize, carry: usize) -> (usize, usize) {
    let (carry1, sum) = half_adder(b, x, y);
    let (carry2, sum) = half_adder(b, sum, carry);
    let carry = b.or_bits(carry1, carry2);
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

/// Construct a AS-Waksman network with xs as inputs, using the
/// recursive construct of the following article:
///
/// Bruno Beauquier, Eric Darrot. On Arbitrary Waksman
/// Networks and their Vulnerability. RR-3788,
/// INRIA. 1999.
///
/// Inputs:
///   - b: circuit builder
///   - xs: ids of input wires to the network, assumes xs.len() > 0
///   - conf: ids of configuration wires of the network, assumes
///   conf.len() == sum {i=1..n} ceil(log2(i)).
///   - one: id of the constant 1
///
/// Returns:
///   - ids of outputs, guarantees res.0.len() == xs.len())
///   - number of configuration inputs used
pub fn waksman<T>(
    b: &mut Builder<T>,
    xs: &[usize],
    conf: &[usize],
    one: usize,
) -> (Vec<usize>, usize) {
    let n = xs.len();
    if n < 1 {
        panic!()
    };
    if n == 1 {
        // assert_eq!(conf.len(), 0, "expected waksman conf.len()==0");
        // When n=1 the network is just a link
        (vec![xs[0]], 0)
    } else if n == 2 {
        // assert_eq!(conf.len(), 1, "expected waksman conf.len()==1");
        // When n=2 the network is just a switch
        let (a, b) = switch(b, xs[0], xs[1], conf[0], one);
        (vec![a, b], 1)
    } else if n % 2 == 0 {
        // ids of input wires for next layer
        let mut ys = vec![];
        let mut zs = vec![];
        // Switches for input layer
        for i in 0..n / 2 {
            let (a, b) = switch(b, xs[2 * i], xs[2 * i + 1], conf[i], one);
            ys.push(a);
            zs.push(b);
        }
        // Construct sub-networks
        let (ys, c1) = waksman(b, &ys, &conf[n / 2..], one);
        let (zs, c2) = waksman(b, &zs, &conf[n / 2 + c1..], one);
        // Switches for output layer
        let mut res = vec![ys[0], zs[0]];
        for i in 0..n / 2 - 1 {
            let (a, b) = switch(b, ys[i + 1], zs[i + 1], conf[n / 2 + c1 + c2 + i], one);
            res.push(a);
            res.push(b);
        }
        (res, n + c1 + c2 - 1)
    } else {
        // ids of input wires for next layer
        let mut ys = vec![xs[0]];
        let mut zs = vec![];
        // Switches for input layer
        for i in 0..n / 2 {
            let (a, b) = switch(b, xs[2 * i + 1], xs[2 * i + 2], conf[i], one);
            ys.push(a);
            zs.push(b);
        }
        // Construct sub-networks
        let (ys, c1) = waksman(b, &ys, &conf[n / 2..], one);
        let (zs, c2) = waksman(b, &zs, &conf[n / 2 + c1..], one);
        // switches for output layer
        let mut res = vec![ys[0]];
        for i in 0..n / 2 {
            let (a, b) = switch(b, ys[i + 1], zs[i], conf[n / 2 + c1 + c2 + i], one);
            res.push(a);
            res.push(b);
        }
        (res, n + c1 + c2 - 1)
    }
}

/// Construct a binary switch with inputs x, y and configuration bit
/// b by computing:
///
///  c: x*(1-b)+ y*b
///  d: x*b    + y*(1-b)
fn switch<T>(b: &mut Builder<T>, x: usize, y: usize, z: usize, one: usize) -> (usize, usize) {
    let one_minus_z = b.sub(one, z);
    let tmp1 = b.mul(x, one_minus_z);
    let tmp2 = b.mul(x, z);
    let tmp3 = b.mul(y, one_minus_z);
    let tmp4 = b.mul(y, z);
    let c = b.add(&[tmp1, tmp4]);
    let d = b.add(&[tmp2, tmp3]);
    (c, d)
}

/// Decodes lower 32 bit instr as encoded by frontend::miniram::encode::encode_instr_u64
///
/// Input: i, the (index of the) constant holding the lower 32 bit
/// of the instruction.
pub fn decode_lo_instr32<T>(
    b: &mut Builder<T>,
    i: usize,
) -> (usize, usize) {
    let i0 = b.decode32(i);
    let arg1 = b.encode4(i0);
    let arg1_word = i;
    (arg1, arg1_word)
}

/// Decodes high 32 bit instr as encoded by
/// frontend::miniram::encode::encode_instr_u64
///
/// Input: i, the (index of the) constant holding the high 32 bit
/// of the instruction.
pub fn decode_hi_instr32<T>(
    b: &mut Builder<T>,
    i: usize,
) -> (usize, usize, usize, usize, usize, usize) {
    // b.dbg()
    // Destruct instruction into its bit-decomposition
    let i0 = b.decode32(i);

    let op = b.encode8(i0 + 24);
    // lsb of op is 1 only for LDR
    let is_load = i0 + 24;
    // next most lsb of op is 1 only for LDR/ STR
    let is_mem = i0 + 25;
    // op >> 5 is 1 only if op is RET
    let is_ret = i0 + (24 + 5);

    let dst = b.encode4(i0 + 16);
    let arg0 = b.encode4(i0 + 8);
    (op, dst, arg0, is_mem, is_load, is_ret)
}


#[cfg(test)]
mod tests {
    use crate::circuit::ARG0;
    use permutation::Permutation;

    use crate::circuit::{builder, eval64};
    use crate::waksman::{self, route};

    use super::*;

    fn arg(id: usize) -> usize {
        ARG0 + id
    }

    fn argv(argc: usize) -> Vec<usize> {
        let mut res = vec![];
        for i in 0..argc {
            res.push(arg(i))
        }
        res
    }

    // #[test]
    // fn enc_dec() {
    //     let mut b = Builder::new(1);
    //     let x = b.decode64(ARG0);
    //     let x0 = b.encode8(x);
    //     let x1 = b.encode8(x + 8);
    //     let x1 = b.encode8(x + 16);
    // }

    fn build_bit_comparator(n: usize) -> builder::Res<u64> {
        let n_in = n * 2;
        let mut b = Builder::new(n_in);
        let one = b.push_const(1);
        let one = b.const_(one);
        let xs = &argv(n_in)[..n];
        let ys = &argv(n_in)[n..];
        let (lt, eq) = word_comparator(&mut b, xs, ys, one);
        let _ = eq;
        b.build(&[lt])
    }

    #[test]
    fn bit_comparator() {
        let c = build_bit_comparator(2);
        let res = eval64(&c, vec![0, 0, 0, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![1, 0, 0, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![0, 1, 0, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![1, 0, 0, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![1, 1, 0, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![0, 0, 1, 0]);
        assert_eq!(res, [1]);

        let res = eval64(&c, vec![0, 0, 0, 1]);
        assert_eq!(res, [1]);

        let res = eval64(&c, vec![0, 0, 1, 1]);
        assert_eq!(res, [1]);

        let res = eval64(&c, vec![1, 0, 1, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![1, 0, 0, 1]);
        assert_eq!(res, [1]);

        let res = eval64(&c, vec![1, 0, 1, 1]);
        assert_eq!(res, [1]);

        let res = eval64(&c, vec![0, 1, 0, 1]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![0, 1, 1, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![0, 1, 1, 1]);
        assert_eq!(res, [1]);

        let c = build_bit_comparator(3);
        let res = eval64(&c, vec![0, 0, 0, 0, 0, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![0, 0, 1, 0, 0, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![0, 1, 0, 0, 0, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![1, 0, 0, 0, 0, 0]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![0, 0, 0, 0, 0, 1]);
        assert_eq!(res, [1]);

        let res = eval64(&c, vec![0, 0, 1, 0, 0, 1]);
        assert_eq!(res, [0]);

        let res = eval64(&c, vec![1, 0, 0, 0, 1, 0]);
        assert_eq!(res, [1]);

        let res = eval64(&c, vec![0, 1, 0, 0, 0, 1]);
        assert_eq!(res, [1]);

        let res = eval64(&c, vec![0, 1, 0, 1, 0, 0]);
        assert_eq!(res, [0])
    }

    /// Generate settings for routing p, and test that routing the
    /// sequential series 0, 1, ..., n yields the result of applying
    /// p to that series.
    fn run_waksman(p: Permutation) {
        let n = p.len();
        let conf = route(&p);
        let n_in = conf.len() + n;
        let mut b = Builder::new(n_in);

        let xs = argv(n_in);
        let one = b.push_const(1);
        let one = b.const_(one);
        let (o, c_) = waksman(&mut b, &xs[..n], &xs[n..], one);
        assert_eq!(conf.len(), c_);
        let c = &b.build(&o);
        let n_ = u64::try_from(n).unwrap();
        let wires = (0..n_).chain(conf.iter().map(|b| u64::from(*b)));

        let res = eval64(c, wires.collect());
        assert_eq!(res, p.inverse().apply_slice((0..n_).collect::<Vec<_>>()));
    }

    #[test]
    fn route_waksman() {
        // n = 2
        run_waksman(Permutation::one(2));
        run_waksman(Permutation::oneline(vec![0, 1]));
        run_waksman(Permutation::oneline(vec![1, 0]));
        // n = 3
        run_waksman(Permutation::one(3));
        run_waksman(Permutation::oneline(vec![0, 1, 2]));
        run_waksman(Permutation::oneline(vec![1, 0, 2]));
        run_waksman(Permutation::oneline(vec![2, 1, 0]));
        run_waksman(Permutation::oneline(vec![1, 2, 0]));
        run_waksman(Permutation::oneline(vec![0, 2, 1]));
        run_waksman(Permutation::oneline(vec![2, 0, 1]));
        // n = 4
        run_waksman(Permutation::one(4));
        run_waksman(Permutation::oneline(vec![0, 1, 2, 3]));
        run_waksman(Permutation::oneline(vec![1, 2, 3, 0]));
        run_waksman(Permutation::oneline(vec![2, 3, 0, 1]));
        run_waksman(Permutation::oneline(vec![3, 0, 1, 2]));
        run_waksman(Permutation::oneline(vec![0, 1, 3, 2]));
        run_waksman(Permutation::oneline(vec![1, 0, 3, 2]));
        run_waksman(Permutation::oneline(vec![1, 0, 2, 3]));
        run_waksman(Permutation::oneline(vec![0, 3, 1, 2]));
        run_waksman(Permutation::oneline(vec![0, 2, 3, 1]));

        run_waksman(Permutation::one(5));
        // n = 8
        run_waksman(Permutation::one(8));
        run_waksman(Permutation::oneline(vec![0, 2, 3, 4, 6, 1, 5, 7]));
        // n = 9
        run_waksman(Permutation::oneline(vec![8, 4, 5, 2, 6, 3, 1, 0, 7]));
    }

    #[test]
    fn route9() {
        let conf = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1,
        ];

        let n_in = conf.len() + 9;
        assert_eq!(conf.len(), waksman::conf_len(9));
        let mut b = Builder::new(n_in);

        let xs = argv(n_in);
        let one = b.push_const(1);
        let one = b.const_(one);
        let (o, _) = waksman(&mut b, &xs[..9], &xs[9..], one);
        let c = &b.build(&o);
        let wires = (0..9).chain(conf);

        let res = eval64(c, wires.collect());
        assert_eq!(res, vec![0, 8, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn cmp_words() {
        let mut b = Builder::new(2);
        let one = b.push_const(1);
        let one = b.const_(one);
        let x0 = b.decode32(ARG0);
        let y0 = b.decode32(ARG0 + 1);
        let xs = &(x0..x0 + 32).collect::<Vec<_>>();
        let ys = &(y0..y0 + 32).collect::<Vec<_>>();
        let (lt, eq) = word_comparator(&mut b, xs, ys, one);
        let c = &b.build(&[lt, eq]);

        let res = eval64(c, vec![0, 1]);
        assert_eq!(res, vec![1, 0]);

        let res = eval64(c, vec![0, 0]);
        assert_eq!(res, vec![0, 1]);

        let res = eval64(c, vec![1, 0]);
        assert_eq!(res, vec![0, 0]);

        let res = eval64(c, vec![1, 1]);
        assert_eq!(res, vec![0, 1]);

        let res = eval64(c, vec![32, 32]);
        assert_eq!(res, vec![0, 1]);

        let res = eval64(c, vec![33, 32]);
        assert_eq!(res, vec![0, 0]);

        let res = eval64(c, vec![24, 32]);
        assert_eq!(res, vec![1, 0])
    }

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
