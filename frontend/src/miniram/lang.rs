use strum_macros::EnumIter;

/// MiniRAM - a TinyRAM inspired language for efficient verification
/// of Zero-Knowledge arguments, intended to be targetet by a
/// compiler for a high-level language
///
/// v0: Harvard architecture, i.e code and data are is seperate
/// memories

pub type Word = u32;

pub type Reg = u8;

// Registers are PC, R1, ..., R15
pub const N_REG: usize = 16;

pub mod reg {
    use super::Reg;

    pub const PC: Reg = 0;
    pub const R1: Reg = 1;
    pub const R2: Reg = 2;
    pub const R3: Reg = 3;
    pub const R4: Reg = 4;
    pub const R5: Reg = 5;
    pub const R6: Reg = 6;
    // pub const R7: Reg = 7;
    // pub const R8: Reg = 8;
    // pub const R9: Reg = 9;
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Inst {
    // Bitwise operations
    // AND(Reg, Reg, Reg),
    // OR(Reg, Reg, Reg),
    // Integer operations
    Add(Reg, Reg, Reg),
    Sub(Reg, Reg, Reg),
    // Move
    Mov(Reg, Val),
    // Memory access
    Ldr(Reg, Reg),
    Str(Reg, Reg),
    // Branching (unconditional and conditional)
    B(Option<Cond>, Reg),
    // Halting
    Ret(Val),
}

#[derive(Debug, Clone, Copy)]
pub enum Val {
    // Value in register
    Reg(Reg),
    // Const
    Const(Word),
}

#[derive(Debug, Eq, PartialEq, Hash, EnumIter, Clone, Copy)]
pub enum Cond {
    // Set when arithmetic intr. resulted in zero
    Z,
}

pub type Prog = Vec<Inst>;
