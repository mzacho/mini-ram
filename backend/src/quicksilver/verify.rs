use utils::channel::*;
use utils::circuit::*;

use crate::quicksilver::vole;
use crate::ProofCtx;

pub fn verify64(c: Circuit<u64>, mut chan: VerifierTcpChannel, mut ctx: ProofCtx) {
    //let n_in = w.len();
    let n_gates = c.n_gates;
    let n_in = c.n_in;
    let n_mul = c.n_mul;
    let n_select = c.n_select;
    let mul_or_select = (n_mul > 0) || n_select > 0;

    let segments = &vole::Segments {
        n_in,
        n_mul: n_mul + n_select * 2,
        n_mul_check: if mul_or_select { 1} else {0}
    };

    let (delta, mut vole) = preprocess_vole(&mut chan, segments);

    println!("Receiving deltas of witness.");
    for i in 0..segments.n_in {
        let d = chan.recv_delta_from_prover();
        println!("  Received d{i}={d}");
        vole.ks_in[i] = vole.ks_in[i].wrapping_sub(delta.wrapping_mul(d));
    }
    println!("Receiving deltas of mult and select gates.");
    for i in 0..segments.n_mul {
        let d = chan.recv_delta_from_prover();
        println!("Received delta{i}={d}");
        vole.ks_mul[i] = vole.ks_mul[i].wrapping_sub(delta.wrapping_mul(d));
    }

    // Choose random challenge _after_ prover has commited to
    // output of mult. gates. This won't be send it if the
    // circuit doesen't have any multiplication or select gates
    let x = ctx.next_u64();
    if (n_mul != 0) || n_select != 0 {
        println!("Sending challenge={x}");
        chan.send_challenge(x);
    }

    let wires = Wires {
        zm: vole.ks_in,
        z2: vec![],
    };

    let (w, keys) = eval(&c, wires, delta, x, vole.ks_mul, &mut chan);
    let n = keys.len();

    if (n_mul > 0) | (n_select > 0) {
        // Assert that mul gates are consistent with input
        let u = chan.recv_u();
        let v = chan.recv_v();
        let uv = u.wrapping_sub(v.wrapping_mul(delta));
        assert_eq!(w.wrapping_add(vole.ks_mul_check[0]), uv)
    }

    println!("Receiving openings (macs) of {n} output values.");
    for key in keys {
        let mac = chan.recv_mac();
        assert_eq!(mac, key);
    }

    println!("Verifier accepts, exiting.");
}

struct Wires {
    zm: Vec<u64>,
    z2: Vec<u64>,
}

type W = u64;
type Key = u64;

fn eval(
    c: &Circuit<u64>,
    mut wires: Wires,
    delta: u64,
    challenge: u64,
    mul_keys: Vec<u64>,
    chan: &mut VerifierTcpChannel,
) -> (W, Vec<Key>) {
    let gates = &c.gates;
    let consts = &c.consts;
    let n_gates = c.n_gates;

    let mut keys = vec![];
    let mut w: u64 = 0;

    let mut i = 0; // ctr gate
    let mut t = 0; // ctr mul/ select
    for _ in 0..n_gates {
        let op = gates[i];
        i += 1;
        let mut res: u64 = 0;
        //dbg!(&op);
        match op {
            // --- binary ops
            OP_XOR => {
                // args: idx1, idx2, ..., idxn
                // outw: x1 xor x2 xor ... xn
                todo!()
                // while gates[i] >= ARG0 {
                //     res_x ^= wires.clear[gates[i] - ARG0];
                //     res_t ^= wires.macs[gates[i] - ARG0];
                //     i += 1;
                //     if i >= gates.len() {
                //         // this gate was the last one
                //         break;
                //     }
                // }
            }
            OP_AND => {
                // args: idx, idy
                // outw: x and y
                todo!()
                // let lhs = clr_w[gates[i] - ARG0];
                // let rhs = clr_w[gates[i + 1] - ARG0];
                // res = lhs & rhs;
                // i += 2;
            }
            OP_AND_CONST => {
                // args: idc, idx
                // outw: x and c
                todo!()
                // let c = consts[gates[i] - ARG0];
                // let x = clr_w[gates[i + 1] - ARG0];
                // res = c & x;
                // i += 2;
            }
            // --- arithmetic ops
            OP_ADD => {
                // args: idx1, idx2, ..., idxn
                // outw: x1 + x2 + ... xn
                while gates[i] >= ARG0 {
                    res = res.wrapping_add(wires.zm[gates[i] - ARG0]);
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
            }
            OP_SUB => {
                // args: idx, idy
                // outw: x - y
                let lhs = wires.zm[gates[i] - ARG0];
                let rhs = wires.zm[gates[i + 1] - ARG0];
                //dbg!(lhs, rhs);
                res = lhs.wrapping_sub(rhs);
                i += 2;
            }
            OP_MUL => {
                // args: idx, idy
                // outw: x * y
                let lhs = wires.zm[gates[i] - ARG0];
                let rhs = wires.zm[gates[i + 1] - ARG0];
                //dbg!(lhs, rhs);
                res = mul_keys[t];

                let t_ = u32::try_from(t).unwrap();

                let b = lhs.wrapping_mul(rhs).wrapping_add(res.wrapping_mul(delta));
                //let tmp = tmp.wrapping_mul(challenge.wrapping_pow(t_));
                w = w.wrapping_add(b);

                i += 2;
                t += 1;
            }
            OP_MUL_CONST => {
                // args: idc, idx
                // outw: c * x
                let c = consts[gates[i] - ARG0];
                let x = wires.zm[gates[i + 1] - ARG0];
                res = c.wrapping_mul(x);
                i += 2;
            }
            OP_SELECT => {
                // args: idi, idx1, idx2, ..., idxn where i <= n
                // outw: xi
                let mut kxi: u64 = 0;
                let mut kbs: u64 = 0;
                let ki = wires.zm[gates[i] - ARG0];
                res = 0;
                i += 1;
                let mut j: u64 = 0;
                while gates[i] >= ARG0 {
                    let kxj = wires.zm[gates[i] - ARG0];
                    let kbj = mul_keys[t];
                    println!("  [select] Using kb{j}={kbj}");
                    let kxjbj = mul_keys[t + 1];
                    println!("  [select] Using kx{j}b{j}={kxjbj}");
                    let kj = 0u64.wrapping_sub(delta.wrapping_mul(j));

                    // Verify bj*(i-j) = 0
                    let b = kbj.wrapping_mul(ki.wrapping_sub(kj));
                    // let tmp = tmp * x.pow(t)
                    w = w.wrapping_add(b);
                    // Verify sharing of bj*xj is consistent
                    let b = kbj
                        .wrapping_mul(kxj)
                        .wrapping_add(kxjbj.wrapping_mul(delta));

                    w = w.wrapping_add(b);

                    res = res.wrapping_add(kxjbj);
                    kbs = kbs.wrapping_add(kbj);
                    t += 2;
                    j += 1;
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
                // Receive mac of sum bj, assert that it opens to 1
                let mac = chan.recv_mac();
                println!("  [select] veriying sum bs, mac={mac}");
                assert_eq!(mac, kbs.wrapping_add(delta));
            }
            OP_SELECT_CONST => {
                // args: idi, idc1, idc2, ..., idcn where i <= n
                // outw: ci
                todo!()
                // let i_ = wires.clrr[gates[i] - ARG0];
                // let it = wires.macs[gates[i] - ARG0];
                // let i_: usize = i_.try_into().ok().unwrap();
                // i += i_ + 1;
                // res_x = consts[gates[i] - ARG0];
                // res_t = todo!();
                // while gates[i] >= ARG0 {
                //     i += 1;
                //     if i >= gates.len() {
                //         break;
                //     }
                // }
            }
            OP_DECODE32 => {
                // args: x where x < 2^32
                // outw: idx1, idx2, ..., idxn s.t sum 2^{i-1}*xi
                todo!()
                // let mut x = clr_w[gates[i] - ARG0];
                // u32::try_from(x).unwrap();
                // //dbg!(x);
                // for _ in 1..32 {
                //     res = u64::from(x.trailing_ones() > 0);
                //     clr_w.push(res);
                //     x >>= 1;
                // }
                // res = u64::from(x.trailing_ones() > 0);
                // i += 1;
            }
            OP_DECODE64 => {
                // args: x where x < 2^64
                // outw: idx1, idx2, ..., idxn s.t sum 2^{i-1}*xi
                todo!()
                // let mut x = clr_w[gates[i] - ARG0];
                // //dbg!(x);
                // for _ in 1..64 {
                //     res = u64::from(x.trailing_ones() > 0);
                //     clr_w.push(res);
                //     x >>= 1;
                // }
                // res = u64::from(x.trailing_ones() > 0);
                // i += 1;
            }
            OP_ENCODE4 => {
                // args: idx1, idx2, idx3, idx4
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                todo!()
                // for k in 0..4 {
                //     let xk = clr_w[gates[i] - ARG0];
                //     res += 2u64.pow(k) * xk;
                //     i += 1;
                // }
            }
            OP_ENCODE8 => {
                // args: idx1, idx2, ..., idx8
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                todo!()
                // for k in 0..8 {
                //     let xk = clr_w[gates[i] - ARG0];
                //     res += 2u64.pow(k) * xk;
                //     i += 1;
                // }
            }
            OP_ENCODE32 => {
                // args: idx1, idx2, ..., idx32
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                todo!()
                // for k in 0..32 {
                //     let xk = clr_w[gates[i] - ARG0];
                //     res += 2u64.pow(k) * xk;
                //     i += 1;
                // }
            }

            // --- mixed ops
            OP_CONV_A2B => {
                // assert x in Z_{2^64} is a bit
                todo!()
                // let x = clr_w[gates[i] - ARG0];
                // assert!(x < 2);
                // // move result
                // res = x;
                // i += 1;
            }
            OP_CONV_B2A => {
                // assert x in Z_{2^64} is a bit
                todo!()
                // let x = clr_w[gates[i] - ARG0];
                // // move result
                // res = x;
                // i += 1;
            }
            OP_CONST => {
                // args: idc
                // outw: c
                let c = consts[gates[i] - ARG0];
                res = 0u64.wrapping_sub(delta.wrapping_mul(c));
                i += 1;
            }
            OP_OUT => {
                // args: idx
                // outw: none
                // out: x
                let key = wires.zm[gates[i] - ARG0];
                keys.push(key);
                i += 1;
            }
            // --- verificatin ops
            //  if check fails, adds 1 to output
            //  if check succeeds, adds 0 to output
            OP_CHECK_Z => (),   // noop
            OP_CHECK_EQ => (),  // noop
            OP_CHECK_AND => (), // noop
            OP_CHECK_ALL_EQ_BUT_ONE => {
                // args: idi, idx1, idy1, idx2, idy2,..., idxn, idyn
                // asserts: xj = yj for j != i
                todo!()
                // let mut res_ = true;
                // let mut i_ = clr_w[gates[i] - ARG0];
                // // dbg!(i_);
                // i += 1;
                // while gates[i] > ARG0 {
                //     if i_ == 0 {
                //         i += 2;
                //         i_ = MAX;
                //         continue;
                //     }
                //     let x = clr_w[gates[i] - ARG0];
                //     let y = clr_w[gates[i + 1] - ARG0];
                //     // dbg!(x, y, i, gates[i], gates[i + 1]);
                //     res_ &= x == y;
                //     i += 2;
                //     i_ -= 1;
                // }
                // assert!(res_)
            }
            OP_DEBUG => {
                dbg!("here");
            }

            _ => panic!("invalid operation"),
        }
        if (op != OP_OUT) & !is_check(op) & !matches!(op, OP_DEBUG) {
            // dbg!(res);
            wires.zm.push(res);
        }
        // dbg!(&wires);
    }
    (w, keys)
}

fn preprocess_vole(
    chan: &mut VerifierTcpChannel,
    segs: &vole::Segments,
) -> (u64, vole::CorrReceiver) {
    let delta = chan.recv_delta_from_dealer();
    println!("Received delta={delta}");

    let ks_in = chan.recv_extend_vole_zm(segs.n_in.try_into().unwrap());
    println!("  Received ks_in={ks_in:?}");
    let ks_mul = chan.recv_extend_vole_zm(segs.n_mul.try_into().unwrap());
    println!("  Received ks_mul={ks_mul:?}");
    let ks_mul_check = chan.recv_extend_vole_zm(segs.n_mul_check.try_into().unwrap());
    println!("  Received ks_select={ks_mul_check:?}");

    (
        delta,
        vole::CorrReceiver {
            ks_in,
            ks_mul,
            ks_mul_check,
        },
    )
}
