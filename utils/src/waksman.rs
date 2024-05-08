use std::collections::HashMap;

use graph_builder::prelude::Graph as _;
use graph_builder::prelude::*;
use permutation::Permutation;

/// A setting of switches of the AS-Waksman network
pub type Config = Vec<bool>;

/// The number of bits required to configure a Waksman network of
/// size n: sum {i=1..n} ceil(log2(i))
pub fn conf_len(n: usize) -> usize {
    let n = i64::try_from(n).unwrap();
    let mut res = 0;
    for i in 1..n {
        res += i.ilog2() + 1
    }
    usize::try_from(res).unwrap()
}

/// Solves the routing problem: Given a permutation p, compute
/// settings for all switches of the AS-Waksman network with size
/// p.len(), such that the network computes p.
///
/// We do this by constructing a bipartite graph where vertices
/// are switches, edges are constraints, and a valid 2-coloring of
/// the vertices of the graph corresponds to a setting of the switches.
///
/// The graph is colored using a simple greedy algoritm that runs in
/// time O(V + E), from pages:
///
/// Thore Husfeldt, Graph colouring algorithms. Chapter 13 of Topics
/// in Chromatic Graph Theory, Cambridge University Press.
///
/// TODE: This currently routes according to p.inverse() instead of p.
pub fn route(p: &Permutation) -> Config {
    if p.len() == 1 {
        vec![]
    } else if p.len() == 2 {
        // Sets the bit for the switch in crate::gadgets::switch
        vec![p.apply_idx(0) == 1]
    } else {
        route_(p)
    }
}

/// A 2-coloring of vertices
type Coloring = HashMap<usize, bool>;

type Graph = UndirectedCsrGraph<usize>;

/// Assumes p.len() >= 4.
pub fn route_(p: &Permutation) -> Config {
    let n = p.len();
    let even = n % 2 == 0;
    // Constraints for input layer
    let start_in = if even { 0 } else { 1 };
    let in_edges = (start_in..n).step_by(2).map(|i| (i, i + 1));

    // Constraints for output layer
    let start_out = if even { 2 } else { 1 };
    let out_edges = (start_out..n)
        .step_by(2)
        .map(|i| (p.apply_idx(i), p.apply_idx(i + 1)));

    // Build graph
    let g = &GraphBuilder::new().edges(in_edges.chain(out_edges)).build();

    // 2-color the graph -
    //
    // If n is even then the first two outputs have a predetermined
    // coloring, as they come from the lower/ upper subnetworks
    // respectively.
    //
    // If n is odd then the first input and the first output have
    // predetermined colorings.
    let mut coloring = Coloring::new();
    if even {
        coloring.insert(p.apply_idx(0), false);
        coloring.insert(p.apply_idx(1), true);
    } else {
        coloring.insert(0, false);
        coloring.insert(p.apply_idx(0), false);
    }
    color(g, &mut coloring);

    // resulting switch configuration
    let mut res = Config::new();

    // Compute config of input layer:
    // Switch i must be set to the color of node 2i
    for i in (start_in..n).step_by(2) {
        res.push(*coloring.get(&i).unwrap())
    }

    // Compute config of subnetworks recursively.
    //
    // Compute the permutation of the first layer (according to
    // the switches just set).
    let mut xs = if even { vec![] } else { vec![0] };
    let mut ys = vec![];
    for i in (start_in..n).step_by(2) {
        if *coloring.get(&i).unwrap() {
            // i is send to upper subnetwork
            ys.push(i);
            xs.push(i + 1);
        } else {
            // i is send to lower subnetwork
            xs.push(i);
            ys.push(i + 1);
        }
    }
    xs.append(&mut ys);
    let p_in = &Permutation::oneline(xs);

    // Compute the permutation up until the last layer
    let mut cs = vec![p.apply_idx(0)];
    let mut ds = if even { vec![p.apply_idx(1)] } else { vec![] };
    for i in (start_out..n).step_by(2) {
        if *coloring.get(&p.apply_idx(i)).unwrap() {
            // p(i) is send to upper subnetwork
            ds.push(p.apply_idx(i));
            cs.push(p.apply_idx(i + 1));
        } else {
            // p(i) is send to lower subnetwork
            cs.push(p.apply_idx(i));
            ds.push(p.apply_idx(i + 1));
        }
    }
    cs.append(&mut ds);
    let p_out = Permutation::oneline(cs);

    // Compose the inverse permutation of the output layer with the
    // permutation of the input layer to compute the permutation of
    // the subnetworks.

    let p_sub = &p_out.inverse() * p_in;
    let p_sub = p_sub.apply_slice((0..n).collect::<Vec<_>>());

    let split = if even { n / 2 } else { n / 2 + 1 };
    let p_lower = &Permutation::oneline(&p_sub[0..split]);
    let p_upper = &Permutation::oneline(
        p_sub[split..]
            .iter()
            .map(|i| i - split)
            .collect::<Vec<usize>>(),
    );

    let res1 = &mut route(p_lower);
    let res2 = &mut route(p_upper);
    res.append(res1);
    res.append(res2);

    // Compute config of output layer:
    // Switch n/2 + i must be set to the color of node p(2i + 2)
    for i in (start_out..n).step_by(2) {
        res.push(*coloring.get(&p.apply_idx(i)).unwrap())
    }
    res
}

/// Finishes the initial 2-coloring clr of g.
/// Assumes g is a bipartite (but not necesarrily connected) graph
/// with 0-indexed vertices.
fn color(g: &Graph, clr: &mut HashMap<usize, bool>) {
    loop {
        // Try to color the graph
        color_connected(g, clr);
        // Check if solution is complete
        if clr.len() == g.node_count() {
            return;
        }
        // Color a node from the next connected component at random
        for i in 0.. {
            if clr.try_insert(i, false).is_ok() {
                break;
            }
        }
    }
}

/// Finishes the initial 2-coloring clr of g.
/// Assumes g is a connected bipartite graph.
fn color_connected(g: &Graph, clr: &mut Coloring) {
    // Work queue
    let mut q = vec![];
    // Parents
    let mut p = HashMap::<usize, usize>::new();
    // Invariant: All nodes in the work queue have a parent

    // Push neigbours of nodes in clr to work queue
    for v in clr.keys() {
        for u in g.neighbors(*v) {
            p.insert(*u, *v);
            q.push(*u)
        }
    }
    while let Some(v) = q.pop() {
        // Color of v.p
        let clr_vp = clr.get(p.get(&v).unwrap()).unwrap();
        // Color v
        clr.insert(v, !clr_vp);
        // Add neighbours of v to work queue if they don't have a parent
        for u in g.neighbors(v) {
            if p.get(u).is_none() {
                p.insert(*u, v);
                q.push(*u)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use graph_builder::Graph as _;

    use super::*;

    #[test]
    fn route_id_even() {
        let p = &Permutation::one(4);
        let _ = route(p);

        let p = &Permutation::one(6);
        let _ = route(p);

        let p = &Permutation::one(8);
        let _ = route(p);

        let p = &Permutation::one(16);
        let _ = route(p);

        let p = &Permutation::one(42);
        let _ = route(p);
    }

    fn assert_coloring(g: &Graph, clr: &Coloring) {
        assert_eq!(g.node_count(), clr.len());
        for u in 0..clr.len() {
            let clr_u = clr.get(&u).unwrap();
            for v in g.neighbors(u) {
                let clr_v = clr.get(v).unwrap();
                assert!(clr_u ^ clr_v)
            }
        }
    }

    #[test]
    fn color_even_graph() {
        let g = &GraphBuilder::new().edges(vec![(0, 1), (2, 3)]).build();

        let clr = &mut Coloring::new();
        color(g, clr);
        assert_coloring(g, clr);

        let g = &GraphBuilder::new()
            .edges(vec![(0, 1), (2, 3), (0, 2), (1, 3)])
            .build();

        let clr = &mut Coloring::new();
        color(g, clr);
        assert_coloring(g, clr);

        // test the constraints of the 8-input Waksman network for
        // permutation 5,4,1,7,0,2,6,3
        let g = &GraphBuilder::new()
            .edges(vec![(0, 1), (2, 3), (4, 5), (6, 7), (1, 7), (0, 2), (6, 3)])
            .build();

        let clr = &mut Coloring::new();
        clr.insert(5, false);
        clr.insert(4, true);
        color(g, clr);
        assert_coloring(g, clr);
    }
}
