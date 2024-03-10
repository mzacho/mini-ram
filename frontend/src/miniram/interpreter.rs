use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::miniram::lang::reg::*;
use crate::miniram::lang::*;

type Mem = HashMap<Word, Word>;
type Store = [Word; N_REG];
type Cflags = HashMap<Cond, bool>;

pub type Res<T> = Result<T, &'static str>;
/// Local state of program execution. Consists of:
/// - Value of all registers
/// - todo: Value of conditional flags
pub type LocalState = Store;

/// Executes prog on args for maximum t steps.
///
/// Returns the result of evaluation, with all local states
/// encountered during evaluation, or an error if the time bound t
/// was exceeded.
pub fn interpret(prog: &Prog, args: Vec<Word>, t: usize) -> Res<(Word, Vec<LocalState>)> {
    let mut mem = init_mem(args);
    let mut st = init_store();
    let mut cfl = init_cflags();
    let mut sts = vec![st];

    let pc = usize::from(PC);
    let mut i = fetch(prog, st[pc])?;
    let res = loop {
        dbg!(&st, i);
        match *i {
            // Inst::AND(dst, x, y) => {
            //     let v = st[x] ^ st[y];
            //     cfl.insert(Cond::Z, v == 0);
            //     st.insert(*dst, v)
            // }
            // Inst::OR(dst, x, y) => {
            //     let v = st[x] | st[y];
            //     cfl.insert(Cond::Z, v == 0);
            //     st.insert(*dst, v)
            // }
            Inst::Add(dst, x, y) => {
                let dst = usize::from(dst);
                let x = usize::from(x);
                let y = usize::from(y);
                let v = st[x] + st[y];
                cfl.insert(Cond::Z, v == 0);
                st[dst] = v
            }
            Inst::Sub(dst, x, y) => {
                let dst = usize::from(dst);
                let x = usize::from(x);
                let y = usize::from(y);
                let v = st[x] - st[y];
                cfl.insert(Cond::Z, v == 0);
                st[dst] = v
            }
            Inst::Mov(dst, v) => {
                let dst = usize::from(dst);
                match v {
                    Val::Reg(src) => st[dst] = st[usize::from(src)],
                    Val::Const(c) => st[dst] = c,
                }
            }
            Inst::Ldr(dst, src) => {
                let dst = usize::from(dst);
                let src = usize::from(src);
                let addr = &st[src];
                st[dst] = mem[addr]
            }
            Inst::Str(dst, src) => {
                let dst = usize::from(dst);
                let src = usize::from(src);
                let addr = st[dst];
                mem.insert(addr, st[src]);
            }
            Inst::B(cond, r) => {
                let pc_ = match cond {
                    Some(cond) => match cond {
                        Cond::Z => {
                            if cfl[&Cond::Z] {
                                st[usize::from(r)]
                            } else {
                                st[pc] + 1
                            }
                        }
                    },
                    None => st[usize::from(r)],
                };
                st[pc] = pc_;
                i = fetch(prog, st[pc])?;
                continue;
            }
            Inst::Ret(v) => {
                let x = match v {
                    Val::Reg(r) => st[usize::from(r)],
                    Val::Const(c) => c,
                };
                // machine returns in r1
                st[1] = x;
                sts.push(record(&st));
                break v;
            }
        };
        inc_pc(&mut st);
        i = fetch(prog, st[pc])?;
        sts.push(record(&st));
        if sts.len() > t {
            return Err("time bound exceeded");
        }
    };

    Ok((
        match res {
            Val::Reg(r) => st[usize::from(r)],
            Val::Const(c) => c,
        },
        sts,
    ))
}

fn fetch(prog: &Prog, pc: Word) -> Res<&Inst> {
    let pc: usize = pc.try_into().unwrap();
    prog.get(pc).ok_or("stuck fetching")
}

#[inline]
fn inc_pc(st: &mut Store) {
    st[usize::from(PC)] += 1
}

/// Add arguments to addresses 0, 1, ..., args.len() in mem
fn init_mem(args: Vec<Word>) -> Mem {
    let mut mem = Mem::new();
    for (k, v) in args.into_iter().enumerate() {
        let k = k.try_into().unwrap();
        assert!(mem.insert(k, v).is_none())
    }
    mem
}

#[inline]
fn init_store() -> Store {
    [0; N_REG]
}

fn init_cflags() -> Cflags {
    let mut cfl = Cflags::new();
    for f in Cond::iter() {
        cfl.insert(f, false);
    }
    cfl
}

/// Records the current local state of the program execution
/// todo: extend with cflags
#[inline]
fn record(st: &Store) -> LocalState {
    *st
}
