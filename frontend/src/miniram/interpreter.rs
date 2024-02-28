use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::miniram::lang::*;

type Mem = HashMap<Word, Word>;
type Store = HashMap<Reg, Word>;
type Cflags = HashMap<Cond, bool>;

pub type Res<T> = Result<T, &'static str>;

pub fn interpret(prog: Prog, args: Vec<Word>) -> Res<Word> {
    let mut mem = init_mem(args);
    let mut st = init_store();
    let mut cfl = init_cflags();

    let pc = &Reg::PC;
    let mut i = fetch(&prog, st[pc])?;
    let res = loop {
        dbg!(&st, i);
        match i {
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
            Inst::ADD(dst, x, y) => {
                let v = st[x] + st[y];
                cfl.insert(Cond::Z, v == 0);
                st.insert(*dst, v)
            }
            Inst::SUB(dst, x, y) => {
                let v = st[x] - st[y];
                cfl.insert(Cond::Z, v == 0);
                st.insert(*dst, v)
            }
            Inst::MOV(dst, v) => match v {
                Val::Reg(src) => st.insert(*dst, st[src]),
                Val::Const(c) => st.insert(*dst, *c),
            },
            Inst::LDR(dst, src) => {
                let addr = st.get(src).unwrap();
                st.insert(*dst, mem[addr])
            }
            Inst::STR(dst, src) => {
                let addr = *st.get(dst).unwrap();
                mem.insert(addr, st[src])
            }
            Inst::B(cond, r) => {
                let pc_ = match cond {
                    Some(cond) => match cond {
                        Cond::Z => {
                            if cfl[&Cond::Z] {
                                st[r]
                            } else {
                                st[pc] + 1
                            }
                        }
                    },
                    None => st[r],
                };
                st.insert(*pc, pc_);
                i = fetch(&prog, st[pc])?;
                continue;
            }
            Inst::RET(v) => break v,
        };
        inc_pc(&mut st);
        i = fetch(&prog, st[pc])?;
    };

    Ok(match res {
        Val::Reg(r) => st[r],
        Val::Const(c) => *c,
    })
}

fn fetch(prog: &Prog, pc: Word) -> Res<&Inst> {
    let pc: usize = pc.try_into().unwrap();
    prog.get(pc).ok_or("stuck fetching")
}

fn inc_pc(st: &mut Store) {
    st.insert(Reg::PC, st[&Reg::PC] + 1);
}

/// Add arguments to addresses 0, 1, ..., args.len() in mem
fn init_mem(args: Vec<Word>) -> Mem {
    let mut mem = Mem::new();
    for (k, v) in args.into_iter().enumerate() {
        let k = k.try_into().unwrap();
        assert!(matches!(mem.insert(k, v), None))
    }
    mem
}

fn init_store() -> Store {
    let mut st = Store::new();
    for c in Reg::iter() {
        st.insert(c, 0);
    }
    st
}

fn init_cflags() -> Cflags {
    let mut cfl = Cflags::new();
    for f in Cond::iter() {
        cfl.insert(f, false);
    }
    cfl
}
