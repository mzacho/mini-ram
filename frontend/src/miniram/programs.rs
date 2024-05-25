use utils::sha256;

use crate::miniram::builder::*;
use crate::miniram::lang::{reg::*, Prog, Reg};

const RES: Reg = R3;

fn mul_() -> Builder {
    let x = R1;
    let y = R2;
    let one = R4;
    let top = R5;
    let bot = R6;

    Builder::new()
        //  fetch args from memory
        .mov_c(x, 0)
        .ldr(x, x)
        .mov_c(y, 1)
        .ldr(y, y)
        //  move one to tmp register
        .mov_c(one, 1)
        .mov_r(RES, y)
        // loop:
        //  prepare address to jump to if i = 1
        .mov_r(top, PC)
        .mov_c(bot, 5)
        .add(bot, PC, bot)
        .sub(x, x, one)
        .b_z(bot)
        .add(RES, RES, y)
        .b(top)
}

/// Computes x * y
/// Precondition: x > 0
#[cfg(test)]
pub fn mul() -> Prog {
    mul_().ret_r(RES).build()
}

/// Computes x * y - z
/// Precondition: x > 0
pub fn mul_eq() -> Prog {
    let z = R1;
    mul_()
        //  fetch arg from memory
        .mov_c(z, 2)
        .ldr(z, z)
        .sub(RES, RES, z)
        .ret_r(RES)
        .build()
}

/// RET 0
pub fn const_0() -> Prog {
    Builder::new().ret_c(0).build()
}

/// MOV r2, 0; RET r2
#[cfg(test)]
pub fn mov0_ret() -> Prog {
    Builder::new().mov_c(2, 0).ret_r(2).build()
}

/// MOV r2, 42; RET r2
#[cfg(test)]
pub fn mov42_ret() -> Prog {
    Builder::new().mov_c(2, 42).ret_r(2).build()
}

/// MOV r2, b100000000000000000000; RET 0
#[cfg(test)]
pub fn mov2pow20_ret0() -> Prog {
    Builder::new().mov_c(2, 2u32.pow(20)).ret_r(3).build()
}

/// MOV r1, 0
/// STR r1, r1
/// RET 0
#[cfg(test)]
pub fn simple_str0() -> Prog {
    Builder::new().mov_c(1, 0).strr(1, 1).ret_c(0).build()
}

/// MOV r1, 1
/// STR r1, r1
/// RET 0
#[cfg(test)]
pub fn simple_str1() -> Prog {
    Builder::new().mov_c(1, 1).strr(1, 1).ret_c(0).build()
}

/// MOV r1, 3
/// MOV r2, 42
/// STR r1, r2
/// LDR r2, r1
/// RET 0
#[cfg(test)]
pub fn str3_42() -> Prog {
    Builder::new()
        .mov_c(1, 3)
        .mov_c(2, 42)
        .strr(1, 2)
        .ldr(2, 1)
        .ret_c(0)
        .build()
}

/// MOV r2, 2
/// STR r2, r2
/// LDR r2, r2
/// MOV r1, 1
/// STR r1, r1
/// LDR r1, r1
/// RET 0
#[cfg(test)]
pub fn str2_ldr2_str1_ldr1() -> Prog {
    Builder::new()
        .mov_c(2, 2)
        .strr(2, 2)
        .ldr(2, 2)
        .mov_c(1, 1)
        .strr(1, 1)
        .ldr(1, 1)
        .ret_c(0)
        .build()
}

/// MOV r2, 1
/// STR r2, r2
/// LDR r2, r2
/// MOV r1, 0
/// STR r1, r1
/// LDR r1, r1
/// RET 0
#[cfg(test)]
pub fn str1_ldr1_str0_ldr0() -> Prog {
    Builder::new()
        .mov_c(2, 1)
        .strr(2, 2)
        .ldr(2, 2)
        .mov_c(1, 0)
        .strr(1, 1)
        .ldr(1, 1)
        .ret_c(0)
        .build()
}

/// MOV r2, 1
/// LDR r2, r2
/// MOV r1, 0
/// STR r2, r2
/// STR r1, r1
/// LDR r1, r1
/// RET 0
#[cfg(test)]
pub fn str1_str0_ldr1_ldr0() -> Prog {
    Builder::new()
        .mov_c(2, 1)
        .strr(2, 2)
        .mov_c(1, 0)
        .strr(1, 1)
        .ldr(2, 2)
        .ldr(1, 1)
        .ret_c(0)
        .build()
}

/// MOV r1, 1
/// MOV r2, 2
/// STR r1, r3   <-- stores 0 at addr 1
/// STR r2, r1   <-- stores 1 at addr 2
/// LDR r4, r2
/// LDR r4, r4
/// RET r4
#[cfg(test)]
pub fn str1_str2_ldr1_ldr2() -> Prog {
    Builder::new()
        .mov_c(1, 1)
        .mov_c(2, 2)
        .strr(1, 3)
        .strr(2, 1)
        .ldr(4, 2)
        .ldr(4, 4)
        .ret_r(4)
        .build()
}

#[cfg(test)]
pub fn str0() -> Prog {
    Builder::new().mov_c(1, 1).strr(2, 1).ret_c(0).build()
}

/// MOV r1, 1
/// STR r1, r1  |
/// ...         |  n times
/// STR r1, r1  |
/// RET 0
#[cfg(test)]
pub fn simple_str_n(n: usize) -> Prog {
    let mut b = Builder::new().mov_c(1, 1);
    for _ in 0..n {
        b = b.strr(1, 1)
    }
    b.ret_c(0).build()
}

/// MOV r1, 1
/// STR r1, r1
/// LDR r1, r1
/// RET 0
#[cfg(test)]
pub fn simple_ldr() -> Prog {
    Builder::new()
        .mov_c(1, 1)
        .strr(1, 1)
        .ldr(1, 1)
        .ret_c(0)
        .build()
}

/// MOV r2, 42
/// MOV r3, r2
/// ADD r4, r15, r3
/// RET 0
#[cfg(test)]
pub fn mov42_movr3_ret0() -> Prog {
    Builder::new()
        .mov_c(2, 42)
        .mov_r(3, 2)
        .add(4, 15, 3)
        .ret_c(0)
        .build()
}

/// MOV r2, 2
/// MOV r3, 2
/// SUB r2, r2, r3
/// RET r2
#[cfg(test)]
pub fn mov_mov_sub_ret() -> Prog {
    Builder::new()
        .mov_c(2, 2)
        .mov_c(3, 2)
        .sub(2, 2, 3)
        .ret_r(2)
        .build()
}

/// MOV r2, 3
/// B r2         <-- skips next instr
/// MOV r3, 42
/// RET 0
#[cfg(test)]
pub fn b_skip() -> Prog {
    Builder::new()
        .mov_c(2, 3)
        .b(2)
        .mov_c(3, 42)
        .ret_c(0)
        .build()
}

/// MOV r2, 4
/// MOV r4, 0      <-- sets Z
/// B Z r2         <-- skips next instr
/// MOV r3, 42
/// RET 0
#[cfg(test)]
pub fn b_z_skip() -> Prog {
    Builder::new()
        .mov_c(2, 4)
        .mov_c(4, 0)
        .b_z(2)
        .mov_c(3, 42)
        .ret_c(0)
        .build()
}

#[cfg(test)]
pub fn ldr_2_args() -> Prog {
    let x = R1;
    let y = R2;

    Builder::new()
        //  fetch args from memory
        .mov_c(x, 0)
        .ldr(x, x)
        .mov_c(y, 1)
        .ldr(y, y)
        // .mov_c(z, 2)
        // .ldr(z, z)
        //  move one to tmp register
        .ret_c(0)
        .build()
}

#[cfg(test)]
pub fn and_000111_111000() -> Prog {
    Builder::new()
        .mov_c(1, 0b000111)
        .mov_c(2, 0b111000)
        .and(3, 2, 1)
        .ret_r(3)
        .build()
}

#[cfg(test)]
pub fn xor_0110_0101() -> Prog {
    Builder::new()
        .mov_c(1, 0b0110)
        .mov_c(2, 0b0101)
        .xor(3, 2, 1)
        .mov_c(4, 0b0110 ^ 0b0101)
        .sub(3, 3, 4)
        .ret_r(3)
        .build()
}

#[cfg(test)]
pub fn shr_5_1654560() -> Prog {
    Builder::new()
        .mov_c(1, 1654560)
        .mov_c(2, 1654560 >> 5)
        .shr(1, 5, 1)
        .sub(3, 2, 1)
        .ret_r(3)
        .build()
}

#[cfg(test)]
pub fn rotr_8_0xb301() -> Prog {
    assert_eq!(0xb301u32.rotate_right(8), 0x10000b3);
    Builder::new()
        .mov_c(1, 0xb301u32)
        .mov_c(2, 0x10000b3)
        .rotr(1, 8, 1)
        .sub(3, 2, 1)
        .ret_r(3)
        .build()
}

/// Returns a program which takes an input x and verifies that
/// compressing x with SHA256 (as described in FIPS 180-2) yields
/// the (hardcoded) mac.
pub fn verify_compress(mac: [u32; 16]) -> Prog {
    let adr_h = 64;
    let mut b = build_compress(false);

    for i in 0..8 {
        b = b.mov_c(4, 0) // Register 4 holds the result
            .mov_c(1, adr_h + i)
            .ldr(2, 1)
            .mov_c(3, mac[usize::try_from(i).unwrap()])
            .sub(2, 2, 3)
            .or(4, 4, 2, 6)
    }
    // Register 4 is 0 only if sha256(input)=mac
    // Add 1 to the result
    b.ret_r(4).build()
}

pub fn compress(verbose: bool) -> Prog {
    let b = build_compress(verbose);
    b.ret_c(0).build()
}

/// Returns a program which takes a 16 word input x and compresses x
/// with SHA256.
///
/// x is assumed to be padded (as described in FIPS 180-2).
pub fn build_compress(verbose: bool) -> Builder {
    let mut b_ = Builder::new();
    // hashes: 8 words
    let adr_h = 64;
    // consts: 64 words
    let adr_k = 72;
    // working vars: 64 words
    let adr_w = 136;
    // registers of local vars
    let a = 8;
    let b = 9;
    let c = 10;
    let d = 11;
    let e = 12;
    let f = 13;
    let g = 14;
    let h = 15;
    // Registers for vars T1 and T2
    let t1 = 7;
    let t2 = 6;

    b_ = add_sha256_consts(b_, adr_h, adr_k);

    // 1. Prepare message schedule W:
    // Uses r1, r2, r3, r4, r5 and r6 as scratch registers
    // todo: r6 is overwrites t1 if input is multiple blocks!
    for t in 0..64u32 {
        if t < 16 {
            // Wt = Mt
            b_ = b_.mov_c(1, t).ldr(2, 1).mov_c(1, adr_w + t).strr(1, 2)
        } else {
            // Wt = s1(Wt-2) + Wt-7 + s0(Wt-15) + Wt-16
            b_ = b_.mov_c(4, adr_w + t - 2).ldr(4, 4);
            b_ = sha256_s1(b_, 4, 5);
            b_ = b_.mov_c(4, adr_w + t - 15).ldr(4, 4);
            b_ = sha256_s0(b_, 4, 6);
            b_ = b_.mov_c(1, adr_w + t - 7).ldr(1, 1);
            b_ = b_.mov_c(2, adr_w + t - 16).ldr(2, 2);
            b_ = b_.add(2, 1, 2).add(2, 2, 5).add(2, 2, 6);
            b_ = b_.mov_c(1, adr_w + t).strr(1, 2)
        }
    }

    // 2. Initialize working vars
    b_ = b_
        .mov_c(1, adr_h)
        .ldr(a, 1)
        .mov_c(1, adr_h + 1)
        .ldr(b, 1)
        .mov_c(1, adr_h + 2)
        .ldr(c, 1)
        .mov_c(1, adr_h + 3)
        .ldr(d, 1)
        .mov_c(1, adr_h + 4)
        .ldr(e, 1)
        .mov_c(1, adr_h + 5)
        .ldr(f, 1)
        .mov_c(1, adr_h + 6)
        .ldr(g, 1)
        .mov_c(1, adr_h + 7)
        .ldr(h, 1);

    // 3. For t = 0 to 63 ...
    for t in 0..64 {
        // update t1
        b_ = sha256_sigma1(b_, e, t1);
        b_ = sha256_ch(b_, e, f, g, 2)
            .add(t1, t1, 2)
            .add(t1, t1, h)
            .mov_c(1, adr_k + t)
            .ldr(1, 1)
            .add(t1, t1, 1)
            .mov_c(1, adr_w + t)
            .ldr(1, 1)
            .add(t1, t1, 1);
        // update t2
        b_ = sha256_sigma0(b_, a, t2);
        b_ = sha256_maj(b_, a, b, c, 1).add(t2, t2, 1);
        // update a, b, ..., g, h
        b_ = b_
            .mov_r(h, g)
            .mov_r(g, f)
            .mov_r(f, e)
            .add(1, d, t1)
            .mov_r(e, 1)
            .mov_r(d, c)
            .mov_r(c, b)
            .mov_r(b, a)
            .add(1, t1, t2)
            .mov_r(a, 1)
    }

    // 4. Update hashes and print them
    b_ = b_
        .mov_c(1, adr_h)
        .ldr(2, 1)
        .add(2, 2, a);
    if verbose {b_ = b_.print(2);};
    b_ = b_.strr(1, 2)
        .mov_c(1, adr_h + 1)
        .ldr(2, 1)
        .add(2, 2, b);
    if verbose {b_ = b_.print(2);};
    b_ = b_.strr(1, 2)
        .mov_c(1, adr_h + 2)
        .ldr(2, 1)
        .add(2, 2, c);
    if verbose {b_ = b_.print(2);};
    b_ = b_.strr(1, 2)
        .mov_c(1, adr_h + 3)
        .ldr(2, 1)
        .add(2, 2, d);
    if verbose {b_ = b_.print(2);};
    b_ = b_.strr(1, 2)
        .mov_c(1, adr_h + 4)
        .ldr(2, 1)
        .add(2, 2, e);
    if verbose {b_ = b_.print(2);};
    b_ = b_.strr(1, 2)
        .mov_c(1, adr_h + 5)
        .ldr(2, 1)
        .add(2, 2, f);
    if verbose {b_ = b_.print(2);};
    b_ = b_.strr(1, 2)
        .mov_c(1, adr_h + 6)
        .ldr(2, 1)
        .add(2, 2, g);
    if verbose {b_ = b_.print(2);};
    b_ = b_.strr(1, 2)
        .mov_c(1, adr_h + 7)
        .ldr(2, 1)
        .add(2, 2, h);
    if verbose {b_ = b_.print(2);};
    b_ = b_.strr(1, 2);
    b_
}

// Initilizes constants in mem addresses 64,..,135:
// Uses r1 and r2 as scratch registers
fn add_sha256_consts(mut b: Builder, mut adr_h: u32, mut adr_k: u32) -> Builder {
    // initial hash values
    let hs = [
        0x6a09e667u32,
        0xbb67ae85u32,
        0x3c6ef372u32,
        0xa54ff53au32,
        0x510e527fu32,
        0x9b05688cu32,
        0x1f83d9abu32,
        0x5be0cd19u32,
    ];
    for h in hs {
        b = b.mov_c(1, h).mov_c(2, adr_h).strr(2, 1);
        adr_h += 1;
    }
    for k in sha256::K32 {
        b = b.mov_c(1, k).mov_c(2, adr_k).strr(2, 1);
        adr_k += 1;
    }
    b
}

// Computes s0(x) = ROTR_7(x) + ROTR_18(x) + SHR_3(x)
// Uses r1, r2, r3 as caller-save registers
fn sha256_s0(b: Builder, x: Reg, dst: Reg) -> Builder {
    b.rotr(1, 7, x)
        .rotr(2, 18, x)
        .shr(3, 3, x)
        .xor(dst, 1, 2)
        .xor(dst, dst, 3)
}

// Computes s1(x) = ROTR_17(x) + ROTR_19(x) + SHR_10(x)
// Uses r1, r2, r3 as caller-save registers
fn sha256_s1(b: Builder, x: Reg, dst: Reg) -> Builder {
    b.rotr(1, 17, x)
        .rotr(2, 19, x)
        .shr(3, 10, x)
        .xor(dst, 1, 2)
        .xor(dst, dst, 3)
}

// Computes sigma0(x) = ROTR_2(x) + ROTR_13(x) + ROTR_22(x)
// Uses r1, r2, r3 as caller-save registers
fn sha256_sigma0(b: Builder, x: Reg, dst: Reg) -> Builder {
    b.rotr(1, 2, x)
        .rotr(2, 13, x)
        .rotr(3, 22, x)
        .xor(dst, 1, 2)
        .xor(dst, dst, 3)
}

// Computes sigma1(x) = ROTR_6(x) + ROTR_11(x) + ROTR_25(x)
// Uses r1, r2, r3 as caller-save registers
fn sha256_sigma1(b: Builder, x: Reg, dst: Reg) -> Builder {
    b.rotr(1, 6, x)
        .rotr(2, 11, x)
        .rotr(3, 25, x)
        .xor(dst, 1, 2)
        .xor(dst, dst, 3)
}

// Computes ch(x, y, z) = z + (x & (y + z))
// Uses r1, r2, r3 as caller-save registers
fn sha256_ch(b: Builder, x: Reg, y: Reg, z: Reg, dst: Reg) -> Builder {
    b.xor(1, y, z).and(2, x, 1).xor(dst, 2, z)
}

// Computes maj(x, y, z) = (x & y) + (x & z) + (y & z)
// Uses r1, r2, r3 as caller-save registers
fn sha256_maj(b: Builder, x: Reg, y: Reg, z: Reg, dst: Reg) -> Builder {
    b.and(1, x, y)
        .and(2, x, z)
        .and(3, y, z)
        .xor(dst, 1, 2)
        .xor(dst, dst, 3)
}

#[test]
#[cfg(test)]
fn test_mul() {
    use crate::miniram::interpreter::interpret;
    let time_bound = 10000;
    let p = &mul();
    let args = vec![3, 4];
    let res = interpret(p, args, time_bound);
    assert_eq!(res.unwrap().0, 12);

    let args = vec![132, 45];
    let res = interpret(p, args, time_bound);
    assert_eq!(res.unwrap().0, 132 * 45);
}

#[test]
#[cfg(test)]
fn test_mul_eq() {
    use crate::miniram::interpreter::interpret;
    let time_bound = 1000;
    let p = &mul_eq();
    let args = vec![3, 4, 12];
    let res = interpret(p, args, time_bound);
    assert_eq!(res.unwrap().0, 0);

    let args = vec![31, 65, 31 * 65];
    let res = interpret(p, args, time_bound);
    assert_eq!(res.unwrap().0, 0);
}
