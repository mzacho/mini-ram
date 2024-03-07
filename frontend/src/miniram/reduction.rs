use crate::miniram::interpreter::*;
use crate::miniram::lang::reg::*;
use crate::miniram::lang::*;

use utils::circuit::builder;

use super::encode::encode;

/// Encodes args as a witness for the correct execution of the
/// MiniRAM program prog (i.e a 0 evaluation).
///
/// The witness consists of the local state of program execution,
/// i.e a Vec<LocalState> that is as long as the time bound t
pub fn encode_witness(prog: &Prog, args: Vec<Word>, t: usize) -> Res<Vec<u64>> {
    let (res, mut lsts) = interpret(prog, args, t)?;
    assert_eq!(res, 0);
    if lsts.len() < t {
        // assume program runs for at least one step
        let last_st = *lsts.last().unwrap();
        for _ in lsts.len()..t {
            lsts.push(last_st);
        }
    }
    Ok(convert_localstates(lsts))
}

fn convert_localstates(lsts: Vec<LocalState>) -> Vec<u64> {
    lsts.iter().flatten().map(|v| u64::from(*v)).collect()
}

/// Generates a circuit for verifying the existence of an input
/// (witness), that will make the program return 0 within time bound
/// t.
///
/// Circuit inputs: [LocalState; t]
///
/// todo: currently the program is hardcoded into the circuit as a
/// constant.
pub fn generate_circuit(prog: &Prog, t: usize) -> builder::Res<u64> {
    let n_in = t * N_REG;
    let mut b = builder::Builder::new(n_in);
    let mut outputs = vec![];
    let mut code = vec![];

    // hard-code program
    let p = encode(prog);
    for instr in p {
        let id = b.push_const(instr);
        code.push(b.const_(id));
        // id of constant gate can be ignored, as the gates are the first
        // to be added to the circuit. We just use the id of the
        // i'th instruction as i when needed.
    }

    // compose transition circuit t times
    for i in 0..t {
        let mut o = transition_circuit(&mut b, i, &code);
        outputs.append(&mut o);
    }

    b.build(&outputs)
}

/// Input:
/// - b: builder with the source code as constants
/// - i: iteration count (0 <= i < time bound t)
/// - code: number of lines of source code
///
/// Output: ids of output nodes
fn transition_circuit(b: &mut builder::Builder<u64>, i: usize, code: &[usize]) -> Vec<usize> {
    // register offset into local state
    let reg_offset = i * 2 * N_REG;
    // fetch instruction
    let i = b.select(usize::from(PC) + reg_offset, code);
    // decode instruction

    todo!()
}

#[cfg(test)]
mod test {
    use utils::circuit::eval64;

    use crate::miniram::programs::mul_eq;

    use super::{encode_witness, generate_circuit};

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
        let c = &generate_circuit(p, t);
        let args = vec![2, 2, 4];
        let w = encode_witness(p, args, 20).unwrap();

        let res = eval64(c, w);

        for v in res {
            assert_eq!(v, 0)
        }
    }
}
