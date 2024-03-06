use crate::miniram::builder::*;
use crate::miniram::lang::{Reg::*, *};

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
        .mov_r(top, Reg::PC)
        .mov_c(bot, 5)
        .add(bot, Reg::PC, bot)
        .sub(x, x, one)
        .b_z(bot)
        .add(RES, RES, y)
        .b(top)
}

/// Computes x * y
/// Invariant: x > 0
pub fn mul() -> Prog {
    mul_()
        .ret_r(RES)
        .build()
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

#[test]
fn test_mul() {
    use crate::miniram::interpreter::interpret;
    let p = mul();
    let args = vec![3, 4];
    let res = interpret(p, args, 42);
    assert_eq!(res.unwrap().0, 12);
}

#[test]
fn test_mul_eq() {
    use crate::miniram::interpreter::interpret;
    let p = mul_eq();
    let args = vec![3, 4, 12];
    let res = interpret(p, args, 42);
    assert_eq!(res.unwrap().0, 0);
}
