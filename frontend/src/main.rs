#![feature(portable_simd)]

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
use utils::sha256;

use backend::ProofCtx;
use utils::circuit::builder::Res as Circuit;
use utils::circuit::circuits;

use crate::miniram::interpreter::interpret;
use crate::miniram::programs;
use crate::miniram::programs::compress;
use crate::miniram::programs::verify_compress;
use crate::miniram::reduction::encode_witness;
use crate::miniram::reduction::generate_circuit;

const PROGRAM_DESC: &str = "VOLE-based ZK proof of correct MiniRAM executions";
const PROGRAM_NAME: &str = "miniram-zk";

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
            run,
            arg,
        }) => {
            println!("Successfully parsed args");

            if let Some(party) = party {
                let deterministic = true;
                let mut ctx = if deterministic {
                    ProofCtx::new_deterministic()
                } else {
                    ProofCtx::new_random()
                };

                match party.as_str() {
                    "prover" | "verifier" => {
                        assert!(port_vole.is_some());
                        let (c, w) = if prog.is_some() & t.is_some() {
                            let prog = prog.unwrap();
                            let t = t.unwrap();
                            let (prog, args) = match prog.as_str() {
                                "mul_eq" => {
                                    let prog = programs::mul_eq();
                                    let args = vec![2, 2, 4];
                                    (prog, args)
                                }
                                "const0" => {
                                    let prog = programs::const_0();
                                    let args = vec![];
                                    (prog, args)
                                }
                                "shr" => {
                                    let prog = programs::shr();
                                    let args = vec![];
                                    (prog, args)
                                }
                                "rotr" => {
                                    let prog = programs::rotr();
                                    let args = vec![];
                                    (prog, args)
                                }
                                "overflowing_add" => {
                                    let prog = programs::overflowing_add();
                                    let args = vec![];
                                    (prog, args)
                                }
                                "verify_compress" => {
                                    let arg = arg.unwrap();
                                    let mut arg = arg.split(',');
                                    let msg = arg.next().unwrap();
                                    let mac = arg.next().unwrap();
                                    let msg_ = sha256::pad(msg);
                                    let mac_ = sha256::parse_mac(mac);
                                    let prog = programs::verify_compress(mac_);
                                    (prog, Vec::from(msg_))
                                }

                                _ => {
                                    println!("don't understand: {}", prog);
                                    exit(1);
                                }
                            };
                            ctx.start_time("encode witness");
                            let w = encode_witness(&prog, args, t, &mut ctx).unwrap(); // todo: handle
                            ctx.stop_time();
                            ctx.start_time("generate circuit");
                            let c = generate_circuit(&prog, t);
                            ctx.stop_time();
                            (c, w)
                        } else if let Some(circuit) = circuit {
                            match circuit.as_str() {
                                "add_eq_42" => {
                                    let c = circuits::add_eq_42();
                                    let w = vec![21, 21];
                                    (c, w)
                                }
                                "add_eq" => {
                                    let c = circuits::add_eq();
                                    let w = vec![21, 21, 42];
                                    (c, w)
                                }
                                "mul_eq" => {
                                    let c = circuits::mul_eq();
                                    let w = vec![2, 2, 4];
                                    (c, w)
                                }
                                "mul_const" => {
                                    let c = circuits::mul_const();
                                    let w = vec![42, 42 * 42];
                                    (c, w)
                                }
                                "mul_mul_eq" => {
                                    let c = circuits::mul_mul_eq();
                                    let w = vec![2, 2, 7, 28];
                                    (c, w)
                                }
                                "pow_eq" => {
                                    let c = circuits::pow();
                                    let w = vec![2, 3, 8];
                                    (c, w)
                                }
                                "select_eq" => {
                                    let c = circuits::select_eq();
                                    let w = vec![0, 0, 1];
                                    (c, w)
                                }
                                "select_eq2" => {
                                    let c = circuits::select_eq2();
                                    let w = vec![2, 1337, 1, 0, 42];
                                    (c, w)
                                }
                                "select_const" => {
                                    let c = circuits::select_const(0, 1);
                                    let w = vec![0];
                                    (c, w)
                                }
                                "select_const_vec" => {
                                    let c = circuits::select_const_vec(&[0, 1, 2, 3, 0, 5]);
                                    let w = vec![4];
                                    (c, w)
                                }
                                "encode4" => {
                                    let c = circuits::encode4(1 + 2 + 8);
                                    let w = vec![1, 1, 0, 1];
                                    (c, w)
                                }
                                "decode32" => {
                                    let c = circuits::decode32();
                                    let w = vec![0];
                                    (c, w)
                                }
                                "w_all_eq_but_one" => {
                                    let c = circuits::check_all_eq_but_one();
                                    let w = vec![1, 43, 43, 2, 3];
                                    (c, w)
                                }
                                "decode32_128_bit" => {
                                    let c = circuits::add_decode32();
                                    let w = vec![1 << 31, 1 << 31];
                                    (c, w)
                                }
                                "add" => {
                                    let arg = arg.unwrap();
                                    let mut arg = arg.split(',');
                                    let x = arg.next().unwrap().parse::<u32>().unwrap();
                                    let y = arg.next().unwrap().parse::<u32>().unwrap();
                                    let c = circuits::add();
                                    let w = vec![x, y];
                                    (c, w)
                                }
                                _ => {
                                    println!("don't understand: {}", circuit);
                                    exit(1);
                                }
                            }
                        } else {
                            println!(
                                "err: want a) prog and
                                time-bound or b) circuit "
                            );
                            exit(1);
                        };
                        match party.as_ref() {
                            "prover" => {
                                print_circuit_stats(&c);
                                run_p(port.unwrap(), port_vole.unwrap(), c, w, ctx)
                            }
                            "verifier" => run_v(port.unwrap(), port_vole.unwrap(), c, ctx),
                            _ => panic!("unreachable"),
                        }
                    }
                    "vole" => run_vole(port.unwrap(), ctx),
                    _ => {
                        println!("don't understand: {}", party);
                        exit(1);
                    }
                }
                .unwrap();
            } else if let Some(prog) = run {
                let time_bound = t.unwrap();
                match prog.as_str() {
                    "verify_compress" => {
                        let arg = arg.unwrap();
                        let mut arg = arg.split(',');
                        let msg = arg.next().unwrap();
                        let mac = arg.next().unwrap();
                        let msg_ = sha256::pad(msg);
                        let mac_ = sha256::parse_mac(mac);
                        let prog = &verify_compress(mac_);
                        println!("Running (verify_compress({mac}))({msg}):");
                        let (res, _) = interpret(prog, Vec::from(msg_), time_bound).unwrap();
                        println!("res={res}");
                    }
                    "compress" => {
                        let arg = arg.unwrap();
                        let prog = &compress(true);

                        let arg_ = sha256::pad(&arg);
                        println!("Running compress({arg}):");
                        let (_, _) = interpret(prog, Vec::from(arg_), time_bound).unwrap();
                        println!();
                    }
                    _ => todo!(),
                }
            } else {
                println!("--run or --party must be set");
                exit(1);
            }
        }
        Err(error) => {
            println!("{}", error);
            exit(1);
        }
    };
}

fn print_circuit_stats<T>(c: &Circuit<T>) {
    let n_in = c.n_in;
    let n_mul = c.n_mul;
    let n_gates = c.gates.len();
    let n_select = c.n_select;
    let n_decode32 = c.n_decode32;
    let n_check_all = c.n_check_all_eq_but_one;
    let n_consts = c.consts.len();
    let n_out = c.n_out;
    println!("Circuit ====================================");
    println!("  number of inputs         : {n_in}");
    println!("  number of gates          : {n_gates}");
    println!("    - multiplication       : {n_mul}");
    println!("    - selects alts.        : {n_select}");
    println!("    - decode32             : {n_decode32}");
    println!("    - check all eq but one : {n_check_all}");
    println!("    - outputs              : {n_out}");
    println!("  number of constants      : {n_consts}");
    println!("============================================");
}

struct ParseRes {
    party: Option<String>,
    port: Option<u16>,
    port_vole: Option<u16>,
    prog: Option<String>,
    t: Option<usize>,
    circuit: Option<String>,
    run: Option<String>,
    arg: Option<String>,
}

fn parse(input: std::env::Args) -> Result<ParseRes, ArgsError> {
    let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
    args.option(
        "p",
        "party",
        "Which party to execute protocol as",
        "PARTY",
        Occur::Optional,
        None,
    );

    args.option(
        "",
        "port",
        "Port of the prover on localhost",
        "PROVER_PORT",
        Occur::Optional,
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

    args.option(
        "",
        "run",
        "Which MiniRAM program to interpret directly",
        "PROG",
        Occur::Optional,
        None,
    );

    args.option(
        "",
        "arg",
        "Which arguments for --run",
        "PROG",
        Occur::Optional,
        None,
    );

    args.parse(input)?;

    let party = args.optional_value_of("party").unwrap();
    let port = args.optional_value_of("port").unwrap();
    let port_vole = args.optional_value_of("vole-port").unwrap();

    let prog = args.optional_value_of("prog").unwrap();
    let t = args.optional_value_of("time-bound").unwrap();
    let circuit = args.optional_value_of("circuit").unwrap();

    let run = args.optional_value_of("run").unwrap();
    let arg = args.optional_value_of("arg").unwrap();

    Ok(ParseRes {
        party,
        port,
        port_vole,
        prog,
        t,
        circuit,
        run,
        arg,
    })
}
