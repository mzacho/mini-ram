use crate::ProofCtx;
use backend::quicksilver::prove::prove64;
use backend::quicksilver::verify::verify64;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use utils::channel;
use utils::channel::*;
use utils::circuit::builder::Res as Circuit;

pub fn run_p(
    port: u16,
    port_vole: u16,
    c: Circuit<u64>,
    w: Vec<u64>,
    ctx: ProofCtx,
) -> std::io::Result<()> {
    print!("Prover: Connecting to VOLE dealer on port {port_vole}... ");
    let stream_vole = TcpStream::connect(format!("127.0.0.1:{port_vole}"))?;
    println!("Connected.");
    print!("Prover: Listening for on port {port} verifier... ");
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;
    let stream_other = listener.accept()?;

    let chan = ProverTcpChannel::new(stream_other.0, stream_vole);
    prove64(c, w, chan, ctx);
    Ok(())
}

pub fn run_v(port: u16, port_vole: u16, c: Circuit<u64>, ctx: ProofCtx) -> std::io::Result<()> {
    print!("Verifier: Connecting to VOLE dealer on port {port_vole}... ");
    let stream_vole = TcpStream::connect(format!("127.0.0.1:{port_vole}"))?;
    println!("Connected.");
    print!("Verifier: Connecting to prover on port {port}... ");
    let stream_other = TcpStream::connect(format!("127.0.0.1:{port}"))?;

    let chan = VerifierTcpChannel::new(stream_other, stream_vole);
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

    for i in 0..n {
        let r = ctx.next_u64();
        let k = ctx.next_u64();
        let m = delta.wrapping_mul(r).wrapping_add(k);
        println!("  i={i}: Sending r={r}, m={m} to prover");
        snd_extend_mac(&mut stream_p, r, m)?;
        println!("       Sending k={k} to verifier");
        snd_extend_key(&mut stream_v, k)?;
    }
    println!("Done, exiting.");
    Ok(())
}

fn snd_extend_mac(stream: &mut TcpStream, r: u64, m: u64) -> std::io::Result<()> {
    let n = stream.write(&r.to_le_bytes())? + stream.write(&m.to_le_bytes())?;
    assert_eq!(n, 16);
    Ok(())
}

fn snd_extend_key(stream: &mut TcpStream, k: u64) -> std::io::Result<()> {
    let n = stream.write(&k.to_le_bytes())?;
    assert_eq!(n, 8);
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
