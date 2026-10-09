// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Disposable inquiry module, temporarily included by pse-math::factorable.
//! It is not a production hasher or supported API. The retained source makes the
//! inquiry reproducible: replace only Builder.interned/new, add the input
//! capture hook at push entry, and invoke measure from the real preparation case.
//! Map-enum dispatch and disabled-capture branches remain in both timed policies;
//! these measurements cannot by themselves qualify an uninstrumented replacement.
#![allow(missing_docs, missing_debug_implementations, reason = "disposable inquiry surface retained as source evidence, not a production API")]
use super::{CasePlan, CaseValues, Constant, FactorableProgram, FactorableRequest, Node, NodeId};
use rustc_hash::FxBuildHasher;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    hash::{BuildHasher, BuildHasherDefault, Hasher},
    hint::black_box,
    sync::{Arc, atomic::{AtomicBool, AtomicU8, Ordering}},
    time::Instant,
};

static POLICY: AtomicU8 = AtomicU8::new(0);
static CAPTURE: AtomicBool = AtomicBool::new(false);
thread_local! { static SNAPSHOT: RefCell<Snapshot> = RefCell::default(); }

#[derive(Default)]
struct Snapshot {
    inputs: BTreeMap<String, usize>,
    attempts: Vec<Node>,
    hits: usize,
    misses: usize,
}
fn shape(node: &Node) -> String {
    match node {
        Node::Var(_) => "variable".into(),
        Node::Aux(_) => "auxiliary".into(),
        Node::Const(Constant::Float(_)) => "constant:binary64".into(),
        Node::Const(c @ Constant::Rational(_)) => format!("constant:rational:allocated{}", c.allocated_bytes()),
        Node::Sum(v) => format!("sum:arity{}", v.len()),
        Node::Product(v) => format!("product:arity{}", v.len()),
        Node::Pow { .. } => "power".into(),
        Node::Exp(_) => "exp".into(),
        Node::Log(_) => "log".into(),
        Node::Abs(_) => "abs".into(),
        Node::Sin(_) => "sin".into(),
        Node::Cos(_) => "cos".into(),
    }
}

#[derive(Default)]
pub struct CollisionHasher;
impl Hasher for CollisionHasher {
    fn finish(&self) -> u64 { 0 }
    fn write(&mut self, _: &[u8]) {}
}
enum Map {
    Std(HashMap<Node, NodeId>),
    Fx(HashMap<Node, NodeId, FxBuildHasher>),
    Collision(HashMap<Node, NodeId, BuildHasherDefault<CollisionHasher>>),
}
pub struct Interner { map: Map, capture: bool }
impl Interner {
    pub fn new(seed: (Node, NodeId)) -> Self {
        let map = match POLICY.load(Ordering::Relaxed) {
            0 => Map::Std(HashMap::new()),
            1 => Map::Fx(HashMap::with_hasher(FxBuildHasher)),
            2 => Map::Collision(HashMap::default()),
            _ => panic!("unknown inquiry policy"),
        };
        let mut result = Self { map, capture: CAPTURE.load(Ordering::Relaxed) };
        result.insert(seed.0, seed.1);
        result
    }
    pub fn capture_input(&self, node: &Node) {
        if self.capture {
            SNAPSHOT.with_borrow_mut(|s| *s.inputs.entry(shape(node)).or_default() += 1);
        }
    }
    pub fn get(&self, node: &Node) -> Option<&NodeId> {
        let result = match &self.map {
            Map::Std(map) => map.get(node),
            Map::Fx(map) => map.get(node),
            Map::Collision(map) => map.get(node),
        };
        if self.capture {
            SNAPSHOT.with_borrow_mut(|s| {
                s.attempts.push(node.clone());
                if result.is_some() { s.hits += 1; } else { s.misses += 1; }
            });
        }
        result
    }
    pub fn insert(&mut self, node: Node, id: NodeId) -> Option<NodeId> {
        match &mut self.map {
            Map::Std(map) => map.insert(node, id),
            Map::Fx(map) => map.insert(node, id),
            Map::Collision(map) => map.insert(node, id),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Replay { nodes: Vec<Node>, ids: Vec<NodeId>, hits: usize }
fn replay<S: BuildHasher>(attempts: &[Node], hasher: S) -> Replay {
    let zero = Node::Const(Constant::integer(0));
    let mut nodes = vec![zero.clone()];
    let mut map = HashMap::with_hasher(hasher);
    map.insert(zero, 0);
    let mut ids = Vec::with_capacity(attempts.len());
    let mut hits = 0;
    for node in attempts {
        let id = if let Some(id) = map.get(node) {
            hits += 1;
            *id
        } else {
            let id = nodes.len();
            nodes.push(node.clone());
            map.insert(node.clone(), id);
            id
        };
        ids.push(id);
    }
    Replay { nodes, ids, hits }
}
fn compare(a: &FactorableProgram, b: &FactorableProgram) {
    assert_eq!(a.nodes, b.nodes);
    assert_eq!(a.variables, b.variables);
    assert_eq!(a.auxiliaries, b.auxiliaries);
    assert_eq!(a.rows, b.rows);
    assert_eq!(a.objective, b.objective);
    assert_eq!(a.obligations, b.obligations);
    assert_eq!(a.implicit, b.implicit);
    assert_eq!(a.native, b.native);
    assert_eq!(a.values, b.values);
    assert_eq!(a.fidelity, b.fidelity);
    assert_eq!(a.structure, b.structure);
    assert_eq!(a.require_exact, b.require_exact);
    assert_eq!(a.key, b.key);
}
fn collision_controls() {
    let half = super::Rational::from(1) / super::Rational::from(2);
    let normalized_half = super::Rational::from(2) / super::Rational::from(4);
    let nodes = vec![
        Node::Const(Constant::Float(0.0)),
        Node::Const(Constant::Float(-0.0)),
        Node::Const(Constant::Rational(half)),
        Node::Const(Constant::Rational(normalized_half)),
        Node::Var(0), Node::Var(1), Node::Sum(vec![1, 2]), Node::Sum(vec![2, 1]),
    ];
    let std = replay(&nodes, std::collections::hash_map::RandomState::new());
    assert_eq!(std.ids, vec![1, 2, 3, 3, 4, 5, 6, 7]);
    assert_eq!(std, replay(&nodes, FxBuildHasher));
    assert_eq!(std, replay(&nodes, BuildHasherDefault::<CollisionHasher>::default()));
}
// Exact terminal framed path for the default FactorableRequest used here. This
// replays canonical field traversal and rational decimal conversion as well as
// BLAKE3; it does not isolate the underlying digest algorithm's cost.
fn terminal_key(program: &FactorableProgram, structure: pse_ids::ContentHash) -> pse_ids::ContentHash {
    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::MathFactorableV2);
    h.hash(&structure).str("branches:auxiliary").u64(100_000).bool(false);
    for (id, bits) in &program.values { h.id(id).u64(*bits); }
    h.u64(program.nodes.len() as u64);
    for node in &program.nodes {
        match node {
            Node::Var(i) => { h.str("var").u64(*i as u64); }
            Node::Aux(i) => { h.str("aux").u64(*i as u64); }
            Node::Const(c) => { h.str("constant"); c.frame(&mut h); }
            Node::Sum(children) | Node::Product(children) => {
                h.str(if matches!(node, Node::Sum(_)) { "sum" } else { "product" }).u64(children.len() as u64);
                for child in children { h.u64(*child as u64); }
            }
            Node::Pow { base, exponent } => { h.str("power").u64(*base as u64); exponent.frame(&mut h); }
            Node::Exp(i) | Node::Log(i) | Node::Abs(i) | Node::Sin(i) | Node::Cos(i) => {
                h.str(match node { Node::Exp(_) => "exp", Node::Log(_) => "log", Node::Abs(_) => "abs", Node::Sin(_) => "sin", _ => "cos" }).u64(*i as u64);
            }
        }
    }
    for block in &program.implicit {
        h.str("checked-selected-graph"); block.selection.frame(&mut h);
    }
    h.finish_hash()
}
#[derive(serde::Serialize)]
pub struct Report {
    pub policy: &'static str,
    pub program_key: String,
    pub structure_key: String,
    pub consumed_values_count: usize,
    pub columns: usize,
    pub rows: usize,
    pub input_shapes: BTreeMap<String, usize>,
    pub lookup_shapes: BTreeMap<String, usize>,
    pub lookups: usize,
    pub hits: usize,
    pub misses: usize,
    pub retained_nodes: usize,
    pub full_program_std_fx_controls: bool,
    pub whole_program_forced_collision: bool,
    pub forced_collision_replay_lookups: usize,
    pub manual_collision_controls: bool,
    pub projection_iterations_per_sample: usize,
    pub replay_iterations_per_sample: usize,
    pub hash_iterations_per_sample: usize,
    pub projection_ns_per_operation: Vec<f64>,
    pub replay_std_ns_per_operation: Vec<f64>,
    pub replay_fx_ns_per_operation: Vec<f64>,
    pub terminal_framed_key_ns_per_operation: Vec<f64>,
    pub structure_framed_key_ns_per_operation: Vec<f64>,
    pub combined_framed_key_paths_ns_per_operation: Vec<f64>,
}
fn samples(iterations: usize, mut operation: impl FnMut()) -> Vec<f64> {
    (0..12).map(|_| {
        let started = Instant::now();
        for _ in 0..iterations { operation(); }
        started.elapsed().as_nanos() as f64 / iterations as f64
    }).collect()
}
fn one_sample(iterations: usize, mut operation: impl FnMut()) -> f64 {
    let started = Instant::now();
    for _ in 0..iterations { operation(); }
    started.elapsed().as_nanos() as f64 / iterations as f64
}
pub fn measure(plan: &CasePlan, values: &CaseValues, cancel: &Arc<AtomicBool>, policy: u8) -> Report {
    let request = FactorableRequest::default();
    let project = || plan.factorable_program(values, &request, 100_000, cancel).unwrap();
    POLICY.store(0, Ordering::Relaxed);
    CAPTURE.store(true, Ordering::Relaxed);
    let standard = project();
    CAPTURE.store(false, Ordering::Relaxed);
    let capture = SNAPSHOT.with_borrow_mut(std::mem::take);
    assert_eq!(capture.misses + 1, standard.nodes.len());
    assert_eq!(terminal_key(&standard, plan.structure().key()), standard.key);
    POLICY.store(1, Ordering::Relaxed);
    compare(&standard, &project());
    let whole_program_forced_collision = standard.nodes.len() <= 512;
    if whole_program_forced_collision {
        POLICY.store(2, Ordering::Relaxed);
        compare(&standard, &project());
    }
    collision_controls();
    let replay_std = replay(&capture.attempts, std::collections::hash_map::RandomState::new());
    assert_eq!(replay_std.nodes, standard.nodes);
    assert_eq!(replay_std.hits, capture.hits);
    assert_eq!(replay_std, replay(&capture.attempts, FxBuildHasher));
    let forced_collision_replay_lookups = if whole_program_forced_collision { capture.attempts.len() } else { capture.attempts.len().min(256) };
    let collision_attempts = &capture.attempts[..forced_collision_replay_lookups];
    assert_eq!(
        replay(collision_attempts, std::collections::hash_map::RandomState::new()),
        replay(collision_attempts, BuildHasherDefault::<CollisionHasher>::default()),
    );
    POLICY.store(policy, Ordering::Relaxed);
    for _ in 0..3 { black_box(project()); }
    let calibration = Instant::now();
    black_box(project());
    let projection_iterations_per_sample = (10_000_000 / calibration.elapsed().as_nanos().max(1)).clamp(1, 1_000) as usize;
    let replay_iterations_per_sample = (100_000 / capture.attempts.len().max(1)).clamp(1, 10_000);
    let hash_iterations_per_sample = (100_000 / standard.nodes.len().max(1)).clamp(1, 10_000);
    let projection_ns_per_operation = samples(projection_iterations_per_sample, || { black_box(project()); });
    let mut replay_std_ns_per_operation = Vec::new();
    let mut replay_fx_ns_per_operation = Vec::new();
    let standard_sample = || one_sample(replay_iterations_per_sample, || { black_box(replay(&capture.attempts, std::collections::hash_map::RandomState::new())); });
    let fx_sample = || one_sample(replay_iterations_per_sample, || { black_box(replay(&capture.attempts, FxBuildHasher)); });
    for index in 0..12 {
        if index % 2 == 0 {
            replay_std_ns_per_operation.push(standard_sample());
            replay_fx_ns_per_operation.push(fx_sample());
        } else {
            replay_fx_ns_per_operation.push(fx_sample());
            replay_std_ns_per_operation.push(standard_sample());
        }
    }
    let terminal_framed_key_ns_per_operation = samples(hash_iterations_per_sample, || { black_box(terminal_key(&standard, standard.structure)); });
    let structure_framed_key_ns_per_operation = samples(hash_iterations_per_sample, || { black_box(plan.structure().key()); });
    let combined_framed_key_paths_ns_per_operation = samples(hash_iterations_per_sample, || {
        black_box(terminal_key(&standard, plan.structure().key()));
        black_box(plan.structure().key());
    });
    let mut lookup_shapes = BTreeMap::new();
    for node in &capture.attempts { *lookup_shapes.entry(shape(node)).or_default() += 1; }
    Report {
        policy: if policy == 0 { "std" } else { "fx" },
        program_key: standard.key.to_string(), structure_key: standard.structure.to_string(),
        consumed_values_count: standard.values.len(), columns: plan.columns().len(), rows: plan.structure().rows().len(),
        input_shapes: capture.inputs, lookup_shapes,
        lookups: capture.attempts.len(), hits: capture.hits, misses: capture.misses,
        retained_nodes: standard.nodes.len(), full_program_std_fx_controls: true,
        whole_program_forced_collision, forced_collision_replay_lookups, manual_collision_controls: true,
        projection_iterations_per_sample, replay_iterations_per_sample, hash_iterations_per_sample,
        projection_ns_per_operation, replay_std_ns_per_operation, replay_fx_ns_per_operation,
        terminal_framed_key_ns_per_operation, structure_framed_key_ns_per_operation,
        combined_framed_key_paths_ns_per_operation,
    }
}
