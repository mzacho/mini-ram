pub mod builder;
pub mod gadgets;

// Operations for evaluation:
// - binary
const OP_XOR: usize = 0;
const OP_AND: usize = 1;
const OP_AND_CONST: usize = 2;
// - arithmetic
const OP_ADD: usize = 3;
const OP_SUB: usize = 4;
const OP_MUL: usize = 5;
const OP_MUL_CONST: usize = 6;
const OP_SELECT: usize = 7;
// - binary and arithmetic
const OP_CONST: usize = 8;
const OP_OUT: usize = 9;
// Operations for verification:
const OP_CHECK_Z: usize = 10;
const OP_CHECK_EQ: usize = 11;
const OP_CHECK_AND: usize = 12;
const OP_CHECK_ALL_EQ_BUT_ONE: usize = 13;

// arg0 is OP_MAX+1
const OP_MAX: usize = 13;

type Gates = [usize];
type W64<'a> = &'a mut [u64];
type C64<'a> = &'a [u64];

/// Evaluate a circuit of u64 values
pub fn eval64(gates: &Gates, wires: W64, n_gates: usize, n_in: usize, consts: C64) -> Vec<u64> {
    #[cfg(test)]
    {
        dbg!(gates);
        assert_eq!(n_in + n_gates - count_outs(gates), wires.len());
        assert_eq!(n_gates, count_ops(gates));
    }
    let mut out = Vec::new();
    let mut i = 0;
    for j in 0..n_gates {
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
            wires[j - out.len() + n_in] = res;
        }
        dbg!(&wires);
    }
    out
}

#[cfg(test)]
/// Counts number of gates that aren't output or verification gates
fn count_ops(gates: &Gates) -> usize {
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
pub fn count_outs(gates: &Gates) -> usize {
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
    fn eval_binary_add() {
        let x = OP_MAX + 1;
        let y = OP_MAX + 2;
        let gates = &[OP_XOR, x, y];
        let n_gates = 1;
        let n_in = 2;
        let consts = &[];

        let wires = &mut [34, 8, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 42)
    }

    #[test]
    fn eval_binary_add_2() {
        let x = OP_MAX + 1;
        let y = OP_MAX + 2;
        // (x + y) + x
        let gates = &[OP_ADD, x, y, OP_ADD, x, y + 1];
        let n_gates = 2;
        let n_in = 2;
        let consts = &[];

        let wires = &mut [34, 8, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 42 + 34)
    }

    #[test]
    fn eval_variadic_add() {
        let x = OP_MAX + 1;
        let y = OP_MAX + 2;
        let z = OP_MAX + 3;
        // (x + y) + x
        let gates = &[OP_ADD, x, y, z];
        let n_gates = 1;
        let n_in = 3;
        let consts = &[];

        let wires = &mut [34, 8, 10, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 52)
    }

    #[test]
    fn eval_mul_simple() {
        let x = OP_MAX + 1;
        let y = OP_MAX + 2;
        // ((x * y) + x) * y
        let gates = &[OP_MUL, x, y, OP_ADD, y + 1, x, OP_MUL, y + 2, y];
        let n_gates = 3;
        let n_in = 2;
        let consts = &[];

        let wires = &mut [2, 5, 0, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], ((2 * 5) + 2) * 5)
    }

    #[test]
    fn eval_mul_const() {
        let x = OP_MAX + 1;
        let c = OP_MAX + 1;
        // (10 + x) * 20
        let gates = &[OP_CONST, c, OP_ADD, x, x + 1, OP_MUL_CONST, c + 1, x + 2];
        let n_gates = 3;
        let n_in = 1;
        let consts = &[10, 20];

        let wires = &mut [7, 0, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], (10 + 7) * 20)
    }

    #[test]
    fn eval_select() {
        let x = OP_MAX + 1;
        let y = OP_MAX + 2;
        let i = OP_MAX + 3;
        // [x, y][i]
        let gates = &[OP_SELECT, i, x, y];
        let n_gates = 1;
        let n_in = 3;
        let consts = &[];

        let wires = &mut [10, 42, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 10);

        wires[2] = 1;
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 42)
    }

    #[test]
    fn eval_select_xor_or() {
        let x = OP_MAX + 1;
        let y = OP_MAX + 2;
        let i = OP_MAX + 3;
        // x + y - [2*x*y, x*y][i]
        // i.e. if i == 0 then compute x xor y
        //      if i == 1 then compute x or y
        let gates = &[
            OP_ADD,
            x,
            y,
            OP_MUL,
            x,
            y,
            OP_MUL_CONST,
            x,
            i + 2,
            OP_SELECT,
            i,
            i + 3,
            i + 2,
            OP_SUB,
            i + 1,
            i + 4,
        ];
        let n_gates = 5;
        let n_in = 3;
        let consts = &[2];

        // test xor
        let wires = &mut [0, 0, 0, 0, 0, 0, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 0);

        let wires = &mut [1, 0, 0, 0, 0, 0, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 1);

        let wires = &mut [0, 1, 0, 0, 0, 0, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 1);

        let wires = &mut [1, 1, 0, 0, 0, 0, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 0);

        // test or
        let wires = &mut [0, 0, 1, 0, 0, 0, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 0);

        let wires = &mut [1, 0, 1, 0, 0, 0, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 1);

        let wires = &mut [0, 1, 1, 0, 0, 0, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 1);

        let wires = &mut [1, 1, 1, 0, 0, 0, 0, 0];
        eval64(gates, wires, n_gates, n_in, consts);
        assert_eq!(wires[wires.len() - 1], 1);
    }
}
