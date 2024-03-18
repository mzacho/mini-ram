use std::cmp::Ordering;

use crate::miniram::interpreter::*;
use crate::miniram::lang::reg::*;
use crate::miniram::lang::*;

use utils::permutation;

use utils::circuit::{
    builder::{self, Builder},
    gadgets, ARG0,
};

use super::encode::encode;

type Witness = Vec<u64>;

/// Encodes args as a witness for the correct execution of the
/// MiniRAM program prog (i.e a 0 evaluation).
///
/// The witness consists of the local state of program execution,
/// i.e a Vec<LocalState> that is as long as the time bound t
pub fn encode_witness(prog: &Prog, args: Vec<Word>, t: usize) -> Res<Witness> {
    let (res, mut lsts) = interpret(prog, args, t)?;
    assert_eq!(res, 0);
    if lsts.len() < t {
        // assume program runs for at least one step
        let last_st = *lsts.last().unwrap();
        for _ in lsts.len()..t {
            lsts.push(last_st);
        }
    }
    Ok(convert_localstates(lsts))
}

/// Convert the local states to the witness, which is a vector W of
/// values from the circuit layed out as
///
///   W = S1, S2, ..., St, S'1, S'2, ..., S't, c1, ..., ck
///
/// where Si represents the i'th local state with the value of the CPU
///
///   Si = pc, r1, ..., r15, Z
///
/// and where the S'1, S'2, ..., S't = sort(S1, S2, ..., St)
/// according to memory accesses with ties broken by timestamp, and
/// where c1, ..., ck are the configuration of the AS-Waksman network
/// that performs the sorting permutation.
fn convert_localstates(lsts: Vec<LocalStateAug>) -> Witness {
    let mut res = vec![];
    // Push S1, S2, ..., St
    for s in lsts.iter() {
        for val in s.st.0 {
            res.push(u64::from(val))
        }
        for flag in s.st.1 {
            res.push(u64::from(flag))
        }
    }

    // Compute the permutation that sorts the states according to
    // memory accesses.
    let _p = permutation::sort(&lsts);

    // // Push S1', ...
    // for s in p.apply_slice(&lsts) {
    //     res.push(s.step);

    // }
    // Compute
    // // Push S'1, S'2, ..., S't
    // // Since [T].sort_by() performs a stable sorting, and lsts is
    // // already sorted by timestamp, then it's enough to sort by
    // // memory location.
    // lsts.into_iter()
    //     .enumerate()
    //     .collect::<Vec<(usize,LocalStateAug)>>()
    //     .sort_by(cmp_mem_loc());

    // todo: compute c1, ..., ck configurations of Benes network
    // mock for now?
    res
}

/// Number of circuit elements (u64) in LocalState
const SIZE_LOCAL_ST: usize = N_REG + N_CFL;

/// Generates a circuit for verifying the existence of an input
/// (witness), that will make the program return 0 within time bound
/// t.
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
        // id of constant gate can be ignored: As the gates are the
        // first to be added to the circuit, we just use i as the
        // id of the i'th (zero indexed) instruction when needed.
    }

    // push constants
    let zero = b.push_const(0);
    let one = b.push_const(1);
    let zero = b.const_(zero);
    let one = b.const_(one);

    // compose transition circuit t times, where the first
    // iteration uses initial values (zeros) for all registers
    outputs.append(&mut first_transition_circuit(&mut b, zero, one));

    for i in 1..t {
        let mut o = transition_circuit(&mut b, i - 1, l, one);
        outputs.append(&mut o);
    }
    b.build(&outputs)
}

fn first_transition_circuit(b: &mut builder::Builder<u64>, zero: usize, one: usize) -> Vec<usize> {
    // Fetch first instruction
    let instr = b.const_(ARG0);

    // Decode it
    let (op, dst, _, _, arg1_word) = gadgets::decode_instr64(b, instr);

    // Get output value of dst register, used for mocking the result
    // of memory operations.
    let dst_out = b.select_range(dst, ARG0, ARG0 + N_REG, 1);
    let cfl_out = ARG0 + N_REG;

    // Compute the result of the ALU at this transition step.
    //
    // Pass zero for the value of (registers referenced by) arg0,
    // arg1 as well as the conditional Z flag. (todo: what if arg0
    // == pc?)
    let alu_in = AluIn {
        op,
        arg0: zero,
        arg1: zero,
        arg1_word,
        cfl_z: zero,
        pc: zero,
    };
    let (res, z) = alu(b, alu_in, dst_out, one);

    // Output res-dst_out and z-cfl_out (both should be zero)
    let check_alu = b.sub(res, dst_out);
    let check_cfl = b.sub(z, cfl_out);

    // Increment pc
    let pc = one;

    // Check all in/ out registers except dst are consistent
    let mut regs = vec![(pc, ARG0)];
    for i in 1..N_REG {
        regs.push((zero, ARG0 + i))
    }
    b.check_all_eq_but_one(dst, &regs);

    vec![check_alu, check_cfl]
}

/// Input:
/// - b: builder with the source code as constants
/// - i: iteration count (0 <= i < time bound t)
/// - l: number of lines of source code
///
/// Output: ids of output nodes
fn transition_circuit(b: &mut builder::Builder<u64>, i: usize, l: usize, one: usize) -> Vec<usize> {
    let k0 = i * SIZE_LOCAL_ST + ARG0;
    let k1 = (i + 1) * SIZE_LOCAL_ST + ARG0;
    let k2 = (i + 2) * SIZE_LOCAL_ST + ARG0;

    // Fetch instruction
    let pc = k0 + usize::from(PC);
    let instr = b.select_const_range(pc, ARG0, ARG0 + l, 1);

    // Decode instruction
    let (op, dst, arg0, arg1, arg1_word) = gadgets::decode_instr64(b, instr);

    // Get value of registers refered to by dst, arg0 and arg1 as
    // well as value of Z flag
    let arg0 = b.select_range(arg0, k0, k1 - N_CFL, 1);
    let arg1 = b.select_range(arg1, k0, k1 - N_CFL, 1);
    let cfl_z = k1 - N_CFL;

    let dst_out = b.select_range(dst, k1, k2 - N_CFL, 1);
    let cfl_z_out = k2 - N_CFL;

    let alu_in = AluIn {
        op,
        arg0,
        arg1,
        arg1_word,
        cfl_z,
        pc,
    };
    let (res, z) = alu(b, alu_in, dst_out, one);

    // Ouput res-dst_out and z-cfl_out - both should be zero if the
    // witness to satisfies the circuit.
    let check_alu = b.sub(res, dst_out);
    let check_cfl = b.sub(z, cfl_z_out);

    // increment pc
    let pc = b.add(&[k0 + usize::from(PC), one]);

    // Check all in/ out registers except dst are consistent
    let mut regs = vec![(pc, k1)];
    for i in 1..N_REG {
        regs.push((k0 + i, k1 + i))
    }
    b.check_all_eq_but_one(dst, &regs);

    vec![check_alu, check_cfl]
}

struct AluIn {
    op: usize,
    arg0: usize,
    arg1: usize,
    arg1_word: usize,
    cfl_z: usize,
    pc: usize,
}

/// Input:
///   - op: opcode of instruction
///   - arg0: value of arg0 (as a 4 bit register).
///   - arg1: value of arg1 (as a 4 bit register).
///   - arg1_w: value of arg1 (as a 32 bit word).
///   - cfl_z: value of the (isZero) Z conditional flag.
///   - dst_out: value of destination register. This should be equal
///   to the output of the alu. This is used for mocking the result
///   of memory operations.
///
/// Output pair (res, z) where:
///   - res: Value of applying op to arg0, arg1 OR dst_out of op is a
///   memory operation (which are checked by a seperate memory
///   consistency circuit)
///   - z: is the boolean value of the Z flag.
///
/// todo? Vector of outputs of the circuit, which should all be zero
fn alu(b: &mut Builder<u64>, in_: AluIn, dst_out: usize, one: usize) -> (usize, usize) {
    // Compute each possible operation of the architecture in order
    // of the encoding of opcodes. Then select the correct value
    // using the opcode.
    let a1 = dst_out; // ldr
    let a0 = dst_out; // str

    let a2 = b.add(&[in_.arg0, in_.arg1]); // add
    let a4 = b.sub(in_.arg0, in_.arg1); // sub
    let a6 = in_.arg1; // mov register
    let a8 = in_.arg1_word; // mov constant
    let a10 = in_.arg1; // b
                        // b z: compute (pc + 1) + cfl_z*(arg1 - (pc + 1))
    let tmp1 = b.add(&[in_.pc, one]);
    let tmp2 = b.sub(in_.arg1, tmp1);
    let tmp3 = b.mul(in_.cfl_z, tmp2);
    let a12 = b.add(&[tmp1, tmp3]);

    let a14 = in_.arg1; // ret register
    let a16 = in_.arg1_word; // ret constant

    // used to trigger a run-time panic if this argument is returned
    // as res. todo: select(in_.op / 2, ids) instead
    let o = usize::MAX;

    let ids = &vec![
        a0, a1, a2, o, a4, o, a6, o, a8, o, a10, o, a12, o, a14, o, a16,
    ];
    let res = b.select(in_.op, ids);

    // Compute the value of the Z flag by destructing res into its
    // bit-decomposition, converting each bit to Z2, OR-ing them
    // all together and negating the ouput.
    let decode_res = b.decode32(res);
    let mut tmp = b.or(decode_res, decode_res + 1); // todo: a2b?
    for i in 2..32 {
        tmp = b.or(tmp, decode_res + i)
    }
    let z = b.xor(&[tmp, one]);
    (res, z)
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
        let t = 100;
        let p = &mul_eq();
        let args = vec![2, 2, 4];
        encode_witness(p, args, t).unwrap();
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
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    #[ignore]
    fn test_long_time_bound() {
        let prog = &const_0();
        let args = vec![];
        let time_bound = 2;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn test_b_skip() {
        let prog = &b_skip();
        let args = vec![];
        let time_bound = 3; // notice: t < len(encode(prog))
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn test_b_z_skip() {
        let prog = &b_z_skip();
        let args = vec![];
        let time_bound = 4; // notice: t < len(encode(prog))
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn test_mov0_const_0() {
        let prog = &mov0_ret();
        let args = vec![];
        let time_bound = 2;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn test_mov_2pow20() {
        let prog = &mov2pow20_ret0();
        let args = vec![];
        let time_bound = 2;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn test_mov42_movr3_ret0() {
        let prog = &mov42_movr3_ret0();
        let args = vec![];
        let time_bound = 4;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    #[ignore]
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
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn convert_and_eval_mul_eq() {
        let prog = &mul_eq();
        let args = vec![2, 2, 4];
        let time_bound = 22;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn convert_and_eval_mul_eq2() {
        let prog = &mul_eq();
        let args = vec![2, 17, 34];
        let time_bound = 22;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    fn convert_and_eval(p: &Prog, args: Vec<Word>, t: usize) -> Vec<u64> {
        let c = &generate_circuit(p, t);
        let w = encode_witness(p, args, t).unwrap();
        dbg!(&w);
        eval64(c, w)
    }
}
