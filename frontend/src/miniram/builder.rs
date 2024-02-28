use crate::miniram::lang::*;

pub struct Builder {
    p: Prog,
}

impl Builder {
    pub fn new() -> Self {
        Builder { p: vec![] }
    }
    pub fn build(self) -> Prog {
        self.p
    }
    pub fn add(mut self, z: Reg, x: Reg, y: Reg) -> Self {
        self.p.push(Inst::ADD(z, x, y));
        self
    }
    pub fn sub(mut self, z: Reg, x: Reg, y: Reg) -> Self {
        self.p.push(Inst::SUB(z, x, y));
        self
    }
    pub fn mov_r(mut self, dst: Reg, src: Reg) -> Self {
        self.p.push(Inst::MOV(dst, Val::Reg(src)));
        self
    }
    pub fn mov_c(mut self, dst: Reg, src: Word) -> Self {
        self.p.push(Inst::MOV(dst, Val::Const(src)));
        self
    }
    pub fn b_z(mut self, dst: Reg) -> Self {
        self.p.push(Inst::B(Some(Cond::Z), dst));
        self
    }
    pub fn b(mut self, dst: Reg) -> Self {
        self.p.push(Inst::B(None, dst));
        self
    }
    pub fn ldr(mut self, dst: Reg, src: Reg) -> Self {
        self.p.push(Inst::LDR(dst, src));
        self
    }

    pub fn ret_r(mut self, r: Reg) -> Self {
        self.p.push(Inst::RET(Val::Reg(r)));
        self
    }
}
