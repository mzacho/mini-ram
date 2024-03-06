use super::OP_AND;
use super::OP_AND_CONST;
use super::OP_ADD;
use super::OP_CONV_A2B;
use super::OP_CONV_B2A;
use super::OP_SUB;
use super::OP_MUL;
use super::OP_MUL_CONST;
use super::OP_CHECK_ALL_EQ_BUT_ONE;
use super::OP_CHECK_AND;
use super::OP_CHECK_EQ;
use super::OP_CHECK_Z;
use super::OP_CONST;
use super::OP_MAX;
use super::OP_OUT;
use super::OP_SELECT;
use super::OP_XOR;

pub struct Builder<T> {
    gates: Vec<usize>,
    consts: Vec<T>,
    cursor_gates: usize,
    cursor_consts: usize,
    n_gates: usize,
    n_outs: usize,
}

impl<T> Builder<T> {
    pub fn new(n_in: usize) -> Self {
        Self {
            gates: Vec::new(),
            consts: Vec::new(),
            cursor_gates: n_in + OP_MAX + 1,
            cursor_consts: OP_MAX + 1,
            n_gates: 0,
            n_outs: 0,
        }
    }

    #[cfg(test)]
    pub fn validate(&self) {
        assert_eq!(super::count_ops(&self.gates), self.n_gates);
        assert_eq!(super::count_outs(&self.gates), self.n_outs);
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
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    pub fn and(&mut self, x: usize, y: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_AND);
        self.gates.push(x);
        self.gates.push(y);
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    pub fn and_const(&mut self, c: usize, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_AND_CONST);
        self.gates.push(c);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
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
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    pub fn sub(&mut self, x: usize, y: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_SUB);
        self.gates.push(x);
        self.gates.push(y);
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    pub fn mul(&mut self, x: usize, y: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_MUL);
        self.gates.push(x);
        self.gates.push(y);
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    pub fn mul_const(&mut self, c: usize, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_MUL_CONST);
        self.gates.push(c);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
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
        self.cursor_gates += 1;
        self.cursor_gates - 1
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
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    pub fn conv_b2a(&mut self, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CONV_B2A);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    pub fn conv_a2b(&mut self, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CONV_A2B);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    // --- zk verification ops

    pub fn check_z(&mut self, x: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CHECK_Z);
        self.gates.push(x);
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    pub fn check_eq(&mut self, x: usize, y: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CHECK_EQ);
        self.gates.push(x);
        self.gates.push(y);
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    pub fn check_and(&mut self, x: usize, y: usize, z: usize) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CHECK_AND);
        self.gates.push(x);
        self.gates.push(y);
        self.gates.push(z);
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    pub fn check_all_eq_but_one(&mut self, i: usize, ids: &[(usize, usize)]) -> usize {
        #[cfg(test)]
        self.validate();
        self.gates.push(OP_CHECK_ALL_EQ_BUT_ONE);
        self.gates.push(i);
        for (x, y) in ids {
            self.gates.push(*x);
            self.gates.push(*y);
        }
        self.n_gates += 1;
        self.cursor_gates += 1;
        self.cursor_gates - 1
    }

    // reduce

    pub fn build(mut self, outputs: &[usize]) -> (Vec<usize>, Vec<T>, usize, usize) {
        #[cfg(test)]
        self.validate();
        for x in outputs {
            self.gates.push(OP_OUT);
            self.gates.push(*x);
            self.n_gates += 1;
            self.n_outs += 1;
        }
        (self.gates, self.consts, self.n_gates, self.n_outs)
    }
}
