use crate::miniram::lang::*;
use crate::miniram::interpreter::*;

/// Encodes args as a witness for the correct execution of the
/// MiniRAM program prog (i.e a 0 evaluation).
///
/// The witness consists of the local state of program execution,
/// i.e a Vec<LocalState> that is as long as the time bound t
fn encode_witness(prog: Prog, args: Vec<Word>, t: usize) -> Res<Vec<LocalState>> {
    let (res, localstate) = interpret(prog, args, t)?;
    assert_eq!(res, 0);
    Ok(localstate)
}

fn generate_circuit() {
    todo!()
}

#[cfg(test)]
mod test {
    use crate::miniram::programs::mul_eq;

    use super::encode_witness;

    #[test]
    fn test_encode_witness() {
        let p = mul_eq();
        let args = vec![2, 2, 4];
        assert!(encode_witness(p, args, 20).is_ok());
    }
}
