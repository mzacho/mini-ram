use zerocopy::IntoBytes;

use crate::ProofCtx;
use backend::quicksilver::prove::prove64;
use backend::quicksilver::verify::verify64;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::simd::Simd;
use utils::channel;
use utils::channel::*;
use utils::circuit::builder::Res as Circuit;

pub fn run_p(
    port: u16,
    port_vole: u16,
    c: Circuit<u64>,
    w: Vec<u64>,
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
    println!("Running prove64");
    ctx.start_time("prover");
    prove64(c, w, chan, ctx);
    Ok(())
}

pub fn run_v(port: u16, port_vole: u16, c: Circuit<u64>, mut ctx: ProofCtx) -> std::io::Result<()> {
    print!("Verifier: Connecting to VOLE dealer on port {port_vole}... ");
    let stream_vole = TcpStream::connect(format!("127.0.0.1:{port_vole}"))?;
    println!("Connected.");
    print!("Verifier: Connecting to prover on port {port}... ");
    let stream_other = TcpStream::connect(format!("127.0.0.1:{port}"))?;
    println!("Connected.");

    println!("Running verify64");
    let chan = VerifierTcpChannel::new(stream_other, stream_vole);
    ctx.start_time("verifier");
    verify64(c, chan, ctx);
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

    let delta = ctx.next_u64();
    println!("Sending delta={delta} to verifier");
    snd_delta(&mut stream_v, delta)?;
    let delta = Simd::from([delta; 64]);

    println!("Sending correlations to both parties...");
    // fill random bytes in 1KB blocks
    let mut buf_val: [u64; 64] = [0; 64];
    let mut buf_key: [u64; 64] = [0; 64];
    let one_tenth_done = (n / 64) / 10;
    let mut ctr = 0;
    ctx.start_time("vole");
    for i in 0..(n / 64) + 3 {
        if (one_tenth_done != 0) && i % one_tenth_done == 0 {
            println!("  [progress] {ctr}%");
            ctr += 10;
        }
        ctx.fill_bytes(&mut buf_val);
        ctx.fill_bytes(&mut buf_key);
        let r = Simd::from(buf_val);
        let k = Simd::from(buf_key);
        let m = (delta * r) + k;
        // println!("  i={i}: Sending r={r}, m={m} to prover");
        snd_extend_mac(&mut stream_p, &r.into(), &m.into())?;
        // println!("       Sending k={k} to verifier");
        snd_extend_key(&mut stream_v, &k.into())?;
    }
    ctx.stop_time();
    println!("Done, exiting.");
    Ok(())
}

fn snd_extend_mac(stream: &mut TcpStream, r: &[u64; 64], m: &[u64; 64]) -> std::io::Result<()> {
    stream.write_all(r.as_bytes())?;
    stream.write_all(m.as_bytes())?;
    Ok(())
}

fn snd_extend_key(stream: &mut TcpStream, k: &[u64; 64]) -> std::io::Result<()> {
    stream.write_all(k.as_bytes())?;
    Ok(())
}

fn snd_delta(stream: &mut TcpStream, delta: u64) -> std::io::Result<()> {
    let n = stream.write(&delta.to_le_bytes())?;
    assert_eq!(n, 8);
    Ok(())
}

fn rcv_extend(stream: &mut TcpStream) -> std::io::Result<u64> {
    Ok(channel::recv_u64(stream))
}

#[cfg(test)]
mod tests {}
