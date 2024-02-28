use crate::miniram::builder::*;
use crate::miniram::lang::{Reg::*, *};

/// Computes x * y
/// Invariant: x > 0
pub fn mul() -> Prog {
    let x = R1;
    let y = R2;
    let res = R3;
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
        .mov_r(res, y)
        // loop:
        //  prepare address to jump to if i = 1
        .mov_r(top, Reg::PC)
        .mov_c(bot, 5)
        .add(bot, Reg::PC, bot)
        .sub(x, x, one)
        .b_z(bot)
        .add(res, res, y)
        .b(top)
        .ret_r(res)
        .build()
}

#[test]
fn test_mul() {
    use crate::miniram::interpreter::interpret;
    let p = mul();
    let args = vec![3, 4];
    let res = interpret(p, args);
    assert_eq!(res.unwrap(), 12);
}
