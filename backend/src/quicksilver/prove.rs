use utils::channel::*;
use utils::circuit::builder::Res as Circuit;

pub fn prove64<T>(c: Circuit<u64>, w: Vec<u64>, channel: T)
where
    T: ZKChannel<u64>,
{
    let n_in = w.len();
    let n_gates = c.n_gates;
    let gates = c.gates;
    let consts = c.consts;

    let _ = n_in;
    let _ = n_gates;
    let _ = gates;
    let _ = consts;
    let _ = channel;

    // let input = U64Circuit {
    //     gates: &c.gates,
    //     consts: &c.consts,
    //     n_gates: c.n_gates,
    //     n_in: w.len(),
    // };
    todo!()
}
