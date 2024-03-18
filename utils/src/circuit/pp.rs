use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::printer::{DotPrinter, PrinterContext};

use std::fmt::Debug;
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;

use crate::circuit::builder::Res as Circuit;

use super::*;

pub fn print<T>(g: &Circuit<T>, w: Option<&[T]>)
where
    T: Display + Debug,
{
    let mut stmts = vec![];
    // input nodes
    if let Some(w) = w {
        for (i, val) in w[0..g.n_in].iter().enumerate() {
            stmts.push(Stmt::Node(in_node(i + ARG0, Some(val))))
        }
    } else {
        for i in 0..g.n_in {
            stmts.push(Stmt::Node(in_node::<T>(i + ARG0, None)))
        }
    }
    // consts
    for (i, c) in g.consts.iter().enumerate() {
        stmts.push(Stmt::Node(const_node(i + ARG0, c)))
    }

    let mut i = 0;
    let mut j = g.n_in;
    for id in ARG0 + g.n_in..ARG0 + g.n_in + g.n_gates {
        let op = g.gates[i];
        let mut args = vec![];
        i += 1;
        while i < g.gates.len() && g.gates[i] >= ARG0 {
            args.push(g.gates[i]);
            i += 1;
        }
        if let Some(w) = w {
            if n_values(op) > 0 {
                stmts.append(&mut node(id, op, &args, Some(&w[j])));
                j += n_values(op);
            } else {
                stmts.append(&mut node::<T>(id, op, &args, None));
            }
        } else {
            stmts.append(&mut node::<T>(id, op, &args, None));
        }
    }
    let g = Graph::DiGraph {
        id: Id::Plain(String::from("g")),
        strict: true,
        stmts,
    };

    let g = g.print(&mut PrinterContext::default());

    let mut file = File::create("graph.dot").unwrap();
    write!(&mut file, "{}", g).unwrap()
}

fn n_values(op: usize) -> usize {
    if matches!(
        op,
        OP_XOR
            | OP_AND
            | OP_AND_CONST
            | OP_ADD
            | OP_SUB
            | OP_MUL
            | OP_MUL_CONST
            | OP_SELECT
            | OP_SELECT_CONST
            | OP_CONV_A2B
            | OP_CONV_B2A
            | OP_ENCODE4
            | OP_ENCODE8
            | OP_ENCODE32
            | OP_CONST
    ) {
        1
    } else if matches!(
        op,
        OP_OUT | OP_CHECK_Z | OP_CHECK_EQ | OP_CHECK_AND | OP_CHECK_ALL_EQ_BUT_ONE
    ) {
        0
    } else if matches!(op, OP_DECODE32) {
        32
    } else if matches!(op, OP_DECODE64) {
        64
    } else {
        panic!("invalid op")
    }
}

fn in_node<T>(i: usize, val: Option<T>) -> Node
where
    T: Display,
{
    let label = if let Some(v) = val {
        format!("\"\\N (arg{}, val: {})\"", i - ARG0, v)
    } else {
        format!("\"\\N (arg{})\"", i - ARG0)
    };
    Node {
        id: NodeId(Id::Plain(format!("{}", i)), None),
        attributes: vec![attr!("label", label)],
    }
}

fn const_node<T>(i: usize, c: T) -> Node
where
    T: std::fmt::Display,
{
    let label = format!("\"\\N (val: {})\"", c);
    Node {
        id: NodeId(Id::Plain(format!("const{}", i)), None),
        attributes: vec![attr!("label", label)],
    }
}

fn node<T>(id: usize, op: usize, args: &[usize], val: Option<T>) -> Vec<Stmt>
where
    T: Display,
{
    let label = if let Some(v) = val {
        // TODO: op with multiple values only show the first
        format!("\"{} (id: {}, val: {})\"", str_of_op(op), id, v)
    } else {
        format!("\"{} ({})\"", str_of_op(op), id)
    };
    let id = NodeId(Id::Plain(format!("{}", id)), None);
    let mut res = vec![Stmt::Node(Node {
        id: id.clone(),
        attributes: vec![attr!("label", label)],
    })];
    for v in args {
        let id_other = if op == OP_CONST {
            format!("const{}", v)
        } else {
            format!("{}", v)
        };
        res.push(Stmt::Edge(Edge {
            ty: EdgeTy::Pair(
                Vertex::N(NodeId(Id::Plain(id_other), None)),
                Vertex::N(id.clone()),
            ),
            attributes: vec![],
        }))
    }
    res
}

fn str_of_op(op: usize) -> &'static str {
    match op {
        OP_XOR => "OP_XOR",
        OP_AND => "OP_AND",
        OP_AND_CONST => "OP_AND_CONST",
        // - arithmetic
        OP_ADD => "OP_ADD",
        OP_SUB => "OP_SUB",
        OP_MUL => "OP_MUL",
        OP_MUL_CONST => "OP_MUL_CONST",
        OP_SELECT => "OP_SELECT",
        OP_SELECT_CONST => "OP_SELECT_CONST",
        // - binary and arithmetic
        OP_CONV_B2A => "OP_CONV_B2A",
        OP_CONV_A2B => "OP_CONV_A2B",
        OP_DECODE32 => "OP_DECODE32",
        OP_DECODE64 => "OP_DECODE64",
        OP_ENCODE4 => "OP_ENCODE4",
        OP_ENCODE8 => "OP_ENCODE8",
        OP_ENCODE32 => "OP_ENCODE32",
        OP_CONST => "OP_CONST",
        OP_OUT => "OP_OUT",
        // Operations for verification:
        OP_CHECK_Z => "OP_CHECK_Z",
        OP_CHECK_EQ => "OP_CHECK_EQ",
        OP_CHECK_AND => "OP_CHECK_AND",
        OP_CHECK_ALL_EQ_BUT_ONE => "OP_CHECK_ALL_EQ_BUT_ONE",
        OP_DEBUG => "OP_DEBUG",
        _ => panic!("invalid op"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::builder::Builder;

    #[test]
    fn pp_graph() {
        let mut b = Builder::<u64>::new(2);
        let c = b.push_const(21);
        let c = b.const_(c);
        let x = b.add(&[c, c]);
        let y = b.add(&[x, ARG0]);
        let o = b.add(&[y, ARG0 + 1]);
        let g = &b.build(&[o]);
        let res = eval64(g, vec![1, 2]);
        assert_eq!(res, vec![45]);
        print(g, None)
    }
}
