use crate::miniram::interpreter::*;
use crate::miniram::lang::reg::*;
use crate::miniram::lang::*;

use utils::circuit::{
    builder::{self, Builder},
    gadgets, ARG0,
};

use super::encode::encode;

/// Encodes args as a witness for the correct execution of the
/// MiniRAM program prog (i.e a 0 evaluation).
///
/// The witness consists of the local state of program execution,
/// i.e a Vec<LocalState> that is as long as the time bound t
pub fn encode_witness(prog: &Prog, args: Vec<Word>, t: usize) -> Res<Vec<u64>> {
    let (res, mut lsts) = interpret(prog, args, t)?;
    assert_eq!(res, 0);
    if lsts.len() < t {
        dbg!(&lsts);
        // assume program runs for at least one step
        let last_st = *lsts.last().unwrap();
        for _ in lsts.len()..t {
            lsts.push(last_st);
        }
    }
    Ok(convert_localstates(lsts))
}

fn convert_localstates(lsts: Vec<LocalState>) -> Vec<u64> {
    lsts.iter().flatten().map(|v| u64::from(*v)).collect()
}

/// Number of circuit elements (u64) in LocalState
const SIZE_LOCAL_ST: usize = N_REG;

/// Generates a circuit for verifying the existence of an input
/// (witness), that will make the program return 0 within time bound
/// t.
///
/// Circuit inputs: [LocalState; t]
///
/// todo: Currently the program is hardcoded into the circuit as a
/// constant. Change to Von Neumann type architecture.
pub fn generate_circuit(prog: &Prog, t: usize) -> builder::Res<u64> {
    let n_in = t * SIZE_LOCAL_ST;
    let mut b = builder::Builder::new(n_in);
    let mut outputs = vec![];

    // hard-code program
    let p = encode(prog);
    let l = p.len();
    for instr in p {
        let _ = b.push_const(instr);
        // id of constant gate can be ignored, as the gates are the
        // first to be added to the circuit. We just use the id of
        // the i'th instruction as i when needed.
    }

    // compose transition circuit t times
    for i in 0..t {
        let mut o = transition_circuit(&mut b, i, l, n_in);
        outputs.append(&mut o);
    }
    b.build(&outputs)
}

/// Input:
/// - b: builder with the source code as constants
/// - i: iteration count (0 <= i < time bound t)
/// - l: number of lines of source code
///
/// Output: ids of output nodes
fn transition_circuit(
    b: &mut builder::Builder<u64>,
    i: usize,
    l: usize,
    n_in: usize,
) -> Vec<usize> {
    // register offset into local state
    let k0 = i * SIZE_LOCAL_ST;
    let k1 = (i + 1) * SIZE_LOCAL_ST;
    let k2 = (i + 2) * SIZE_LOCAL_ST;
    // fetch instruction
    b.offset_arg0();
    let pc = b.select_range(k0 + usize::from(PC), 0, n_in, 1);
    let instr = b.const_(pc);
    // decode instruction
    let (op, dst, arg0, arg1) = gadgets::decode_instr64(b, instr);
    // get value of registers
    #[rustfmt::skip]
    let _dst_in  = b.select_range(dst , k0 + ARG0, k1 + ARG0, 1);
    let arg0_in = b.select_range(arg0, k0 + ARG0, k1 + ARG0, 1);
    let arg1_in = b.select_range(arg1, k0 + ARG0, k1 + ARG0, 1);

    #[rustfmt::skip]
    let dst_out  = b.select_range(dst , k1 + ARG0, k2 + ARG0, 1);
    let _arg0_out = b.select_range(arg0, k1 + ARG0, k2 + ARG0, 1);
    let _arg1_out = b.select_range(arg1, k1 + ARG0, k2 + ARG0, 1);

    let res = alu(b, op, arg1_in, arg0_in, dst_out);

    // Output res-dst_out (should be zero)
    let check_alu = b.sub(res, dst_out);

    // Check all in/ out registers are consistent but dst
    let regs = (k0 + ARG0..k1 + ARG0).zip(k1 + ARG0..k2 + ARG0);
    let regs = &regs.collect::<Vec<(usize, usize)>>();
    b.check_all_eq_but_one(dst, regs);

    vec![check_alu]
}

/// Input:
///   - op: opcode of instruction
///   - arg0: value of arg0
///   - arg1: vaule of arg1
///   - dst_out: value of destination register. This should be equal
///   to the output of the alu
///
/// Output:
///   - Value of applying op to arg0, arg1 OR dst_out of op is a
///   memory operation (which are checked by a seperate memory
///   consistency circuit)
///
/// todo? Vector of outputs of the circuit, which should all be zero
fn alu(b: &mut Builder<u64>, op: usize, arg1_in: usize, arg0_in: usize, dst_out: usize) -> usize {
    // Compute each possible operation of the architecture in order
    // of the encoding of opcodes. Then select the correct value
    // using the opcode.
    let a0 = b.add(&[arg0_in, arg1_in]); // add
    let a1 = b.sub(arg0_in, arg1_in); // sub
    let a2 = arg1_in; // mov register
    let a3 = arg1_in; // mov constant
    let a4 = dst_out; // ldr
    let a5 = dst_out; // str
    let a6 = arg1_in; // b
    let a7 = arg1_in; // b z (TODO)
    let a8 = arg1_in; // ret register
    let a9 = arg1_in; // ret constant

    let ids = &vec![a0, a1, a2, a3, a4, a5, a6, a7, a8, a9];
    b.select(op, ids)
}

#[cfg(test)]
mod test {
    use utils::circuit::eval64;

    use crate::miniram::lang::Prog;
    use crate::miniram::lang::Word;
    use crate::miniram::programs::*;

    use super::{encode_witness, generate_circuit};

    #[test]
    fn test_encode_witness() {
        let p = &mul_eq();
        let args = vec![2, 2, 4];
        assert!(encode_witness(p, args, 20).is_ok());
    }

    #[test]
    fn test_generate_circuit() {
        let t = 20;
        let p = &mul_eq();
        let _ = generate_circuit(p, t);
    }

    #[test]
    fn test_const_0() {
        let prog = &const_0();
        let args = vec![];
        let time_bound = 1;
        let res = convert_and_eval(prog, args, time_bound);
        for v in res {
            assert_eq!(v, 0)
        }
    }

    #[test]
    fn test_mov0_const_0() {
        let prog = &mov0_ret();
        let args = vec![];
        let time_bound = 2;
        let res = convert_and_eval(prog, args, time_bound);
        for v in res {
            assert_eq!(v, 0)
        }
    }

    #[test]
    fn test_mov42_ret() {
        let prog = &mov42_ret();
        let args = vec![];
        let time_bound = 2;
        let res = convert_and_eval(prog, args, time_bound);

        // assert ouput is not all zeros
        assert_ne!(res.iter().filter(|v| **v != 0).count(), 0);
    }

    #[test]
    fn test_mov_mov_sub_ret() {
        let prog = &mov_mov_sub_ret();
        let args = vec![];
        let time_bound = 4;
        let res = convert_and_eval(prog, args, time_bound);
        for v in res {
            assert_eq!(v, 0)
        }
    }

    #[test]
    fn convert_and_eval_mul_eq() {
        let prog = &mul_eq();
        let args = vec![2, 2, 4];
        let time_bound = 1000;
        let res = convert_and_eval(prog, args, time_bound);
        for v in res {
            assert_eq!(v, 0)
        }
    }

    fn convert_and_eval(p: &Prog, args: Vec<Word>, t: usize) -> Vec<u64> {
        let c = &generate_circuit(p, t);
        let w = encode_witness(p, args, t).unwrap();
        dbg!(&w);
        eval64(c, w)
    }
}
