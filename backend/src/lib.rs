#![allow(unused_variables)]
extern crate utils;

use rand::rngs::StdRng;
use rand::RngCore;
use rand::SeedableRng;

pub mod quicksilver;

pub struct ProofCtx {
    rng: StdRng,
}

impl ProofCtx {
    pub fn new_deterministic() -> Self {
        #[rustfmt::skip]
        let seed = [0,0,0,0, 30,0,20,10, 200,1,10,1, 0,0,0,0,
                    0,2,100,100, 20,10,5,0, 0,0,0,0, 0,0,0,0];
        let rng = StdRng::from_seed(seed);
        Self { rng }
    }

    pub fn new_random() -> Self {
        Self {
            rng: StdRng::from_rng(rand::thread_rng()).unwrap(),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }
}
