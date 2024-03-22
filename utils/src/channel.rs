use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub struct ProverTcpChannel {
    stream_vole: TcpStream,
    stream_other: TcpStream,
}

pub struct VerifierTcpChannel {
    stream_vole: TcpStream,
    stream_other: TcpStream,
}

impl ProverTcpChannel {
    pub fn new(so: TcpStream, sv: TcpStream) -> Self {
        Self {
            stream_other: so,
            stream_vole: sv,
        }
    }

    // --- comm. with vole dealer

    pub fn send_extend_vole_z2(&mut self, n: u64) {
        let x = self.stream_vole.write(&n.to_le_bytes()).unwrap();
        assert_eq!(x, 8);
    }
    pub fn send_extend_vole_zm(&mut self, n: u64) {
        self.send_extend_vole_z2(n)
    }
    pub fn recv_extend_vole_z2(&mut self, n: u64) -> (Vec<u64>, Vec<u64>) {
        let mut xs = vec![];
        let mut macs = vec![];
        for _ in 0..n {
            // read point of evaluation
            let x = recv_u64(&mut self.stream_vole);
            xs.push(x);
            // read mac on point
            let t = recv_u64(&mut self.stream_vole);
            macs.push(t);
        }
        (xs, macs)
    }
    pub fn recv_extend_vole_zm(&mut self, n: u64) -> (Vec<u64>, Vec<u64>) {
        self.recv_extend_vole_z2(n)
    }

    // --- comm. with verifier

    pub fn send_delta(&mut self, delta: u64) {
        let n = self.stream_other.write(&delta.to_le_bytes()).unwrap();
        assert_eq!(n, 8)
    }

    pub fn send_mac(&mut self, mac: u64) {
        let n = self.stream_other.write(&mac.to_le_bytes()).unwrap();
        assert_eq!(n, 8)
    }

    pub fn recv_challenge(&mut self) -> u64 {
        recv_u64(&mut self.stream_other)
    }

    pub fn send_u(&mut self, x: u64) {
        let x = self.stream_other.write(&x.to_le_bytes()).unwrap();
        assert_eq!(x, 8);
    }

    pub fn send_v(&mut self, x: u64) {
        let x = self.stream_other.write(&x.to_le_bytes()).unwrap();
        assert_eq!(x, 8);
    }
}

impl VerifierTcpChannel {
    pub fn new(so: TcpStream, sv: TcpStream) -> Self {
        Self {
            stream_other: so,
            stream_vole: sv,
        }
    }

    // --- comm. with vole dealer

    pub fn recv_delta_from_dealer(&mut self) -> u64 {
        recv_u64(&mut self.stream_vole)
    }

    pub fn recv_extend_vole_z2(&mut self, n: u64) -> Vec<u64> {
        let mut keys = vec![];
        for _ in 0..n {
            // read key
            keys.push(recv_u64(&mut self.stream_vole));
        }
        keys
    }

    pub fn recv_extend_vole_zm(&mut self, n: u64) -> Vec<u64> {
        self.recv_extend_vole_z2(n)
    }

    // --- comm. with prover

    pub fn recv_delta_from_prover(&mut self) -> u64 {
        recv_u64(&mut self.stream_other)
    }

    pub fn recv_mac(&mut self) -> u64 {
        recv_u64(&mut self.stream_other)
    }

    pub fn send_challenge(&mut self, x: u64) {
        let x = self.stream_other.write(&x.to_le_bytes()).unwrap();
        assert_eq!(x, 8);
    }

    pub fn recv_u(&mut self) -> u64 {
        recv_u64(&mut self.stream_other)
    }

    pub fn recv_v(&mut self) -> u64 {
        recv_u64(&mut self.stream_other)
    }
}

pub fn recv_u64(stream: &mut TcpStream) -> u64 {
    let mut buf: [u8; 8] = [0; 8];
    let mut n = 0;
    loop {
        n += stream.read(&mut buf[n..]).unwrap();
        if n == 8 {
            break;
        }
    }
    u64::from_le_bytes(buf)
}
