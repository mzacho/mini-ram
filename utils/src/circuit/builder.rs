use super::ARG0;
use super::OP_ADD;
use super::OP_AND;
use super::OP_AND_CONST;
use super::OP_CHECK_ALL_EQ_BUT_ONE;
use super::OP_CHECK_AND;
use super::OP_CHECK_EQ;
use super::OP_CHECK_Z;
use super::OP_CONST;
use super::OP_CONV_A2B;
use super::OP_CONV_B2A;
use super::OP_DECODE32;
use super::OP_DECODE64;
use super::OP_ENCODE32;
use super::OP_ENCODE4;
use super::OP_ENCODE8;
use super::OP_MUL;
use super::OP_MUL_CONST;
use super::OP_OUT;
use super::OP_SELECT;
use super::OP_SELECT_CONST;
use super::OP_SUB;
use super::OP_XOR;

pub struct Builder<T> {
    gates: Vec<usize>,
    consts: Vec<T>,
    cursor_wires: usize,
    cursor_consts: usize,
    n_gates: usize,
    n_in: usize,
    n_mul: usize,
    n_out: usize,
    offset_arg0: bool,
}

#[derive(Debug)]
pub struct Res<T> {
    pub gates: Vec<usize>,
    pub consts: Vec<T>,
    pub n_gates: usize,
    pub n_in: usize,
    pub n_mul: usize,
    pub n_out: usize,
}

impl<T> Builder<T> {
    pub fn new(n_in: usize) -> Self {
        Self {
            gates: Vec::new(),
            consts: Vec::new(),
            cursor_wires: n_in + ARG0,
            cursor_consts: ARG0,
            n_gates: 0,
            n_in,
            n_mul: 0,
            n_out: 0,
            offset_arg0: false,
        }
    }

    #[cfg(test)]
    pub fn validate(&self) {
        assert_eq!(super::count_ops(&self.gates), self.n_gates);
        assert_eq!(super::count_out(&self.gates), self.n_out);
        assert_eq!(super::count_mul(&self.gates), self.n_mul);
    }

    // --- binary ops

    pub fn xor(&mut self, ids: &[usize]) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_XOR);
        for id in ids {
            self.gates.push(*id);
        }
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn and(&mut self, x: usize, y: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_AND);
        self.gates.push(x);
        self.gates.push(y);
        self.n_gates += 1;
        self.n_mul += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn and_const(&mut self, c: usize, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_AND_CONST);
        self.gates.push(c);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    // convenience
    pub fn or(&mut self, x: usize, y: usize) -> usize {
        #[cfg(test)]
        self.validate();
        let xor = self.xor(&[x, y]);
        let and = self.and(x, y);
        self.xor(&[xor, and])
    }

    // --- arithmetic ops

    pub fn add(&mut self, ids: &[usize]) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_ADD);
        for id in ids {
            self.gates.push(*id);
        }
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn sub(&mut self, x: usize, y: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_SUB);
        self.gates.push(x);
        self.gates.push(y);
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn mul(&mut self, x: usize, y: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_MUL);
        self.gates.push(x);
        self.gates.push(y);
        self.n_gates += 1;
        self.n_mul += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn mul_const(&mut self, c: usize, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_MUL_CONST);
        self.gates.push(c);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn select(&mut self, i: usize, ids: &[usize]) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_SELECT);
        self.gates.push(i);
        for id in ids {
            self.gates.push(*id);
        }
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn select_range(&mut self, i: usize, from: usize, to: usize, step: usize) -> usize {
        self.select_range_(i, from, to, step, OP_SELECT)
    }

    pub fn select_const_range(&mut self, i: usize, from: usize, to: usize, step: usize) -> usize {
        self.select_range_(i, from, to, step, OP_SELECT_CONST)
    }

    pub fn select_range_(
        &mut self,
        mut i: usize,
        mut from: usize,
        mut to: usize,
        step: usize,
        op: usize,
    ) -> usize {
        if !matches!(op, OP_SELECT | OP_SELECT_CONST) {
            panic!("invalid select op")
        }
        #[cfg(test)]
        self.validate();
        if self.offset_arg0 {
            i += ARG0;
            from += ARG0;
            to += ARG0;
            self.offset_arg0 = false;
        }

        self.gates.push(op);
        self.gates.push(i);
        for id in (from..to).step_by(step) {
            self.gates.push(id);
        }
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn decode32(&mut self, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_DECODE32);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_wires += 32;
        self.cursor_wires - 32
    }

    pub fn decode64(&mut self, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_DECODE64);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_wires += 64;
        self.cursor_wires - 64
    }

    pub fn encode8(&mut self, x1: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_ENCODE8);
        for id in x1..x1 + 8 {
            self.gates.push(id);
        }
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn encode4(&mut self, x1: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_ENCODE4);
        for id in x1..x1 + 4 {
            self.gates.push(id);
        }
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn encode32(&mut self, x1: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_ENCODE32);
        for id in x1..x1 + 32 {
            self.gates.push(id);
        }
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    // --- mixed ops

    pub fn push_const(&mut self, c: T) -> usize {
        #[cfg(test)]
        self.validate();
        self.consts.push(c);
        self.cursor_consts += 1;
        self.cursor_consts - 1
    }

    pub fn const_(&mut self, c: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CONST);
        self.gates.push(c);
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn conv_b2a(&mut self, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CONV_B2A);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    pub fn conv_a2b(&mut self, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CONV_A2B);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_wires += 1;
        self.cursor_wires - 1
    }

    // --- zk verification ops

    pub fn check_z(&mut self, x: usize) {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CHECK_Z);
        self.gates.push(x);
        self.n_gates += 1;
    }

    pub fn check_eq(&mut self, x: usize, y: usize) {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CHECK_EQ);
        self.gates.push(x);
        self.gates.push(y);
        self.n_gates += 1;
    }

    pub fn check_and(&mut self, x: usize, y: usize, z: usize) {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CHECK_AND);
        self.gates.push(x);
        self.gates.push(y);
        self.gates.push(z);
        self.n_gates += 1;
    }

    pub fn check_all_eq_but_one(&mut self, i: usize, ids: &[(usize, usize)]) {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CHECK_ALL_EQ_BUT_ONE);
        self.gates.push(i);
        for (x, y) in ids {
            self.gates.push(*x);
            self.gates.push(*y);
        }
        self.n_gates += 1;
    }

    /// Put builder into state where all ids are offset by ARG0, but
    /// only for next instruction. (and currently only for
    /// select_range)
    pub fn offset_arg0(&mut self) {
        self.offset_arg0 = true
    }

    pub fn debug(&mut self) {
        use super::OP_DEBUG;

        self.gates.push(OP_DEBUG);
        self.n_gates += 1;
    }

    /// Reduce builder to its result
    pub fn build(mut self, outputs: &[usize]) -> Res<T> {
        #[cfg(test)]
        self.validate();
        for x in outputs {
            self.gates.push(OP_OUT);
            self.gates.push(*x);
            self.n_gates += 1;
            self.n_out += 1;
        }
        Res {
            gates: self.gates,
            consts: self.consts,
            n_gates: self.n_gates,
            n_out: self.n_out,
            n_in: self.n_in,
            n_mul: self.n_mul,
        }
    }
}
