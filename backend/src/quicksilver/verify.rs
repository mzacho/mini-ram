use utils::channel::*;
use utils::circuit::builder::Res as Circuit;

pub fn verify64(c: Circuit<u64>, mut chan: VerifierTcpChannel) {
    //let n_in = w.len();
    let n_gates = c.n_gates;
    let n_in = c.n_in;
    let n_mul = c.n_mul;
    let gates = c.gates;
    let consts = c.consts;

    let vole_size = n_in + n_mul + 1; // + 2 * q;
    let (delta, keys) = preprocess_vole(&mut chan, vole_size);

    // todo!()
}

fn preprocess_vole(chan: &mut VerifierTcpChannel, n: usize) -> (u64, Vec<u64>) {
    let vole_size = u64::try_from(n).unwrap();

    let delta = chan.recv_delta();
    let vole_zm = chan.recv_extend_vole_zm(vole_size);
    (delta, vole_zm)
}
