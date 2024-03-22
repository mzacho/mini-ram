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
#[cfg(test)]
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
