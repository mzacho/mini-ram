use utils::channel::*;
use utils::circuit::*;

use crate::quicksilver::vole;
use crate::ProofCtx;

pub fn verify32(c: Circuit<u32>, mut chan: VerifierTcpChannel, mut ctx: ProofCtx) {
    let check_mul = (c.n_mul > 0)
        || (c.n_select_alt > 0)
        || (c.n_select_const_alt > 0)
        || (c.n_decode32 > 0)
        || (c.n_check_all_eq_but_one > 0);
    let n_openings = c.n_out + c.n_decode32;

    #[rustfmt::skip]
    let segments = &vole::Segments {
        n_in: c.n_in,
        n_mul: c.n_mul
            + c.n_select_alt * 2
            + c.n_select_const_alt
            + c.n_decode32 * 32
            + c.n_check_all_eq_but_one,
        n_mul_check: if check_mul { 1 } else { 0 },
        n_openings: c.n_out
            + c.n_decode32
            + c.n_select
            + c.n_select_const
    };

    ctx.start_time("preprocess vole");
    let (delta, mut vole) = preprocess_vole(&mut chan, segments);
    ctx.stop_time();

    ctx.start_time("receiving deltas of witness");
    for i in 0..segments.n_in {
        let d = chan.recv_delta_from_prover();
        vole.ks_in[i] = vole.ks_in[i].wrapping_sub(delta.wrapping_mul(d));
    }
    ctx.stop_time();

    ctx.start_time("receiving deltas of mult and select gates.");
    for i in 0..segments.n_mul {
        let d = chan.recv_delta_from_prover();
        vole.ks_mul[i] = vole.ks_mul[i].wrapping_sub(delta.wrapping_mul(d));
    }
    ctx.stop_time();

    // Choose random challenge _after_ prover has commited to
    // output of mult. gates. This is unused in eval if circuit
    // doesen't have any multiplication or select gates and therefore
    // won't be send it this case.
    let x = ctx.next_u128();
    if check_mul {
        chan.send_challenge(x);
    }

    let wires = Wires {
        zm: vole.ks_in,
        // z2: vec![],
    };

    ctx.start_time("evaluating circuit");
    let (w, keys) = eval(&c, wires, delta, x, vole.ks_mul, &mut chan);
    let n = keys.len();
    ctx.stop_time();

    if check_mul {
        // Assert that mul gates are consistent with input
        let u = chan.recv_u();
        let v = chan.recv_v();
        let uv = u.wrapping_sub(v.wrapping_mul(delta));
        assert_eq!(w.wrapping_add(vole.ks_mul_check[0]), uv);
    }

    println!("Receiving openings (macs) of {n} output values.");
    ctx.start_time("openings");
    for (i, kx) in keys.into_iter().enumerate() {
        let z = chan.recv_val();
        let tz = chan.recv_mac();
        let kr = vole.ks_openings[i];

        assert_eq!(
            tz,
            delta
                .wrapping_mul(z)
                .wrapping_add(kx)
                .wrapping_add(kr.wrapping_mul(1 << 32))
        );
    }
    ctx.stop_time();

    println!("Verifier accepts, exiting.");
}

#[allow(dead_code)]
struct Wires {
    zm: Vec<u128>,
    // z2: Vec<u64>,
}

type W = u128;
type Key = u128;

fn eval(
    c: &Circuit<u32>,
    mut wires: Wires,
    delta: u128,
    challenge: u128,
    mul_keys: Vec<u128>,
    chan: &mut VerifierTcpChannel,
) -> (W, Vec<Key>) {
    let gates = &c.gates;
    let consts = &c.consts;
    let n_gates = c.n_gates;

    let mut outputs = vec![];
    let mut w: Key = 0;

    let mut i = 0; // ctr gate
    let mut t = 0; // ctr mul/ select
    for _ in 0..n_gates {
        let op = gates[i];
        i += 1;
        let mut res: Key = 0;
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
                res = (c as Key).wrapping_mul(x);
                i += 2;
            }
            OP_SELECT => {
                // args: idi, idx1, idx2, ..., idxn where i <= n
                // outw: xi
                let mut kbs: Key = 0;
                let ki = wires.zm[gates[i] - ARG0];
                res = 0;
                i += 1;
                let mut j: Key = 0;
                while gates[i] >= ARG0 {
                    let kxj = wires.zm[gates[i] - ARG0];
                    let kbj = mul_keys[t];
                    let kxjbj = mul_keys[t + 1];
                    let kj = 0u128.wrapping_sub(delta.wrapping_mul(j));

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
                // Assert that sum bjs opens to 1
                outputs.push(kbs.wrapping_add(delta))
            }
            OP_SELECT_CONST => {
                // args: idi, idc1, idc2, ..., idcn where i <= n
                // outw: ci
                let mut kbs: Key = 0;
                let ki = wires.zm[gates[i] - ARG0];
                res = 0;
                i += 1;
                let mut j: Key = 0;
                while gates[i] >= ARG0 {
                    let cj = consts[gates[i] - ARG0];
                    let kcj = 0u128.wrapping_sub(delta.wrapping_mul(cj as Key));
                    let kbj = mul_keys[t];
                    let kcjbj = (cj as Key).wrapping_mul(kbj);
                    let kj = 0u128.wrapping_sub(delta.wrapping_mul(j));

                    // Verify bj*(i-j) = 0
                    let b = kbj.wrapping_mul(ki.wrapping_sub(kj));
                    // let tmp = tmp * x.pow(t)
                    w = w.wrapping_add(b);

                    res = res.wrapping_add(kcjbj);
                    kbs = kbs.wrapping_add(kbj);
                    t += 1;
                    j += 1;
                    i += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
                // Assert that sum bjs opens to 1
                outputs.push(kbs.wrapping_add(delta))
            }
            OP_DECODE32 => {
                // args: x where x < 2^32
                // outw: idx1, idx2, ..., idxn s.t sum 2^{i-1}*xi
                let kx = wires.zm[gates[i] - ARG0];
                let mut sum: Key = 0;
                for i in 0..32 {
                    let kxi = mul_keys[t];

                    // Verify xi(1-xi) = 0
                    let k1 = 0u128.wrapping_sub(delta);
                    let b = kxi.wrapping_mul(k1.wrapping_sub(kxi));
                    w = w.wrapping_add(b);

                    sum = sum.wrapping_add(2u128.pow(i).wrapping_mul(kxi));

                    if i != 31 {
                        wires.zm.push(kxi);
                        t += 1;
                    } else {
                        res = kxi;
                        t += 1;
                    }
                }
                // Verify x - sum opens to 0
                outputs.push(kx.wrapping_sub(sum));
                i += 1;
            }
            OP_DECODE64 => {
                panic!("unsupported")
            }
            OP_ENCODE4 => {
                // args: idx1, idx2, idx3, idx4
                // outw: sum 2^{i-1}*xi
                //
                // assumes xs are all bits so no overflow happens.
                res = 0;
                for k in 0..4 {
                    let key = wires.zm[gates[i] - ARG0];
                    res = res.wrapping_add(2u128.pow(k).wrapping_mul(key));
                    i += 1;
                }
            }
            OP_ENCODE5 => {
                // args: idx1, idx2, idx3, idx4, idx5
                // outw: sum 2^{i-1}*xi
                //
                // assumes xs are all bits so no overflow happens.
                res = 0;
                for k in 0..5 {
                    let key = wires.zm[gates[i] - ARG0];
                    res = res.wrapping_add(2u128.pow(k).wrapping_mul(key));
                    i += 1;
                }
            }

            OP_ENCODE8 => {
                // args: idx1, idx2, ..., idx8
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                res = 0;
                for k in 0..8 {
                    let key = wires.zm[gates[i] - ARG0];
                    res = res.wrapping_add(2u128.pow(k).wrapping_mul(key));
                    i += 1;
                }
            }
            OP_ENCODE32 => {
                // args: idx1, idx2, ..., idx32
                // outw: sum 2^{i-1}*xi
                //
                // assumse xs are all bits so no overflow happens.
                res = 0;
                for k in 0..32 {
                    let key = wires.zm[gates[i] - ARG0];
                    res = res.wrapping_add(2u128.pow(k).wrapping_mul(key));
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
                let c = consts[gates[i] - ARG0];
                res = 0u128.wrapping_sub(delta.wrapping_mul(c as Key));
                i += 1;
            }
            OP_OUT => {
                // args: idx
                // outw: none
                // out: x
                let key = wires.zm[gates[i] - ARG0];
                outputs.push(key);
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
                let ki = wires.zm[gates[i] - ARG0];
                let mut sum: Key = 0;
                let mut j: Key = 0;
                i += 1;
                while gates[i] >= ARG0 {
                    let kxj = wires.zm[gates[i] - ARG0];
                    let kyj = wires.zm[gates[i + 1] - ARG0];
                    let kj = 0u128.wrapping_sub(delta.wrapping_mul(j));
                    let k1 = 0u128.wrapping_sub(delta);
                    let kbj = mul_keys[t];

                    // Verify that (bj-1)*(i-j) opens to 0
                    let b = (kbj.wrapping_sub(k1)).wrapping_mul(ki.wrapping_sub(kj));
                    // let tmp = tmp * x.pow(t)
                    w = w.wrapping_add(b);

                    // Verify that bj*(xj-yj) opens to 0
                    let b = kbj.wrapping_mul(kxj.wrapping_sub(kyj));
                    // let tmp = tmp * x.pow(t)
                    w = w.wrapping_add(b);

                    sum = sum.wrapping_add(kbj);

                    i += 2;
                    j += 1;
                    t += 1;
                    if i >= gates.len() {
                        break;
                    }
                }
                // Verify sum - n-1 opens to 0
                let mac = chan.recv_mac();
                assert_eq!(mac, sum.wrapping_add(delta.wrapping_mul(j - 1)));
            }
            OP_DEBUG => {
                let msg = gates[i] - ARG0;
                dbg!(msg);
                i += 1;
            }
            OP_DEBUG_WIRE => {
                let id = gates[i] - ARG0;
                dbg!(wires.zm[id]);
                i += 1;
            }

            _ => panic!("invalid operation"),
        }
        if (op != OP_OUT) & !is_check(op) & !matches!(op, OP_DEBUG | OP_DEBUG_WIRE) {
            wires.zm.push(res);
        }
        // dbg!(&wires);
    }
    (w, outputs)
}

fn preprocess_vole(
    chan: &mut VerifierTcpChannel,
    segs: &vole::Segments,
) -> (u128, vole::CorrReceiver) {
    let verbose = false;
    let delta = chan.recv_delta_from_dealer();
    if verbose {
        println!("Received delta={delta}")
    };

    let ks_in = chan.recv_extend_vole_zm(segs.n_in);
    if verbose {
        println!("  Received ks_in={ks_in:?}")
    };
    let ks_openings = chan.recv_extend_vole_zm(segs.n_openings);
    if verbose {
        println!("  Received ks_out={ks_openings:?}")
    };
    let ks_mul = chan.recv_extend_vole_zm(segs.n_mul);
    if verbose {
        println!("  Received ks_mul={ks_mul:?}")
    };
    let ks_mul_check = chan.recv_extend_vole_zm(segs.n_mul_check);
    if verbose {
        println!("  Received ks_mul_check={ks_mul_check:?}")
    };

    (
        delta,
        vole::CorrReceiver {
            ks_in,
            ks_openings,
            ks_mul,
            ks_mul_check,
        },
    )
}
