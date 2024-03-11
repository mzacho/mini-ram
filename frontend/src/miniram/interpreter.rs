use std::collections::HashMap;

use crate::miniram::lang::reg::*;
use crate::miniram::lang::*;

type Mem = HashMap<Word, Word>;
type Store = [Word; N_REG];
type Cflags = [bool; N_CFL];

pub type Res<T> = Result<T, &'static str>;
/// Local state of program execution. Consists of:
/// - Value of all registers
/// - todo: Value of conditional flags
pub type LocalState = (Store, Cflags);

/// Executes prog on args for maximum t steps.
///
/// Returns the result of evaluation, with all local states
/// encountered during evaluation, or an error if the time bound t
/// was exceeded.
pub fn interpret(prog: &Prog, args: Vec<Word>, t: usize) -> Res<(Word, Vec<LocalState>)> {
    let mut mem = init_mem(args);
    let mut st = init_store();
    let mut cfl = init_cflags();
    let mut sts = vec![];

    let pc = usize::from(PC);
    let mut i = fetch(prog, st[pc])?;
    let res = loop {
        dbg!(&st, &cfl, i);
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
                set_flags(&mut cfl, v);
                st[dst] = v
            }
            Inst::Sub(dst, x, y) => {
                let dst = usize::from(dst);
                let x = usize::from(x);
                let y = usize::from(y);
                let v = st[x] - st[y];
                set_flags(&mut cfl, v);
                st[dst] = v
            }
            Inst::Mov(dst, v) => {
                let dst = usize::from(dst);
                let v = match v {
                    Val::Reg(src) => st[usize::from(src)],
                    Val::Const(c) => c,
                };
                set_flags(&mut cfl, v);
                st[dst] = v
            }
            Inst::Ldr(dst, src) => {
                let dst = usize::from(dst);
                let src = usize::from(src);
                let addr = &st[src];
                let v = mem[addr];
                set_flags(&mut cfl, v);
                st[dst] = v
            }
            Inst::Str(dst, src) => {
                let dst = usize::from(dst);
                let src = usize::from(src);
                let addr = st[dst];
                set_flags(&mut cfl, addr);
                mem.insert(addr, st[src]);
            }
            Inst::B(cond, r) => {
                let pc_ = match cond {
                    Some(cond) => match cond {
                        Cond::Z => {
                            if cfl[0] {
                                st[usize::from(r)]
                            } else {
                                st[pc] + 1
                            }
                        }
                    },
                    None => st[usize::from(r)],
                };
                set_flags(&mut cfl, pc_);
                st[pc] = pc_;
                i = fetch(prog, st[pc])?;
                sts.push(record(&st, &cfl));
                continue;
            }
            Inst::Ret(v) => {
                let v = match v {
                    Val::Reg(r) => st[usize::from(r)],
                    Val::Const(c) => c,
                };
                set_flags(&mut cfl, v);
                // machine returns in r1
                st[1] = v;
                inc_pc(&mut st);
                sts.push(record(&st, &cfl));
                break v;
            }
        };
        inc_pc(&mut st);
        i = fetch(prog, st[pc])?;
        sts.push(record(&st, &cfl));
        if sts.len() >= t {
            return Err("time bound exceeded");
        }
    };

    dbg!(&st, &cfl);
    Ok((res, sts))
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

#[inline]
fn init_cflags() -> Cflags {
    [false; N_CFL]
}

/// Sets conditional flags:
///  - cfl[0] = 1  iff  v == 0  (i.e cfl[0] is the flag Z)
#[inline]
fn set_flags(cfl: &mut Cflags, v: Word) {
    cfl[0] = v == 0
}

/// Records the current local state of the program execution
/// todo: extend with cflags
#[inline]
fn record(st: &Store, cfl: &Cflags) -> LocalState {
    (*st, *cfl)
}
