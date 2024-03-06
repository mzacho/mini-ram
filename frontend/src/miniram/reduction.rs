use crate::miniram::lang::*;
use crate::miniram::interpreter::*;

use utils::circuit::builder;

use super::encode::encode;

/// Encodes args as a witness for the correct execution of the
/// MiniRAM program prog (i.e a 0 evaluation).
///
/// The witness consists of the local state of program execution,
/// i.e a Vec<LocalState> that is as long as the time bound t
fn encode_witness(prog: &Prog, args: Vec<Word>, t: usize) -> Res<Vec<LocalState>> {
    let (res, mut lsts) = interpret(prog, args, t)?;
    assert_eq!(res, 0);
    if lsts.len() < t {
        // assume program runs for at least one step
        let last_st = *lsts.last().unwrap();
        for _ in lsts.len()..t {
            lsts.push(last_st);
        }
    }
    Ok(lsts)
}

/// Generates a circuit for verifying the existence of an input
/// (witness), that will make the program return 0 within time bound
/// t.
///
/// Circuit inputs: [LocalState; t]
///
/// todo: currently the program is hardcoded into the circuit as a
/// constant.
fn generate_circuit(prog: &Prog, t: usize) -> builder::Res<u64> {
    let n_in = t * N_REG;
    let mut b = builder::Builder::new(n_in);
    let outputs = vec![];

    // hard-code program
    let p = encode(prog);
    for instr in p {
        b.push_const(instr);
    }
    b.build(&outputs)
}

fn convert_localstates(lsts: Vec<LocalState>) -> Vec<u64> {
    lsts.iter().flatten().map(|v| u64::from(*v)).collect()
}

#[cfg(test)]
mod test {
    use utils::circuit::eval64;

    use crate::miniram::programs::mul_eq;

    use super::{convert_localstates, encode_witness, generate_circuit};

    #[test]
    fn test_encode_witness() {
        let p = &mul_eq();
        let args = vec![2, 2, 4];
        assert!(encode_witness(p, args, 20).is_ok());
    }

    #[test]
    fn test_generate_circuit() {
        let t = 20;
        let p = &mul_eq();
        let _ = generate_circuit(p, t);
    }

    #[test]
    fn test_convert_and_eval_mul_eq() {
        let t = 20;
        let p = &mul_eq();
        let c = generate_circuit(p, t);
        let args = vec![2, 2, 4];
        let lsts = encode_witness(p, args, 20).unwrap();
        let mut w = convert_localstates(lsts);
        let n_in = w.len();
        let res = eval64(&c.gates, &mut w, c.n_gates, n_in,
                         &c.consts);

        for v in res {
            assert_eq!(v, 0)
        }
    }
}
