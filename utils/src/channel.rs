use zerocopy::transmute;

use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub struct ProverTcpChannel {
    stream_vole: TcpStream,
    stream_other: TcpStream,
    delta_ctr: usize,
    mac_ctr: usize,
    verbose: bool,
}

pub struct VerifierTcpChannel {
    stream_vole: TcpStream,
    stream_other: TcpStream,
    delta_ctr: usize,
    mac_ctr: usize,
    verbose: bool,
}

impl ProverTcpChannel {
    pub fn new(so: TcpStream, sv: TcpStream) -> Self {
        Self {
            stream_other: so,
            stream_vole: sv,
            delta_ctr: 0,
            mac_ctr: 0,
            verbose: false,
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
    pub fn recv_extend_vole_z2(&mut self, _n: usize) -> (Vec<u64>, Vec<u64>) {
        todo!()
    }
    pub fn recv_extend_vole_zm(&mut self, n: usize) -> (Vec<u64>, Vec<u64>) {
        let mut xs = vec![];
        let mut macs = vec![];

        // Read in chunks of 64 u64's at a time
        for _ in 0..(n >> 6) + 1 {
            // read point of evaluation
            let xs_ = &recv_64_u64(&mut self.stream_vole);
            xs.extend_from_slice(xs_);
            // read mac on point
            let macs_ = &recv_64_u64(&mut self.stream_vole);
            macs.extend(macs_);
        }
        xs.truncate(n);
        macs.truncate(n);
        (xs, macs)
    }

    // --- comm. with verifier

    pub fn send_delta(&mut self, delta: u64) {
        let ctr = self.delta_ctr;
        if self.verbose {
            println!("  [chan] Sending delta{ctr}={delta}")
        };
        let n = self.stream_other.write(&delta.to_le_bytes()).unwrap();
        assert_eq!(n, 8);
        self.delta_ctr += 1;
    }

    pub fn send_mac(&mut self, mac: u64) {
        let ctr = self.mac_ctr;
        if self.verbose {
            println!("  [chan] Sending mac{ctr}={mac}")
        };
        let n = self.stream_other.write(&mac.to_le_bytes()).unwrap();
        assert_eq!(n, 8);
        self.mac_ctr += 1;
    }

    pub fn recv_challenge(&mut self) -> u64 {
        let x = recv_u64(&mut self.stream_other);
        if self.verbose {
            println!("  [chan] Received challenge={x}")
        };
        x
    }

    pub fn send_u(&mut self, x: u64) {
        if self.verbose {
            println!("  [chan] Sending u={x}")
        };
        let x = self.stream_other.write(&x.to_le_bytes()).unwrap();
        assert_eq!(x, 8);
    }

    pub fn send_v(&mut self, x: u64) {
        if self.verbose {
            println!("  [chan] Sending v={x}")
        };
        let x = self.stream_other.write(&x.to_le_bytes()).unwrap();
        assert_eq!(x, 8);
    }
}

impl VerifierTcpChannel {
    pub fn new(so: TcpStream, sv: TcpStream) -> Self {
        Self {
            stream_other: so,
            stream_vole: sv,
            delta_ctr: 0,
            mac_ctr: 0,
            verbose: false,
        }
    }

    // --- comm. with vole dealer

    pub fn recv_delta_from_dealer(&mut self) -> u64 {
        recv_u64(&mut self.stream_vole)
    }

    pub fn recv_extend_vole_z2(&mut self, _n: usize) -> Vec<u64> {
        todo!()
    }

    pub fn recv_extend_vole_zm(&mut self, n: usize) -> Vec<u64> {
        let mut keys = vec![];

        // Read in chunks of 64 u64's at a time
        for _ in 0..(n >> 6) + 1 {
            let keys_ = &recv_64_u64(&mut self.stream_vole);
            keys.extend(keys_);
        }
        keys.truncate(n);
        keys
    }

    // --- comm. with prover

    pub fn recv_delta_from_prover(&mut self) -> u64 {
        let x = recv_u64(&mut self.stream_other);
        let ctr = self.delta_ctr;
        if self.verbose {
            println!("  [chan] Received delta{ctr}={x}")
        };
        self.delta_ctr += 1;
        x
    }

    pub fn recv_mac(&mut self) -> u64 {
        let x = recv_u64(&mut self.stream_other);
        let ctr = self.mac_ctr;
        if self.verbose {
            println!("  [chan] Received mac{ctr}={x}")
        };
        self.mac_ctr += 1;
        x
    }

    pub fn send_challenge(&mut self, x: u64) {
        let x = self.stream_other.write(&x.to_le_bytes()).unwrap();
        if self.verbose {
            println!("  [chan] Sending challenge={x}")
        };
        assert_eq!(x, 8);
    }

    pub fn recv_u(&mut self) -> u64 {
        let x = recv_u64(&mut self.stream_other);
        if self.verbose {
            println!("  [chan] Received u={x}")
        };
        x
    }

    pub fn recv_v(&mut self) -> u64 {
        let x = recv_u64(&mut self.stream_other);
        if self.verbose {
            println!("  [chan] Received v={x}")
        };
        x
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

pub fn recv_64_u64(stream: &mut TcpStream) -> [u64; 64] {
    let mut buf: [u8; 8 * 64] = [0; 8 * 64];
    stream.read_exact(&mut buf).unwrap();
    transmute!(buf)
}
