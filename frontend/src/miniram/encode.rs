use self::reg::{PC, R1};

use super::lang::Inst::*;
use super::lang::*;

/// Type of an encoded instruction.
///
/// Intructions are encoded into a 64 bit word consisting of
/// 4 fields:
///
///   #1 (8 bits): opcode          - only uses 6 bits
///   #2 (8 bits): dst  (register) - only uses 4 bits
///   #3 (8 bits): arg0 (register) - only uses 4 bits
///   #4 (8 bits): unused
///   #5 (32 bits): arg1 (register or word)
type EInst64 = u64;

pub type EProg = Vec<EInst64>;

pub fn encode(p: &Prog) -> EProg {
    p.iter().map(encode_instr).collect()
}

fn encode_instr(i: &Inst) -> EInst64 {
    match i {
        Add(x, y, z) => {
            let opcode = 0b100;
            let dst = encode_reg(x);
            let arg0 = encode_reg(y);
            let arg1 = u32::from(encode_reg(z));
            encode_instr_u64(opcode, dst, arg0, arg1)
        }
        Sub(x, y, z) => {
            let opcode = 0b1000;
            let dst = encode_reg(x);
            let arg0 = encode_reg(y);
            let arg1 = u32::from(encode_reg(z));
            encode_instr_u64(opcode, dst, arg0, arg1)
        }
        And(x,y,z) => {
            let opcode = 0;
            let dst = encode_reg(x);
            let arg0 = encode_reg(y);
            let arg1 = u32::from(encode_reg(z));
            encode_instr_u64(opcode, dst, arg0, arg1)
        }
        Mov(x, y) => {
            let opcode = 0b1100;
            let dst = encode_reg(x);
            let arg0 = 0;
            let (arg1, op_offset) = encode_val(y);
            encode_instr_u64(opcode + op_offset, dst, arg0, arg1)
        }
        Ldr(dst, src) => {
            let opcode = 0b11;
            let dst = encode_reg(dst);
            let arg0 = encode_reg(src);
            let arg1 = 0;
            encode_instr_u64(opcode, dst, arg0, arg1)
        }
        Str(dst, src) => {
            let opcode = 0b10;
            let dst = encode_reg(dst);
            let arg0 = encode_reg(src);
            let arg1 = 0;
            // NOTICE: we encode the register dst, holding the
            // address to be stored into, in field #3, so fetching the
            // address is done in the same way for LDR/ STR, in the
            // memory consistency sub-circuit.

            encode_instr_u64(opcode, arg0, dst, arg1)
        }
        B(x, y) => {
            let opcode = match x {
                None => 0b10100,
                Some(Cond::Z) => 0b11000,
            };
            let dst = PC;
            let arg0 = 0;
            // todo: use arg0 instead, so arg1 can hold an
            //       offset
            let arg1 = u32::from(encode_reg(y));
            encode_instr_u64(opcode, dst, arg0, arg1)
        }
        Ret(x) => {
            let opcode = 0b100000;
            let dst = R1; // machine returns in R1
            let arg0 = 0;
            let (arg1, op_offset) = encode_val(x);
            encode_instr_u64(opcode + op_offset, dst, arg0, arg1)
        }
    }
}

fn encode_instr_u64(opcode: u8, dst: u8, arg0: u8, arg1: u32) -> u64 {
    let field1: u64 = u64::from(opcode) << 56;
    let field2: u64 = u64::from(dst) << 48;
    let field3: u64 = u64::from(arg0) << 40;
    // field 4 is blank
    let field5: u64 = u64::from(arg1);
    field1 ^ field2 ^ field3 ^ field5
}

#[inline]
fn encode_reg(reg: &Reg) -> u8 {
    *reg
}

/// Returns encoded value and opcode offset
fn encode_val(v: &Val) -> (u32, u8) {
    match v {
        Val::Reg(r) => {
            let r = encode_reg(r);
            (u32::from(r), 0)
        }
        Val::Const(c) => (*c, 0b100),
    }
}

#[cfg(test)]
mod tests {
    use super::encode_instr;
    use crate::miniram::lang::reg::*;
    use crate::miniram::lang::Inst::*;

    #[test]
    fn test_encode() {
        let i = Add(R1, R1, R1);
        let enc = encode_instr(&i);
        //          op      dst     arg0    blank                             arg0
        assert_eq!(
            enc,
            0b0000_0100_0000_0001_0000_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001
        );

        let i = Add(R2, R1, R1);
        let enc = encode_instr(&i);
        //          op      dst     arg0    blank                             arg0
        assert_eq!(
            enc,
            0b0000_0100_0000_0010_0000_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001
        );

        let i = Sub(R2, R2, R2);
        let enc = encode_instr(&i);
        //          op      dst     arg0    blank                             arg0
        assert_eq!(
            enc,
            0b0000_1000_0000_0010_0000_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010
        );
    }
}
