use zerocopy::transmute;

/// Pads a message x according to FIPS 180-2.
///
/// x is assumed to have a max length (in bytes) of 55, so the
/// actual compression of x will only use 1 round of hashing.
pub fn pad(msg: &str) -> Vec<u32> {
    // MiniRAM memory is 2^32 bits i.e 2^29 words
    // 2^3 + 2^7 words used for consts in SHA256, i.e they must also
    // fit in memory
    let n_blocks = if msg.len() >= usize::pow(2, 29) - usize::pow(2, 3) - usize::pow(2, 7) {
        panic!("msg too long")
    } else {
        // last 8 bytes reserved to encode msg length
        //      1 byte reserverd to prepend 1 to msg
        usize::max(1, (msg.len() + 9).div_ceil(64))
    };

    let l = msg.len();
    let mut bytes = vec![0u8; 64 * n_blocks];
    for (i, b) in msg.as_bytes().iter().enumerate() {
        bytes[i] = *b;
    }
    bytes[l] = 0x80; // add 1 to the end of the string

    let l = l * 8;
    let l: [u8; 8] = u64::try_from(l).unwrap().to_be_bytes();
    let last_block = n_blocks * 64 - 8;
    for (i, b) in l.iter().enumerate() {
        bytes[last_block + i] = *b;
    }
    let mut res = vec![0u32; n_blocks * 16];
    for i in 0..n_blocks * 16 {
        let block = [
            bytes[i * 4],
            bytes[i * 4 + 1],
            bytes[i * 4 + 2],
            bytes[i * 4 + 3],
        ];
        res[i] = u32::from_be(transmute!(block));
    }
    res
}

pub fn parse_mac(mac: &str) -> [u32; 16] {
    let mut bytes = [0; 64];
    for (i, x) in hex::decode(mac).unwrap().iter().enumerate() {
        bytes[i] = *x;
    }
    let ints: [u32; 16] = transmute!(bytes);
    let mut res = [0u32; 16];
    for (i, x) in ints.iter().enumerate() {
        res[i] = u32::from_be(*x);
    }
    res
}

/// Constants necessary for SHA-256 family of digests.
pub const K32: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];
