pub mod builder;
pub mod circuits;
pub mod gadgets;
pub mod pp;

// Operations for evaluation:
// - binary
pub const OP_XOR: usize = 0;
pub const OP_AND: usize = 1;
pub const OP_AND_CONST: usize = 2;
// - arithmetic
pub const OP_ADD: usize = 3;
pub const OP_SUB: usize = 4;
pub const OP_MUL: usize = 5;
pub const OP_MUL_CONST: usize = 6;
pub const OP_SELECT: usize = 7;
pub const OP_SELECT_CONST: usize = 19;
// - binary and arithmetic
pub const OP_CONV_B2A: usize = 8;
pub const OP_CONV_A2B: usize = 9;
pub const OP_DECODE32: usize = 22;
pub const OP_DECODE64: usize = 16;
pub const OP_ENCODE4: usize = 21;
pub const OP_ENCODE8: usize = 17;
pub const OP_ENCODE32: usize = 18;
pub const OP_CONST: usize = 10;
pub const OP_OUT: usize = 11;
// Operations for verification:
pub const OP_CHECK_Z: usize = 12;
pub const OP_CHECK_EQ: usize = 13;
pub const OP_CHECK_AND: usize = 14;
pub const OP_CHECK_ALL_EQ_BUT_ONE: usize = 15;

pub const OP_DEBUG: usize = 20;

/// Index of the first two arguments
pub const ARG0: usize = 23;
pub const ARG1: usize = ARG0 + 1;

use std::u64::MAX;
use std::u32::MAX as U32MAX;

pub use builder::Res as Circuit;

// pub struct U64Circuit<'a> {
//     pub gates: &'a [usize],
//     pub consts: &'a [u64],
//     pub n_gates: usize,
//     pub n_in: usize,
// }

/// Evaluate a circuit of u64 values
pub fn eval64(c: &Circuit<u64>, mut wires: Vec<u64>) -> Vec<u64> {
    // dbg!(c);
    let gates = &c.gates;
    let consts = &c.consts;
    let n_gates = c.n_gates;
    assert_eq!(c.n_in, wires.len());
    assert_eq!(n_gates, count_ops(gates));
    let mut out = Vec::new();
    let mut i = 0;
    for _ in 0..n_gates {
        let op = gates[i];
        i += 1;
        let mut res = 0;
        //dbg!(&op);
        match op {
            // --- binary ops
            OP_XOR => {
                // args: idx1, idx2, ..., idxn
                // outw: x1 xor x2 xor ... xn
                while gates[i] >= ARG0 {
                    res ^= wires[gates[i] - ARG0];
                    i += 1;
                    if i >= gates.len() {
                        // this gate was the last one
                        break;
                    }
                }
            }
            OP_AND => {
                // args: idx, idy
                // outw: x and y
                let lhs = wires[gates[i] - ARG0];
                let rhs = wires[gates[i + 1] - ARG0];
                res = lhs & rhs;
                i += 2;
            }
            OP_AND_CONST => {
                // args: idc, idx
                // outw: x and c
                let c = consts[gates[i] - ARG0];
                let x = wires[gates[i + 1] - ARG0];
                res = c & x;
                i += 2;
            }
            // --- arithmetic ops
            OP_ADD => {
                // args: idx1, idx2, ..., idxn
                // outw: x1 + x2 + ... xn
                while gates[i] >= ARG0 {
                    res = res.wrapping_add(wires[gates[i] - ARG0]);
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
            }
            OP_SUB => {
                // args: idx, idy
                // outw: x - y
                let lhs = wires[gates[i] - ARG0];
                let rhs = wires[gates[i + 1] - ARG0];
                //dbg!(lhs, rhs);
                res = lhs.wrapping_sub(rhs);
                i += 2;
            }
            OP_MUL => {
                // args: idx, idy
                // outw: x * y
                let lhs = wires[gates[i] - ARG0];
                let rhs = wires[gates[i + 1] - ARG0];
                //dbg!(lhs, rhs);
                res = lhs.wrapping_mul(rhs);
                i += 2;
            }
            OP_MUL_CONST => {
                // args: idc, idx
                // outw: c * x
                let c = consts[gates[i] - ARG0];
                let x = wires[gates[i + 1] - ARG0];
                res = c.wrapping_mul(x);
                i += 2;
            }
            OP_SELECT => {
                // args: idi, idx1, idx2, ..., idxn where i <= n
                // outw: xi
                let i_ = wires[gates[i] - ARG0];
                let i_: usize = i_.try_into().ok().unwrap();
                //dbg!(i_);
                i += i_ + 1;
                res = wires[gates[i] - ARG0];
                while gates[i] >= ARG0 {
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
            }
            OP_SELECT_CONST => {
                // args: idi, idc1, idc2, ..., idcn where i <= n
                // outw: ci
                let i_ = wires[gates[i] - ARG0];
                let i_: usize = i_.try_into().ok().unwrap();
                i += i_ + 1;
                res = consts[gates[i] - ARG0];
                while gates[i] >= ARG0 {
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
            }
            OP_DECODE32 => {
                // args: x where x < 2^32
                // outw: idx1, idx2, ..., idxn s.t sum 2^{i-1}*xi
                let mut x = wires[gates[i] - ARG0];
                u32::try_from(x).unwrap();
                //dbg!(x);
                for _ in 1..32 {
                    res = u64::from(x.trailing_ones() > 0);
                    wires.push(res);
                    x >>= 1;
                }
                res = u64::from(x.trailing_ones() > 0);
                i += 1;
            }
            OP_DECODE64 => {
                // args: x where x < 2^64
                // outw: idx1, idx2, ..., idxn s.t sum 2^{i-1}*xi
                let mut x = wires[gates[i] - ARG0];
                //dbg!(x);
                for _ in 1..64 {
                    res = u64::from(x.trailing_ones() > 0);
                    wires.push(res);
                    x >>= 1;
                }
                res = u64::from(x.trailing_ones() > 0);
                i += 1;
            }
            OP_ENCODE4 => {
                // args: idx1, idx2, idx3, idx4
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                // TODO: Assert that xs are bits
                for k in 0..4 {
                    let xk = wires[gates[i] - ARG0];
                    res += 2u64.pow(k) * xk;
                    i += 1;
                }
            }
            OP_ENCODE8 => {
                // args: idx1, idx2, ..., idx8
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                // TODO: Assert that xs are bits
                for k in 0..8 {
                    let xk = wires[gates[i] - ARG0];
                    res += 2u64.pow(k) * xk;
                    i += 1;
                }
            }
            OP_ENCODE32 => {
                // args: idx1, idx2, ..., idx32
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                // TODO: Assert that xs are bits
                for k in 0..32 {
                    let xk = wires[gates[i] - ARG0];
                    res += 2u64.pow(k) * xk;
                    i += 1;
                }
            }

            // --- mixed ops
            OP_CONV_A2B => {
                // assert x in Z_{2^64} is a bit
                let x = wires[gates[i] - ARG0];
                assert!(x < 2);
                // move result
                res = x;
                i += 1;
            }
            OP_CONV_B2A => {
                // assert x in Z_{2^64} is a bit
                let x = wires[gates[i] - ARG0];
                // move result
                res = x;
                i += 1;
            }
            OP_CONST => {
                // args: idc
                // outw: c
                res = consts[gates[i] - ARG0];
                i += 1;
            }
            OP_OUT => {
                // args: idx
                // outw: none
                // out: x
                out.push(wires[gates[i] - ARG0]);
                i += 1;
            }
            // --- verificatin ops
            //  if check fails, adds 1 to output
            //  if check succeeds, adds 0 to output
            OP_CHECK_Z => (),   // noop
            OP_CHECK_EQ => (),  // noop
            OP_CHECK_AND => (), // noop
            OP_CHECK_ALL_EQ_BUT_ONE => {
                // args: idi, idx1, idy1, idx2, idy2,..., idxn, idyn
                // asserts: xj = yj for j != i
                let mut res_ = true;
                let mut i_ = wires[gates[i] - ARG0];
                // dbg!(i_);
                i += 1;
                while gates[i] > ARG0 {
                    if i_ == 0 {
                        i += 2;
                        i_ = MAX;
                        continue;
                    }
                    let x = wires[gates[i] - ARG0];
                    let y = wires[gates[i + 1] - ARG0];
                    // dbg!(x, y, i, gates[i], gates[i + 1]);
                    res_ &= x == y;
                    i += 2;
                    i_ -= 1;
                }
                assert!(res_)
            }
            OP_DEBUG => {
                dbg!("here");
            }

            _ => panic!("invalid operation"),
        }
        if (op != OP_OUT) & !is_check(op) & !matches!(op, OP_DEBUG) {
            // dbg!(res);
            wires.push(res);
        }
        // dbg!(&wires);
    }
    pp::print(c, Some(&wires));
    out
}

/// Evaluate a circuit of u32 values
pub fn eval32(c: &Circuit<u32>, mut wires: Vec<u32>) -> Vec<u32> {
    // dbg!(c);
    let gates = &c.gates;
    let consts = &c.consts;
    let n_gates = c.n_gates;
    assert_eq!(c.n_in, wires.len());
    assert_eq!(n_gates, count_ops(gates));
    let mut out = Vec::new();
    let mut i = 0;
    for _ in 0..n_gates {
        let op = gates[i];
        i += 1;
        let mut res = 0;
        // dbg!(&op);
        match op {
            // --- binary ops
            OP_XOR => {
                // args: idx1, idx2, ..., idxn
                // outw: x1 xor x2 xor ... xn
                while gates[i] >= ARG0 {
                    res ^= wires[gates[i] - ARG0];
                    i += 1;
                    if i >= gates.len() {
                        // this gate was the last one
                        break;
                    }
                }
            }
            OP_AND => {
                // args: idx, idy
                // outw: x and y
                let lhs = wires[gates[i] - ARG0];
                let rhs = wires[gates[i + 1] - ARG0];
                res = lhs & rhs;
                i += 2;
            }
            OP_AND_CONST => {
                // args: idc, idx
                // outw: x and c
                let c = consts[gates[i] - ARG0];
                let x = wires[gates[i + 1] - ARG0];
                res = c & x;
                i += 2;
            }
            // --- arithmetic ops
            OP_ADD => {
                // args: idx1, idx2, ..., idxn
                // outw: x1 + x2 + ... xn
                while gates[i] >= ARG0 {
                    res = res.wrapping_add(wires[gates[i] - ARG0]);
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
            }
            OP_SUB => {
                // args: idx, idy
                // outw: x - y
                let lhs = wires[gates[i] - ARG0];
                let rhs = wires[gates[i + 1] - ARG0];
                //dbg!(lhs, rhs);
                res = lhs.wrapping_sub(rhs);
                i += 2;
            }
            OP_MUL => {
                // args: idx, idy
                // outw: x * y
                let lhs = wires[gates[i] - ARG0];
                let rhs = wires[gates[i + 1] - ARG0];
                //dbg!(lhs, rhs);
                res = lhs.wrapping_mul(rhs);
                i += 2;
            }
            OP_MUL_CONST => {
                // args: idc, idx
                // outw: c * x
                // dbg!(i, gates[i] - ARG0);
                let c = consts[gates[i] - ARG0];
                let x = wires[gates[i + 1] - ARG0];
                res = c.wrapping_mul(x);
                i += 2;
            }
            OP_SELECT => {
                // args: idi, idx1, idx2, ..., idxn where i <= n
                // outw: xi
                let i_ = wires[gates[i] - ARG0];
                let i_: usize = i_.try_into().ok().unwrap();
                //dbg!(i_);
                i += i_ + 1;
                res = wires[gates[i] - ARG0];
                while gates[i] >= ARG0 {
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
            }
            OP_SELECT_CONST => {
                // args: idi, idc1, idc2, ..., idcn where i <= n
                // outw: ci
                let i_ = wires[gates[i] - ARG0];
                let i_: usize = i_.try_into().ok().unwrap();
                i += i_ + 1;
                res = consts[gates[i] - ARG0];
                while gates[i] >= ARG0 {
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
            }
            OP_DECODE32 => {
                // args: x where x < 2^32
                // outw: idx1, idx2, ..., idxn s.t sum 2^{i-1}*xi
                let mut x = wires[gates[i] - ARG0];
                //dbg!(x);
                for _ in 1..32 {
                    res = u32::from(x.trailing_ones() > 0);
                    wires.push(res);
                    x >>= 1;
                }
                res = u32::from(x.trailing_ones() > 0);
                i += 1;
            }
            OP_DECODE64 => {
                panic!("unsupported")
            }
            OP_ENCODE4 => {
                // args: idx1, idx2, idx3, idx4
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                // TODO: Assert that xs are bits
                for k in 0..4 {
                    let xk = wires[gates[i] - ARG0];
                    res += 2u32.pow(k) * xk;
                    i += 1;
                }
            }
            OP_ENCODE8 => {
                // args: idx1, idx2, ..., idx8
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                // TODO: Assert that xs are bits
                for k in 0..8 {
                    let xk = wires[gates[i] - ARG0];
                    res += 2u32.pow(k) * xk;
                    i += 1;
                }
            }
            OP_ENCODE32 => {
                // args: idx1, idx2, ..., idx32
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                // TODO: Assert that xs are bits
                for k in 0..32 {
                    let xk = wires[gates[i] - ARG0];
                    res += 2u32.pow(k) * xk;
                    i += 1;
                }
            }

            // --- mixed ops
            OP_CONV_A2B => {
                // assert x in Z_{2^32} is a bit
                let x = wires[gates[i] - ARG0];
                assert!(x < 2);
                // move result
                res = x;
                i += 1;
            }
            OP_CONV_B2A => {
                // assert x in Z_{2^32} is a bit
                let x = wires[gates[i] - ARG0];
                // move result
                res = x;
                i += 1;
            }
            OP_CONST => {
                // args: idc
                // outw: c
                res = consts[gates[i] - ARG0];
                i += 1;
            }
            OP_OUT => {
                // args: idx
                // outw: none
                // out: x
                // dbg!(i);
                out.push(wires[gates[i] - ARG0]);
                i += 1;
            }
            // --- verificatin ops
            //  if check fails, adds 1 to output
            //  if check succeeds, adds 0 to output
            OP_CHECK_Z => (),   // noop
            OP_CHECK_EQ => (),  // noop
            OP_CHECK_AND => (), // noop
            OP_CHECK_ALL_EQ_BUT_ONE => {
                // args: idi, idx1, idy1, idx2, idy2,..., idxn, idyn
                // asserts: xj = yj for j != i
                let mut res_ = true;
                let mut i_ = wires[gates[i] - ARG0];
                dbg!(i_);
                i += 1;
                while gates[i] > ARG0 {
                    if i_ == 0 {
                        i += 2;
                        i_ = U32MAX;
                        continue;
                    }
                    let x = wires[gates[i] - ARG0];
                    let y = wires[gates[i + 1] - ARG0];
                    dbg!(x, y, i, gates[i], gates[i + 1]);
                    res_ &= x == y;
                    i += 2;
                    i_ -= 1;
                }
                assert!(res_)
            }
            OP_DEBUG => {
                dbg!("here");
            }

            _ => panic!("invalid operation"),
        }
        if (op != OP_OUT) & !is_check(op) & !matches!(op, OP_DEBUG) {
            // dbg!(res);
            wires.push(res);
        }
        // dbg!(&wires);
    }
    pp::print(c, Some(&wires));
    out
}


/// Counts number of gates
fn count_ops(gates: &[usize]) -> usize {
    let mut res = 0;
    for i in gates {
        if *i < ARG0 {
            res += 1;
        }
    }
    res
}

/// Counts number of output gates
pub fn count_out(gates: &[usize]) -> usize {
    let mut res = 0;
    for i in gates {
        if *i == OP_OUT {
            res += 1;
        }
    }
    res
}

/// Counts number of mul or and gates
pub fn count_mul(gates: &[usize]) -> usize {
    let mut res = 0;
    for i in gates {
        if (*i == OP_MUL) | (*i == OP_AND) {
            res += 1;
        }
    }
    res
}

pub fn is_check(op: usize) -> bool {
    matches!(
        op,
        OP_CHECK_Z | OP_CHECK_EQ | OP_CHECK_AND | OP_CHECK_ALL_EQ_BUT_ONE
    )
}

#[cfg(test)]
mod tests {
    use self::builder::Builder;

    use super::*;
    #[test]
    fn eval_xor() {
        let x = ARG0;
        let y = ARG0 + 1;
        let gates = vec![OP_XOR, x, y, OP_OUT, y + 1];
        let n_gates = 2;
        let consts = vec![];

        let wires = vec![34, 8];
        let c = &Circuit {
            gates,
            consts,
            n_gates,
            n_in: wires.len(),
            n_out: 1,
            n_mul: 0,
            n_select: 0,
            n_select_const: 0,
            n_decode32: 0,
            n_check_all_eq_but_one: 0,
        };
        let res = eval64(c, wires);
        assert_eq!(*res.last().unwrap(), 42)
    }

    #[test]
    fn eval_add() {
        let x = ARG0;
        let y = ARG0 + 1;
        // (x + y) + x
        let gates = vec![OP_ADD, x, y, OP_ADD, x, y + 1, OP_OUT, y + 2];
        let n_gates = 3;
        let consts = vec![];

        let wires = vec![34, 8];
        let c = &Circuit {
            gates,
            consts,
            n_gates,
            n_in: wires.len(),
            n_out: 1,
            n_mul: 0,
            n_select: 0,
            n_select_const: 0,
            n_decode32: 0,
            n_check_all_eq_but_one: 0,
        };
        let res = eval64(c, wires);
        assert_eq!(*res.last().unwrap(), 42 + 34)
    }

    #[test]
    fn eval_decode() {
        // in: n, x where x < 2^32
        // out: x1 xor x2 xor ... xor x32
        let x = ARG0;
        let mut b = Builder::new(1);
        let x1 = b.decode32(x);
        let out = b.xor_bits(&(x1..x1 + 32).collect::<Vec<usize>>());
        let c = &b.build(&[out]);

        let wires = vec![0b1001010];
        let res = eval64(c, wires);
        assert_eq!(*res.last().unwrap(), 1);

        let wires = vec![0b0];
        let res = eval64(c, wires);
        assert_eq!(*res.last().unwrap(), 0);

        let wires = vec![0b010];
        let res = eval64(c, wires);
        assert_eq!(*res.last().unwrap(), 1);

        let wires = vec![0b010101];
        let res = eval64(c, wires);
        assert_eq!(*res.last().unwrap(), 1);
    }

    #[test]
    fn output_const() {
        let mut b = Builder::<u64>::new(1);
        let c = b.push_const(42);
        let c = b.const_(c);
        let g = &b.build(&[c]);
        let res = eval64(g, vec![0]);
        assert_eq!(res, vec![42]);
    }

    #[test]
    fn add_const() {
        let mut b = Builder::<u64>::new(1);
        let ten = b.push_const(21);
        let c1 = b.const_(ten);
        let c2 = b.const_(ten);
        let o = b.add(&[c1, c2]);
        let g = &b.build(&[o]);
        let res = eval64(g, vec![0]);
        assert_eq!(res, vec![42]);
    }

    #[test]
    fn aritmetic_xor() {
        let mut b = Builder::<u64>::new(2);
        b.disable_z2_ops();
        let z = b.xor_bits(&[ARG0, ARG0 + 1]);
        let c = &b.build(&[z]);

        assert_eq!(eval64(c, vec![0, 0]), vec![0]);
        assert_eq!(eval64(c, vec![0, 1]), vec![1]);
        assert_eq!(eval64(c, vec![1, 0]), vec![1]);
        assert_eq!(eval64(c, vec![1, 1]), vec![0]);
    }

    // #[test]
    // fn eval_add_variadic() {
    //     let x = OP_MAX + 1;
    //     let y = OP_MAX + 2;
    //     let z = OP_MAX + 3;
    //     // (x + y) + x
    //     let gates = &[OP_ADD, x, y, z];
    //     let n_gates = 1;
    //     let n_in = 3;
    //     let consts = &[];

    //     let wires = &mut [34, 8, 10, 0];
    //     let c = U64Circuit {
    //         gates,
    //         consts,
    //         n_gates, n_in: wires.len(),
    //         n_in,
    //     };
    //     eval64(c, wires);
    //     assert_eq!(wires[wires.len() - 1], 52)
    // }

    // #[test]
    // fn eval_mul() {
    //     let x = OP_MAX + 1;
    //     let y = OP_MAX + 2;
    //     // ((x * y) + x) * y
    //     let gates = &[OP_MUL, x, y, OP_ADD, y + 1, x, OP_MUL, y + 2, y];
    //     let n_gates = 3;
    //     let n_in = 2;
    //     let consts = &[];

    //     let wires = &mut [2, 5, 0, 0, 0];
    //     let c = U64Circuit {
    //         gates,
    //         consts,
    //         n_gates, n_in: wires.len(),
    //         n_in,
    //     };
    //     eval64(c, wires);
    //     assert_eq!(wires[wires.len() - 1], ((2 * 5) + 2) * 5)
    // }

    // #[test]
    // fn eval_mul_const() {
    //     let x = OP_MAX + 1;
    //     let c = OP_MAX + 1;
    //     // (10 + x) * 20
    //     let gates = &[OP_CONST, c, OP_ADD, x, x + 1, OP_MUL_CONST, c + 1, x + 2];
    //     let n_gates = 3;
    //     let n_in = 1;
    //     let consts = &[10, 20];

    //     let wires = &mut [7, 0, 0, 0];
    //     let c = U64Circuit {
    //         gates,
    //         consts,
    //         n_gates, n_in: wires.len(),
    //         n_in,
    //     };
    //     eval64(c, wires);
    //     assert_eq!(wires[wires.len() - 1], (10 + 7) * 20)
    // }

    // #[test]
    // fn eval_select() {
    //     let x = OP_MAX + 1;
    //     let y = OP_MAX + 2;
    //     let i = OP_MAX + 3;
    //     // [x, y][i]
    //     let gates = &[OP_SELECT, i, x, y];
    //     let n_gates = 1;
    //     let n_in = 3;
    //     let consts = &[];

    //     let wires = &mut [10, 42, 0, 0];
    //     let c = U64Circuit {
    //         gates,
    //         consts,
    //         n_gates, n_in: wires.len(),
    //         n_in,
    //     };
    //     eval64(c, wires);
    //     assert_eq!(wires[wires.len() - 1], 10);

    //     wires[2] = 1;
    //     let c = U64Circuit {
    //         gates,
    //         consts,
    //         n_gates, n_in: wires.len(),
    //         n_in,
    //     };
    //     eval64(c, wires);
    //     assert_eq!(wires[wires.len() - 1], 42)
    // }

    // #[test]
    // fn eval_select_xor_or() {
    //     let x = OP_MAX + 1;
    //     let y = OP_MAX + 2;
    //     let i = OP_MAX + 3;
    //     // x + y - [2*x*y, x*y][i]
    //     // i.e. if i == 0 then compute x xor y
    //     //      if i == 1 then compute x or y
    //     let gates = &[
    //         OP_ADD,
    //         x,
    //         y,
    //         OP_MUL,
    //         x,
    //         y,
    //         OP_MUL_CONST,
    //         x,
    //         i + 2,
    //         OP_SELECT,
    //         i,
    //         i + 3,
    //         i + 2,
    //         OP_SUB,
    //         i + 1,
    //         i + 4,
    //     ];
    //     let n_gates = 5;
    //     let n_in = 3;
    //     let consts = &[2];

    //     // test xor
    //     let wires = &mut [0, 0, 0, 0, 0, 0, 0, 0];
    //     eval64(
    //         U64Circuit {
    //             gates,
    //             consts,
    //             n_gates, n_in: wires.len(),
    //             n_in,
    //         },
    //         wires,
    //     );
    //     assert_eq!(wires[wires.len() - 1], 0);

    //     let wires = &mut [1, 0, 0, 0, 0, 0, 0, 0];
    //     eval64(
    //         U64Circuit {
    //             gates,
    //             consts,
    //             n_gates, n_in: wires.len(),
    //             n_in,
    //         },
    //         wires,
    //     );
    //     assert_eq!(wires[wires.len() - 1], 1);

    //     let wires = &mut [0, 1, 0, 0, 0, 0, 0, 0];
    //     eval64(
    //         U64Circuit {
    //             gates,
    //             consts,
    //             n_gates, n_in: wires.len(),
    //             n_in,
    //         },
    //         wires,
    //     );
    //     assert_eq!(wires[wires.len() - 1], 1);

    //     let wires = &mut [1, 1, 0, 0, 0, 0, 0, 0];
    //     eval64(
    //         U64Circuit {
    //             gates,
    //             consts,
    //             n_gates, n_in: wires.len(),
    //             n_in,
    //         },
    //         wires,
    //     );
    //     assert_eq!(wires[wires.len() - 1], 0);

    //     // test or
    //     let wires = &mut [0, 0, 1, 0, 0, 0, 0, 0];
    //     eval64(
    //         U64Circuit {
    //             gates,
    //             consts,
    //             n_gates, n_in: wires.len(),
    //             n_in,
    //         },
    //         wires,
    //     );
    //     assert_eq!(wires[wires.len() - 1], 0);

    //     let wires = &mut [1, 0, 1, 0, 0, 0, 0, 0];
    //     eval64(
    //         U64Circuit {
    //             gates,
    //             consts,
    //             n_gates, n_in: wires.len(),
    //             n_in,
    //         },
    //         wires,
    //     );
    //     assert_eq!(wires[wires.len() - 1], 1);

    //     let wires = &mut [0, 1, 1, 0, 0, 0, 0, 0];
    //     eval64(
    //         U64Circuit {
    //             gates,
    //             consts,
    //             n_gates, n_in: wires.len(),
    //             n_in,
    //         },
    //         wires,
    //     );
    //     assert_eq!(wires[wires.len() - 1], 1);

    //     let wires = &mut [1, 1, 1, 0, 0, 0, 0, 0];
    //     eval64(
    //         U64Circuit {
    //             gates,
    //             consts,
    //             n_gates, n_in: wires.len(),
    //             n_in,
    //         },
    //         wires,
    //     );
    //     assert_eq!(wires[wires.len() - 1], 1);
    // }
}
