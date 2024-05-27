use crate::miniram::interpreter::*;
use crate::miniram::lang::reg::*;
use crate::miniram::lang::*;

use backend::ProofCtx;
use utils::{permutation, waksman};

use utils::circuit::{
    builder::{self, Builder},
    gadgets, ARG0, ARG1,
};

use super::encode::encode;

type Witness = Vec<Word>;

/// Encodes args as a witness for the correct execution of the
/// MiniRAM program prog (i.e a 0 evaluation).
///
/// The witness consists of the local state of program execution,
/// i.e a Vec<LocalState> that is as long as the time bound t
pub fn encode_witness(prog: &Prog, args: Vec<Word>, t: usize, ctx: &mut ProofCtx) -> Res<Witness> {
    ctx.start_time("interpret program");
    let (res, mut lsts) = interpret(prog, args, t)?;
    ctx.stop_time();
    assert_eq!(res, 0);
    if lsts.len() < t {
        // assume program runs for at least one step
        let last_st = *lsts.last().unwrap();
        for _ in lsts.len()..t {
            lsts.push(last_st);
        }
    }
    Ok(convert_localstates(lsts, ctx))
}

/// Convert the local states to the witness, which is a vector W of
/// values from the circuit layed out as
///
///   W = S1, S2, ..., St, c1, ..., ck
///
/// where Si represents the i'th local state with the value of the CPU
///
///   Si = pc, r1, ..., r15, Z
///
/// and where c1, ..., ck is the configuration of the AS-Waksman network
/// that sorts S1, S2, ..., St according to memory accesses with
/// ties broken by timestamp.
///
/// TODO: fix docs of:
///
/// encoded as
///
///   S'j = i, addr_i, val_i, is_load_i
///
///   - i         is the timestamp (step index, i.e the states
///               original index in the trace) of the operation.
///
///   - addr_i    is the address accessed by the opretion, or 0 if the
///               operation was not a memory operation.
///
///   - val_i     is the value read/ written
///
///   - is_load_i is 1 only if the operation was a LDR
fn convert_localstates(lsts: Vec<LocalStateAug>, ctx: &mut ProofCtx) -> Witness {
    let mut res = vec![];
    // Push S1, S2, ..., St
    for s in lsts.iter() {
        for val in s.st.0 {
            // Push value of registers
            res.push(val)
        }
        for flag in s.st.1 {
            // Push value of conditional flags
            res.push(Word::from(flag))
        }
    }
    // Compute the permutation that sorts the states according to
    // memory accesses.
    ctx.start_time("compute sorting permutation");
    let p = permutation::sort(&lsts);
    ctx.stop_time();

    // Push configuration of permutation network
    ctx.start_time("route sorting permutation of witness");
    for c in waksman::route(&p.inverse()) {
        // TODO: Coniguration could maybe be packed? Seems a bit
        // overkill to use a ring element per bit
        res.push(Word::from(c));
    }
    ctx.stop_time();

    // dbg!(&lsts);
    // dbg!(&p.apply_slice(&lsts));
    res
}

/// Number of circuit elements (u32) in one LocalState of the trace
const SIZE_LOCAL_ST: usize = N_REG + N_CFL;

/// Generates a circuit for verifying the existence of an input
/// (witness), that will make the program return 0 within time bound
/// t.
///
/// todo: Currently the program is hardcoded into the circuit as a
/// constant. If time permits, it would be nice to change this to a
/// Von Neumann type architecture.
pub fn generate_circuit(prog: &Prog, time_bound: usize) -> builder::Res<Word> {
    let n_in = time_bound * SIZE_LOCAL_ST + waksman::conf_len(time_bound);

    // id of first permutation network config
    let in_pconf = time_bound * SIZE_LOCAL_ST + ARG0;

    let mut b = builder::Builder::new(n_in);
    let mut outputs = vec![];

    // hard-code program
    let p = encode(prog);
    let n_instr = p.len();
    for instr in &p {
        let lo = *instr as u32;
        let hi = (*instr >> 32) as u32;
        let _ = b.push_const(hi);
        let _ = b.push_const(lo);
        // id of constant gate can be ignored: As the gates are the
        // first to be added to the circuit, we just use 2i || 2i+1
        // as the (zero indexed) instruction.
    }

    // push constants
    let id_zero = b.push_const(0);
    let id_one = b.push_const(1);
    let id_two = b.push_const(2);
    let zero = b.const_(id_zero);
    let one = b.const_(id_one);

    // input of permutation networks
    let mut perm_in_0 = vec![];
    let mut perm_in_1 = vec![];
    let mut perm_in_2 = vec![];
    let mut perm_in_3 = vec![];
    let mut ctr = zero;

    // compose transition circuit t times, where the first
    // iteration uses initial values (zeros) for all registers
    let (mut o, adr, v, is_load) = fst_trans_circ(&mut b, zero, one);
    outputs.append(&mut o);
    perm_in_0.push(ctr);
    perm_in_1.push(adr);
    perm_in_2.push(v);
    perm_in_3.push(is_load);

    for step in 1..time_bound {
        // b.debug(step);
        // if step < p.len() {
        //     b.debug(p[step].try_into().unwrap());
        // }
        let (mut o, adr, v, is_load) = trans_circ(&mut b, step - 1, n_instr, zero, one, id_two);
        ctr = b.add(&[ctr, one]);
        outputs.append(&mut o);
        perm_in_0.push(ctr);
        perm_in_1.push(adr);
        perm_in_2.push(v);
        perm_in_3.push(is_load);
    }
    // Add the permutation networks
    let conf = &(in_pconf..n_in + ARG0).collect::<Vec<_>>();

    let (po0, _) = gadgets::waksman(&mut b, &perm_in_0, conf, one);
    let (po1, _) = gadgets::waksman(&mut b, &perm_in_1, conf, one);
    let (po2, _) = gadgets::waksman(&mut b, &perm_in_2, conf, one);
    let (po3, _) = gadgets::waksman(&mut b, &perm_in_3, conf, one);

    for step in 1..time_bound {
        let x = (po0[step - 1], po1[step - 1], po2[step - 1], po3[step - 1]);
        let y = (po0[step], po1[step], po2[step], po3[step]);
        let mut o = mem_consistency_circ(&mut b, x, y, one);
        outputs.append(&mut o);
    }

    b.build(&outputs)
}

/// Inputs:
///   - x: (t, adr, v, is_load) for i'th state in the sorted trace
///   - y: (t, adr, v, is_load) for i+1'th state in the sorted trace
///
/// Check that elements are sorted:
///
///   adr1 < adr2 OR (adr1 = adr2 AND t1 < t2)
///
/// Check memory accesses are sequentially consistent:
///
///   (adr1 = adr2 AND is_load2) => v1 = v2
///
/// We will allow reading arbitrary values from unitialized
/// memory. This is also convenient for reading program arguments,
/// that the memory is initialized with.
fn mem_consistency_circ(
    b: &mut Builder<Word>,
    x: (usize, usize, usize, usize),
    y: (usize, usize, usize, usize),
    one: usize,
) -> Vec<usize> {
    let (t1, adr1, v1) = (x.0, x.1, x.2);
    let (t2, adr2, v2, is_load2) = (y.0, y.1, y.2, y.3);

    // The addresses are already decoded previously in the instruction
    // fetching - don't think its possible to reuse those bits here
    // though, since the permutation network is 'in between'.

    let adr1_bits = b.decode32(adr1);
    let adr2_bits = b.decode32(adr2);
    let t1_bits = b.decode32(t1);
    let t2_bits = b.decode32(t2);

    let xs = &(adr1_bits..adr1_bits + 32).collect::<Vec<_>>();
    let ys = &(adr2_bits..adr2_bits + 32).collect::<Vec<_>>();
    let (adr_lt, adr_eq) = gadgets::word_comparator(b, xs, ys, one);

    let xs = &(t1_bits..t1_bits + 32).collect::<Vec<_>>();
    let ys = &(t2_bits..t2_bits + 32).collect::<Vec<_>>();
    let (t_lt, _) = gadgets::word_comparator(b, xs, ys, one);

    let tmp = b.and_bits(adr_eq, t_lt);
    let tmp = b.or_bits(adr_lt, tmp);
    let check_sorted = b.xor_bits(&[tmp, one]);

    let tmp = b.sub(v1, v2);
    let tmp = b.mul(is_load2, tmp);
    let check_mem = b.mul(adr_eq, tmp);

    vec![check_sorted, check_mem]
}

/// Returns (outputs, addr, val, is_load) where
///
///   - outputs is the ids of all output nodes of the circuit
///
///   - addr    is the address of the current memory instruction (or 0,
///             if the instruction is not a LDR/ STR)
///
///   - val     is the value read/ written by the current memory
///             instruction (or 0, if the instruction is not a LDR/
///             STR)
///
///  - is_load  is 1 only if the current instruction was a LDR
fn fst_trans_circ(
    b: &mut builder::Builder<Word>,
    zero: usize,
    one: usize,
) -> (Vec<usize>, usize, usize, usize) {
    // Fetch first instruction
    let instr_hi = b.const_(ARG0);
    let instr_lo = b.const_(ARG1);

    // Decode it
    let (op, dst, _, is_mem, is_load, is_ret, field4) = gadgets::decode_hi_instr32(b, instr_hi);
    let (_, arg1_word) = gadgets::decode_lo_instr32(b, instr_lo);

    let is_str = b.xor_bits(&[is_mem, is_load]);

    // Get output value of dst register, used for mocking the result
    // of memory operations for ALU sub-circuit, and getting the
    // value of LDR operations for the memory consistency sub-circuit.
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
        field4,
    };
    let (res, z) = alu(b, alu_in, dst_out, zero, one);

    // Output res-dst_out and z-cfl_out (both should be zero)
    let check_alu = b.sub(res, dst_out);
    let check_cfl = b.sub(z, cfl_out);

    // Conditional flag isn't set when op is STR
    let tmp = b.xor_bits(&[is_str, one]);
    let check_cfl = b.mul(tmp, check_cfl);

    // Increment pc if op is not RET
    let tmp = b.sub(one, is_ret);
    let pc = b.mul(tmp, one);

    // Check all in/ out registers except dst are consistent
    let mut regs = vec![(pc, ARG0)];
    for i in 1..N_REG {
        regs.push((zero, ARG0 + i))
    }
    b.check_all_eq_but_one(dst, &regs);

    // Compute the memory address, if instruction is a LDR/STR.
    // Since the address is loaded from a register, which are all
    // initialized to 0, and this is the first instructions, then
    // the any LDR/ STR must use zero as the address.
    //
    // We use address 0 for instructions that don't use memory (see
    // trans_circ), thus mem_addr is 1 (for address 0) if
    // instruction is a LDR/STR.
    let mem_addr = b.mul(is_mem, one);

    // Compute the the value read/ written for LDR/STR instructions.
    // Similar to mem_addr this is zero for the first instruction
    let mem_val = zero;

    (vec![check_alu, check_cfl], mem_addr, mem_val, is_load)
}

/// Input:
/// - b: builder with the source code as constants
/// - i: iteration count (0 <= i < time bound t)
/// - l: number of lines of source code
/// - one: index of constant one
/// - two: index of constant two
///
/// Returns: the same as first_transition_circuit
fn trans_circ(
    b: &mut builder::Builder<Word>,
    i: usize,
    l: usize,
    zero: usize,
    one: usize,
    id_two: usize,
) -> (Vec<usize>, usize, usize, usize) {
    let k0 = i * SIZE_LOCAL_ST + ARG0;
    let k1 = (i + 1) * SIZE_LOCAL_ST + ARG0;
    let k2 = (i + 2) * SIZE_LOCAL_ST + ARG0;

    // for (i, w) in (k0..k1).enumerate() {
    //     b.debug(i);
    //     b.debug_wire(w);
    // }

    // Fetch instruction
    let pc = k0 + usize::from(PC);
    let pc_hi = b.mul_const(id_two, pc);
    let pc_lo = b.add(&[one, pc_hi]);
    let instr_hi = b.select_const_range(pc_hi, ARG0, ARG0 + 2 * l, 1);
    let instr_lo = b.select_const_range(pc_lo, ARG0, ARG0 + 2 * l + 1, 1);

    // Decode instruction
    let (op, dst, arg0, is_mem, is_load, is_ret, field4) = gadgets::decode_hi_instr32(b, instr_hi);
    let (arg1, arg1_word) = gadgets::decode_lo_instr32(b, instr_lo);

    let is_str = b.xor_bits(&[is_mem, is_load]);

    // Get value of registers refered to by dst, arg0 and arg1 as
    // well as value of Z flag
    let dst_in = b.select_range(dst, k0, k1 - N_CFL, 1);
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
        field4,
    };
    let (res, z) = alu(b, alu_in, dst_out, zero, one);

    // Ouput res-dst_out and z-cfl_out - both should be zero if the
    // witness satisfies the circuit.
    let check_alu = b.sub(res, dst_out);
    let check_cfl = b.sub(z, cfl_z_out);

    // Conditional flag isn't set when op is STR
    let tmp = b.xor_bits(&[is_str, one]);
    let check_cfl = b.mul(tmp, check_cfl);

    // Increment pc if op is not ret
    let tmp = b.sub(one, is_ret);
    let pc = b.add(&[k0 + usize::from(PC), tmp]);

    // Check all in/ out registers except dst are consistent
    let mut regs = vec![(pc, k1)];
    for i in 1..N_REG {
        regs.push((k0 + i, k1 + i))
    }
    b.check_all_eq_but_one(dst, &regs);

    // Compute the memory address, if instruction is a LDR/STR.
    // Set mem_addr to 0 for instructions that don't use memory,
    // and add 1 to the actual address. This makes the highest
    // address unavailable to programs.
    let tmp = b.add(&[arg0, one]);
    let mem_addr = b.mul(is_mem, tmp);

    // Compute the the value read/ written for LDR/STR instructions.
    // The value is only used if is_load is set, so garbage is sent
    // for instructions that don't access memory.
    let tmp1 = b.mul(is_load, dst_out);
    let tmp2 = b.sub(one, is_load);
    let tmp3 = b.mul(tmp2, dst_in);
    let mem_val = b.add(&[tmp1, tmp3]);

    (vec![check_alu, check_cfl], mem_addr, mem_val, is_load)
}

struct AluIn {
    op: usize,
    arg0: usize,
    arg1: usize,
    arg1_word: usize,
    cfl_z: usize,
    pc: usize,
    field4: usize,
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
fn alu(
    b: &mut Builder<Word>,
    in_: AluIn,
    dst_out: usize,
    zero: usize,
    one: usize,
) -> (usize, usize) {
    // Compute each possible operation of the architecture in order
    // of the encoding of opcodes. Then select the correct value
    // using the opcode.
    let a2 = dst_out; // str
    let a3 = dst_out; // ldr

    let a4 = b.add(&[in_.arg0, in_.arg1]); // add
    let a8 = b.sub(in_.arg0, in_.arg1); // sub
    let a12 = in_.arg1; // mov register
    let a16 = in_.arg1_word; // mov constant
    let a20 = in_.arg1; // b
                        // b z: compute (pc + 1) + cfl_z*(arg1 - (pc + 1))
    let tmp1 = b.add(&[in_.pc, one]);
    let tmp2 = b.sub(in_.arg1, tmp1);
    let tmp3 = b.mul(in_.cfl_z, tmp2);
    let a24 = b.add(&[tmp1, tmp3]);

    let a32 = in_.arg1; // ret register
    let a36 = in_.arg1_word; // ret constant

    let arg0bits = b.decode32(in_.arg0);
    let arg1bits = b.decode32(in_.arg1);
    // --- bitwise operations
    // and/ xor
    let a0 = gadgets::bitwise_and_u32_bits(b, arg0bits, arg1bits);
    let a28 = gadgets::bitwise_xor_u32_bits(b, arg0bits, arg1bits);
    // shr
    let shrs = (1..32).map(|i| {
        // encode the bits 0 0 0 ... x0 x1 ... x(32-i)
        let mut xs = (arg0bits + i..arg0bits + 32).chain((0..i).map(|_| zero));
        b.encode32_range(core::array::from_fn(|_| xs.next().unwrap()))
    });
    // shr 0 arg0, shr 1 arg0, ..., shr 31 arg0
    let shrs = &[in_.arg0].into_iter().chain(shrs).collect::<Vec<_>>();
    // shr field4 arg0
    let a64 = b.select(in_.field4, shrs);

    // rotr
    let rotrs = (1..32).map(|i| {
        // encode the bits x(32-i+1) x(32-i+2) ... x0 x1 ... x(32-i)
        let mut xs = (arg0bits + i..arg0bits + 32).chain(arg0bits..arg0bits + i);
        b.encode32_range(core::array::from_fn(|_| xs.next().unwrap()))
    });
    // rotr 0 arg0, rotr 1 arg0, ..., rotr 31 arg0
    let rotrs = &[in_.arg0].into_iter().chain(rotrs).collect::<Vec<_>>();
    // rotr field4 arg0
    let a68 = b.select(in_.field4, rotrs);

    // print
    let a72 = dst_out;

    // todo: select(in_.op / 4, ids) instead
    let mut ids = [ARG0; 73];
    ids[0] = a0;
    ids[2] = a2;
    ids[3] = a3;
    ids[4] = a4;
    ids[8] = a8;
    ids[12] = a12;
    ids[16] = a16;
    ids[20] = a20;
    ids[24] = a24;
    ids[28] = a28;
    ids[32] = a32;
    ids[36] = a36;
    ids[64] = a64;
    ids[68] = a68;
    ids[72] = a72;

    let res = b.select(in_.op, &ids);

    // Compute the value of the Z flag by destructing res into its
    // bit-decomposition, converting each bit to Z2, OR-ing them
    // all together and negating the ouput.
    let decode_res = b.decode32(res);
    let mut tmp = b.or_bits(decode_res, decode_res + 1); // todo: a2b?
    for i in 2..32 {
        tmp = b.or_bits(tmp, decode_res + i)
    }
    let z = b.xor_bits(&[tmp, one]);
    (res, z)
}

#[cfg(test)]
mod test {
    use backend::ProofCtx;
    use utils::circuit::eval32;

    use crate::miniram::lang::Prog;
    use crate::miniram::lang::Word;
    use crate::miniram::programs;
    use crate::miniram::programs::*;

    use super::{encode_witness, generate_circuit};

    #[test]
    fn encod_witness() {
        let t = 100;
        let p = &mul_eq();
        let args = vec![2, 2, 4];
        let ctx = &mut ProofCtx::new_deterministic();
        encode_witness(p, args, t, ctx).unwrap();
    }

    #[test]
    fn gen_circuit() {
        let t = 20;
        let p = &mul_eq();
        let _ = generate_circuit(p, t);
    }

    #[test]
    fn const0() {
        let prog = &const_0();
        let args = vec![];
        let time_bound = 1;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    //#[ignore]
    fn long_time_bound() {
        let prog = &const_0();
        let args = vec![];
        let time_bound = 10;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn b() {
        let prog = &b_skip();
        let args = vec![];
        let time_bound = 3; // notice: t < len(encode(prog))
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn overflowing_add() {
        let prog = &programs::overflowing_add();
        let args = vec![];
        let time_bound = 3;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn b_z() {
        let prog = &b_z_skip();
        let args = vec![];
        let time_bound = 4; // notice: t < len(encode(prog))
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mov_ret() {
        let prog = &mov0_ret();
        let args = vec![];
        let time_bound = 2;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mem_write0() {
        let prog = &simple_str0();
        let args = vec![];
        let time_bound = 3;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mem_write1() {
        let prog = &simple_str1();
        let args = vec![];
        let time_bound = 3;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mem_str3_42() {
        let prog = &str3_42();
        let args = vec![];
        let time_bound = 5;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mem_str2_ldr2_str1_ldr1() {
        let prog = &str2_ldr2_str1_ldr1();
        let args = vec![];
        let time_bound = 7;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mem_str1_ldr1_str0_ldr0() {
        let prog = &str1_ldr1_str0_ldr0();
        let args = vec![];
        let time_bound = 7;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mem_str1_str0_ldr1_ldr0() {
        let prog = &str1_str0_ldr1_ldr0();
        let args = vec![];
        let time_bound = 7;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mem_str1_str2_ldr1_ldr2() {
        let prog = &str1_str2_ldr1_ldr2();
        let args = vec![];
        let time_bound = 7;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mem_str0() {
        let prog = &str0();
        let args = vec![];
        let time_bound = 3;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res)
    }

    #[test]
    fn mem_write1_7times() {
        let n = 7;
        let prog = &simple_str_n(n);
        let args = vec![];
        let time_bound = n + 2;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn load() {
        let prog = &simple_ldr();
        let args = vec![];
        let time_bound = 4;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mov_2pow20() {
        let prog = &mov2pow20_ret0();
        let args = vec![];
        let time_bound = 2;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mov42_movr3() {
        let prog = &mov42_movr3_ret0();
        let args = vec![];
        let time_bound = 4;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    #[ignore = "program doesn't return 0"]
    fn mov42() {
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
    fn ldr_args() {
        let prog = &ldr_2_args();
        let args = vec![1, 2];
        let time_bound = 5;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn and() {
        let prog = &and_000111_111000();
        let args = vec![];
        let time_bound = 5;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn xor() {
        let prog = &xor_0110_0101();
        let args = vec![];
        let time_bound = 6;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn shr() {
        let prog = &shr_5_1654560();
        let args = vec![];
        let time_bound = 5;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn rotr() {
        let prog = &rotr_8_0xb301();
        let args = vec![];
        let time_bound = 5;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mul_1_1_eq_1() {
        let prog = &mul_eq();
        let args = vec![1, 1, 1];
        let time_bound = 15;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mul_1_2_eq_2() {
        let prog = &mul_eq();
        dbg!(&prog);
        let args = vec![1, 2, 2];
        let time_bound = 15;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mul_2_2_eq_4() {
        let prog = &mul_eq();
        let args = vec![2, 2, 4];
        let time_bound = 22;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    #[test]
    fn mul_2_17_eq_34() {
        let prog = &mul_eq();
        let args = vec![2, 17, 34];
        let time_bound = 22;
        let res = convert_and_eval(prog, args, time_bound);
        assert_eq!(vec![0; res.len()], res);
    }

    fn convert_and_eval(p: &Prog, args: Vec<Word>, t: usize) -> Vec<u32> {
        let c = &generate_circuit(p, t);
        //pp::print(c, None);
        let ctx = &mut ProofCtx::new_deterministic();
        let w = encode_witness(p, args, t, ctx).unwrap();
        // dbg!(&w);
        eval32(c, w)
    }
}
