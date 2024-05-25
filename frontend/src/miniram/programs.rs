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
pub fn verify_compress(mac: [u32; 8]) -> Prog {
    todo!()
}

/// Returns a program which takes a 16 word input x and compresses x
/// with SHA256.
///
/// x is assumed to be padded (as described in FIPS 180-2).
pub fn compress() -> Prog {
    let mut b = Builder::new();
    b = add_sha256_consts(b);

    // Load initial state (hash) into r8,..,r15
    for i in 0..8 {
        b = b.mov_c(1, 64 + u32::from(i)).ldr(i + 8, 1)
    }

    // Do 64 rounds of hashing
    for i in 0..64 {
        let t0 = u8::wrapping_sub(8, i) % 8;
        let t1 = u8::wrapping_sub(8 + 1, i) % 8;
        let t2 = u8::wrapping_sub(8 + 2, i) % 8;
        let t3 = u8::wrapping_sub(8 + 3, i) % 8;
        let t4 = u8::wrapping_sub(8 + 4, i) % 8;
        let t5 = u8::wrapping_sub(8 + 5, i) % 8;
        let t6 = u8::wrapping_sub(8 + 6, i) % 8;
        let t7 = u8::wrapping_sub(8 + 7, i) % 8;

        let i = u32::from(i);
        if i < 16 {
            b = sha256_rounda(b, i, t0, t1, t2, t3, t4, t5, t6, t7);
        } else {
            b = sha256_roundb(b, i, t0, t1, t2, t3, t4, t5, t6, t7);
        }
    }

    // Working variables:
    // a, b, c, d, e, f, g, h at adr 128, .., 135
    // Temporay words T1, T2 at addr 136, 137
    b.ret_c(0).build()
}

// Uses r4 as Wi
#[allow(clippy::too_many_arguments)]
fn sha256_rounda(
    b: Builder,
    i: u32,
    t0: Reg,
    t1: Reg,
    t2: Reg,
    t3: Reg,
    t4: Reg,
    t5: Reg,
    t6: Reg,
    t7: Reg,
) -> Builder {
    // Load input word
    let b = b.mov_c(4, i);
    sha256_roundtail(b, i, t0, t1, t2, t3, t4, t5, t6, t7)
}

#[allow(clippy::too_many_arguments)]
fn sha256_roundb(
    b: Builder,
    i: u32,
    t0: Reg,
    t1: Reg,
    t2: Reg,
    t3: Reg,
    t4: Reg,
    t5: Reg,
    t6: Reg,
    t7: Reg,
) -> Builder {
    b
}

#[allow(clippy::too_many_arguments)]
fn sha256_roundtail(
    b: Builder,
    i: u32,
    t0: Reg,
    t1: Reg,
    t2: Reg,
    t3: Reg,
    t4: Reg,
    t5: Reg,
    t6: Reg,
    t7: Reg,
) -> Builder {
    // Part 0
    b = sha256_
}


// Initilizes constants in mem addresses 64,..,135
// Uses r1 and r2 as scratch registers
fn add_sha256_consts(mut b: Builder) -> Builder {
    // Move initial hash value to adr 64,65,..,71
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
    let mut adr = 64u32;
    for h in hs {
        b = b.mov_c(1, h).mov_c(2, adr).strr(2, 1);
        adr += 1;
    }
    // Move K0,..,K63 to adr 72,...,135
    let mut adr = 72u32;
    for h in sha256::K32 {
        b = b.mov_c(1, h).mov_c(2, adr).strr(2, 1);
        adr += 1;
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

// Computes ch(x, y, z) = (x & z) + (x & y)
// Uses r1, r2, r3 as caller-save registers
fn sha256_ch(b: Builder, x: Reg, y: Reg, z: Reg, dst: Reg) -> Builder {
    b.and(1, x, y).and(2, x, z).xor(dst, 1, 2)
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
