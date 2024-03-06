use super::lang::Inst::*;
use super::lang::*;

/// encoded instruction
/// todo: describe format
type EInst64 = u64;

pub type EProg = Vec<EInst64>;

pub fn encode(p: &Prog) -> EProg {
    p.iter().map(|i| encode_instr(i)).collect()
}

fn encode_instr(i: &Inst) -> EInst64 {
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
        Val::Const(c) => (*c, 1),
    }
}

#[cfg(test)]
mod tests {
    use super::encode_instr;
    use crate::miniram::lang::reg::*;
    use crate::miniram::lang::Inst::*;

    #[test]
    fn test_encode() {
        let i = ADD(R1, R1, R1);
        let enc = encode_instr(&i);
        //          op      dst     arg0    blank                             arg1
        assert_eq!(
            enc,
            0b00000000_00000001_00000001_00000000_00000000000000000000000000000001
        );

        let i = ADD(R2, R1, R1);
        let enc = encode_instr(&i);
        //          op      dst     arg0    blank                             arg1
        assert_eq!(
            enc,
            0b00000000_00000010_00000001_00000000_00000000000000000000000000000001
        );

        let i = SUB(R2, R2, R2);
        let enc = encode_instr(&i);
        //          op      dst     arg0    blank                             arg1
        assert_eq!(
            enc,
            0b00000001_00000010_00000010_00000000_00000000000000000000000000000010
        );
    }
}
