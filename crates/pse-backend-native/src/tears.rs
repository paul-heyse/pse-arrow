// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Weighted tear selection is a `HiGHS` MILP; petgraph independently checks its witness.
#[cfg(feature = "highs")]
use crate::quality::Tolerances;
#[cfg(feature = "highs")]
use crate::solve::*;
use crate::{CoefficientProblem, OracleContract, ProblemError, Variable};
use pse_ids::{Frame, FramedHasher, SemanticId};
use pse_math::binding::ObjectiveSense;
use pse_model::generated::enums::ModelingVariableDomain;
use pse_structural::flowsheet::{FlowGraph, Policy};
use std::collections::{BTreeMap, BTreeSet};
/// Exact finite MILP projection with stable decision-group/native column maps.
#[derive(Debug)]
pub struct TearProblem {
    /// Native coefficient model.
    pub problem: CoefficientProblem,
    /// Decision binary columns; order columns precede them.
    pub decisions: BTreeMap<SemanticId, usize>,
}
fn id(frame: Frame, source: SemanticId) -> SemanticId {
    let mut h = FramedHasher::new(frame);
    h.id(&source);
    h.finish_id()
}
// Components are structural facts. Decision columns remain global because one
// authored group can remove connections in several independent components.
fn order_components(graph: &FlowGraph) -> BTreeMap<SemanticId, (usize, usize)> {
    let self_loops: BTreeSet<_> = graph
        .declaration()
        .connections
        .iter()
        .filter(|edge| edge.from == edge.to)
        .map(|edge| edge.from)
        .collect();
    graph
        .components()
        .into_iter()
        .enumerate()
        .filter(|(_, nodes)| {
            nodes.len() > 1 || nodes.first().is_some_and(|node| self_loops.contains(node))
        })
        .flat_map(|(component, nodes)| {
            let width = nodes.len();
            nodes
                .into_iter()
                .map(move |node| (node, (component, width)))
        })
        .collect()
}

/// Known native MILP construction arrays plus the independent graph witness.
/// HiGHS opaque storage remains the execution's foreign allocation allowance.
pub fn construction_allocation_bound(graph: &FlowGraph) -> Result<usize, ProblemError> {
    let d = graph.declaration();
    let overflow = || ProblemError::Contract("tear construction extent overflow".into());
    // Admission precedes SCC construction: use the conservative structural
    // ceiling here, rather than allocating a temporary decomposition uncharged.
    let rows = d.connections.len();
    let variables = d
        .nodes
        .len()
        .checked_add(d.decisions.len())
        .ok_or_else(overflow)?;
    let entries = rows.checked_mul(3).ok_or_else(overflow)?;
    let mut bytes = graph
        .tear_allocation_bound()
        .map_err(|e| ProblemError::Contract(e.to_string()))?;
    for (count, width) in [
        (
            variables,
            size_of::<Variable>() + size_of::<ModelingVariableDomain>() + 3 * size_of::<f64>(),
        ),
        (
            d.nodes.len(),
            11 * size_of::<(SemanticId, usize)>() + 16 * size_of::<usize>(),
        ),
        // SCC member vectors, component lookup and singleton self-loop set.
        (
            d.nodes.len(),
            size_of::<Vec<SemanticId>>()
                + 11 * size_of::<(SemanticId, (usize, usize))>()
                + 16 * size_of::<usize>(),
        ),
        (
            d.decisions.len(),
            11 * size_of::<(SemanticId, usize)>() + 16 * size_of::<usize>(),
        ),
        (
            rows,
            size_of::<SemanticId>() + size_of::<(f64, f64)>() + 2 * size_of::<f64>(),
        ),
        // Triplet input, faer sorted/deduplicated source, CSC output and permutations.
        (
            entries,
            size_of::<faer::sparse::Triplet<usize, usize, f64>>()
                + size_of::<[usize; 8]>()
                + 2 * size_of::<f64>(),
        ),
        (
            variables.checked_add(1).ok_or_else(overflow)?,
            4 * size_of::<usize>(),
        ),
    ] {
        bytes = count
            .checked_mul(2)
            .and_then(|n| n.checked_add(4))
            .and_then(|n| n.checked_mul(width))
            .and_then(|n| bytes.checked_add(n))
            .ok_or_else(overflow)?;
    }
    bytes
        .checked_add(size_of::<TearProblem>() + size_of::<CoefficientProblem>())
        .ok_or_else(overflow)
}
/// `r_u + 1 <= r_v + |SCC| t_group(edge)` for each intra-SCC occurrence.
/// Inter-component edges cannot be cyclic; globally shared decision columns keep
/// every authored group cost and policy intact without independent solve/merge.
pub fn compile(graph: &FlowGraph) -> Result<TearProblem, ProblemError> {
    let d = graph.declaration();
    let n = d.nodes.len();
    if n == 0 {
        return Err(ProblemError::Contract(
            "empty graph needs no native tear attempt".into(),
        ));
    }
    let components = order_components(graph);
    let order_nodes: Vec<_> = d
        .nodes
        .iter()
        .filter(|node| components.contains_key(&node.id))
        .collect();
    let mut variables: Vec<_> = order_nodes
        .iter()
        .map(|node| Variable {
            id: id(Frame::TearOrderV1, node.id),
            lower: 0.0,
            upper: (components[&node.id].1 - 1) as f64,
        })
        .collect();
    let nodes: BTreeMap<_, _> = order_nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id, i))
        .collect();
    let mut decisions = BTreeMap::new();
    let mut domains = vec![ModelingVariableDomain::Continuous; order_nodes.len()];
    let mut objective = vec![0.0; order_nodes.len()];
    for g in &d.decisions {
        decisions.insert(g.id, variables.len());
        variables.push(Variable {
            id: id(Frame::TearDecisionV1, g.id),
            lower: if g.policy == Policy::Mandatory {
                1.0
            } else {
                0.0
            },
            upper: if g.policy == Policy::Forbidden {
                0.0
            } else {
                1.0
            },
        });
        domains.push(ModelingVariableDomain::Binary);
        objective.push(g.cost);
    }
    let mut rows = Vec::new();
    let mut entries = Vec::new();
    for e in &d.connections {
        let Some((from, to)) = components.get(&e.from).zip(components.get(&e.to)) else {
            continue;
        };
        if from.0 != to.0 {
            continue;
        }
        let r = rows.len();
        rows.push(e.id);
        entries.push(faer::sparse::Triplet::new(r, nodes[&e.from], 1.0));
        entries.push(faer::sparse::Triplet::new(r, nodes[&e.to], -1.0));
        entries.push(faer::sparse::Triplet::new(
            r,
            decisions[&e.decision],
            -(from.1 as f64),
        ));
    }
    let bounds = vec![(f64::NEG_INFINITY, -1.0); rows.len()];
    let constraints =
        faer::sparse::SparseColMat::try_new_from_triplets(rows.len(), variables.len(), &entries)
            .map_err(|e| ProblemError::Internal(e.to_string()))?;
    let problem = CoefficientProblem {
        contract: OracleContract {
            identity: graph.key(),
            variables,
            rows,
            derivatives: pse_kernels::DerivativeOrder::Value,
            smoothness: pse_kernels::DerivativeOrder::Value,
        },
        objective,
        objective_constant: 0.0,
        sense: ObjectiveSense::Minimize,
        domains,
        assumptions: graph.key(),
        constraints,
        hessian: None,
        bounds,
        objectives: Vec::new(),
    };
    problem.validate()?;
    Ok(TearProblem { problem, decisions })
}
/// A selected tear set with a separate native assurance and independently checked DAG.
#[derive(Clone, Debug)]
pub struct TearReport {
    owner: Option<std::sync::Arc<dyn pse_math::AllocationOwner>>,
    /// Explicit selected decision groups.
    pub decisions: BTreeSet<SemanticId>,
    /// All removed original connection occurrences.
    pub connections: Vec<SemanticId>,
    /// Remaining graph's prerequisite-first unit order.
    pub order: Vec<SemanticId>,
    /// Sum of authored decision weights.
    pub cost: f64,
    /// Explicit policy/assurance label.
    pub method: &'static str,
}
impl TearReport {
    /// Retain result accounting when the graph witness leaves its outer envelope.
    pub fn with_owner(mut self, owner: std::sync::Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.owner = Some(owner);
        self
    }
}
fn recover(
    graph: &FlowGraph,
    selected: BTreeSet<SemanticId>,
    method: &'static str,
) -> Result<TearReport, ProblemError> {
    let order = graph
        .witness(&selected)
        .map_err(|e| ProblemError::Contract(e.to_string()))?;
    let d = graph.declaration();
    let cost = d
        .decisions
        .iter()
        .filter(|g| selected.contains(&g.id))
        .map(|g| g.cost)
        .sum();
    let connections = d
        .connections
        .iter()
        .filter(|e| selected.contains(&e.decision))
        .map(|e| e.id)
        .collect();
    Ok(TearReport {
        owner: None,
        decisions: selected,
        connections,
        order,
        cost,
        method,
    })
}
/// Explicit unweighted heuristic. Its computed authored cost is not an optimality claim.
pub fn heuristic(graph: &FlowGraph) -> Result<TearReport, ProblemError> {
    let selected = graph
        .unweighted_heuristic()
        .map_err(|e| ProblemError::Contract(e.to_string()))?;
    recover(
        graph,
        selected,
        "petgraph greedy feedback arc set; unweighted heuristic",
    )
}
/// Execute after runtime admission; returned `None` means no usable native incumbent.
#[cfg(feature = "highs")]
pub fn solve(
    graph: &FlowGraph,
    controls: &Controls,
    execution: Execution,
) -> Result<(Option<TearReport>, SolveReport), ProblemError> {
    let p = compile(graph)?;
    let compatibility = Compatibility {
        layout: graph.key(),
        profile: graph.key(),
        data: graph.key(),
        backend: Backend::Highs,
    };
    let mut session = crate::highs::Session::new(&p.problem, None, compatibility, &execution)?;
    // The tear MILP is a 0/1 problem of this analysis, not of a model: its feasibility
    // budget is the default policy's integrality budget.
    let policy = pse_model::numerics::NumericalPolicy::default();
    let accuracy = ResolvedAccuracy::from_policy(&policy, policy.integrality)?;
    let t = Tolerances {
        variables: vec![accuracy.feasibility; p.problem.contract.variables.len()],
        rows: vec![accuracy.feasibility; p.problem.bounds.len()],
        integrality: accuracy.integrality,
    };
    let report = session.solve(
        &p.problem,
        &pse_math::normalization::Normalization::identity(
            p.problem.contract.variables.len(),
            p.problem.contract.rows.len(),
        ),
        controls,
        &accuracy,
        &crate::highs::Settings::default(),
        execution,
        &t,
        None,
    )?;
    let Some(candidate) = &report.candidate else {
        return Ok((None, report));
    };
    if !report
        .quality
        .as_ref()
        .is_some_and(crate::quality::Quality::feasible)
    {
        return Ok((None, report));
    }
    let selected = p
        .decisions
        .iter()
        .filter(|(_, c)| candidate.primal[**c] > 0.5)
        .map(|(id, _)| *id)
        .collect();
    let tears = recover(
        graph,
        selected,
        "HiGHS weighted feedback-edge MILP; native optimality/limit status retained",
    )?;
    Ok((Some(tears), report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_structural::flowsheet::{Connection, Decision, Declaration, Node};
    fn sid(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn graph(edges: &[(usize, usize, usize)]) -> FlowGraph {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = pse_quantity::standard::ids::quantity("neutral");
        let unit = registry.quantity_type(q).unwrap().canonical_unit;
        let node_count = edges
            .iter()
            .flat_map(|(from, to, _)| [*from, *to])
            .max()
            .map_or(3, |n| (n + 1).max(3));
        let mut nodes: Vec<_> = (0..node_count)
            .map(|i| Node {
                id: sid(i as u8 + 1),
                ports: vec![],
            })
            .collect();
        let mut connections = vec![];
        let mut decisions = BTreeMap::new();
        for (i, &(from, to, group)) in edges.iter().enumerate() {
            let source = sid(80 + 2 * i as u8);
            let target = sid(81 + 2 * i as u8);
            nodes[from].ports.push(pse_kernels::Port {
                id: source,
                quantity: q,
                unit,
            });
            nodes[to].ports.push(pse_kernels::Port {
                id: target,
                quantity: q,
                unit,
            });
            let decision = sid(40 + group as u8);
            decisions.insert(
                decision,
                Decision {
                    id: decision,
                    cost: 1.0 + group as f64,
                    policy: Policy::Free,
                },
            );
            connections.push(Connection {
                id: sid(20 + i as u8),
                from: sid(from as u8 + 1),
                to: sid(to as u8 + 1),
                decision,
                bindings: vec![(source, target)],
            });
        }
        FlowGraph::admit(
            Declaration {
                nodes,
                connections,
                decisions: decisions.into_values().collect(),
            },
            &registry,
            pse_structural::projection::GraphLimits {
                nodes: 20,
                edges: 20,
            },
        )
        .unwrap()
    }
    #[test]
    fn tear_milp_source_demand_preserves_actual_tiny_projection() {
        let graph = graph(&[(0, 1, 0), (1, 2, 1), (2, 0, 2)]);
        let demand = construction_allocation_bound(&graph).unwrap();
        assert!(demand < 1 << 20);
        let problem = compile(&graph).unwrap();
        assert_eq!(problem.problem.contract.identity, graph.key());
        assert_eq!(problem.problem.assumptions, graph.key());
        assert_eq!(problem.problem.contract.variables.len(), 6);
        assert_eq!(problem.problem.constraints.nrows(), 3);
        assert_eq!(problem.problem.constraints.compute_nnz(), 9);
        assert!(demand > 9 * size_of::<faer::sparse::Triplet<usize, usize, f64>>());
    }
    #[test]
    fn exhaustive_tiny_milp_feasibility_matches_independent_dag_witness() {
        for edges in [
            vec![(0, 1, 0), (1, 2, 1), (2, 0, 2)],
            vec![(0, 1, 0), (0, 1, 1), (1, 0, 2)],
            vec![(0, 0, 0), (0, 1, 1), (1, 2, 1)],
            vec![(0, 1, 0), (1, 0, 0)],
            vec![(0, 1, 0), (1, 0, 1), (1, 2, 2), (2, 3, 0), (3, 2, 3)],
            vec![(0, 1, 0), (1, 2, 1)],
        ] {
            let g = graph(&edges);
            let p = compile(&g).unwrap();
            let groups: Vec<_> = p.decisions.keys().copied().collect();
            for mask in 0..1usize << groups.len() {
                let selected: BTreeSet<_> = groups
                    .iter()
                    .enumerate()
                    .filter_map(|(i, id)| (mask & (1 << i) != 0).then_some(*id))
                    .collect();
                let witness = g.witness(&selected).is_ok();
                let order_count = p.problem.contract.variables.len() - groups.len();
                let ranges: Vec<_> = p.problem.contract.variables[..order_count]
                    .iter()
                    .map(|v| v.upper as usize + 1)
                    .collect();
                let assignments: usize = ranges.iter().product();
                let feasible = (0..assignments).any(|mut ranks| {
                    let mut x: Vec<_> = ranges
                        .iter()
                        .map(|width| {
                            let rank = (ranks % width) as f64;
                            ranks /= width;
                            rank
                        })
                        .collect();
                    x.extend(groups.iter().map(|id| f64::from(selected.contains(id))));
                    if x.iter()
                        .zip(&p.problem.contract.variables)
                        .any(|(value, variable)| *value < variable.lower || *value > variable.upper)
                    {
                        return false;
                    }
                    let mut rows = vec![0.0; p.problem.constraints.nrows()];
                    for (c, v) in x.iter().enumerate() {
                        for (r, a) in p
                            .problem
                            .constraints
                            .row_idx_of_col(c)
                            .zip(p.problem.constraints.val_of_col(c))
                        {
                            rows[r] += v * a;
                        }
                    }
                    rows.iter()
                        .zip(&p.problem.bounds)
                        .all(|(v, (lower, upper))| *v >= *lower && *v <= *upper)
                });
                assert_eq!(feasible, witness, "edges={edges:?}, mask={mask}");
            }
        }
    }
    #[test]
    fn tear_components_keep_shared_groups_and_drop_only_inter_component_rows() {
        let g = graph(&[(0, 1, 0), (1, 0, 1), (1, 2, 2), (2, 3, 0), (3, 2, 3)]);
        let p = compile(&g).unwrap();
        assert_eq!(p.problem.constraints.nrows(), 4);
        assert_eq!(p.problem.contract.variables.len(), 8);
        assert!(
            p.problem.contract.variables[..4]
                .iter()
                .all(|v| v.upper == 1.0)
        );
        let shared = p.decisions[&sid(40)];
        assert_eq!(p.problem.constraints.val_of_col(shared), &[-2.0, -2.0]);
        assert_eq!(p.problem.objective[shared], 1.0, "shared cost occurs once");
        assert!(
            !p.problem.contract.rows.contains(&sid(22)),
            "acyclic bridge has no order row"
        );
        assert!(construction_allocation_bound(&g).unwrap() > 0);
    }
    #[cfg(feature = "highs")]
    #[test]
    fn tear_component_native_optimum_charges_shared_group_once() {
        let g = graph(&[(0, 1, 0), (1, 0, 1), (1, 2, 2), (2, 3, 0), (3, 2, 3)]);
        let controls = Controls::default();
        let (selected, report) =
            solve(&g, &controls, Execution::new(Default::default(), &controls)).unwrap();
        let selected = selected.unwrap();
        assert_eq!(selected.decisions, BTreeSet::from([sid(40)]));
        assert_eq!(selected.cost, 1.0);
        assert_eq!(selected.order.len(), 4);
        assert!(
            report
                .quality
                .as_ref()
                .is_some_and(crate::quality::Quality::feasible)
        );
        assert!(report.candidate.is_some());
    }
    #[cfg(feature = "highs")]
    fn limited_native_graph() -> FlowGraph {
        let edges: Vec<_> = (0..5)
            .flat_map(|from| {
                (0..5)
                    .filter(move |to| *to != from)
                    .map(move |to| (from, to))
            })
            .enumerate()
            .map(|(group, (from, to))| (from, to, group))
            .collect();
        graph(&edges)
    }
    #[cfg(feature = "highs")]
    #[test]
    fn tear_native_solution_limit_keeps_feasible_original_witness_and_limit_status() {
        let g = limited_native_graph();
        let controls = Controls {
            options: Options::from([
                ("presolve".into(), OptionValue::Text("off".into())),
                ("mip_max_improving_sols".into(), OptionValue::Integer(1)),
            ]),
            ..Controls::default()
        };
        let (selected, report) =
            solve(&g, &controls, Execution::new(Default::default(), &controls)).unwrap();
        assert_eq!(
            report.termination.category,
            Termination::SolutionLimit,
            "native status must remain a limit: {:?}",
            report.termination
        );
        let selected = selected.expect("actual feasible incumbent survives the limit");
        assert_eq!(selected.order, g.witness(&selected.decisions).unwrap());
        assert!(
            report
                .quality
                .as_ref()
                .is_some_and(crate::quality::Quality::feasible)
        );
        assert!(selected.method.contains("limit status retained"));
        assert_eq!(
            selected.cost,
            g.declaration()
                .decisions
                .iter()
                .filter(|decision| selected.decisions.contains(&decision.id))
                .map(|decision| decision.cost)
                .sum::<f64>()
        );
    }
    #[cfg(feature = "highs")]
    #[test]
    fn tear_native_interruption_without_incumbent_never_invents_a_tear_set() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        #[derive(Debug)]
        struct StopAtNativeEntry {
            cancel: Arc<AtomicBool>,
            entered: AtomicBool,
        }
        impl ProgressTap for StopAtNativeEntry {
            fn observe(&self, event: &Event) {
                if event.phase == "highs.callback" {
                    self.entered.store(true, Ordering::Release);
                    self.cancel.store(true, Ordering::Release);
                }
            }
        }
        let g = limited_native_graph();
        let controls = Controls {
            options: Options::from([
                ("presolve".into(), OptionValue::Text("off".into())),
                ("mip_heuristic_effort".into(), OptionValue::Real(0.0)),
                (
                    "mip_heuristic_run_feasibility_jump".into(),
                    OptionValue::Bool(false),
                ),
            ]),
            ..Controls::default()
        };
        let tap = Arc::new(StopAtNativeEntry {
            cancel: Arc::default(),
            entered: AtomicBool::new(false),
        });
        let mut execution = Execution::new(tap.cancel.clone(), &controls);
        execution.progress = Arc::new(Progress::tapped(controls.history, tap.clone()));
        let (selected, report) = solve(&g, &controls, execution).unwrap();
        assert!(
            tap.entered.load(Ordering::Acquire),
            "actual linked callback must run"
        );
        assert_eq!(
            report.termination.category,
            Termination::Cancelled,
            "{:?}",
            report.termination
        );
        assert!(
            report.candidate.is_none(),
            "no original candidate: {:?}",
            report.candidate
        );
        assert!(
            selected.is_none(),
            "native interruption grants no invented witness"
        );
    }
    #[test]
    fn tear_acyclic_decisions_preserve_mandatory_forbidden_and_self_loop_policy() {
        let base = graph(&[(0, 0, 0), (0, 1, 1), (1, 2, 2)]);
        let mut declaration = base.declaration().clone();
        declaration.decisions.iter_mut().for_each(|g| {
            if g.id == sid(41) {
                g.policy = Policy::Mandatory;
            }
            if g.id == sid(42) {
                g.policy = Policy::Forbidden;
            }
        });
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let g = FlowGraph::admit(
            declaration,
            &registry,
            pse_structural::projection::GraphLimits {
                nodes: 20,
                edges: 20,
            },
        )
        .unwrap();
        let p = compile(&g).unwrap();
        assert_eq!(p.problem.constraints.nrows(), 1);
        assert_eq!(p.problem.contract.variables.len(), 4);
        assert_eq!(p.problem.contract.variables[0].upper, 0.0);
        assert_eq!(
            p.problem.contract.variables[p.decisions[&sid(41)]].lower,
            1.0
        );
        assert_eq!(
            p.problem.contract.variables[p.decisions[&sid(42)]].upper,
            0.0
        );
        let report = recover(&g, BTreeSet::from([sid(40), sid(41)]), "fixture").unwrap();
        assert_eq!(report.cost, 3.0);
        assert_eq!(report.connections.len(), 2);
        assert_eq!(report.order.len(), 3);
    }
    #[test]
    fn grouped_parallel_tears_and_self_loops_preserve_policy() {
        let g = graph(&[(0, 0, 0), (0, 1, 1), (1, 0, 1)]);
        let r = heuristic(&g).unwrap();
        assert!(r.decisions.contains(&sid(40)));
        assert_eq!(r.connections.len(), 3);
        assert!(r.method.contains("heuristic"));
    }
}
