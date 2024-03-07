use std::{io::Write, net::TcpStream};

use super::ZKChannel;

pub struct TcpChannel {
    stream_other: TcpStream,
    stream_vole: TcpStream,
}

impl TcpChannel {
    pub fn new(so: TcpStream, sv: TcpStream) -> Self {
        Self {
            stream_other: so,
            stream_vole: sv,
        }
    }
}

impl ZKChannel<u64> for TcpChannel {
    fn extend_vole(&mut self, n: u64) {
        let n = self.stream_vole.write(&n.to_le_bytes()).unwrap();
        assert_eq!(n, 1)
    }
    fn send_delta(&mut self, delta: u64) {
        let n = self.stream_other.write(&delta.to_le_bytes()).unwrap();
        assert_eq!(n, 8)
    }
}
