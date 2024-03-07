use backend::quicksilver::prove::prove64;
use backend::quicksilver::verify::verify64;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use utils::channel::impls::TcpChannel;
use utils::circuit::builder::Res as Circuit;

pub fn run_p(port: u16, port_vole: u16, c: Circuit<u64>, w: Vec<u64>) -> std::io::Result<()> {
    let stream_vole = TcpStream::connect(format!("127.0.0.1:{port_vole}"))?;
    // listen for verifier connect
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;
    let stream_other = listener.accept()?;

    let chan = TcpChannel::new(stream_other.0, stream_vole);
    prove64(c, w, chan);
    Ok(())
}

pub fn run_v(port: u16, port_vole: u16, c: Circuit<u64>) -> std::io::Result<()> {
    let stream_vole = TcpStream::connect(format!("127.0.0.1:{port_vole}"))?;
    // connect to prover
    let stream_other = TcpStream::connect(format!("127.0.0.1:{port}"))?;

    let chan = TcpChannel::new(stream_other, stream_vole);
    verify64(c, chan);
    Ok(())
}

pub fn run_vole(port: u16) -> std::io::Result<()> {
    // accept init requests
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;
    let mut stream_p = listener.accept()?.0;
    println!("prover connected");
    let mut stream_v = listener.accept()?.0;
    println!("both clients connected, entering eval loop");

    let delta = 0;
    // spin up two treads handling requests for each party?
    loop {
        // Only the prover sends extend message.
        // Todo: is this secure?
        let n = rcv_extend(&mut stream_p)?;
        for _ in 0..n {
            let r = 0;
            let k = 0;
            let m = delta * r + k;
            snd_extend_mac(&mut stream_p, r, m)?;
            snd_extend_key(&mut stream_v, k)?;
        }
    }
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

fn rcv_extend(stream: &mut TcpStream) -> std::io::Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    assert_eq!(stream.read(&mut buf)?, 8);
    Ok(u64::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {}
