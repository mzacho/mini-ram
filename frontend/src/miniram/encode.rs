use super::lang::Inst::*;
use super::lang::*;
use strum::IntoEnumIterator;

/// encoded instruction
/// todo: describe format
type EInst64 = u64;

pub type EProg = Vec<EInst64>;

pub fn encode(p: Prog) -> EProg {
    p.iter().map(|i| encode_(i)).collect()
}

fn encode_(i: &Inst) -> EInst64 {
    match i {
        ADD(x, y, z) => {
            let opcode = 0;
            let dst = encode_reg(x);
            let arg1 = encode_reg(y);
            let arg2 = u32::from(encode_reg(z));
            encode_instr_u64(opcode, dst, arg1, arg2)
        }
        SUB(x, y, z) => {
            let opcode = 1;
            let dst = encode_reg(x);
            let arg1 = encode_reg(y);
            let arg2 = u32::from(encode_reg(z));
            encode_instr_u64(opcode, dst, arg1, arg2)
        }
        MOV(x, y) => {
            let opcode = 2;
            let dst = encode_reg(x);
            let arg1 = 0;
            let (arg2, op_offset) = encode_val(y);
            encode_instr_u64(opcode + op_offset, dst, arg1, arg2)
        }
        LDR(x, y) => {
            let opcode = 4;
            let dst = encode_reg(x);
            let arg1 = encode_reg(y);
            let arg2 = 0;
            encode_instr_u64(opcode, dst, arg1, arg2)
        }
        STR(x, y) => {
            let opcode = 5;
            let dst = encode_reg(x);
            let arg1 = encode_reg(y);
            let arg2 = 0;
            encode_instr_u64(opcode, dst, arg1, arg2)
        }
        B(x, y) => {
            let opcode = match x {
                None => 6,
                Some(Cond::Z) => 7,
            };
            let dst = 0;
            let arg1 = 0;
            let arg2 = u32::from(encode_reg(y));
            encode_instr_u64(opcode, dst, arg1, arg2)
        }
        RET(x) => {
            let opcode = 8;
            let dst = 0;
            let arg1 = 0;
            let (arg2, op_offset) = encode_val(x);
            encode_instr_u64(opcode + op_offset, dst, arg1, arg2)
        }
    }
}

fn encode_instr_u64(opcode: u8, dst: u8, arg1: u8, arg2: u32) -> u64 {
    let field1: u64 = u64::from(opcode) << 56;
    let field2: u64 = u64::from(dst) << 48;
    let field3: u64 = u64::from(arg1) << 40;
    // field 4 is blank
    let field5: u64 = u64::from(arg2);
    field1 ^ field2 ^ field3 ^ field5
}

fn encode_reg(reg: &Reg) -> u8 {
    #[rustfmt::skip]
    let x = Reg::iter()
        .enumerate()
        .find(|(_, r)| *r == *reg)
        .unwrap().0;
    u8::try_from(x).ok().unwrap()
}

/// Returns encoded value and opcode offset
fn encode_val(v: &Val) -> (u32, u8) {
    match v {
        Val::Reg(r) => {
            let r = encode_reg(r);
            (u32::from(r), 0)
        }
        Val::Const(c) => (*c, 1),
    }
}
