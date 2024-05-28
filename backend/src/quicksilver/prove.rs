use super::vole;
use utils::circuit::*;

use utils::channel::*;

use crate::ProofCtx;

pub fn prove32(c: Circuit<u32>, w: Vec<u32>, mut chan: ProverTcpChannel, mut ctx: ProofCtx) {
    let check_mul = (c.n_mul > 0)
        || (c.n_select_alt > 0)
        || (c.n_select_const_alt > 0)
        || (c.n_decode32 > 0)
        || (c.n_check_all_eq_pairs > 0);
    let n_openings = c.n_out + c.n_decode32;

    #[rustfmt::skip]
    let segments = &vole::Segments {
        n_in: w.len(),
        n_mul: c.n_mul
            + c.n_select_alt * 2
            + c.n_select_const_alt
            + c.n_decode32 * 32
            + c.n_check_all_eq_pairs,
        n_mul_check: if check_mul { 1 } else { 0 },
        n_openings: c.n_out
            + c.n_decode32
            + c.n_select
            + c.n_select_const
            + c.n_check_all_eq
    };

    ctx.start_time("preprocess vole");
    let voles = preprocess_vole(&mut chan, segments);
    ctx.stop_time();

    ctx.start_time("sending deltas of witness");
    for (i, (wi, xi)) in w.iter().zip(&voles.xs_in).enumerate() {
        let delta = (*wi as u128).wrapping_sub(*xi);
        chan.send_delta(delta);
    }
    ctx.stop_time();

    let wires = Wires {
        clear: w.into_iter().map(|w| w as u128).collect::<Vec<_>>(),
        macs: voles.mc_in,
    };
    ctx.start_time("evaluating circuit");
    let (outputs, mult_checks) = eval(&c, wires, voles.xs_mul, voles.mc_mul, &mut chan);
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

    ctx.start_time("sending openings");
    for (i, (x, tx)) in outputs.iter().enumerate() {
        // Assert that prover is honest
        assert_eq!(x % (1 << 32), 0);
        // Mask upper 96 bits of x by opening x + r * 2^32
        let r = voles.xs_openings[i];
        let tr = voles.mc_openings[i];
        let z = x.wrapping_add(r.wrapping_mul(1 << 32));
        let tz = tx.wrapping_add(tr.wrapping_mul(1 << 32));

        chan.send_val(z);
        chan.send_mac(tz);
    }
    ctx.stop_time();

    println!("Done, exiting.");
}

fn compute_a0a1(x: u128, mult_checks: Vec<A0A1>) -> (u128, u128) {
    let mut x: u128 = 0;
    let mut y: u128 = 0;
    for (t, (a0, a1)) in mult_checks.into_iter().enumerate() {
        let t = u128::try_from(t).unwrap();
        x = x.wrapping_add(a0.wrapping_mul(1)); //x.wrapping_pow(t)));
        y = y.wrapping_add(a1.wrapping_mul(1)); //y.wrapping_pow(t)));
    }
    (x, y)
}

fn preprocess_vole(chan: &mut ProverTcpChannel, segs: &vole::Segments) -> vole::CorrSender {
    let verbose = false;
    let n = segs.size().try_into().unwrap();

    if verbose {
        println!("Sending extend VOLE n={n}")
    };
    chan.send_extend_vole_zm(n);

    let (xs_in, mc_in) = chan.recv_extend_vole_zm(segs.n_in);
    if verbose {
        println!("  Correlations for witness:")
    };
    if verbose {
        println!("  Received xs={xs_in:?}, macs={mc_in:?}")
    };

    let (xs_openings, mc_openings) = chan.recv_extend_vole_zm(segs.n_openings);
    if verbose {
        println!("  Correlations for openings:")
    };
    if verbose {
        println!("  Received xs={xs_openings:?}, macs={mc_openings:?}")
    };

    let (xs_mul, mc_mul) = chan.recv_extend_vole_zm(segs.n_mul);
    if verbose {
        println!("  Correlations for multiplications:")
    };
    if verbose {
        println!("  Received xs={xs_mul:?}, macs={mc_mul:?}")
    };

    let (xs_mul_check, mc_mul_check) = chan.recv_extend_vole_zm(segs.n_mul_check);
    if verbose {
        println!("  Correlations for multiplication checks:")
    };
    if verbose {
        println!("  Received xs={xs_mul_check:?}, macs={mc_mul_check:?}")
    };

    vole::CorrSender {
        xs_in,
        mc_in,
        xs_openings,
        mc_openings,
        xs_mul,
        mc_mul,
        xs_mul_check,
        mc_mul_check,
    }
}

type ValWithMac = (u128, u128);
type A0A1 = (u128, u128);

struct Wires {
    clear: Vec<u128>, // values don't get reduced mod 2^32
    macs: Vec<u128>,
}

/// Evaluation circuit on clear-text witness as well as macs
///
/// Outputs are pairs of (x, t) where x is the value on the wire and
/// t is its tag, as well as a0 and a1 for multiplication checks
fn eval(
    c: &Circuit<u32>,
    mut wires: Wires,
    xs_mul: Vec<u128>,
    mc_mul: Vec<u128>,
    chan: &mut ProverTcpChannel,
) -> (Vec<ValWithMac>, Vec<A0A1>) {
    let gates = &c.gates;
    let consts = &c.consts;
    let n_gates = c.n_gates;

    let zero: u128 = 0;
    let one: u128 = 1;
    let two: u128 = 2;

    assert_eq!(c.n_in, wires.clear.len());
    assert_eq!(c.n_in, wires.macs.len());
    //assert_eq!(c.n_mul, xs_mul.len());
    //assert_eq!(c.n_mul, mc_mul.len());
    assert_eq!(n_gates, count_ops(gates));

    let mut out = vec![];
    let mut a0a1 = vec![];

    let mut i = 0; // ctr gate
    let mut t = 0; // ctr mul
    for _ in 0..n_gates {
        let op = gates[i];
        i += 1;
        let mut res_x: u128 = 0;
        let mut res_t: u128 = 0;
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
                    res_x = res_x.wrapping_add(wires.clear[gates[i] - ARG0]);
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
                let lhs = wires.clear[gates[i] - ARG0];
                let rhs = wires.clear[gates[i + 1] - ARG0];
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
                let lhs = wires.clear[gates[i] - ARG0];
                let rhs = wires.clear[gates[i + 1] - ARG0];
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
                let x = wires.clear[gates[i + 1] - ARG0];
                let t = wires.macs[gates[i + 1] - ARG0];
                res_x = (c as u128).wrapping_mul(x);
                res_t = (c as u128).wrapping_mul(t);
                i += 2;
            }
            OP_SELECT => {
                // args: idi, idx1, idx2, ..., idxn where i <= n
                // outw: xi
                let i_ = wires.clear[gates[i] - ARG0];
                let it = wires.macs[gates[i] - ARG0];
                let mut iu: usize = i_.try_into().ok().unwrap();
                i += 1;
                res_x = wires.clear[gates[i + iu] - ARG0];
                res_t = 0;
                let mut bst: u128 = 0;
                let mut bs: u128 = 0;
                let mut j = 0;
                while gates[i] >= ARG0 {
                    let xj = wires.clear[gates[i] - ARG0];
                    let xjt = wires.macs[gates[i] - ARG0];
                    let bj: u128 = if iu == 0 { 1 } else { 0 };
                    bs = bs.wrapping_add(bj);

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
                out.push((bs.wrapping_sub(1), bst));
            }
            OP_SELECT_CONST => {
                // args: idi, idc1, idc2, ..., idcn where i <= n
                // outw: ci
                let i_ = wires.clear[gates[i] - ARG0];
                let it = wires.macs[gates[i] - ARG0];
                let mut iu: usize = i_.try_into().ok().unwrap();
                i += 1;
                res_x = consts[gates[i + iu] - ARG0] as u128;
                res_t = 0;
                let mut bst: u128 = 0;
                let mut bs: u128 = 0;
                let mut j = 0;
                while gates[i] >= ARG0 {
                    let cj = consts[gates[i] - ARG0];
                    let cjt = 0;
                    let bj: u128 = if iu == 0 { 1 } else { 0 };
                    bs = bs.wrapping_add(bj);

                    // Commit to bj
                    let bjt = mc_mul[t];
                    // Send delta
                    let r = xs_mul[t];
                    let d = bj.wrapping_sub(xs_mul[t]);
                    chan.send_delta(d);
                    // Add tag to sum of bj tags
                    bst = bst.wrapping_add(bjt);
                    // Compute tag of bj*cj
                    let bct = (cj as u128).wrapping_mul(bjt);
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
                out.push((bs.wrapping_sub(1), bst));
            }
            OP_DECODE32 => {
                // args: x' where x' < 2^128
                // outw: idx1, idx2, ..., idxn s.t x = sum 2^{i-1}*xi
                //       where x = x' mod 2^32
                let mut x_ = wires.clear[gates[i] - ARG0];
                let x_init = wires.clear[gates[i] - ARG0];
                let x = x_ % (1 << 32);
                let tx_ = wires.macs[gates[i] - ARG0];
                let mut tsum: u128 = 0;
                let mut sum: u128 = 0;
                for i in 0..32 {
                    let xi = u128::from(x_.trailing_ones() > 0);
                    // Commit to xi
                    let txi = mc_mul[t];
                    // Send delta
                    let d = xi.wrapping_sub(xs_mul[t]);
                    chan.send_delta(d);
                    // Prove xi(1-xi) opens to 0
                    let a0 = txi.wrapping_mul(zero.wrapping_sub(txi));
                    let a1 = txi
                        .wrapping_mul(one.wrapping_sub(xi))
                        .wrapping_add(zero.wrapping_sub(txi).wrapping_mul(xi));
                    a0a1.push((a0, a1));

                    tsum = tsum.wrapping_add(two.pow(i).wrapping_mul(txi));
                    sum = sum.wrapping_add(two.pow(i).wrapping_mul(xi));

                    if i != 31 {
                        wires.clear.push(xi);
                        wires.macs.push(txi);
                        x_ >>= 1;
                        t += 1;
                    } else {
                        res_x = xi;
                        res_t = txi;
                        t += 1;
                    }
                }
                // Prove pow sum opens to x
                out.push((x_init.wrapping_sub(sum), tx_.wrapping_sub(tsum)));
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
                    let x = wires.clear[gates[i] - ARG0];
                    let t = wires.macs[gates[i] - ARG0];
                    assert_eq!(x * (1 - x), 0);
                    res_x = res_x.wrapping_add(2u128.pow(k).wrapping_mul(x));
                    res_t = res_t.wrapping_add(2u128.pow(k).wrapping_mul(t));
                    i += 1;
                }
            }
            OP_ENCODE5 => {
                // args: idx1, idx2, idx3, idx4, idx5
                // outw: sum 2^{i-1}*xi
                //
                // assumes xs are all bits so no overflow happens.
                res_x = 0;
                res_t = 0;
                for k in 0..5 {
                    let x = wires.clear[gates[i] - ARG0];
                    let t = wires.macs[gates[i] - ARG0];
                    assert_eq!(x * (1 - x), 0);
                    res_x = res_x.wrapping_add(2u128.pow(k).wrapping_mul(x));
                    res_t = res_t.wrapping_add(2u128.pow(k).wrapping_mul(t));
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
                    let x = wires.clear[gates[i] - ARG0];
                    let t = wires.macs[gates[i] - ARG0];
                    assert_eq!(x * (1 - x), 0);
                    res_x = res_x.wrapping_add(2u128.pow(k).wrapping_mul(x));
                    res_t = res_t.wrapping_add(2u128.pow(k).wrapping_mul(t));
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
                    let x = wires.clear[gates[i] - ARG0];
                    let t = wires.macs[gates[i] - ARG0];
                    assert_eq!(x * (1 - x), 0);
                    res_x = res_x.wrapping_add(2u128.pow(k).wrapping_mul(x));
                    res_t = res_t.wrapping_add(2u128.pow(k).wrapping_mul(t));
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
                res_x = consts[gates[i] - ARG0] as u128;
                res_t = 0;
                i += 1;
            }
            OP_OUT => {
                // args: idx
                // outw: none
                // out: x
                let x = wires.clear[gates[i] - ARG0];
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
                let i_ = wires.clear[gates[i] - ARG0];
                let it = wires.macs[gates[i] - ARG0];
                let mut iu: usize = i_.try_into().ok().unwrap();
                i += 1;
                let mut j = 0;
                let mut tsum: u128 = 0;
                let mut sum: u128 = 0;
                while gates[i] >= ARG0 {
                    let bj: u128 = if iu == 0 { 0 } else { 1 };
                    let xj = wires.clear[gates[i] - ARG0];
                    let yj = wires.clear[gates[i + 1] - ARG0];
                    let xjt = wires.macs[gates[i] - ARG0];
                    let yjt = wires.macs[gates[i + 1] - ARG0];
                    sum = sum.wrapping_add(bj);

                    // Commit to bj
                    let bjt = mc_mul[t];
                    // Send delta
                    let r = xs_mul[t];
                    let d = bj.wrapping_sub(xs_mul[t]);
                    chan.send_delta(d);
                    tsum = tsum.wrapping_add(bjt);

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
                out.push((sum.wrapping_sub(j - 1), tsum));
            }
            OP_DEBUG => {
                let msg = gates[i] - ARG0;
                dbg!(msg);
                i += 1;
            }
            OP_DEBUG_WIRE => {
                let id = gates[i] - ARG0;
                dbg!(wires.clear[id]);
                i += 1;
            }

            _ => panic!("invalid operation"),
        }
        if (op != OP_OUT) & !is_check(op) & !matches!(op, OP_DEBUG | OP_DEBUG_WIRE) {
            wires.clear.push(res_x);
            wires.macs.push(res_t);
        }
        // dbg!(&wires);
    }
    //pp::print(c, Some(&clr_w));
    (out, a0a1)
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
