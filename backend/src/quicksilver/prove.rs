use utils::channel::*;
use utils::circuit::builder::Res as Circuit;

pub fn prove64(c: Circuit<u64>, w: Vec<u64>, mut chan: ProverTcpChannel) {
    let n_in = w.len();
    let n_mul = c.n_mul;
    let n_gates = c.n_gates;
    let gates = c.gates;
    let consts = c.consts;

    let vole_size = n_in + n_mul + 1; // + 2 * q;

    let (xs, macs) = preprocess_vole(&mut chan, vole_size);

    //todo!()
}

fn preprocess_vole(chan: &mut ProverTcpChannel, n: usize) -> (Vec<u64>, Vec<u64>) {
    let vole_size = u64::try_from(n).unwrap();

    chan.send_extend_vole_zm(vole_size);
    chan.recv_extend_vole_zm(vole_size)
}
