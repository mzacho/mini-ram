use strum_macros::EnumIter;

/// MiniRAM - a TinyRAM inspired language for efficient verification
/// of Zero-Knowledge arguments, intended to be targetet by a
/// compiler for a high-level language
///
/// v0: Harvard architecture, i.e code and data are is seperate
/// memories

pub type Word = u32;

#[derive(Debug, Eq, PartialEq, Hash, EnumIter, Clone, Copy)]
pub enum Reg {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    R16,
    PC,
}

#[derive(Debug)]
pub enum Inst {
    // Bitwise operations
    // AND(Reg, Reg, Reg),
    // OR(Reg, Reg, Reg),
    // Integer operations
    ADD(Reg, Reg, Reg),
    SUB(Reg, Reg, Reg),
    // Move
    MOV(Reg, Val),
    // Memory access
    LDR(Reg, Reg),
    STR(Reg, Reg),
    // Branching (unconditional and conditional)
    B(Option<Cond>, Reg),
    // Halting
    RET(Val),
}

#[derive(Debug)]
pub enum Val {
    // Value in register
    Reg(Reg),
    // Const
    Const(Word),
}

#[derive(Debug, Eq, PartialEq, Hash, EnumIter)]
pub enum Cond {
    // Set when arithmetic intr. resulted in zero
    Z,
}

pub type Prog = Vec<Inst>;
