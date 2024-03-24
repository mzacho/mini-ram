use super::vole;
use utils::circuit::*;

use utils::channel::*;

use crate::ProofCtx;

pub fn prove64(c: Circuit<u64>, w: Vec<u64>, mut chan: ProverTcpChannel, mut ctx: ProofCtx) {
    let n_in = w.len();
    let n_mul = c.n_mul;
    let n_select = c.n_select;
    let n_select_const = c.n_select_const;
    let n_decode32 = c.n_decode32;
    let n_decode64 = c.n_decode64;
    let n_check_all_eq_but_one = c.n_check_all_eq_but_one;
    let check_mul = (n_mul > 0)
        || (n_select > 0)
        || (n_select_const > 0)
        || (n_decode32 > 0)
        || (n_decode64 > 0)
        || (n_check_all_eq_but_one > 0);

    let segments = &vole::Segments {
        n_in,
        n_mul: n_mul
            + n_select * 2
            + n_select_const
            + n_decode32 * 32
            + n_decode64 * 64
            + n_check_all_eq_but_one,
        n_mul_check: if check_mul { 1 } else { 0 },
    };

    ctx.start_time("preprocess vole");
    let voles = preprocess_vole(&mut chan, segments);
    ctx.stop_time();

    ctx.start_time("sending deltas of witness");
    for (i, (wi, xi)) in w.iter().zip(&voles.xs_in).enumerate() {
        let delta = wi.wrapping_sub(*xi);
        chan.send_delta(delta);
    }
    ctx.stop_time();

    let wires = Wires {
        cler: w,
        macs: voles.mc_in,
    };
    ctx.start_time("evaluating circuit");
    let (outputs, mult_checks, openings) = eval(&c, wires, voles.xs_mul, voles.mc_mul, &mut chan);
    ctx.stop_time();

    ctx.start_time("opening outputs");
    for mac in openings {
        chan.send_mac(mac)
    }
    ctx.stop_time();

    if check_mul {
        let x = chan.recv_challenge();

        ctx.start_time("computing a0a1");
        let (a0, a1) = compute_a0a1(x, mult_checks);
        ctx.stop_time();

        // Compute U and V by masking with rvole correlation
        let u = a0.wrapping_add(voles.mc_mul_check[0]);
        let v = a1.wrapping_add(voles.xs_mul_check[0]);

        // Send to V
        chan.send_u(u);
        chan.send_v(v);
    }

    ctx.start_time("sending opening of outputs.");
    for (val, mac) in outputs {
        // Assert that prover is honest
        assert_eq!(val, 0);
        chan.send_mac(mac);
    }
    ctx.stop_time();

    println!("Done, exiting.");
}

fn compute_a0a1(x: u64, mult_checks: Vec<A0A1>) -> (u64, u64) {
    let mut x: u64 = 0;
    let mut y: u64 = 0;
    for (t, (a0, a1)) in mult_checks.into_iter().enumerate() {
        let t = u32::try_from(t).unwrap();
        x = x.wrapping_add(a0.wrapping_mul(1)); //x.wrapping_pow(t)));
        y = y.wrapping_add(a1.wrapping_mul(1)); //y.wrapping_pow(t)));
    }
    (x, y)
}

fn preprocess_vole(chan: &mut ProverTcpChannel, segs: &vole::Segments) -> vole::CorrSender {
    let n = segs.size().try_into().unwrap();

    println!("Sending extend VOLE n={n}");
    chan.send_extend_vole_zm(n);

    let (xs_in, mc_in) = chan.recv_extend_vole_zm(segs.n_in);
    // println!("  Correlations for witness:");
    // println!("  Received xs={xs_in:?}, macs={mc_in:?}");

    let (xs_mul, mc_mul) = chan.recv_extend_vole_zm(segs.n_mul);
    // println!("  Correlations for multiplications:");
    // println!("  Received xs={xs_mul:?}, macs={mc_mul:?}");

    let (xs_mul_check, mc_mul_check) = chan.recv_extend_vole_zm(segs.n_mul_check);
    // println!("  Correlations for multiplication checks:");
    // println!("  Received xs={xs_mul_check:?}, macs={mc_mul_check:?}");

    vole::CorrSender {
        xs_in,
        mc_in,
        xs_mul,
        mc_mul,
        xs_mul_check,
        mc_mul_check,
    }
}

struct Wires {
    cler: Vec<u64>,
    macs: Vec<u64>,
}

type ValWithMac = (u64, u64);
type A0A1 = (u64, u64);
type Mac = u64;

/// Evaluation circuit on clear-text witness as well as macs
///
/// Outputs are pairs of (x, t) where x is the value on the wire and
/// t is its tag, as well as a0 and a1 for multiplication checks
fn eval(
    c: &Circuit<u64>,
    mut wires: Wires,
    xs_mul: Vec<u64>,
    mc_mul: Vec<u64>,
    chan: &mut ProverTcpChannel,
) -> (Vec<ValWithMac>, Vec<A0A1>, Vec<Mac>) {
    let gates = &c.gates;
    let consts = &c.consts;
    let n_gates = c.n_gates;

    assert_eq!(c.n_in, wires.cler.len());
    assert_eq!(c.n_in, wires.macs.len());
    //assert_eq!(c.n_mul, xs_mul.len());
    //assert_eq!(c.n_mul, mc_mul.len());
    assert_eq!(n_gates, count_ops(gates));

    let mut out = vec![];
    let mut a0a1 = vec![];
    let mut openings = vec![];

    let mut i = 0; // ctr gate
    let mut t = 0; // ctr mul
    for _ in 0..n_gates {
        let op = gates[i];
        i += 1;
        let mut res_x: u64 = 0;
        let mut res_t: u64 = 0;
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
                    res_x = res_x.wrapping_add(wires.cler[gates[i] - ARG0]);
                    res_t = res_t.wrapping_add(wires.macs[gates[i] - ARG0]);
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
            }
            OP_SUB => {
                // args: idx, idy
                // outw: x - y
                let lhs = wires.cler[gates[i] - ARG0];
                let rhs = wires.cler[gates[i + 1] - ARG0];
                //dbg!(lhs, rhs);
                res_x = lhs.wrapping_sub(rhs);

                let lhs = wires.macs[gates[i] - ARG0];
                let rhs = wires.macs[gates[i + 1] - ARG0];
                //dbg!(lhs, rhs);
                res_t = lhs.wrapping_sub(rhs);
                i += 2;
            }
            OP_MUL => {
                // args: idx, idy
                // outw: x * y
                let lhs = wires.cler[gates[i] - ARG0];
                let rhs = wires.cler[gates[i + 1] - ARG0];
                //dbg!(lhs, rhs);
                res_x = lhs.wrapping_mul(rhs);
                res_t = mc_mul[t];

                chan.send_delta(res_x.wrapping_sub(xs_mul[t]));

                let lhs_t = wires.macs[gates[i] - ARG0];
                let rhs_t = wires.macs[gates[i + 1] - ARG0];
                let a0 = lhs_t.wrapping_mul(rhs_t);
                let a1 = lhs_t
                    .wrapping_mul(rhs)
                    .wrapping_add(rhs_t.wrapping_mul(lhs))
                    .wrapping_sub(res_t);

                a0a1.push((a0, a1));

                i += 2;
                t += 1;
            }
            OP_MUL_CONST => {
                // args: idc, idx
                // outw: c * x
                let c = consts[gates[i] - ARG0];
                let x = wires.cler[gates[i + 1] - ARG0];
                let t = wires.macs[gates[i + 1] - ARG0];
                res_x = c.wrapping_mul(x);
                res_t = c.wrapping_mul(t);
                i += 2;
            }
            OP_SELECT => {
                // args: idi, idx1, idx2, ..., idxn where i <= n
                // outw: xi
                let i_ = wires.cler[gates[i] - ARG0];
                let it = wires.macs[gates[i] - ARG0];
                let mut iu: usize = i_.try_into().ok().unwrap();
                i += 1;
                res_x = wires.cler[gates[i + iu] - ARG0];
                res_t = 0;
                let mut bst: u64 = 0;
                let mut j = 0;
                while gates[i] >= ARG0 {
                    let xj = wires.cler[gates[i] - ARG0];
                    let xjt = wires.macs[gates[i] - ARG0];
                    let bj: u64 = if iu == 0 { 1 } else { 0 };

                    // Commit to bj
                    let bjt = mc_mul[t];
                    // Send delta
                    let r = xs_mul[t];
                    let d = bj.wrapping_sub(xs_mul[t]);
                    chan.send_delta(d);
                    // Add tag to sum of bj tags
                    bst = bst.wrapping_add(bjt);
                    // Commit to bj*xj
                    let bxt = mc_mul[t + 1];
                    // Send delta
                    let r = xs_mul[t + 1];
                    let d = (bj * xj).wrapping_sub(xs_mul[t + 1]);
                    chan.send_delta(d);
                    // Add tag to result
                    res_t = res_t.wrapping_add(bxt);
                    // Prove bj(i_-j) opens to 0
                    let a0 = bjt.wrapping_mul(it);
                    let a1 = bjt
                        .wrapping_mul(i_.wrapping_sub(j))
                        .wrapping_add(it.wrapping_mul(bj));
                    a0a1.push((a0, a1));

                    // Prove sharing of bj*xj is consistent
                    let a0 = bjt.wrapping_mul(xjt);
                    let a1 = bjt
                        .wrapping_mul(xj)
                        .wrapping_add(xjt.wrapping_mul(bj))
                        .wrapping_sub(bxt);
                    a0a1.push((a0, a1));

                    iu = iu.wrapping_sub(1);
                    t += 2;
                    j += 1;
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
                // Prove that sum of bs opens to 1
                openings.push(bst);
            }
            OP_SELECT_CONST => {
                // args: idi, idc1, idc2, ..., idcn where i <= n
                // outw: ci
                let i_ = wires.cler[gates[i] - ARG0];
                let it = wires.macs[gates[i] - ARG0];
                let mut iu: usize = i_.try_into().ok().unwrap();
                i += 1;
                res_x = consts[gates[i + iu] - ARG0];
                res_t = 0;
                let mut bst: u64 = 0;
                let mut j = 0;
                while gates[i] >= ARG0 {
                    let cj = consts[gates[i] - ARG0];
                    let cjt = 0;
                    let bj: u64 = if iu == 0 { 1 } else { 0 };

                    // Commit to bj
                    let bjt = mc_mul[t];
                    // Send delta
                    let r = xs_mul[t];
                    let d = bj.wrapping_sub(xs_mul[t]);
                    chan.send_delta(d);
                    // Add tag to sum of bj tags
                    bst = bst.wrapping_add(bjt);
                    // Compute tag of bj*cj
                    let bct = cj.wrapping_mul(bjt);
                    // Add tag to result
                    res_t = res_t.wrapping_add(bct);

                    // Prove bj(i_-j) opens to 0
                    let a0 = bjt.wrapping_mul(it);
                    let a1 = bjt
                        .wrapping_mul(i_.wrapping_sub(j))
                        .wrapping_add(it.wrapping_mul(bj));
                    a0a1.push((a0, a1));

                    iu = iu.wrapping_sub(1);
                    t += 1;
                    j += 1;
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
                // Prove that sum of bs opens to 1
                openings.push(bst);
            }
            OP_DECODE32 => {
                // args: x where x < 2^32
                // outw: idx1, idx2, ..., idxn s.t sum 2^{i-1}*xi
                let mut x = wires.cler[gates[i] - ARG0];
                let xt = wires.macs[gates[i] - ARG0];
                u32::try_from(x).unwrap();
                let mut sum: u64 = 0;
                for i in 0..32 {
                    let xi = u64::from(x.trailing_ones() > 0);
                    // Commit to xi
                    let xit = mc_mul[t];
                    // Send delta
                    let d = xi.wrapping_sub(xs_mul[t]);
                    chan.send_delta(d);
                    // Prove xi(1-xi) opens to 0
                    let a0 = xit.wrapping_mul(0u64.wrapping_sub(xit));
                    let a1 = xit
                        .wrapping_mul(1u64.wrapping_sub(xi))
                        .wrapping_add(0u64.wrapping_sub(xit).wrapping_mul(xi));
                    a0a1.push((a0, a1));

                    sum = sum.wrapping_add(2u64.pow(i).wrapping_mul(xit));

                    if i != 31 {
                        wires.cler.push(xi);
                        wires.macs.push(xit);
                        x >>= 1;
                        t += 1;
                    } else {
                        res_x = xi;
                        res_t = xit;
                        t += 1;
                    }
                }
                // Prove pow sum opens to x
                openings.push(sum.wrapping_sub(xt));
                i += 1;
            }
            OP_DECODE64 => {
                // args: x where x < 2^64
                // outw: idx1, idx2, ..., idxn s.t sum 2^{i-1}*xi
                let mut x = wires.cler[gates[i] - ARG0];
                let xt = wires.macs[gates[i] - ARG0];
                let mut sum: u64 = 0;
                for i in 0..64 {
                    let xi = u64::from(x.trailing_ones() > 0);
                    // Commit to xi
                    let xit = mc_mul[t];
                    // Send delta
                    let d = xi.wrapping_sub(xs_mul[t]);
                    chan.send_delta(d);
                    // Prove xi(1-xi) opens to 0
                    let a0 = xit.wrapping_mul(0u64.wrapping_sub(xit));
                    let a1 = xit
                        .wrapping_mul(1u64.wrapping_sub(xi))
                        .wrapping_add(0u64.wrapping_sub(xit).wrapping_mul(xi));
                    a0a1.push((a0, a1));

                    sum = sum.wrapping_add(2u64.pow(i).wrapping_mul(xit));

                    if i != 63 {
                        wires.cler.push(xi);
                        wires.macs.push(xit);
                        x >>= 1;
                        t += 1;
                    } else {
                        res_x = xi;
                        res_t = xit;
                        t += 1;
                    }
                }
                // Prove pow sum opens to x
                openings.push(sum.wrapping_sub(xt));
                i += 1;
            }
            OP_ENCODE4 => {
                // args: idx1, idx2, idx3, idx4
                // outw: sum 2^{i-1}*xi
                //
                // assumes xs are all bits so no overflow happens.
                res_x = 0;
                res_t = 0;
                for k in 0..4 {
                    let x = wires.cler[gates[i] - ARG0];
                    let t = wires.macs[gates[i] - ARG0];
                    assert_eq!(x * (1 - x), 0);
                    res_x = res_x.wrapping_add(2u64.pow(k).wrapping_mul(x));
                    res_t = res_t.wrapping_add(2u64.pow(k).wrapping_mul(t));
                    i += 1;
                }
            }
            OP_ENCODE8 => {
                // args: idx1, idx2, ..., idx8
                // outw: sum 2^{i-1}*xi
                //
                // assumes xs are all bits so no overflow happens.
                res_x = 0;
                res_t = 0;
                for k in 0..8 {
                    let x = wires.cler[gates[i] - ARG0];
                    let t = wires.macs[gates[i] - ARG0];
                    assert_eq!(x * (1 - x), 0);
                    res_x = res_x.wrapping_add(2u64.pow(k).wrapping_mul(x));
                    res_t = res_t.wrapping_add(2u64.pow(k).wrapping_mul(t));
                    i += 1;
                }
            }
            OP_ENCODE32 => {
                // args: idx1, idx2, ..., idx32
                // outw: sum 2^{i-1}*xi
                //
                // assumes xs are all bits so no overflow happens.
                res_x = 0;
                res_t = 0;
                for k in 0..32 {
                    let x = wires.cler[gates[i] - ARG0];
                    let t = wires.macs[gates[i] - ARG0];
                    assert_eq!(x * (1 - x), 0);
                    res_x = res_x.wrapping_add(2u64.pow(k).wrapping_mul(x));
                    res_t = res_t.wrapping_add(2u64.pow(k).wrapping_mul(t));
                    i += 1;
                }
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
                res_x = consts[gates[i] - ARG0];
                res_t = 0;
                i += 1;
            }
            OP_OUT => {
                // args: idx
                // outw: none
                // out: x
                let x = wires.cler[gates[i] - ARG0];
                let t = wires.macs[gates[i] - ARG0];
                out.push((x, t));
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
                let i_ = wires.cler[gates[i] - ARG0];
                let it = wires.macs[gates[i] - ARG0];
                let mut iu: usize = i_.try_into().ok().unwrap();
                i += 1;
                let mut j = 0;
                let mut sum: u64 = 0;
                while gates[i] >= ARG0 {
                    let bj: u64 = if iu == 0 { 0 } else { 1 };
                    let xj = wires.cler[gates[i] - ARG0];
                    let yj = wires.cler[gates[i + 1] - ARG0];
                    let xjt = wires.macs[gates[i] - ARG0];
                    let yjt = wires.macs[gates[i + 1] - ARG0];

                    // Commit to bj
                    let bjt = mc_mul[t];
                    // Send delta
                    let r = xs_mul[t];
                    let d = bj.wrapping_sub(xs_mul[t]);
                    chan.send_delta(d);
                    sum = sum.wrapping_add(bjt);

                    // Prove (bj-1)*(i-j) opens to 0
                    let a0 = bjt.wrapping_mul(it);
                    let a1 = bjt
                        .wrapping_mul(i_.wrapping_sub(j))
                        .wrapping_add(it.wrapping_mul(bj.wrapping_sub(1)));
                    a0a1.push((a0, a1));

                    // Prove bj*(xj-yj) opens to 0
                    let a0 = bjt.wrapping_mul(xjt.wrapping_sub(yjt));
                    let a1 = bjt
                        .wrapping_mul(xj.wrapping_sub(yj))
                        .wrapping_add((xjt.wrapping_sub(yjt)).wrapping_mul(bj));
                    a0a1.push((a0, a1));

                    iu = iu.wrapping_sub(1);
                    i += 2;
                    t += 1;
                    j += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
                // Prove that sum{j=1,..,n} bj = n-1
                openings.push(sum)
            }
            OP_DEBUG => {
                dbg!("here");
            }

            _ => panic!("invalid operation"),
        }
        if (op != OP_OUT) & !is_check(op) & !matches!(op, OP_DEBUG) {
            wires.cler.push(res_x);
            wires.macs.push(res_t);
        }
        // dbg!(&wires);
    }
    //pp::print(c, Some(&clr_w));
    (out, a0a1, openings)
}

/// Counts number of gates
fn count_ops(gates: &[usize]) -> usize {
    let mut res = 0;
    for i in gates {
        if *i < ARG0 {
            res += 1;
        }
    }
    res
}

/// Counts number of output gates
pub fn count_out(gates: &[usize]) -> usize {
    let mut res = 0;
    for i in gates {
        if *i == OP_OUT {
            res += 1;
        }
    }
    res
}

/// Counts number of mul or and gates
pub fn count_mul(gates: &[usize]) -> usize {
    let mut res = 0;
    for i in gates {
        if (*i == OP_MUL) | (*i == OP_AND) {
            res += 1;
        }
    }
    res
}

pub fn is_check(op: usize) -> bool {
    matches!(
        op,
        OP_CHECK_Z | OP_CHECK_EQ | OP_CHECK_AND | OP_CHECK_ALL_EQ_BUT_ONE
    )
}
