pub mod builder;
pub mod circuits;
pub mod gadgets;

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
// - binary and arithmetic
pub const OP_CONV_B2A: usize = 8;
pub const OP_CONV_A2B: usize = 9;
pub const OP_CONST: usize = 10;
pub const OP_OUT: usize = 11;
// Operations for verification:
pub const OP_CHECK_Z: usize = 12;
pub const OP_CHECK_EQ: usize = 13;
pub const OP_CHECK_AND: usize = 14;
pub const OP_CHECK_ALL_EQ_BUT_ONE: usize = 15;

// arg0 is OP_MAX+1
pub const OP_MAX: usize = 15;

use builder::Res as Circuit;

// pub struct U64Circuit<'a> {
//     pub gates: &'a [usize],
//     pub consts: &'a [u64],
//     pub n_gates: usize,
//     pub n_in: usize,
// }

/// Evaluate a circuit of u64 values
pub fn eval64(c: &Circuit<u64>, mut wires: Vec<u64>) -> Vec<u64> {
    let gates = &c.gates;
    let consts = &c.consts;
    let n_gates = c.n_gates;
    #[cfg(test)]
    assert_eq!(n_gates, count_ops(&gates));
    let mut out = Vec::new();
    let mut i = 0;
    for _ in 0..n_gates {
        let op = gates[i];
        let mut arg = gates[i + 1];
        i = i + 1;
        let mut res = 0;
        dbg!(&op);
        match op {
            // --- binary ops
            OP_XOR => {
                while arg > OP_MAX {
                    res ^= wires[arg - OP_MAX - 1];
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                    arg = gates[i];
                }
            }
            OP_AND => {
                let lhs = wires[arg - OP_MAX - 1];
                let rhs = wires[gates[i + 1] - OP_MAX - 1];
                res = lhs & rhs;
                i += 2;
            }
            OP_AND_CONST => {
                let c = consts[arg - OP_MAX - 1];
                let arg = gates[i + 1];
                res = c & wires[arg - OP_MAX - 1];
                i += 2;
            }
            // --- arithmetic ops
            OP_ADD => {
                while arg > OP_MAX {
                    res += wires[arg - OP_MAX - 1];
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                    arg = gates[i];
                }
            }
            OP_SUB => {
                let lhs = wires[arg - OP_MAX - 1];
                let rhs = wires[gates[i + 1] - OP_MAX - 1];
                res = lhs - rhs;
                i += 2;
            }
            OP_MUL => {
                let lhs = wires[arg - OP_MAX - 1];
                let rhs = wires[gates[i + 1] - OP_MAX - 1];
                res = lhs * rhs;
                i += 2;
            }
            OP_MUL_CONST => {
                let c = consts[arg - OP_MAX - 1];
                let arg = gates[i + 1];
                res = c * wires[arg - OP_MAX - 1];
                i += 2;
            }
            OP_SELECT => {
                let idx = wires[arg - OP_MAX - 1];
                let idx: usize = idx.try_into().ok().unwrap();
                i += idx + 1;
                arg = gates[i];
                res = wires[arg - OP_MAX - 1];
                while arg > OP_MAX {
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                    arg = gates[i];
                }
            }
            // --- mixed ops
            OP_CONV_A2B => {
                // assert x in Z_{2^64} is a bit
                let x = wires[arg - OP_MAX - 1];
                assert!(x < 2);
                // move result
                res = x;
                i += 1;
            }
            OP_CONV_B2A => {
                // assert x in Z_{2^64} is a bit
                let x = wires[arg - OP_MAX - 1];
                // move result
                res = x;
                i += 1;
            }
            OP_CONST => {
                res = consts[arg - OP_MAX - 1];
                i += 1;
            }
            OP_OUT => {
                dbg!(wires[arg - OP_MAX - 1]);
                out.push(wires[arg - OP_MAX - 1]);
                i += 1;
            }
            // --- zk verificatin ops
            OP_CHECK_Z => (),              // noop
            OP_CHECK_EQ => (),             // noop
            OP_CHECK_AND => (),            // noop
            OP_CHECK_ALL_EQ_BUT_ONE => (), // noop
            _ => panic!("invalid operation"),
        }
        if op != OP_OUT {
            dbg!(res);
            // wires[j - out.len() + n_in] = res;
            wires.push(res);
        }
        dbg!(&wires);
    }
    out
}

#[cfg(test)]
/// Counts number of gates that aren't output or verification gates
fn count_ops(gates: &[usize]) -> usize {
    let mut res = 0;
    for i in gates {
        if *i <= OP_MAX {
            res += 1;
        }
    }
    res
}

#[cfg(test)]
/// Counts number of output gates
pub fn count_outs(gates: &[usize]) -> usize {
    let mut res = 0;
    for i in gates {
        if *i == OP_OUT {
            res += 1;
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eval_xor() {
        let x = OP_MAX + 1;
        let y = OP_MAX + 2;
        let gates = vec![OP_XOR, x, y, OP_OUT, y + 1];
        let n_gates = 2;
        let consts = vec![];

        let wires = vec![34, 8];
        let c = &Circuit {
            gates,
            consts,
            n_gates,
            n_out: 1,
        };
        let res = eval64(c, wires);
        assert_eq!(*res.last().unwrap(), 42)
    }

    #[test]
    fn eval_add() {
        let x = OP_MAX + 1;
        let y = OP_MAX + 2;
        // (x + y) + x
        let gates = vec![OP_ADD, x, y, OP_ADD, x, y + 1, OP_OUT, y + 2];
        let n_gates = 3;
        let consts = vec![];

        let wires = vec![34, 8];
        let c = &Circuit {
            gates,
            consts,
            n_gates,
            n_out: 1,
        };
        let res = eval64(c, wires);
        assert_eq!(*res.last().unwrap(), 42 + 34)
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
    //         n_gates,
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
    //         n_gates,
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
    //         n_gates,
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
    //         n_gates,
    //         n_in,
    //     };
    //     eval64(c, wires);
    //     assert_eq!(wires[wires.len() - 1], 10);

    //     wires[2] = 1;
    //     let c = U64Circuit {
    //         gates,
    //         consts,
    //         n_gates,
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
    //             n_gates,
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
    //             n_gates,
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
    //             n_gates,
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
    //             n_gates,
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
    //             n_gates,
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
    //             n_gates,
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
    //             n_gates,
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
    //             n_gates,
    //             n_in,
    //         },
    //         wires,
    //     );
    //     assert_eq!(wires[wires.len() - 1], 1);
    // }
}
