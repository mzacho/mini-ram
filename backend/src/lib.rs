#![allow(unused_variables)]
extern crate utils;

use std::time::Instant;

use rand::rngs::StdRng;
use rand::RngCore;
use rand::SeedableRng;

pub mod quicksilver;

pub struct ProofCtx {
    rng: StdRng,
    instants: Vec<Instant>,
    msgs: Vec<&'static str>,
    indent: usize,
}

impl ProofCtx {
    pub fn new_deterministic() -> Self {
        #[rustfmt::skip]
        let seed = [0,0,0,0, 30,0,20,10, 200,1,10,1, 0,0,0,0,
                    0,2,100,100, 20,10,5,0, 0,0,0,0, 0,0,0,0];
        let rng = StdRng::from_seed(seed);
        // There's no use of making time tracking deterministic..
        let instants = vec![Instant::now()];
        Self {
            rng,
            instants,
            indent: 0,
            msgs: vec![],
        }
    }

    pub fn new_random() -> Self {
        Self {
            rng: StdRng::from_rng(rand::thread_rng()).unwrap(),
            instants: vec![Instant::now()],
            indent: 0,
            msgs: vec![],
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// Starts a new timer.
    pub fn start_time(&mut self, msg: &'static str) {
        self.instants.push(Instant::now());
        self.msgs.push(msg);
        self.indent += 1;
    }

    /// Stops last timer. Returns current time if no timer is
    /// running.
    pub fn stop_time(&mut self) {
        let now = Instant::now();
        if let Some(last_time) = self.instants.pop() {
            let msg = self.msgs.pop().unwrap();
            let d = now.duration_since(last_time).as_millis();
            self.indent -= 1;
            let indent = indent(&self.indent);
            println!("  [bench] {indent}{msg}: {d}ms");
        } else {
            println!(
                "  [bench] WARNING: stop_time called without
any timer running"
            );
        };
    }
}

fn indent(x: &usize) -> String {
    let mut res = String::from("");
    for _ in 0..*x {
        res = format!("{res} ")
    }
    res
}
