use std::cmp::Eq;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::miniram::lang::reg::*;
use crate::miniram::lang::*;

type Mem = HashMap<Word, Word>;
type Store = [Word; N_REG];
type Cflags = [bool; N_CFL];

pub type Res<T> = Result<T, &'static str>;
/// Local state of program execution. Consists of:
/// - Value of all registers
/// - Value of conditional flags
pub type LocalState = (Store, Cflags);

/// Local state augmented with information on whether the current
/// instruction needs memory access
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct LocalStateAug {
    pub st: LocalState,
    pub ma: MemAccess,
    pub step: u64,
}

impl Ord for LocalStateAug {
    /// States whose operations access memory are ordered before
    /// states whose operations are arithmetic or moves etc.
    ///
    /// If both operations access memory, then the states are
    /// ordered according to 1) memory address (lower addresses are
    /// ordered before higher addresses) and 2) step index
    /// (i.e timestamp, lower timestamp is ordered before higher
    /// timestamps).
    ///
    /// Operations that don't access memory are ordered acceording
    /// to their timestamp.
    ///
    /// Examples:
    ///
    /// Prog:
    ///
    ///  ADD r1, r2, r3  ,   S1
    ///  MOV r2, r1      ,   S2
    ///
    /// is sorted as S1, S2 as memory is not touched.
    ///
    /// Prog:
    ///
    ///  ADD r1, r2, r3  ,   S1
    ///  LDR r2, r1      ,   S2
    ///  STR r1, r2      ,   S3
    ///
    /// is sorted as S2, S3, S1 as the memory operations are on the
    /// same address, and thus are sorted according to their step index
    /// (before the arithmetic operation).
    ///
    /// Prog:
    ///
    ///  MOV r1, 1       ,   S1
    ///  MOV r2, 2       ,   S2
    ///  LDR r3, r2      ,   S3
    ///  STR r1, r4      ,   S4
    ///
    /// is sorted as S4, S3, S1, S2 as the memory operations are on
    /// different addresses, and the address of STR is less than the
    /// address of the LDR operation.
    fn cmp(&self, other: &Self) -> Ordering {
        use MemAccess::*;
        if matches!((self.ma, other.ma), (None, None)) {
            self.step.cmp(&other.step)
        } else if matches!((self.ma, other.ma), (None, _)) {
            Ordering::Less
        } else if matches!((self.ma, other.ma), (_, None)) {
            Ordering::Greater
        } else {
            // both instructions access memory,
            // sort according to location.
            let addr1 = match self.ma {
                Read { addr, .. } | Write { addr, .. } => addr,
                None => panic!("unreachable"),
            };
            let addr2 = match other.ma {
                Read { addr, .. } | Write { addr, .. } => addr,
                None => panic!("unreachable"),
            };
            addr1.cmp(&addr2)
        }
    }
}

impl PartialOrd for LocalStateAug {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MemAccess {
    None,
    Read { addr: Word, val: Word },
    Write { addr: Word, val: Word },
}

/// Executes prog on args for maximum t steps.
///
/// Returns the result of evaluation, with all local states
/// encountered during evaluation, or an error if the time bound t
/// was exceeded.
pub fn interpret(prog: &Prog, args: Vec<Word>, t: usize) -> Res<(Word, Vec<LocalStateAug>)> {
    let mut mem = init_mem(args);
    let mut st = init_store();
    let mut cfl = init_cflags();
    let mut sts = vec![];

    let pc = usize::from(PC);
    let mut i = fetch(prog, st[pc])?;
    let res = loop {
        // dbg!(&st, &cfl, i);
        let ma = match *i {
            Inst::And(dst, x, y) => {
                let dst = usize::from(dst);
                let x = usize::from(x);
                let y = usize::from(y);
                let v = st[x] & st[y];
                st[dst] = v;
                set_flags(&mut cfl, v);
                MemAccess::None
            }
            Inst::Xor(dst, x, y) => {
                let dst = usize::from(dst);
                let x = usize::from(x);
                let y = usize::from(y);
                let v = st[x] ^ st[y];
                st[dst] = v;
                set_flags(&mut cfl, v);
                MemAccess::None
            }
            Inst::Shr(dst, x, y) => {
                let dst = usize::from(dst);
                let y = usize::from(y);
                let v = st[y] >> x;
                st[dst] = v;
                set_flags(&mut cfl, v);
                MemAccess::None
            }
            Inst::Rotr(dst, x, y) => {
                let dst = usize::from(dst);
                let y = usize::from(y);
                let v = st[y].rotate_right(x);
                st[dst] = v;
                set_flags(&mut cfl, v);
                MemAccess::None
            }
            Inst::Add(dst, x, y) => {
                let dst = usize::from(dst);
                let x = usize::from(x);
                let y = usize::from(y);
                let v = st[x] + st[y];
                set_flags(&mut cfl, v);
                st[dst] = v;
                MemAccess::None
            }
            Inst::Sub(dst, x, y) => {
                let dst = usize::from(dst);
                let x = usize::from(x);
                let y = usize::from(y);
                let v = st[x].wrapping_sub(st[y]);
                set_flags(&mut cfl, v);
                st[dst] = v;
                MemAccess::None
            }
            Inst::Mov(dst, v) => {
                let dst = usize::from(dst);
                let v = match v {
                    Val::Reg(src) => st[usize::from(src)],
                    Val::Const(c) => c,
                };
                set_flags(&mut cfl, v);
                st[dst] = v;
                MemAccess::None
            }
            Inst::Ldr(dst, src) => {
                let dst = usize::from(dst);
                let src = usize::from(src);
                let addr = st[src];
                let val = mem[&addr];
                set_flags(&mut cfl, val);
                st[dst] = val;
                MemAccess::Read { addr, val }
            }
            Inst::Str(dst, src) => {
                let dst = usize::from(dst);
                let src = usize::from(src);
                let addr = st[dst];
                let val = st[src];
                //set_flags(&mut cfl, addr);
                mem.insert(addr, val);
                MemAccess::Write { addr, val }
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
                sts.push(record(&st, &cfl, MemAccess::None, sts.len()));
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
                //inc_pc(&mut st);
                sts.push(record(&st, &cfl, MemAccess::None, sts.len()));
                break v;
            }
            Inst::Print(r) => {
                let x = st[usize::from(r)];
                println!("{x}");
                MemAccess::None
            }
        };
        inc_pc(&mut st);
        i = fetch(prog, st[pc])?;
        sts.push(record(&st, &cfl, ma, sts.len()));
        if sts.len() >= t {
            return Err("time bound exceeded");
        }
    };

    // dbg!(&st, &cfl);
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

/// Add arguments to addresses 0, 1, ..., args.len()-1 in mem
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
#[inline]
fn record(st: &Store, cfl: &Cflags, ma: MemAccess, step: usize) -> LocalStateAug {
    LocalStateAug {
        st: (*st, *cfl),
        ma,
        step: u64::try_from(step).unwrap(),
    }
}
