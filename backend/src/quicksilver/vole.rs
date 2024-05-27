#[derive(Debug)]
pub struct Segments {
    pub n_in: usize,
    pub n_mul: usize,
    pub n_mul_check: usize,
    pub n_openings: usize,
}

pub struct CorrSender {
    pub xs_in: Vec<u128>,
    pub mc_in: Vec<u128>,
    pub xs_mul: Vec<u128>,
    pub mc_mul: Vec<u128>,
    pub xs_mul_check: Vec<u128>,
    pub mc_mul_check: Vec<u128>,
    pub xs_openings: Vec<u128>,
    pub mc_openings: Vec<u128>,
}

pub struct CorrReceiver {
    pub ks_in: Vec<u128>,
    pub ks_mul: Vec<u128>,
    pub ks_mul_check: Vec<u128>,
    pub ks_openings: Vec<u128>,
}

impl Segments {
    pub fn size(&self) -> usize {
        self.n_in + self.n_mul + self.n_mul_check + self.n_openings
    }
}

pub fn deal() {}
