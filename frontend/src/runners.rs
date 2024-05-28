use zerocopy::IntoBytes;

use crate::miniram::lang::Word;
use crate::ProofCtx;
use backend::quicksilver::prove::prove32;
use backend::quicksilver::verify::verify32;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use utils::channel;
use utils::channel::*;
use utils::circuit::builder::Res as Circuit;

pub fn run_p(
    port: u16,
    port_vole: u16,
    c: Circuit<Word>,
    w: Vec<Word>,
    mut ctx: ProofCtx,
) -> std::io::Result<()> {
    print!("Prover: Connecting to VOLE dealer on port {port_vole}... ");
    let stream_vole = TcpStream::connect(format!("127.0.0.1:{port_vole}"))?;
    println!("Connected.");
    print!("Prover: Listening for verifier on port {port}... ");
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;
    let stream_other = listener.accept()?;
    println!("Verifier connected.");

    let chan = ProverTcpChannel::new(stream_other.0, stream_vole);
    println!("Running prover");
    ctx.start_time("prover");
    prove32(c, w, chan, ctx);
    Ok(())
}

pub fn run_v(
    port: u16,
    port_vole: u16,
    c: Circuit<Word>,
    mut ctx: ProofCtx,
) -> std::io::Result<()> {
    print!("Verifier: Connecting to VOLE dealer on port {port_vole}... ");
    let stream_vole = TcpStream::connect(format!("127.0.0.1:{port_vole}"))?;
    println!("Connected.");
    print!("Verifier: Connecting to prover on port {port}... ");
    let stream_other = TcpStream::connect(format!("127.0.0.1:{port}"))?;
    println!("Connected.");

    println!("Running verifier");
    let chan = VerifierTcpChannel::new(stream_other, stream_vole);
    ctx.start_time("verifier");
    verify32(c, chan, ctx);
    Ok(())
}

pub fn run_vole(port: u16, mut ctx: ProofCtx) -> std::io::Result<()> {
    // accept init requests
    println!("VOLE dealer listening for connections on port {port}...");
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;
    let mut stream_p = listener.accept()?.0;
    println!("Prover connected");
    let mut stream_v = listener.accept()?.0;
    println!("Both clients connected, waiting for extend message from prover...");

    let n = rcv_extend(&mut stream_p)?;
    println!("Received extend n={n}, generating vole correlations\n...");

    let delta = ctx.next_u128();
    println!("Sending delta={delta} to verifier");
    snd_delta(&mut stream_v, delta)?;
    //let delta = Simd::from([delta; 64]);

    println!("Sending correlations to both parties...");
    // fill random bytes in 1KB blocks
    let mut buf_val: [u128; 32] = [0; 32];
    let mut buf_key: [u128; 32] = [0; 32];
    let one_tenth_done = (n / 32) / 10;
    let mut ctr = 0;
    ctx.start_time("vole");
    for i in 0..(n / 32) + 4 {
        if (one_tenth_done != 0) && i % one_tenth_done == 0 {
            println!("  [progress] {ctr}%");
            ctr += 10;
        }
        ctx.fill_bytes(&mut buf_val);
        ctx.fill_bytes(&mut buf_key);
        let r = buf_val; // let r = Simd::from(buf_val);
        let k = buf_key; // let k = Simd::from(buf_key);
        let mut m: [u128; 32] = [0; 32];
        for j in 0..32 {
            m[j] = (delta.wrapping_mul(r[j])).wrapping_add(k[j]);
        }
        //println!("  i={i}: Sending r={r:?}, m={m:?} to prover");
        snd_extend_mac(&mut stream_p, &r, &m)?;
        //println!("       Sending k={k:?} to verifier");
        snd_extend_key(&mut stream_v, &k)?;
    }
    ctx.stop_time();
    println!("Done, exiting.");
    Ok(())
}

fn snd_extend_mac(stream: &mut TcpStream, r: &[u128; 32], m: &[u128; 32]) -> std::io::Result<()> {
    stream.write_all(r.as_bytes())?;
    stream.write_all(m.as_bytes())?;
    Ok(())
}

fn snd_extend_key(stream: &mut TcpStream, k: &[u128; 32]) -> std::io::Result<()> {
    stream.write_all(k.as_bytes())?;
    Ok(())
}

fn snd_delta(stream: &mut TcpStream, delta: u128) -> std::io::Result<()> {
    let n = stream.write(&delta.to_le_bytes())?;
    assert_eq!(n, std::mem::size_of::<u128>());
    Ok(())
}

fn rcv_extend(stream: &mut TcpStream) -> std::io::Result<u64> {
    Ok(channel::recv_u64(stream))
}

#[cfg(test)]
mod tests {}
