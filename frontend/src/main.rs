extern crate args;
extern crate getopts;
extern crate utils;

mod miniram;
mod runners;
// mod arm
// mod arm::parser

use args::Args;
use args::ArgsError;
use getopts::Occur;
use runners::run_p;
use runners::run_v;
use runners::run_vole;
use std::env;
use std::process::exit;

use utils::circuit::circuits;

use crate::miniram::programs;
use crate::miniram::reduction::encode_witness;
use crate::miniram::reduction::generate_circuit;

const PROGRAM_DESC: &str = "VOLE-based ZK proof of correct MiniRAM executions";
const PROGRAM_NAME: &str = "miniram-zk";

pub struct ProofCtx {
    //rng: dyn Rng
}

/// Options:
///  -p, --party:
///  --ip-other: IP of the other party
///  --ip-vole:  IP of trusted VOLE dealer
fn main() {
    // parse input ARM prog
    // translate ARM prog to MiniRAM
    // encode witness
    //
    match parse(env::args()) {
        Ok(ParseRes {
            party,
            port,
            port_vole,
            prog,
            t,
            circuit,
        }) => {
            println!("Successfully parsed args");
            let ctx = ProofCtx {};
            match party.as_str() {
                "prover" | "verifier" => {
                    assert!(port_vole.is_some());
                    let (c, w) = if prog.is_some() & t.is_some() {
                        let prog = prog.unwrap();
                        let t = t.unwrap();
                        match prog.as_str() {
                            "mul_eq" => {
                                let prog = programs::mul_eq();
                                let args = vec![2, 2, 4];
                                let w = encode_witness(&prog, args, t).unwrap(); // todo: handle
                                let c = generate_circuit(&prog, t);
                                (c, w)
                            }
                            _ => {
                                println!("don't understand: {}", prog);
                                exit(1);
                            }
                        }
                    } else if let Some(circuit) = circuit {
                        match circuit.as_str() {
                            "mul_eq" => {
                                let c = circuits::mul_eq();
                                let w = vec![2, 2, 4];
                                (c, w)
                            }
                            _ => {
                                println!("don't understand: {}", circuit);
                                exit(1);
                            }
                        }
                    } else {
                        println!("err: no prog and time-bound or curcuit ");
                        exit(1);
                    };
                    match party.as_ref() {
                        "prover" => run_p(port, port_vole.unwrap(), c, w, ctx),
                        "verifier" => run_v(port, port_vole.unwrap(), c, ctx),
                        _ => panic!("unreachable"),
                    }
                }
                "vole" => run_vole(port, ctx),
                _ => {
                    println!("don't understand: {}", party);
                    exit(1);
                }
            }
            .unwrap();
        }
        Err(error) => {
            println!("{}", error);
            exit(1);
        }
    };
}

struct ParseRes {
    party: String,
    port: u16,
    port_vole: Option<u16>,
    prog: Option<String>,
    t: Option<usize>,
    circuit: Option<String>,
}

fn parse(input: std::env::Args) -> Result<ParseRes, ArgsError> {
    let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
    args.option(
        "p",
        "party",
        "Which party to execute protocol as",
        "PARTY",
        Occur::Req,
        None,
    );

    args.option(
        "",
        "port",
        "Port of the prover running on localhost",
        "PROVER_PORT",
        Occur::Req,
        None,
    );

    args.option(
        "v",
        "vole-port",
        "Port of the trusted VOLE dealer running on the localhost",
        "VOLE_PORT",
        Occur::Optional,
        None,
    );

    args.option(
        "x",
        "prog",
        "Which test program/args to use (cannot be used with -b)",
        "PROG",
        Occur::Optional,
        None,
    );

    args.option(
        "t",
        "time-bound",
        "Max steps of program to verify (must be used with -x)",
        "TIME_BOUND",
        Occur::Optional,
        None,
    );

    args.option(
        "c",
        "circuit",
        "Which test circuit/ witness to use (cannot be used with -x)",
        "CIRCUIT",
        Occur::Optional,
        None,
    );

    args.parse(input)?;

    let party = args.value_of("party").unwrap();
    let port = args.value_of("port").unwrap();
    let port_vole = args.optional_value_of("vole-port").unwrap();

    let prog = args.optional_value_of("prog").unwrap();
    let t = args.optional_value_of("time-bound").unwrap();
    let circuit = args.optional_value_of("circuit").unwrap();

    Ok(ParseRes {
        party,
        port,
        port_vole,
        prog,
        t,
        circuit,
    })
}
