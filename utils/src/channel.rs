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

    pub fn send_delta(&mut self, delta: u64) {
        let n = self.stream_other.write(&delta.to_le_bytes()).unwrap();
        assert_eq!(n, 8)
    }
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
        let mut buf: [u8; 8] = [0; 8];
        for _ in 0..n {
            // read point of evaluation
            let x = self.stream_vole.read(&mut buf).unwrap();
            assert_eq!(x, 8);
            xs.push(u64::from_le_bytes(buf));
            // read mac on point
            let x = self.stream_vole.read(&mut buf).unwrap();
            assert_eq!(x, 8);
            macs.push(u64::from_le_bytes(buf));
        }
        (xs, macs)
    }
    pub fn recv_extend_vole_zm(&mut self, n: u64) -> (Vec<u64>, Vec<u64>) {
        self.recv_extend_vole_z2(n)
    }

    // --- comm. with verifier
}

impl VerifierTcpChannel {
    pub fn new(so: TcpStream, sv: TcpStream) -> Self {
        Self {
            stream_other: so,
            stream_vole: sv,
        }
    }

    // --- comm. with vole dealer

    pub fn recv_delta(&mut self) -> u64 {
        let mut buf: [u8; 8] = [0; 8];
        let x = self.stream_vole.read(&mut buf).unwrap();
        assert_eq!(x, 8);
        u64::from_le_bytes(buf)
    }

    pub fn recv_extend_vole_z2(&mut self, n: u64) -> Vec<u64> {
        let mut keys = vec![];
        let mut buf: [u8; 8] = [0; 8];
        for _ in 0..n {
            // read key
            let x = self.stream_vole.read(&mut buf).unwrap();
            assert_eq!(x, 8);
            keys.push(u64::from_le_bytes(buf));
        }
        keys
    }

    pub fn recv_extend_vole_zm(&mut self, n: u64) -> Vec<u64> {
        self.recv_extend_vole_z2(n)
    }

    // --- comm. with prover

    pub fn send_challenge(&mut self, x: u64) {
        let x = self.stream_other.write(&x.to_le_bytes()).unwrap();
        assert_eq!(x, 8);
    }
}

// pub trait ZKChannel {
//     fn send_delta(&mut self, delta: u64);
//     fn recv_delta(&mut self, delta: u64);
//     fn send_extend_voleZ2(&mut self, n: u64);
//     fn send_extend_voleZm(&mut self, n: u64);
//     fn recv_extend_voleZ2(&mut self) -> (Vec<u64>, Vec<u64>);
//     fn recv_extend_voleZm(&mut self) -> (Vec<u64>, Vec<u64>);
// }
