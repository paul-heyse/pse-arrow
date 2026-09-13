// PROBE 3 — petgraph: re-sources the claims the deleted petgraph.md carried.
//   (a) is tarjan_scc's output order stable, and is order *within* a component arbitrary?
//   (b) does maximum_matching really ignore edge direction (the amendment-B claim)?
//   (c) does feedback_arc_set exist and solve §12.5's tear-selection shape?
use petgraph::algo::{tarjan_scc, kosaraju_scc, condensation, toposort, is_bipartite_undirected};
use petgraph::algo::matching::{maximum_matching, greedy_matching};
use petgraph::algo::feedback_arc_set::greedy_feedback_arc_set;
use petgraph::graph::{DiGraph, NodeIndex, UnGraph};
use petgraph::visit::EdgeRef;

fn build(order: &[(usize, usize)], n: usize) -> DiGraph<usize, ()> {
    let mut g = DiGraph::new();
    let idx: Vec<NodeIndex> = (0..n).map(|i| g.add_node(i)).collect();
    for &(a, b) in order { g.add_edge(idx[a], idx[b], ()); }
    g
}

fn scc_shape(g: &DiGraph<usize, ()>) -> Vec<Vec<usize>> {
    tarjan_scc(g).into_iter().map(|c| c.into_iter().map(|n| g[n]).collect()).collect()
}

fn probe3a() {
    println!("== PROBE 3a: tarjan_scc determinism — component order and intra-component order");
    // One 3-cycle {0,1,2}, one 2-cycle {3,4}, plus an acyclic tail 5.
    let e1 = [(0,1),(1,2),(2,0),(3,4),(4,3),(2,3),(3,5)];
    let mut e2 = e1.to_vec(); e2.reverse();               // same graph, edges inserted in reverse
    let a = scc_shape(&build(&e1, 6));
    let b = scc_shape(&build(&e2, 6));
    println!("   insertion order 1 : {a:?}");
    println!("   insertion order 2 : {b:?}");
    println!("   identical across insertion orders : {}", a == b);
    let sets_a: Vec<_> = a.iter().map(|c| { let mut c = c.clone(); c.sort(); c }).collect();
    let sets_b: Vec<_> = b.iter().map(|c| { let mut c = c.clone(); c.sort(); c }).collect();
    println!("   same components once sorted       : {}", sets_a == sets_b);
    println!("   -> if the first is false and the second true, intra-component order is");
    println!("      an artifact of traversal, which is exactly what §5.1 must not depend on.");
    let k = kosaraju_scc(&build(&e1, 6)).into_iter().map(|c| c.into_iter().map(|n| n.index()).collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("   kosaraju_scc on the same graph    : {k:?}");
}

fn probe3b() {
    println!("== PROBE 3b: maximum_matching — does it ignore edge direction?");
    // A bipartite incidence graph: equations {0,1} x variables {2,3}, directed eq -> var.
    let g = build(&[(0,2),(0,3),(1,2)], 4);
    let m = maximum_matching(&g);
    let edges: Vec<(usize, usize)> = m.edges().map(|(a,b)| (a.index(), b.index())).collect();
    println!("   directed eq->var graph, maximum_matching : {edges:?}  size={}", edges.len());
    // Reverse every edge. If direction were honoured, the result could differ.
    let gr = build(&[(2,0),(3,0),(2,1)], 4);
    let mr: Vec<(usize, usize)> = maximum_matching(&gr).edges().map(|(a,b)| (a.index(), b.index())).collect();
    println!("   every edge reversed                      : {mr:?}  size={}", mr.len());
    println!("   same matching size either way            : {}", edges.len() == mr.len());
    let gm = greedy_matching(&g);
    println!("   greedy_matching size                     : {}", gm.edges().count());
    // Is the input recognised as bipartite at all?
    let u: UnGraph<usize, ()> = UnGraph::from_edges([(0u32,2u32),(0,3),(1,2)]);
    println!("   is_bipartite_undirected (from node 0)    : {}", is_bipartite_undirected(&u, 0.into()));
    println!("   -> petgraph ships general-graph matching; nothing here is bipartite-specialised.");
}

fn probe3c() {
    println!("== PROBE 3c: feedback_arc_set — §12.5 tear selection");
    // A flowsheet-shaped digraph with one recycle: 0->1->2->3->1
    let g = build(&[(0,1),(1,2),(2,3),(3,1)], 4);
    let fas: Vec<(usize, usize)> = greedy_feedback_arc_set(&g)
        .map(|e| (g[e.source()], g[e.target()])).collect();
    println!("   recycle 0->1->2->3->1, greedy_feedback_arc_set : {fas:?}");
    println!("   toposort before removal : {:?}", toposort(&g, None).map(|v| v.iter().map(|n| g[*n]).collect::<Vec<_>>()));
    let cond = condensation(build(&[(0,1),(1,2),(2,3),(3,1)], 4), true);
    println!("   condensation node count : {} (from 4)", cond.node_count());
    println!("   -> a tear set IS a feedback arc set; §12.5 lists tear selection as hand-written.");
}

fn main() { probe3a(); println!(); probe3b(); println!(); probe3c(); }
