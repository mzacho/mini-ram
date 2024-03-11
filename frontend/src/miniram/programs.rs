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
/// Invariant: x > 0
#[cfg(test)]
pub fn mul() -> Prog {
    mul_().ret_r(RES).build()
}

/// Computes x * y - z
/// Invariant: x > 0, x * y > z
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
