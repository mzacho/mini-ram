use zerocopy::transmute;

use std::{
    io::{Read, Write},
    net::TcpStream,
};

const DEBUG: bool = false;

pub struct ProverTcpChannel {
    stream_vole: TcpStream,
    stream_other: TcpStream,
    delta_ctr: usize,
    mac_ctr: usize,
    val_ctr: usize,
    verbose: bool,
}

pub struct VerifierTcpChannel {
    stream_vole: TcpStream,
    stream_other: TcpStream,
    delta_ctr: usize,
    mac_ctr: usize,
    val_ctr: usize,
    verbose: bool,
}

impl ProverTcpChannel {
    pub fn new(so: TcpStream, sv: TcpStream) -> Self {
        Self {
            stream_other: so,
            stream_vole: sv,
            delta_ctr: 0,
            mac_ctr: 0,
            val_ctr: 0,
            verbose: DEBUG,
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
    pub fn recv_extend_vole_zm(&mut self, n: usize) -> (Vec<u128>, Vec<u128>) {
        let mut xs = vec![];
        let mut macs = vec![];
        if n == 0 {
            return (xs, macs);
        }

        // Read in chunks of 32 u128's at a time
        for _ in 0..(n / 32) + 1 {
            // read point of evaluation
            let xs_ = &recv_32_u128(&mut self.stream_vole);
            xs.extend_from_slice(xs_);
            // read mac on point
            let macs_ = &recv_32_u128(&mut self.stream_vole);
            macs.extend(macs_);
        }
        xs.truncate(n);
        macs.truncate(n);
        (xs, macs)
    }

    // --- comm. with verifier

    pub fn send_delta(&mut self, delta: u128) {
        let ctr = self.delta_ctr;
        if self.verbose {
            println!("  [chan] Sending delta{ctr}={delta}")
        };
        let n = self.stream_other.write(&delta.to_le_bytes()).unwrap();
        assert_eq!(n, std::mem::size_of::<u128>());
        self.delta_ctr += 1;
    }

    pub fn send_mac(&mut self, mac: u128) {
        let ctr = self.mac_ctr;
        if self.verbose {
            println!("  [chan] Sending mac{ctr}={mac}")
        };
        let n = self.stream_other.write(&mac.to_le_bytes()).unwrap();
        assert_eq!(n, std::mem::size_of::<u128>());
        self.mac_ctr += 1;
    }

    pub fn send_val(&mut self, val: u128) {
        let ctr = self.val_ctr;
        if self.verbose {
            println!("  [chan] Sending val{ctr}={val}")
        };
        let n = self.stream_other.write(&val.to_le_bytes()).unwrap();
        assert_eq!(n, std::mem::size_of::<u128>());
        self.val_ctr += 1;
    }

    pub fn recv_challenge(&mut self) -> u128 {
        let x = recv_u128(&mut self.stream_other);
        if self.verbose {
            println!("  [chan] Received challenge={x}")
        };
        x
    }

    pub fn send_u(&mut self, x: u128) {
        if self.verbose {
            println!("  [chan] Sending u={x}")
        };
        let x = self.stream_other.write(&x.to_le_bytes()).unwrap();
        assert_eq!(x, std::mem::size_of::<u128>());
    }

    pub fn send_v(&mut self, x: u128) {
        if self.verbose {
            println!("  [chan] Sending v={x}")
        };
        let x = self.stream_other.write(&x.to_le_bytes()).unwrap();
        assert_eq!(x, std::mem::size_of::<u128>());
    }
}

impl VerifierTcpChannel {
    pub fn new(so: TcpStream, sv: TcpStream) -> Self {
        Self {
            stream_other: so,
            stream_vole: sv,
            delta_ctr: 0,
            mac_ctr: 0,
            val_ctr: 0,
            verbose: DEBUG,
        }
    }

    // --- comm. with vole dealer

    pub fn recv_delta_from_dealer(&mut self) -> u128 {
        recv_u128(&mut self.stream_vole)
    }

    pub fn recv_extend_vole_z2(&mut self, _n: usize) -> Vec<u64> {
        todo!()
    }

    pub fn recv_extend_vole_zm(&mut self, n: usize) -> Vec<u128> {
        let mut keys = vec![];
        if n == 0 {
            return keys;
        }

        // Read in chunks of 32 u128's at a time
        for _ in 0..(n / 32) + 1 {
            let keys_ = &recv_32_u128(&mut self.stream_vole);
            keys.extend(keys_);
        }
        keys.truncate(n);
        keys
    }

    // --- comm. with prover

    pub fn recv_delta_from_prover(&mut self) -> u128 {
        let x = recv_u128(&mut self.stream_other);
        let ctr = self.delta_ctr;
        if self.verbose {
            println!("  [chan] Received delta{ctr}={x}")
        };
        self.delta_ctr += 1;
        x
    }

    pub fn recv_mac(&mut self) -> u128 {
        let x = recv_u128(&mut self.stream_other);
        let ctr = self.mac_ctr;
        if self.verbose {
            println!("  [chan] Received mac{ctr}={x}")
        };
        self.mac_ctr += 1;
        x
    }

    pub fn recv_val(&mut self) -> u128 {
        let x = recv_u128(&mut self.stream_other);
        let ctr = self.val_ctr;
        if self.verbose {
            println!("  [chan] Received val{ctr}={x}")
        };
        self.val_ctr += 1;
        x
    }

    pub fn send_challenge(&mut self, x: u128) {
        if self.verbose {
            println!("  [chan] Sending challenge={x}")
        };
        let n = self.stream_other.write(&x.to_le_bytes()).unwrap();
        assert_eq!(n, std::mem::size_of::<u128>());
    }

    pub fn recv_u(&mut self) -> u128 {
        let x = recv_u128(&mut self.stream_other);
        if self.verbose {
            println!("  [chan] Received u={x}")
        };
        x
    }

    pub fn recv_v(&mut self) -> u128 {
        let x = recv_u128(&mut self.stream_other);
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

pub fn recv_u128(stream: &mut TcpStream) -> u128 {
    let mut buf: [u8; 16] = [0; 16];
    let mut n = 0;
    loop {
        n += stream.read(&mut buf[n..]).unwrap();
        if n == 16 {
            break;
        }
    }
    u128::from_le_bytes(buf)
}

pub fn recv_64_u64(stream: &mut TcpStream) -> [u64; 64] {
    let mut buf: [u8; 8 * 64] = [0; 8 * 64];
    stream.read_exact(&mut buf).unwrap();
    transmute!(buf)
}

pub fn recv_32_u128(stream: &mut TcpStream) -> [u128; 32] {
    let mut buf: [u8; 8 * 64] = [0; 8 * 64];
    stream.read_exact(&mut buf).unwrap();
    transmute!(buf)
}
