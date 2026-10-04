// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded selection among verified implicit roots. Smooth branches do not prove a smooth selector.
use super::{InnerSolver, Options, Problem};
use crate::{
    MathError,
    guarded::{CompiledBody, Worker},
};
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// Controls issued for one numerical refinement of an already proven winning sheet.
pub(crate) struct NumericalRefinement<'a> {
    pub parameters: &'a [f64],
    pub unknown_scales: &'a [f64],
    pub row_scales: &'a [f64],
    pub root_allowance: f64,
    pub residual_allowance: f64,
    pub linear: Option<(&'a [f64], f64)>,
    pub product: (
        pse_ids::ContentHash,
        pse_ids::ContentHash,
        pse_ids::ContentHash,
    ),
    pub deadline: Instant,
    pub cancel: &'a Arc<AtomicBool>,
}

/// One compiled alternative and its separately demanded eligibility/criterion programs.
#[derive(Debug)]
pub struct RegimeFactoryBranch {
    /// Root problem of this alternative.
    pub residual: super::Factory,
    /// Program returning one indicator: zero when the alternative is ineligible.
    pub eligibility: Arc<CompiledBody>,
    /// Program returning the selection score and its absolute tie tolerance.
    pub criterion: Arc<CompiledBody>,
    /// Conditional exact nonlinear selection program; static graph evidence needs none.
    pub isolation: Option<Arc<crate::factorable::RootIsolationProgram>>,
}
/// Attempt-bound alternative selection using the same injected native root capability.
#[derive(Debug)]
pub struct RegimeFactory {
    /// Provider contract shared by every alternative.
    pub spec: pse_kernels::ProviderSpec,
    /// Alternatives in authored order.
    pub alternatives: Vec<RegimeFactoryBranch>,
    /// Maximum number of alternatives the selector admits.
    pub maximum_regimes: usize,
    /// Total wall-clock allowance shared by all alternative solves.
    pub time_limit: Duration,
    /// Outer cancellation owner for providers created without a scope.
    pub cancel: Arc<AtomicBool>,
    /// Validated mathematics injected alongside the numerical root solver.
    pub verifier: Option<Arc<dyn super::SelectionVerifier>>,
}
impl RegimeFactory {
    /// Same admitted factory/worker/verifier allowance used by ordinary implicit registration.
    pub fn retained_bytes(&self) -> Result<usize, MathError> {
        super::ImplicitFactory::regime_bytes(self)
    }
    /// Prepare the identical selected worker used by ordinary implicit providers.
    /// This worker is concrete so admitted reduced reconstruction can consume its
    /// accepted sheet/chart and action enclosures. No configuration is resolved twice
    /// and no native solver is introduced here. Factory admission still owns its lease.
    pub fn prepare_selection(
        &self,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<RegimeSelection, pse_kernels::ProviderError> {
        self.prepare_selection_offsets(scope, None)
    }
    /// Actual numerical residual and exact verifier DAG consume identical authored
    /// offsets. Callers check named correspondence against the original contract.
    pub fn prepare_selection_offsets(
        &self,
        scope: pse_kernels::ExecutionScope,
        offsets: Option<&[f64]>,
    ) -> Result<RegimeSelection, pse_kernels::ProviderError> {
        scope.check()?;
        if offsets.is_some_and(|values| {
            values.len() != self.spec.outputs.len() || values.iter().any(|v| !v.is_finite())
        }) {
            return Err(pse_kernels::ProviderError::Contract(
                "selected residual offset extent/value".into(),
            ));
        }
        if self.spec.smoothness > self.spec.derivatives
            || self.alternatives.iter().any(|r| {
                r.residual.spec.inputs != self.spec.inputs
                    || r.residual.spec.outputs != self.spec.outputs
                    || r.residual.unknowns.iter().map(|u| u.id).ne(self
                        .spec
                        .outputs
                        .iter()
                        .map(|p| p.id))
            })
        {
            return Err(pse_kernels::ProviderError::Contract(
                "regime selector coordinates or derivative contract".into(),
            ));
        }
        let alternatives = self
            .alternatives
            .iter()
            .map(|r| {
                Ok(Regime {
                    problem: match offsets {
                        None => r.residual.problem(scope.clone())?,
                        Some(offsets) => {
                            let problem = r.residual.problem(scope.clone())?;
                            let owned = Arc::try_unwrap(problem).map_err(|_| {
                                pse_kernels::ProviderError::Contract(
                                    "fresh selected residual has an unexpected shared owner".into(),
                                )
                            })?;
                            Arc::new(
                                owned
                                    .with_residual_offsets(offsets)
                                    .map_err(super::provider_error)?,
                            )
                        }
                    },
                    solver: r.residual.solver.clone(),
                    configuration: super::ConfigurationWorker::new(
                        r.residual.configuration.clone(),
                        r.residual.hints.as_ref(),
                        r.residual.terms.as_ref(),
                    ),
                    eligibility: r.eligibility.worker_scoped(scope.clone()),
                    criterion: r.criterion.worker_scoped(scope.clone()),
                    eligibility_source: r.eligibility.clone(),
                    criterion_source: r.criterion.clone(),
                    isolation: match (&r.isolation, offsets) {
                        (Some(program), Some(offsets)) => {
                            let mut program = program.as_ref().clone();
                            for (residual, offset) in program.residuals.iter_mut().zip(offsets) {
                                let constant = program.nodes.len();
                                program.nodes.push(crate::factorable::Node::Const(
                                    crate::factorable::Constant::Float(-*offset),
                                ));
                                let difference = program.nodes.len();
                                program
                                    .nodes
                                    .push(crate::factorable::Node::Sum(vec![*residual, constant]));
                                *residual = difference;
                            }
                            Some(Arc::new(program))
                        }
                        _ => r.isolation.clone(),
                    },
                })
            })
            .collect::<Result<_, _>>()?;
        let mut selection = RegimeSelection::new(
            self.spec.id,
            alternatives,
            self.maximum_regimes,
            self.time_limit,
        )
        .map_err(super::provider_error)?;
        selection.verifier = self.verifier.clone();
        selection.scope = Some(scope.clone());
        scope.check()?;
        Ok(selection)
    }
}
impl pse_kernels::ProviderFactory for RegimeFactory {
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        &self.spec
    }
    fn configuration_key(&self) -> pse_ids::ContentHash {
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::ImplicitRegimeConfigurationV2);
        hash.hash(&self.spec.identity())
            .u64(self.maximum_regimes as u64)
            .u64(self.time_limit.as_secs())
            .u64(self.time_limit.subsec_nanos() as u64)
            .u64(self.alternatives.len() as u64);
        if let Some(verifier) = &self.verifier {
            hash.bool(true).hash(&verifier.identity());
        } else {
            hash.bool(false);
        }
        for branch in &self.alternatives {
            hash.hash(&branch.residual.configuration_key())
                .bool(branch.isolation.is_some());
        }
        hash.finish_hash()
    }
    fn create(&self) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        self.create_scoped(pse_kernels::ExecutionScope::new(self.cancel.clone(), None))
    }
    fn create_scoped(
        &self,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        scope.check()?;
        let cancel = scope.cancellation().clone();
        let selection = self.prepare_selection(scope.clone())?;
        scope.check()?;
        Ok(Box::new(SelectedProvider {
            spec: self.spec.clone(),
            selection,
            cancel,
            requested_output: self
                .alternatives
                .iter()
                .map(|r| r.residual.requirements.requested_output)
                .min()
                .ok_or_else(|| {
                    pse_kernels::ProviderError::Contract("empty regime demand".into())
                })?,
        }))
    }
}
#[derive(Debug)]
struct SelectedProvider {
    requested_output: DerivativeOrder,
    spec: pse_kernels::ProviderSpec,
    selection: RegimeSelection,
    cancel: Arc<AtomicBool>,
}
impl pse_kernels::Provider for SelectedProvider {
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        &self.spec
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &pse_kernels::ProviderRequest,
        context: &pse_kernels::EvaluationContext<'_>,
    ) -> Result<pse_kernels::ProviderValues, pse_kernels::ProviderError> {
        request.validate(&self.spec, context)?;
        if request.order > self.requested_output {
            return Err(pse_kernels::ProviderError::Contract(
                "implicit regime output exceeds compiled demand".into(),
            ));
        }
        if !std::ptr::eq(context.cancelled, self.cancel.as_ref()) {
            return Err(pse_kernels::ProviderError::Contract(
                "regime selection requires its admitted outer cancellation owner".into(),
            ));
        }
        let selected = self
            .selection
            .evaluate(inputs, request.order, &self.cancel)
            .map_err(super::provider_error)?;
        let values = pse_kernels::ProviderValues {
            values: request
                .outputs
                .iter()
                .map(|i| selected.values[*i])
                .collect(),
            jacobian: request
                .outputs
                .iter()
                .flat_map(|i| {
                    let n = inputs.len();
                    selected
                        .jacobian
                        .get(i * n..(i + 1) * n)
                        .unwrap_or_default()
                        .iter()
                        .copied()
                })
                .collect(),
            hessians: request
                .outputs
                .iter()
                .flat_map(|i| {
                    let n = inputs.len() * inputs.len();
                    selected
                        .hessians
                        .get(i * n..(i + 1) * n)
                        .unwrap_or_default()
                        .iter()
                        .copied()
                })
                .collect(),
        };
        values.validate(&self.spec, request)?;
        Ok(values)
    }
}

/// One alternative residual system with its own deterministic start and physical bounds.
#[derive(Debug)]
pub struct Regime {
    problem: Arc<Problem>,
    solver: Arc<dyn InnerSolver>,
    configuration: super::ConfigurationWorker,
    eligibility: Worker,
    criterion: Worker,
    eligibility_source: Arc<CompiledBody>,
    criterion_source: Arc<CompiledBody>,
    isolation: Option<Arc<crate::factorable::RootIsolationProgram>>,
}

#[cfg(test)]
#[path = "implicit_regime_deadline_tests.rs"]
mod deadline_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        implicit::Unknown,
        typed::{Binary, BodyBuilder, BodyLimits},
    };
    #[derive(Debug)]
    struct Roots {
        fail: Option<SemanticId>,
        wrong: bool,
    }
    impl InnerSolver for Roots {
        fn minimum_order(&self) -> DerivativeOrder {
            DerivativeOrder::First
        }
        fn identity(&self) -> pse_ids::ContentHash {
            crate::implicit::solver_identity(&format!(
                "test.regime-roots.v1:{:?}:{}",
                self.fail, self.wrong
            ))
        }
        fn solve(
            &self,
            problem: Arc<Problem>,
            _: &[f64],
            options: &Options,
            _: &Arc<AtomicBool>,
        ) -> Result<Vec<f64>, MathError> {
            if self.fail == Some(problem.id) {
                return Err(MathError::Domain {
                    source_id: problem.id,
                    requirement: "controlled nonconvergence",
                });
            }
            Ok(vec![if self.wrong { 0. } else { options.start[0] }])
        }
    }
    fn selection(eligibility: [bool; 2]) -> RegimeSelection {
        selection_with_singular_rival(eligibility, false)
    }
    fn selection_with_singular_rival(eligibility: [bool; 2], singular: bool) -> RegimeSelection {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let unit = registry
            .unit(registry.quantity_type(q).unwrap().canonical_unit)
            .unwrap()
            .clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let unknown = pse_ids::named_id(SemanticId::NIL, "unknown");
        let alternatives = [-1., 1.]
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                let id = pse_ids::named_id(SemanticId::NIL, &format!("regime-{i}"));
                let body = |kind| {
                    let mut b = BodyBuilder::new(
                        crate::initialize().unwrap(),
                        &registry,
                        &pse_quantity::standard::StandardInvariantChecker,
                        2,
                        BodyLimits::default(),
                    )
                    .unwrap();
                    let y = b.input(0, q, pse_quantity::IndexSet::new(), id).unwrap();
                    let p = b.input(1, q, pse_quantity::IndexSet::new(), id).unwrap();
                    let literal = |b: &mut BodyBuilder<'_>, v| {
                        b.literal(
                            v,
                            &unit,
                            pse_quantity::literal::LiteralContext::Explicit { quantity_type: q },
                            id,
                        )
                        .unwrap()
                    };
                    let outputs = match kind {
                        0 => {
                            let root = literal(&mut b, value);
                            let residual = b.binary(Binary::Sub, y, root, None, id).unwrap();
                            vec![if singular && i == 1 {
                                b.binary(Binary::Mul, residual.clone(), residual, None, id)
                                    .unwrap()
                            } else {
                                residual
                            }]
                        }
                        1 => vec![literal(&mut b, if eligibility[i] { 1. } else { 0. })],
                        _ => {
                            let delta = b.binary(Binary::Sub, y, p, None, id).unwrap();
                            vec![
                                b.binary(Binary::Mul, delta.clone(), delta, None, id)
                                    .unwrap(),
                                literal(&mut b, 1e-8),
                            ]
                        }
                    };
                    Arc::new(
                        b.finish(
                            &outputs,
                            if kind == 0 {
                                DerivativeOrder::Second
                            } else {
                                DerivativeOrder::First
                            },
                            crate::library::Optimization::default(),
                            &cancel,
                        )
                        .unwrap(),
                    )
                };
                let problem = Arc::new(
                    Problem::new(
                        id,
                        pse_ids::ContentHash::from_bytes([i as u8; 32]),
                        vec![Unknown {
                            id: unknown,
                            lower: -2.,
                            upper: 2.,
                        }],
                        vec![pse_ids::named_id(id, "row")],
                        1,
                        body(0),
                        100,
                    )
                    .unwrap(),
                );
                let options = Options {
                    start: vec![value],
                    variable_nominals: vec![1.],
                    variable_tolerance: vec![1e-9],
                    residual_tolerance: vec![1e-9],
                    iterations: 10,
                    time_limit: Duration::from_secs(1),
                    derivative_tolerance: 1e-10,
                };
                Regime::new(
                    problem,
                    options,
                    body(1),
                    body(2),
                    Arc::new(Roots {
                        fail: None,
                        wrong: false,
                    }),
                )
                .unwrap()
            })
            .collect();
        RegimeSelection::new(SemanticId::NIL, alternatives, 2, Duration::from_secs(1)).unwrap()
    }
    #[derive(Debug)]
    struct GroupProof {
        calls: Arc<std::sync::atomic::AtomicUsize>,
        refusal: Option<super::super::SelectionProofRefusal>,
        oversized: bool,
    }
    impl super::super::SelectionVerifier for GroupProof {
        fn identity(&self) -> pse_ids::ContentHash {
            super::super::solver_identity("test.minimum-selection-proof.v1")
        }
        fn workspace_bytes(
            &self,
            _: &[Arc<crate::factorable::RootIsolationProgram>],
        ) -> Result<usize, MathError> {
            Ok(0)
        }
        fn promote(
            &self,
            request: &super::super::SelectionProofRequest<'_>,
            chart: &super::super::SelectionChart,
        ) -> Result<super::super::SelectionEvidence, MathError> {
            chart.validate_scope(request, self.identity())?;
            if let Some(refusal) = self.refusal {
                return Ok(super::super::SelectionEvidence::Incomplete(refusal));
            }
            let mut promoted = chart.clone();
            promoted.order = request.order;
            Ok(super::super::SelectionEvidence::Unique(promoted))
        }
        fn certify(
            &self,
            request: &super::super::SelectionProofRequest<'_>,
        ) -> Result<super::super::SelectionEvidence, MathError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            assert_eq!(request.alternatives.len(), 2);
            if let Some(reason) = self.refusal {
                return Ok(super::super::SelectionEvidence::Incomplete(reason));
            }
            let intervals = |radius: f64| {
                request
                    .candidate
                    .iter()
                    .map(|value| super::super::ProofInterval {
                        lower: value - radius,
                        upper: value + radius,
                    })
                    .collect()
            };
            let mut chart = super::super::SelectionChart {
                selection: request.selection,
                alternatives: request
                    .alternatives
                    .iter()
                    .map(|alternative| super::super::SelectionScope {
                        id: alternative.id,
                        program: alternative.program.clone(),
                        residual_identity: alternative.residual_identity,
                        unknowns: alternative.unknowns.to_vec(),
                    })
                    .collect(),
                winner: request.winner,
                verifier_identity: self.identity(),
                parameters: request
                    .parameters
                    .iter()
                    .map(|value| super::super::ProofInterval {
                        lower: value - 0.5,
                        upper: value + 0.5,
                    })
                    .collect(),
                existence: intervals(0.01),
                uniqueness: intervals(0.1),
                order: request.order,
            };
            if self.oversized {
                chart.parameters.reserve(10_000);
            }
            Ok(super::super::SelectionEvidence::Unique(chart))
        }
    }
    pub(super) fn group_selection(
        refusal: Option<super::super::SelectionProofRefusal>,
        oversized: bool,
    ) -> (RegimeSelection, Arc<std::sync::atomic::AtomicUsize>) {
        use crate::factorable::{Constant, Node, RootIsolationProgram};
        let mut selected = selection([true, true]);
        for (index, regime) in selected.alternatives.iter_mut().enumerate() {
            // Exact projection of the simple authored test bodies: y - root and (y-p)^2.
            let root = if index == 0 { -1. } else { 1. };
            regime.isolation = Some(Arc::new(RootIsolationProgram {
                inputs: 2,
                nodes: vec![
                    Node::Var(0),
                    Node::Var(1),
                    Node::Const(Constant::Float(-root)),
                    Node::Sum(vec![0, 2]),
                    Node::Const(Constant::Float(-1.)),
                    Node::Product(vec![1, 4]),
                    Node::Sum(vec![0, 5]),
                    Node::Pow {
                        base: 6,
                        exponent: Constant::Float(2.),
                    },
                    Node::Const(Constant::Float(1e-8)),
                ],
                residuals: vec![3],
                eligibility: vec![],
                criterion: [7, 8],
                obligations: vec![],
                derivative_obligations: vec![],
            }));
        }
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        selected.verifier = Some(Arc::new(GroupProof {
            calls: calls.clone(),
            refusal,
            oversized,
        }));
        (selected, calls)
    }
    fn fail_rival(selected: &mut RegimeSelection) {
        let id = selected.alternatives[1].problem.id;
        selected.alternatives[1].solver = Arc::new(Roots {
            fail: Some(id),
            wrong: false,
        });
    }
    #[test]
    fn implicit_regimes_group_proof_keeps_failed_rival_and_reuses_neighborhood() {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut selected, calls) = group_selection(None, false);
        fail_rival(&mut selected);
        // Value-only operational selection still cannot erase a failed rival.
        assert!(matches!(
            selected.evaluate(&[-2.], DerivativeOrder::Value, &cancel),
            Err(MathError::Domain {
                requirement: "controlled nonconvergence",
                ..
            })
        ));
        let first = selected
            .evaluate(&[-2.], DerivativeOrder::First, &cancel)
            .unwrap();
        assert_eq!(first.values, vec![-1.]);
        assert_eq!((first.examined, first.eligible), (2, 1));
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        selected
            .evaluate(&[-1.99], DerivativeOrder::Value, &cancel)
            .unwrap();
        selected
            .evaluate(&[-1.98], DerivativeOrder::First, &cancel)
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        selected
            .evaluate(&[-1.98], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        selected
            .evaluate(&[-1.5], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 2);
        // Exact physical domains of even a failed rival invalidate the cached chart.
        let rival = &mut selected.alternatives[1];
        let options = rival
            .configuration
            .resolve(&mut rival.problem, &[-1.5], &cancel, None)
            .unwrap();
        let mut bounds = rival.problem.unknowns.clone();
        bounds[0].upper = 1.9;
        rival.configuration = super::super::ConfigurationWorker::new(
            super::super::Configuration::Fixed(bounds, options),
            None,
            None,
        );
        selected
            .evaluate(&[-1.5], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 3);
        let program = selected.alternatives[1].isolation.as_ref().unwrap();
        let mut changed = program.as_ref().clone();
        changed.nodes.push(crate::factorable::Node::Const(
            crate::factorable::Constant::Float(0.),
        ));
        selected.alternatives[1].isolation = Some(Arc::new(changed));
        selected
            .evaluate(&[-1.5], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 4);
        selected.alternatives[1].solver = Arc::new(Roots {
            fail: None,
            wrong: false,
        });
        assert!(matches!(
            selected.evaluate(&[2.], DerivativeOrder::Second, &cancel),
            Err(MathError::Provider {
                cause: pse_kernels::ProviderError::RegimeCrossing { .. },
                ..
            })
        ));
        assert_eq!(calls.load(Ordering::Relaxed), 4);
        fail_rival(&mut selected);
        // Cached mathematics never substitutes for a new original residual verification.
        selected.alternatives[0].solver = Arc::new(Roots {
            fail: None,
            wrong: true,
        });
        assert!(matches!(
            selected.evaluate(&[-1.5], DerivativeOrder::Second, &cancel),
            Err(MathError::Domain {
                requirement: "inner root did not satisfy original residual budgets",
                ..
            })
        ));
        assert_eq!(calls.load(Ordering::Relaxed), 4);
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            selected.evaluate(&[-1.5], DerivativeOrder::Second, &cancel),
            Err(MathError::Cancelled)
        ));
        assert_eq!(calls.load(Ordering::Relaxed), 4);
    }
    #[test]
    fn implicit_regimes_group_proof_does_not_require_regular_losing_roots() {
        let cancel = Arc::new(AtomicBool::new(false));
        let mut operational = selection_with_singular_rival([true, true], true);
        assert!(matches!(
            operational.evaluate(&[-2.], DerivativeOrder::First, &cancel),
            Err(MathError::Domain { .. })
        ));
        let (mut selected, calls) = group_selection(None, false);
        selected.alternatives[1].problem = operational.alternatives.remove(1).problem;
        let mut projection = selected.alternatives[1]
            .isolation
            .as_ref()
            .unwrap()
            .as_ref()
            .clone();
        projection.nodes.push(crate::factorable::Node::Pow {
            base: 3,
            exponent: crate::factorable::Constant::Float(2.),
        });
        projection.residuals = vec![projection.nodes.len() - 1];
        selected.alternatives[1].isolation = Some(Arc::new(projection));
        let result = selected
            .evaluate(&[-2.], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(result.values, vec![-1.]);
        assert_eq!(result.jacobian, vec![0.]);
        assert_eq!(result.hessians, vec![0.]);
        assert_eq!((result.examined, result.eligible), (2, 2));
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
    #[derive(Debug)]
    struct FatalRoots(&'static str);
    impl InnerSolver for FatalRoots {
        fn minimum_order(&self) -> DerivativeOrder {
            DerivativeOrder::First
        }
        fn identity(&self) -> pse_ids::ContentHash {
            super::super::solver_identity("test.fatal-regime-roots.v1")
        }
        fn solve(
            &self,
            problem: Arc<Problem>,
            _: &[f64],
            _: &Options,
            cancel: &Arc<AtomicBool>,
        ) -> Result<Vec<f64>, MathError> {
            match self.0 {
                "resource" => Err(MathError::Limit("controlled root resources")),
                "cancel" => {
                    cancel.store(true, Ordering::Release);
                    Err(MathError::Domain {
                        source_id: problem.id,
                        requirement: "cancelled iterate",
                    })
                }
                _ => Err(MathError::Library(
                    "controlled native internal failure".into(),
                )),
            }
        }
    }
    #[test]
    fn implicit_regimes_group_proposals_propagate_fatal_errors_without_proof() {
        for failure in ["resource", "cancel", "internal"] {
            let cancel = Arc::new(AtomicBool::new(false));
            let (mut selected, calls) = group_selection(None, false);
            selected.alternatives[1].solver = Arc::new(FatalRoots(failure));
            let result = selected.evaluate(&[-2.], DerivativeOrder::First, &cancel);
            match failure {
                "resource" => assert!(matches!(
                    result,
                    Err(MathError::Limit("controlled root resources"))
                )),
                "cancel" => assert!(matches!(result, Err(MathError::Cancelled))),
                _ => assert!(matches!(result, Err(MathError::Library(_)))),
            }
            assert_eq!(calls.load(Ordering::Relaxed), 0);
            assert!(selected.chart.is_none());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut selected, calls) = group_selection(None, false);
        fail_rival(&mut selected);
        let winner = &mut selected.alternatives[0];
        winner.solver = Arc::new(Roots {
            fail: Some(winner.problem.id),
            wrong: false,
        });
        assert!(matches!(
            selected.evaluate(&[-2.], DerivativeOrder::First, &cancel),
            Err(MathError::Domain {
                requirement: "controlled nonconvergence",
                ..
            })
        ));
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }
    #[test]
    fn implicit_regimes_partial_proof_admission_refuses_local_demand() {
        let cancel = Arc::new(AtomicBool::new(false));
        for missing_projection in [false, true] {
            let (mut selected, calls) = group_selection(None, false);
            if missing_projection {
                selected.alternatives[1].isolation = None;
            } else {
                selected.verifier = None;
            }
            assert_eq!(
                selected
                    .evaluate(&[-2.], DerivativeOrder::Value, &cancel)
                    .unwrap()
                    .values,
                vec![-1.]
            );
            assert!(matches!(
                selected.evaluate(&[-2.], DerivativeOrder::First, &cancel),
                Err(MathError::Contract(_))
            ));
            assert_eq!(calls.load(Ordering::Relaxed), 0);
        }
    }
    #[test]
    fn implicit_regimes_group_proof_refusals_and_cache_extent_are_bounded() {
        let cancel = Arc::new(AtomicBool::new(false));
        for (refusal, oversized) in [
            (Some(super::super::SelectionProofRefusal::Resource), false),
            (None, true),
        ] {
            let (mut selected, calls) = group_selection(refusal, oversized);
            for _ in 0..2 {
                assert!(matches!(
                    selected.evaluate(&[-2.], DerivativeOrder::First, &cancel),
                    Err(MathError::Limit(_))
                ));
                assert!(selected.chart.is_none());
            }
            assert_eq!(calls.load(Ordering::Relaxed), 2);
        }
    }
    #[test]
    fn implicit_regimes_choose_verified_eligible_minimum_and_refuse_ties() {
        let cancel = Arc::new(AtomicBool::new(false));
        let mut selected = selection([true, true]);
        let negative = selected
            .evaluate(&[-2.], DerivativeOrder::Value, &cancel)
            .unwrap();
        let positive = selected
            .evaluate(&[2.], DerivativeOrder::Value, &cancel)
            .unwrap();
        assert_eq!(negative.values, vec![-1.]);
        assert_eq!(positive.values, vec![1.]);
        assert_ne!(negative.id, positive.id);
        assert_eq!(positive.examined, 2);
        assert_eq!(positive.eligible, 2);
        assert!(matches!(
            selected.evaluate(&[0.], DerivativeOrder::Value, &cancel),
            Err(MathError::Domain {
                requirement: "implicit regime selection is tied within physical tolerance",
                ..
            })
        ));
        let jet = selected
            .evaluate(&[2.], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(jet.jacobian, vec![0.0]);
        assert_eq!(jet.hessians, vec![0.0]);
        for order in [DerivativeOrder::First, DerivativeOrder::Value] {
            assert!(matches!(
                selected.evaluate(&[-2.], order, &cancel),
                Err(MathError::Provider {
                    cause: pse_kernels::ProviderError::RegimeCrossing { bound, selected: crossed, .. },
                    ..
                }) if bound == positive.id && crossed == negative.id
            ));
        }
        let mut filtered = selection([true, false]);
        assert_eq!(
            filtered
                .evaluate(&[2.], DerivativeOrder::Value, &cancel)
                .unwrap()
                .values,
            vec![-1.]
        );
    }
    #[test]
    fn implicit_regimes_refuse_incomplete_or_invalid_candidate_evidence() {
        let cancel = Arc::new(AtomicBool::new(false));
        let mut selected = selection([true, true]);
        let failed = selected.alternatives[1].problem.id;
        selected.alternatives[1].solver = Arc::new(Roots {
            fail: Some(failed),
            wrong: false,
        });
        assert!(matches!(
            selected.evaluate(&[-2.], DerivativeOrder::Value, &cancel),
            Err(MathError::Domain {
                requirement: "controlled nonconvergence",
                ..
            })
        ));
        selected.alternatives[1].solver = Arc::new(Roots {
            fail: None,
            wrong: true,
        });
        assert!(matches!(
            selected.evaluate(&[-2.], DerivativeOrder::Value, &cancel),
            Err(MathError::Domain {
                requirement: "inner root did not satisfy original residual budgets",
                ..
            })
        ));
        let mut empty = selection([false, false]);
        assert!(matches!(
            empty.evaluate(&[2.], DerivativeOrder::Value, &cancel),
            Err(MathError::Domain {
                requirement: "no eligible implicit regime",
                ..
            })
        ));
        selected.time_limit = Duration::from_nanos(1);
        assert!(matches!(
            selected.evaluate(&[2.], DerivativeOrder::Value, &cancel),
            Err(MathError::Limit(_))
        ));
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            empty.evaluate(&[2.], DerivativeOrder::Value, &cancel),
            Err(MathError::Cancelled)
        ));
    }
    #[derive(Debug)]
    struct CountedRoots {
        calls: Arc<std::sync::atomic::AtomicUsize>,
        mode: u8,
    }
    impl InnerSolver for CountedRoots {
        fn minimum_order(&self) -> DerivativeOrder {
            DerivativeOrder::First
        }
        fn identity(&self) -> pse_ids::ContentHash {
            super::super::solver_identity(&format!("test.counted-roots:{}", self.mode))
        }
        fn solve(
            &self,
            _: Arc<Problem>,
            parameters: &[f64],
            options: &Options,
            _: &Arc<AtomicBool>,
        ) -> Result<Vec<f64>, MathError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(vec![match self.mode {
                1 => parameters[0],
                2 => {
                    if parameters[0] < 0.0 {
                        -1.0
                    } else {
                        1.0
                    }
                }
                _ => options.start[0],
            }])
        }
    }
    #[derive(Debug)]
    struct CountedPromotion {
        verifier: Arc<dyn super::super::SelectionVerifier>,
        calls: Arc<std::sync::atomic::AtomicUsize>,
    }
    impl super::super::SelectionVerifier for CountedPromotion {
        fn identity(&self) -> pse_ids::ContentHash {
            self.verifier.identity()
        }
        fn workspace_bytes(
            &self,
            p: &[Arc<crate::factorable::RootIsolationProgram>],
        ) -> Result<usize, MathError> {
            self.verifier.workspace_bytes(p)
        }
        fn certify(
            &self,
            r: &super::super::SelectionProofRequest<'_>,
        ) -> Result<super::super::SelectionEvidence, MathError> {
            self.verifier.certify(r)
        }
        fn promote(
            &self,
            r: &super::super::SelectionProofRequest<'_>,
            c: &super::super::SelectionChart,
        ) -> Result<super::super::SelectionEvidence, MathError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.verifier.promote(r, c)
        }
    }
    #[test]
    fn selected_products_promote_without_proposal_or_exclusion_replay_and_accuracy_is_separate() {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut selected, proofs) = group_selection(None, false);
        let proposals = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        for regime in &mut selected.alternatives {
            regime.solver = Arc::new(CountedRoots {
                calls: proposals.clone(),
                mode: 0,
            });
        }
        let promotions = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        selected.verifier = Some(Arc::new(CountedPromotion {
            verifier: selected.verifier.take().unwrap(),
            calls: promotions.clone(),
        }));
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
            DerivativeOrder::First,
        ] {
            selected.evaluate(&[-2.0], order, &cancel).unwrap();
        }
        assert_eq!(proposals.load(Ordering::Relaxed), 2);
        assert_eq!(proofs.load(Ordering::Relaxed), 1);
        assert_eq!(promotions.load(Ordering::Relaxed), 1);
        let winner = &mut selected.alternatives[0];
        let mut options = winner
            .configuration
            .resolve(&mut winner.problem, &[-2.0], &cancel, None)
            .unwrap();
        options.derivative_tolerance *= 0.5;
        winner.configuration = super::super::ConfigurationWorker::new(
            super::super::Configuration::Fixed(winner.problem.unknowns.clone(), options),
            None,
            None,
        );
        selected
            .evaluate(&[-2.0], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(proposals.load(Ordering::Relaxed), 3);
        assert_eq!(proofs.load(Ordering::Relaxed), 1);
        let original_sheet = selected.sheet;
        selected
            .evaluate(&[-1.9], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(proposals.load(Ordering::Relaxed), 4);
        assert_eq!(proofs.load(Ordering::Relaxed), 1);
        assert_eq!(selected.sheet, original_sheet);
        // A changed criterion projection cannot inherit the old exclusion scope.
        let mut projection = selected.alternatives[0]
            .isolation
            .as_ref()
            .unwrap()
            .as_ref()
            .clone();
        projection.nodes.push(crate::factorable::Node::Const(
            crate::factorable::Constant::Float(1e-8),
        ));
        projection.criterion[1] = projection.nodes.len() - 1;
        selected.alternatives[0].isolation = Some(Arc::new(projection));
        selected
            .evaluate(&[-1.9], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(proposals.load(Ordering::Relaxed), 6);
        assert_eq!(proofs.load(Ordering::Relaxed), 2);
    }
    #[derive(Debug)]
    struct UniformProof {
        moving: bool,
        calls: Arc<std::sync::atomic::AtomicUsize>,
        chain_refusal: Option<super::super::SelectionProofRefusal>,
    }
    impl super::super::SelectionVerifier for UniformProof {
        fn identity(&self) -> pse_ids::ContentHash {
            super::super::solver_identity("test.known-uniform-linear-or-constant-root")
        }
        fn workspace_bytes(
            &self,
            _: &[Arc<crate::factorable::RootIsolationProgram>],
        ) -> Result<usize, MathError> {
            Ok(0)
        }
        fn certify(
            &self,
            r: &super::super::SelectionProofRequest<'_>,
        ) -> Result<super::super::SelectionEvidence, MathError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            let interval = |value: f64, radius: f64| super::super::ProofInterval {
                lower: value - radius,
                upper: value + radius,
            };
            Ok(super::super::SelectionEvidence::Unique(
                super::super::SelectionChart {
                    selection: r.selection,
                    alternatives: r
                        .alternatives
                        .iter()
                        .map(|a| super::super::SelectionScope {
                            id: a.id,
                            program: a.program.clone(),
                            residual_identity: a.residual_identity,
                            unknowns: a.unknowns.to_vec(),
                        })
                        .collect(),
                    winner: r.winner,
                    verifier_identity: self.identity(),
                    parameters: r.parameters.iter().map(|p| interval(*p, 0.25)).collect(),
                    existence: r
                        .candidate
                        .iter()
                        .map(|p| interval(*p, if self.moving { 0.3 } else { 0.01 }))
                        .collect(),
                    uniqueness: r
                        .candidate
                        .iter()
                        .map(|p| interval(*p, if self.moving { 0.5 } else { 0.1 }))
                        .collect(),
                    order: r.order,
                },
            ))
        }
        fn connect_chain(
            &self,
            r: &super::super::ChartChainRequest<'_>,
        ) -> Result<super::super::ChartChainEvidence, MathError> {
            r.validate(self.identity())?;
            if let Some(reason) = self.chain_refusal {
                return Ok(super::super::ChartChainEvidence::Refused {
                    reason,
                    work: super::super::ChartChainWork {
                        charts: 0,
                        connections: 0,
                        proof_cells: 2,
                    },
                });
            }
            if r.coverage == super::super::ChartChainCoverage::SelectedFunction {
                return Ok(super::super::ChartChainEvidence::Incomplete(
                    super::super::SelectionProofRefusal::Coverage,
                ));
            }
            // Exact analytic test source y=p has one regular root at every parameter;
            // constant two-root source cannot connect its disconnected endpoint sheets.
            Ok(if self.moving {
                super::super::ChartChainEvidence::Connected(super::super::ChartChainProof {
                    coverage: super::super::ChartChainCoverage::RootSheet,
                    charts: 1,
                    connections: 2,
                    proof_cells: 3,
                })
            } else {
                super::super::ChartChainEvidence::Incomplete(
                    super::super::SelectionProofRefusal::Chart,
                )
            })
        }
        fn connect(
            &self,
            r: &super::super::SelectionProofRequest<'_>,
            a: &super::super::SelectionChart,
            b: &super::super::SelectionChart,
        ) -> Result<bool, MathError> {
            a.validate_scope(r, self.identity())?;
            b.validate_scope(r, self.identity())?;
            // Fixture supplies exact analytic root y=p of its unchanged linear source.
            Ok(self.moving
                && a.uniqueness[0].contains(r.parameters[0])
                && b.uniqueness[0].contains(r.parameters[0]))
        }
    }
    fn one_regime(
        moving: bool,
    ) -> (
        RegimeSelection,
        Arc<std::sync::atomic::AtomicUsize>,
        Arc<std::sync::atomic::AtomicUsize>,
    ) {
        use crate::factorable::{Constant, Node};
        let (mut selected, _) = group_selection(None, false);
        selected.alternatives.pop();
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let id = selected.alternatives[0].problem.id;
        let mut body = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            BodyLimits::default(),
        )
        .unwrap();
        let y = body.input(0, q, pse_quantity::IndexSet::new(), id).unwrap();
        let p = body.input(1, q, pse_quantity::IndexSet::new(), id).unwrap();
        let residual = if moving {
            body.binary(Binary::Sub, y, p, None, id).unwrap()
        } else {
            let square = body.binary(Binary::Mul, y.clone(), y, None, id).unwrap();
            let unit = registry
                .unit(registry.quantity_type(q).unwrap().canonical_unit)
                .unwrap();
            let one = body
                .literal(
                    1.0,
                    unit,
                    pse_quantity::literal::LiteralContext::Explicit { quantity_type: q },
                    id,
                )
                .unwrap();
            body.binary(Binary::Sub, square, one, None, id).unwrap()
        };
        let body = Arc::new(
            body.finish(
                &[residual],
                DerivativeOrder::Second,
                crate::library::Optimization::default(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap(),
        );
        let previous = &selected.alternatives[0].problem;
        selected.alternatives[0].problem = Arc::new(
            Problem::new(
                id,
                pse_ids::ContentHash::from_bytes([55; 32]),
                previous.unknowns.clone(),
                previous.rows.clone(),
                1,
                body,
                100,
            )
            .unwrap(),
        );
        let mut projection = selected.alternatives[0]
            .isolation
            .as_ref()
            .unwrap()
            .as_ref()
            .clone();
        if moving {
            projection.residuals = vec![6];
        } else {
            projection.nodes.push(Node::Pow {
                base: 0,
                exponent: Constant::Float(2.0),
            });
            let square = projection.nodes.len() - 1;
            projection.nodes.push(Node::Const(Constant::Float(-1.0)));
            let minus = projection.nodes.len() - 1;
            projection.nodes.push(Node::Sum(vec![square, minus]));
            projection.residuals = vec![projection.nodes.len() - 1];
        }
        selected.alternatives[0].isolation = Some(Arc::new(projection));
        let proposals = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        selected.alternatives[0].solver = Arc::new(CountedRoots {
            calls: proposals.clone(),
            mode: if moving { 1 } else { 2 },
        });
        let proofs = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        selected.verifier = Some(Arc::new(UniformProof {
            moving,
            calls: proofs.clone(),
            chain_refusal: None,
        }));
        (selected, proposals, proofs)
    }
    #[test]
    fn selected_sheet_moves_with_uniform_chart_and_certified_common_anchor() {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut selected, proposals, proofs) = one_regime(true);
        let first = selected
            .evaluate(&[0.0], DerivativeOrder::First, &cancel)
            .unwrap();
        assert_eq!(first.jacobian, [1.0]);
        let sheet = selected.sheet;
        let nearby = selected
            .evaluate(&[0.1], DerivativeOrder::First, &cancel)
            .unwrap();
        assert_eq!(nearby.values, [0.1]);
        assert_eq!(nearby.jacobian, [1.0]);
        assert_eq!(proofs.load(Ordering::Relaxed), 1);
        let bridged = selected
            .evaluate(&[0.3], DerivativeOrder::First, &cancel)
            .unwrap();
        assert_eq!(bridged.values, [0.3]);
        assert_eq!(selected.sheet, sheet);
        assert_eq!(proposals.load(Ordering::Relaxed), 3);
        assert_eq!(proofs.load(Ordering::Relaxed), 2);
    }
    #[test]
    fn finite_chart_chain_transports_sheet_without_replaying_unchanged_exclusion() {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut selected, proposals, proofs) = one_regime(true);
        selected
            .evaluate(&[0.0], DerivativeOrder::First, &cancel)
            .unwrap();
        let sheet = selected.sheet;
        let endpoint = selected
            .evaluate(&[1.0], DerivativeOrder::First, &cancel)
            .unwrap();
        assert_eq!(endpoint.values, [1.0]);
        assert_eq!(endpoint.jacobian, [1.0]);
        assert_eq!(selected.sheet, sheet);
        assert_eq!(
            selected.chart_chain_proof(),
            Some(super::super::ChartChainProof {
                coverage: super::super::ChartChainCoverage::RootSheet,
                charts: 1,
                connections: 2,
                proof_cells: 3
            })
        );
        assert_eq!(proposals.load(Ordering::Relaxed), 2);
        assert_eq!(proofs.load(Ordering::Relaxed), 2);
        selected
            .evaluate(&[1.0], DerivativeOrder::First, &cancel)
            .unwrap();
        assert_eq!(proposals.load(Ordering::Relaxed), 2);
        assert_eq!(proofs.load(Ordering::Relaxed), 2);
    }
    #[test]
    fn root_sheet_chain_does_not_satisfy_selected_function_coverage() {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut selected, _, _) = one_regime(true);
        selected
            .require_transport_coverage(super::super::ChartChainCoverage::SelectedFunction)
            .unwrap();
        let first = selected
            .evaluate(&[0.0], DerivativeOrder::First, &cancel)
            .unwrap();
        let chart = selected.chart.clone().unwrap();
        assert!(matches!(
            selected.evaluate(&[1.0], DerivativeOrder::First, &cancel),
            Err(MathError::Domain {
                requirement: "selected root chart-chain coverage is not certified",
                ..
            })
        ));
        assert_eq!(
            selected.numerical.as_ref().unwrap().selected.values,
            first.values
        );
        assert_eq!(
            selected.chart.as_ref().unwrap().uniqueness,
            chart.uniqueness
        );
        assert_eq!(selected.chart_chain_proof(), None);
        assert!(
            selected
                .require_transport_coverage(super::super::ChartChainCoverage::RootSheet)
                .is_err()
        );
    }
    #[test]
    fn disjoint_winners_in_one_regime_do_not_establish_selected_sheet_continuity() {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut selected, _, _) = one_regime(false);
        let first = selected
            .evaluate(&[-1.0], DerivativeOrder::First, &cancel)
            .unwrap();
        let sheet = selected.sheet;
        let chart = selected.chart.clone().unwrap();
        assert!(matches!(
            selected.evaluate(&[1.0], DerivativeOrder::First, &cancel),
            Err(MathError::Domain {
                requirement: "selected root chart-chain overlap is not certified",
                ..
            })
        ));
        assert_eq!(selected.sheet, sheet);
        assert_eq!(
            selected.chart.as_ref().unwrap().uniqueness,
            chart.uniqueness
        );
        assert_eq!(
            selected.numerical.as_ref().unwrap().selected.values,
            first.values
        );
    }
    #[test]
    fn chart_chain_refusals_retain_the_specific_requirement_and_observed_work() {
        use super::super::SelectionProofRefusal;
        for (reason, requirement) in [
            (
                SelectionProofRefusal::Boundary,
                "selected root chart-chain guard or boundary margin is not certified",
            ),
            (
                SelectionProofRefusal::Unsupported,
                "selected root chart-chain projection is unsupported",
            ),
            (
                SelectionProofRefusal::Chart,
                "selected root chart-chain overlap is not certified",
            ),
        ] {
            let cancel = Arc::new(AtomicBool::new(false));
            let (mut selected, _, proofs) = one_regime(true);
            selected.verifier = Some(Arc::new(UniformProof {
                moving: true,
                calls: proofs,
                chain_refusal: Some(reason),
            }));
            selected
                .evaluate(&[0.0], DerivativeOrder::First, &cancel)
                .unwrap();
            let source = selected.id;
            assert!(matches!(
                selected.evaluate(&[1.0], DerivativeOrder::First, &cancel),
                Err(MathError::Domain { source_id, requirement: observed })
                    if source_id == source && observed == requirement
            ));
            assert_eq!(selected.chain_work.unwrap().proof_cells, 2);
            assert_eq!(selected.observed_chain.proof_cells, 2);
            assert_eq!(selected.chart_chain_proof(), None);
        }
    }
}
impl Regime {
    /// Assessment inputs are the root coordinates followed by the residual inputs.
    /// Eligibility returns one Boolean indicator; criterion returns score and absolute tie tolerance.
    /// The compiler owns their physical typing and common score convention.
    pub fn new(
        problem: Arc<Problem>,
        options: Options,
        eligibility: Arc<CompiledBody>,
        criterion: Arc<CompiledBody>,
        solver: Arc<dyn InnerSolver>,
    ) -> Result<Self, MathError> {
        problem.validate_options(&options)?;
        let configuration = super::ConfigurationWorker::new(
            super::Configuration::Fixed(problem.unknowns.clone(), options),
            None,
            None,
        );
        Ok(Self {
            problem,
            solver,
            configuration,
            eligibility: eligibility.worker(),
            criterion: criterion.worker(),
            eligibility_source: eligibility,
            criterion_source: criterion,
            isolation: None,
        })
    }
    fn assess(
        &mut self,
        parameters: &[f64],
        point: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Option<(f64, f64)>, MathError> {
        let inputs = point.iter().chain(parameters).copied().collect::<Vec<_>>();
        let mut providers = self
            .problem
            .providers
            .lock()
            .map_err(|_| MathError::Library("regime provider lock poisoned".into()))?;
        let eligible = self
            .eligibility
            .evaluate(&inputs, order, &mut providers, cancel)?;
        match eligible.values.as_slice() {
            [value] if *value == 0. => return Ok(None),
            [value] if *value == 1. => {}
            _ => {
                return Err(MathError::Contract(
                    "regime eligibility requires one Boolean indicator".into(),
                ));
            }
        }
        let criterion = self
            .criterion
            .evaluate(&inputs, order, &mut providers, cancel)?;
        let [score, tolerance] = criterion.values.as_slice() else {
            return Err(MathError::Contract(
                "regime criterion requires score and tolerance".into(),
            ));
        };
        if !score.is_finite() || !tolerance.is_finite() || *tolerance < 0. {
            return Err(MathError::Domain {
                source_id: self.problem.id,
                requirement: "finite regime score and nonnegative tie tolerance",
            });
        }
        Ok(Some((*score, *tolerance)))
    }
}
/// The selected values retain the winning semantic regime identity and evidence extent.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedRegime {
    /// Identity of the winning alternative.
    pub id: SemanticId,
    /// Solved unknowns of the winning alternative.
    pub values: Vec<f64>,
    /// Local derivatives of the selected regular branch, in parameter order.
    pub jacobian: Vec<f64>,
    /// Local second partials, output-major full symmetric matrices.
    pub hessians: Vec<f64>,
    /// Alternatives considered.
    pub examined: usize,
    /// Alternatives that solved, verified and were eligible.
    pub eligible: usize,
}
/// Alternative solves share a total wall allowance and one outer cancellation owner.
#[derive(Debug)]
pub struct RegimeSelection {
    id: SemanticId,
    alternatives: Vec<Regime>,
    time_limit: Duration,
    derivative_branch: Option<SemanticId>,
    verifier: Option<Arc<dyn super::SelectionVerifier>>,
    scope: Option<pse_kernels::ExecutionScope>,
    chart: Option<super::SelectionChart>,
    chart_bytes: usize,
    chain_coverage: super::ChartChainCoverage,
    last_chain: Option<super::ChartChainProof>,
    chain_work: Option<super::ChartChainWork>,
    observed_chain: super::ChartChainWork,
    observed_action_cells: u64,
    numerical: Option<SelectionProduct>,
    point_refinement: Option<PointRefinement>,
    observed_point_cells: u64,
    sheet: Option<pse_ids::ContentHash>,
    sheet_parameters: Vec<f64>,
}
#[derive(Debug)]
struct SelectionProduct {
    key: pse_ids::ContentHash,
    refinement_base: Option<pse_ids::ContentHash>,
    refined_options: Option<Options>,
    proposals: Vec<Option<Vec<f64>>>,
    selected: SelectedRegime,
    order: DerivativeOrder,
    sources: Vec<(
        crate::guarded::PreparedSupport,
        crate::guarded::PreparedSupport,
        Option<Arc<crate::factorable::RootIsolationProgram>>,
    )>,
}
#[derive(Debug)]
struct PointRefinement {
    key: pse_ids::ContentHash,
    intervals: Vec<super::ProofInterval>,
    inverse_norm_upper: f64,
}
impl RegimeSelection {
    /// Alternatives must define the same ordered unknowns and independent input arity.
    /// A failed solve is not evidence that its regime is physically ineligible.
    pub fn new(
        id: SemanticId,
        alternatives: Vec<Regime>,
        maximum_regimes: usize,
        time_limit: Duration,
    ) -> Result<Self, MathError> {
        if maximum_regimes == 0
            || alternatives.is_empty()
            || alternatives.len() > maximum_regimes
            || time_limit.is_zero()
        {
            return Err(MathError::Limit("implicit regime extent or time allowance"));
        }
        let first = &alternatives[0].problem;
        let mut seen = std::collections::BTreeSet::new();
        if alternatives.iter().any(|r| {
            !seen.insert(r.problem.id)
                || r.problem.inputs != first.inputs
                || !r
                    .problem
                    .unknowns
                    .iter()
                    .map(|u| u.id)
                    .eq(first.unknowns.iter().map(|u| u.id))
        }) {
            return Err(MathError::Contract(
                "implicit regimes require unique identities and common coordinates".into(),
            ));
        }
        let chart_bytes = chart_extent(&alternatives)?;
        Ok(Self {
            id,
            alternatives,
            time_limit,
            derivative_branch: None,
            verifier: None,
            scope: None,
            chart: None,
            chart_bytes,
            chain_coverage: super::ChartChainCoverage::RootSheet,
            last_chain: None,
            chain_work: None,
            observed_chain: super::ChartChainWork::default(),
            observed_action_cells: 0,
            numerical: None,
            point_refinement: None,
            observed_point_cells: 0,
            sheet: None,
            sheet_parameters: Vec::new(),
        })
    }
    /// Choose the consumed transport meaning before this worker binds a root sheet.
    /// Changing an existing lineage's history cannot manufacture stronger coverage.
    pub fn require_transport_coverage(
        &mut self,
        coverage: super::ChartChainCoverage,
    ) -> Result<(), MathError> {
        if self.sheet.is_some() && self.chain_coverage != coverage {
            return Err(MathError::Contract(
                "set chart-chain coverage before binding a root sheet".into(),
            ));
        }
        self.chain_coverage = coverage;
        Ok(())
    }
    /// Actual most recent chain proof, separate from endpoint selection and accuracy.
    pub fn chart_chain_proof(&self) -> Option<super::ChartChainProof> {
        self.last_chain
    }
    /// Actual work in the latest evaluate attempt, including refused/cancelled chains.
    /// Observing work never commits its partial chart coverage or root lineage.
    pub fn chart_chain_work(&self) -> Option<super::ChartChainWork> {
        self.chain_work
    }
    /// Cumulative actual chain work over this admitted worker, including refusals.
    /// Subsequent cheap Value/First calls do not erase the earlier work observations.
    pub fn observed_chain_work(&self) -> super::ChartChainWork {
        self.observed_chain
    }
    /// Cumulative actual attempted interval IFT action cells, including refusals.
    pub fn observed_action_cells(&self) -> u64 {
        self.observed_action_cells
    }
    /// Cumulative actual fixed-parameter root/inverse proof cells, including refusals.
    pub fn observed_point_cells(&self) -> u64 {
        self.observed_point_cells
    }
    /// Bound mathematical root-sheet identity, independent of numerical cache mutation.
    pub fn sheet_identity(&self) -> Option<pse_ids::ContentHash> {
        self.sheet
    }
    /// Accepted selected-root proof. The borrower cannot mutate retained evidence.
    pub fn selected_chart(&self) -> Option<&super::SelectionChart> {
        self.chart.as_ref()
    }
    /// Actual interval-enclosed derivative action using the original accepted chart.
    /// The numerical selected worker still owns iteration and IFT tangent generation.
    pub fn enclose_action(
        &mut self,
        parameters: &[f64],
        direction: &[f64],
        cancel: &Arc<AtomicBool>,
    ) -> Result<super::RootActionEvidence, MathError> {
        let started = Instant::now();
        let selected = self.evaluate(parameters, DerivativeOrder::First, cancel)?;
        let chart = self.chart.as_ref().ok_or_else(|| {
            MathError::Contract("selected action requires an original verified root chart".into())
        })?;
        let verifier = self.verifier.as_ref().ok_or_else(|| {
            MathError::Contract("selected action requires its admitted verifier".into())
        })?;
        let alternatives = self
            .alternatives
            .iter()
            .map(|r| {
                Ok(super::SelectionAlternative {
                    id: r.problem.id,
                    program: r.isolation.as_ref().ok_or_else(|| {
                        MathError::Contract("selected action lacks its original projection".into())
                    })?,
                    residual_identity: r.problem.identity,
                    unknowns: &r.problem.unknowns,
                })
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let mut remaining = self.time_limit.saturating_sub(started.elapsed());
        if let Some(deadline) = self
            .scope
            .as_ref()
            .and_then(pse_kernels::ExecutionScope::deadline)
        {
            remaining = remaining.min(deadline.saturating_duration_since(Instant::now()));
        }
        let request = super::SelectionProofRequest {
            selection: self.id,
            alternatives: &alternatives,
            winner: chart.winner,
            parameters,
            candidate: &selected.values,
            order: DerivativeOrder::First,
            time_limit: remaining,
            cancel,
        };
        let evidence = verifier.enclose_action(&request, chart, direction)?;
        let cells = match &evidence {
            super::RootActionEvidence::Enclosed { proof_cells, .. }
            | super::RootActionEvidence::Incomplete { proof_cells, .. }
            | super::RootActionEvidence::Interrupted { proof_cells } => *proof_cells,
        };
        self.observed_action_cells =
            self.observed_action_cells
                .checked_add(cells)
                .ok_or(MathError::Limit(
                    "selected action work observation overflow",
                ))?;
        if matches!(evidence, super::RootActionEvidence::Interrupted { .. }) {
            return Err(MathError::Cancelled);
        }
        Ok(evidence)
    }
    /// One product shares this declared child allowance and original task deadline.
    pub(super) fn refinement_deadline(&self) -> Result<Instant, MathError> {
        let local = Instant::now()
            .checked_add(self.time_limit)
            .ok_or(MathError::Limit("implicit refinement deadline"))?;
        Ok(self
            .scope
            .as_ref()
            .and_then(pse_kernels::ExecutionScope::deadline)
            .map_or(local, |outer| outer.min(local)))
    }
    pub(super) fn refinement_checkpoint(
        &self,
        deadline: Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), MathError> {
        if let Some(scope) = &self.scope {
            scope.check().map_err(crate::error::scope_error)?;
        }
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(MathError::Scope(pse_kernels::ProviderError::Deadline));
        }
        Ok(())
    }
    /// Point contraction retains competitive selection separately from root accuracy.
    pub(super) fn refine_point(
        &mut self,
        parameters: &[f64],
        unknown_scales: &[f64],
        row_scales: &[f64],
        max_cells: u64,
        deadline: Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<super::RootPointEvidence, MathError> {
        self.refinement_checkpoint(deadline, cancel)?;
        let selected = self.numerical.as_ref().ok_or_else(|| {
            MathError::Contract(
                "point refinement requires an actual selected numerical product".into(),
            )
        })?;
        let chart = self.chart.as_ref().ok_or_else(|| {
            MathError::Contract("point refinement requires original selected coverage".into())
        })?;
        let verifier = self.verifier.as_ref().ok_or_else(|| {
            MathError::Contract("point refinement requires an admitted verifier".into())
        })?;
        let n = self
            .alternatives
            .get(chart.winner)
            .ok_or_else(|| MathError::Contract("point refinement chart winner extent".into()))?
            .problem
            .unknowns
            .len();
        if unknown_scales.len() != n
            || row_scales.len() != n
            || unknown_scales
                .iter()
                .chain(row_scales)
                .any(|s| !s.is_finite() || *s <= 0.)
        {
            return Err(MathError::Contract(
                "point refinement physical scale extent/value".into(),
            ));
        }
        let alternatives = self
            .alternatives
            .iter()
            .map(|r| {
                Ok(super::SelectionAlternative {
                    id: r.problem.id,
                    program: r.isolation.as_ref().ok_or_else(|| {
                        MathError::Contract("point refinement original projection missing".into())
                    })?,
                    residual_identity: r.problem.identity,
                    unknowns: &r.problem.unknowns,
                })
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let request = super::SelectionProofRequest {
            selection: self.id,
            alternatives: &alternatives,
            winner: chart.winner,
            parameters,
            candidate: &selected.selected.values,
            order: DerivativeOrder::First,
            time_limit: deadline.saturating_duration_since(Instant::now()),
            cancel,
        };
        chart.validate(&request, verifier.identity())?;
        let mut key = pse_ids::FramedHasher::new(pse_ids::Frame::ImplicitNumericalProductV1);
        key.str("fixed-selected-root-inverse").hash(&selected.key);
        for s in unknown_scales.iter().chain(row_scales) {
            key.f64(*s);
        }
        let key = key.finish_hash();
        if let Some(cached) = self.point_refinement.as_ref().filter(|p| p.key == key) {
            return Ok(super::RootPointEvidence::Enclosed {
                intervals: cached.intervals.clone(),
                inverse_norm_upper: cached.inverse_norm_upper,
                proof_cells: 0,
            });
        }
        if max_cells == 0 {
            return Ok(super::RootPointEvidence::Incomplete {
                reason: super::SelectionProofRefusal::Resource,
                proof_cells: 0,
            });
        }
        let result =
            verifier.refine_point(&request, chart, unknown_scales, row_scales, max_cells)?;
        let cells = match &result {
            super::RootPointEvidence::Enclosed { proof_cells, .. }
            | super::RootPointEvidence::Incomplete { proof_cells, .. }
            | super::RootPointEvidence::Interrupted { proof_cells } => *proof_cells,
        };
        self.observed_point_cells =
            self.observed_point_cells
                .checked_add(cells)
                .ok_or(MathError::Limit(
                    "selected point proof observation overflow",
                ))?;
        if cells > max_cells {
            return Err(MathError::Contract(
                "point verifier exceeded remaining proof allowance".into(),
            ));
        }
        self.refinement_checkpoint(deadline, cancel)?;
        if let super::RootPointEvidence::Enclosed {
            intervals,
            inverse_norm_upper,
            ..
        } = &result
        {
            if intervals.len() != n
                || intervals
                    .iter()
                    .zip(&chart.uniqueness)
                    .any(|(e, u)| !e.valid() || e.lower < u.lower || e.upper > u.upper)
                || !inverse_norm_upper.is_finite()
                || *inverse_norm_upper <= 0.
            {
                return Err(MathError::Contract(
                    "fixed root/inverse evidence extent or original uniqueness coverage".into(),
                ));
            }
            self.point_refinement = Some(PointRefinement {
                key,
                intervals: intervals.clone(),
                inverse_norm_upper: *inverse_norm_upper,
            });
        }
        Ok(result)
    }
    /// Actual bounded interval action at the fixed parameter root, without new
    /// competitive coverage or a renewed product deadline.
    pub(super) fn enclose_action_bounded(
        &mut self,
        parameters: &[f64],
        direction: &[f64],
        max_cells: u64,
        deadline: Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<super::RootActionEvidence, MathError> {
        self.refinement_checkpoint(deadline, cancel)?;
        let selected = self.numerical.as_ref().ok_or_else(|| {
            MathError::Contract("bounded action requires a selected numerical product".into())
        })?;
        let chart = self.chart.as_ref().ok_or_else(|| {
            MathError::Contract("bounded action requires original selected chart".into())
        })?;
        let verifier = self.verifier.as_ref().ok_or_else(|| {
            MathError::Contract("bounded action requires admitted verifier".into())
        })?;
        let alternatives = self
            .alternatives
            .iter()
            .map(|r| {
                Ok(super::SelectionAlternative {
                    id: r.problem.id,
                    program: r.isolation.as_ref().ok_or_else(|| {
                        MathError::Contract("bounded action original projection missing".into())
                    })?,
                    residual_identity: r.problem.identity,
                    unknowns: &r.problem.unknowns,
                })
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let request = super::SelectionProofRequest {
            selection: self.id,
            alternatives: &alternatives,
            winner: chart.winner,
            parameters,
            candidate: &selected.selected.values,
            order: DerivativeOrder::First,
            time_limit: deadline.saturating_duration_since(Instant::now()),
            cancel,
        };
        chart.validate(&request, verifier.identity())?;
        let result = verifier.enclose_action_bounded(&request, chart, direction, max_cells)?;
        let cells = match &result {
            super::RootActionEvidence::Enclosed { proof_cells, .. }
            | super::RootActionEvidence::Incomplete { proof_cells, .. }
            | super::RootActionEvidence::Interrupted { proof_cells } => *proof_cells,
        };
        self.observed_action_cells =
            self.observed_action_cells
                .checked_add(cells)
                .ok_or(MathError::Limit(
                    "selected action proof observation overflow",
                ))?;
        if cells > max_cells {
            return Err(MathError::Contract(
                "action verifier exceeded remaining proof allowance".into(),
            ));
        }
        self.refinement_checkpoint(deadline, cancel)?;
        Ok(result)
    }
    /// Correct only the already certified winning sheet using existing native
    /// iteration/IFT actions. Strictly stronger consumed controls stay separately
    /// keyed while their unchanged original source/configuration admits later reuse.
    pub(super) fn refine_numerical(
        &mut self,
        controls: NumericalRefinement<'_>,
    ) -> Result<SelectedRegime, MathError> {
        let NumericalRefinement {
            parameters,
            unknown_scales,
            row_scales,
            root_allowance,
            residual_allowance,
            linear,
            product,
            deadline,
            cancel,
        } = controls;
        self.refinement_checkpoint(deadline, cancel)?;
        let previous = self
            .numerical
            .as_ref()
            .ok_or_else(|| {
                MathError::Contract("numerical refinement requires selected product".into())
            })?
            .selected
            .clone();
        let mut configurations = Vec::with_capacity(self.alternatives.len());
        for r in &mut self.alternatives {
            configurations.push(r.configuration.resolve(
                &mut r.problem,
                parameters,
                cancel,
                None,
            )?);
        }
        let base = self.product_key(parameters, &configurations);
        if !self.product_matches(base) {
            return Err(MathError::Contract(
                "numerical refinement source/configuration changed".into(),
            ));
        }
        let chart = self.chart.as_ref().ok_or_else(|| {
            MathError::Contract("numerical refinement requires certified winning chart".into())
        })?;
        let winner = chart.winner;
        let alternatives = self
            .alternatives
            .iter()
            .map(|r| {
                Ok(super::SelectionAlternative {
                    id: r.problem.id,
                    program: r.isolation.as_ref().ok_or_else(|| {
                        MathError::Contract(
                            "numerical refinement original projection missing".into(),
                        )
                    })?,
                    residual_identity: r.problem.identity,
                    unknowns: &r.problem.unknowns,
                })
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let verifier = self.verifier.as_ref().ok_or_else(|| {
            MathError::Contract("numerical refinement requires original verifier".into())
        })?;
        let request = super::SelectionProofRequest {
            selection: self.id,
            alternatives: &alternatives,
            winner,
            parameters,
            candidate: &previous.values,
            order: DerivativeOrder::First,
            time_limit: deadline.saturating_duration_since(Instant::now()),
            cancel,
        };
        chart.validate(&request, verifier.identity())?;
        let mut options = self
            .numerical
            .as_ref()
            .and_then(|p| p.refined_options.clone())
            .unwrap_or_else(|| configurations[winner].clone());
        if unknown_scales.len() != options.variable_tolerance.len()
            || row_scales.len() != options.residual_tolerance.len()
            || unknown_scales
                .iter()
                .chain(row_scales)
                .any(|s| !s.is_finite() || *s <= 0.)
        {
            return Err(MathError::Contract(
                "normalized refinement physical control extent/value".into(),
            ));
        }
        if !root_allowance.is_finite()
            || root_allowance <= 0.
            || !residual_allowance.is_finite()
            || residual_allowance <= 0.
        {
            return Err(MathError::Refinement {
                product: product.0,
                source_key: product.1,
                validity: product.2,
                reason: crate::derived::RefinementRefusal::Precision,
            });
        }
        for (t, s) in options.variable_tolerance.iter_mut().zip(unknown_scales) {
            *t = t.min(s * root_allowance);
        }
        for (t, s) in options.residual_tolerance.iter_mut().zip(row_scales) {
            *t = t.min(s * residual_allowance);
        }
        if options
            .variable_tolerance
            .iter()
            .chain(&options.residual_tolerance)
            .any(|t| !t.is_finite() || *t <= 0.)
        {
            return Err(MathError::Refinement {
                product: product.0,
                source_key: product.1,
                validity: product.2,
                reason: crate::derived::RefinementRefusal::Precision,
            });
        }
        if let Some((direction, backward_allowance)) = linear {
            let n = previous.values.len();
            let p = parameters.len();
            let width = n + p;
            if direction.len() != p {
                return Err(MathError::Contract(
                    "actual action backward controller direction extent".into(),
                ));
            }
            if !backward_allowance.is_finite() || backward_allowance <= 0. {
                return Err(MathError::Refinement {
                    product: product.0,
                    source_key: product.1,
                    validity: product.2,
                    reason: crate::derived::RefinementRefusal::Precision,
                });
            }
            let jet = self.alternatives[winner].problem.evaluate(
                parameters,
                &previous.values,
                DerivativeOrder::First,
                cancel,
            )?;
            if jet.jacobian.len() != n * width || previous.jacobian.len() != n * p {
                return Err(MathError::Contract(
                    "action controller residual jet extent".into(),
                ));
            }
            // This observed floating denominator is an Estimated control translation,
            // not an interval defect bound or a forward accuracy certificate.
            let mut denominator = 0.0_f64;
            for row in 0..n {
                let mut weighted = 0.0;
                for j in 0..p {
                    let mut scale = jet.jacobian[row * width + n + j].abs();
                    for k in 0..n {
                        scale = (scale
                            + (jet.jacobian[row * width + k] * previous.jacobian[k * p + j])
                                .abs()
                                .next_up())
                        .next_up();
                    }
                    weighted = (weighted + (direction[j].abs() * scale).next_up()).next_up();
                }
                denominator = denominator.max((weighted / row_scales[row]).next_up());
            }
            if !denominator.is_finite() {
                return Err(MathError::Refinement {
                    product: product.0,
                    source_key: product.1,
                    validity: product.2,
                    reason: crate::derived::RefinementRefusal::Precision,
                });
            }
            if denominator > 0. {
                options.derivative_tolerance = options
                    .derivative_tolerance
                    .min(backward_allowance / denominator);
            }
            if !options.derivative_tolerance.is_finite() || options.derivative_tolerance <= 0. {
                return Err(MathError::Refinement {
                    product: product.0,
                    source_key: product.1,
                    validity: product.2,
                    reason: crate::derived::RefinementRefusal::Precision,
                });
            }
        }
        options.start = previous.values.clone();
        options.time_limit = options
            .time_limit
            .min(deadline.saturating_duration_since(Instant::now()));
        configurations[winner] = options.clone();
        let actual_key = self.product_key(parameters, &configurations);
        let r = &mut self.alternatives[winner];
        let point = r
            .solver
            .solve(r.problem.clone(), parameters, &options, cancel)?;
        r.problem.verify(parameters, &point, &options, cancel)?;
        if point
            .iter()
            .zip(&chart.uniqueness)
            .any(|(v, u)| !u.interior_contains(*v))
        {
            return Err(MathError::Domain {
                source_id: self.id,
                requirement: "refined numerical proposal left original winning uniqueness chart",
            });
        }
        if r.assess(parameters, &point, DerivativeOrder::First, cancel)?
            .is_none()
        {
            return Err(MathError::Domain {
                source_id: self.id,
                requirement: "refined winning numerical proposal is ineligible",
            });
        }
        let derivatives =
            r.problem
                .derivatives(parameters, &point, DerivativeOrder::First, &options, cancel)?;
        let selected = SelectedRegime {
            id: previous.id,
            values: point.clone(),
            jacobian: derivatives.jacobian,
            hessians: Vec::new(),
            examined: previous.examined,
            eligible: previous.eligible,
        };
        self.refinement_checkpoint(deadline, cancel)?;
        let mut proposals = self
            .numerical
            .as_ref()
            .ok_or_else(|| MathError::Contract("lost selected numerical product".into()))?
            .proposals
            .clone();
        proposals[winner] = Some(point);
        self.retain_product(
            actual_key,
            proposals,
            selected.clone(),
            DerivativeOrder::First,
        );
        let product = self
            .numerical
            .as_mut()
            .ok_or_else(|| MathError::Contract("lost refined numerical product".into()))?;
        product.refinement_base = Some(base);
        product.refined_options = Some(options);
        self.point_refinement = None;
        Ok(selected)
    }
    fn product_key(&self, parameters: &[f64], configurations: &[Options]) -> pse_ids::ContentHash {
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::ImplicitNumericalProductV1);
        h.id(&self.id).u64(self.alternatives.len() as u64);
        for (regime, options) in self.alternatives.iter().zip(configurations) {
            h.hash(&super::numerical_product_key(
                &regime.problem,
                parameters,
                options,
                regime.solver.identity(),
            ));
        }
        h.finish_hash()
    }
    fn product_matches(&self, key: pse_ids::ContentHash) -> bool {
        self.numerical.as_ref().is_some_and(|product| {
            (product.key == key || product.refinement_base == Some(key))
                && product.sources.len() == self.alternatives.len()
                && product.sources.iter().zip(&self.alternatives).all(
                    |((eligibility, criterion, program), regime)| {
                        eligibility == regime.eligibility_source.prepared_support()
                            && criterion == regime.criterion_source.prepared_support()
                            && program == &regime.isolation
                    },
                )
        })
    }
    fn retain_product(
        &mut self,
        key: pse_ids::ContentHash,
        proposals: Vec<Option<Vec<f64>>>,
        selected: SelectedRegime,
        order: DerivativeOrder,
    ) {
        if self.product_matches(key)
            && self
                .numerical
                .as_ref()
                .is_some_and(|p| p.order >= order && p.selected == selected)
        {
            return;
        }
        self.numerical = Some(SelectionProduct {
            key,
            refinement_base: None,
            refined_options: None,
            proposals,
            selected,
            order,
            sources: self
                .alternatives
                .iter()
                .map(|r| {
                    (
                        r.eligibility_source.prepared_support().clone(),
                        r.criterion_source.prepared_support().clone(),
                        r.isolation.clone(),
                    )
                })
                .collect(),
        });
    }
    /// Minimize the authored criterion over all verified eligible alternatives. Ties refuse.
    /// Exact nonlinear projections use one complete competitive-root exclusion proof and
    /// winning-branch derivatives. Operational selection retains its all-alternative
    /// regularity and local continuity checks. The first derivative request binds this
    /// worker to its winning branch; later crossings refuse.
    pub fn evaluate(
        &mut self,
        parameters: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<SelectedRegime, MathError> {
        self.chain_work = None;
        let local = order > DerivativeOrder::Value || self.derivative_branch.is_some();
        if local
            && self
                .alternatives
                .iter()
                .any(|regime| regime.isolation.is_some())
        {
            if self.verifier.is_none()
                || self
                    .alternatives
                    .iter()
                    .any(|regime| regime.isolation.is_none())
            {
                return Err(MathError::Contract(
                    "nonlinear selection requires complete projections and its admitted verifier"
                        .into(),
                ));
            }
            return self.evaluate_certified(parameters, order, cancel);
        }
        self.evaluate_operational(parameters, order, cancel)
    }

    fn evaluate_certified(
        &mut self,
        parameters: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<SelectedRegime, MathError> {
        let deadline = Instant::now()
            .checked_add(self.time_limit)
            .ok_or(MathError::Limit("implicit regime deadline"))?;
        let scope = self.scope.clone();
        let deadline = scope
            .as_ref()
            .and_then(pse_kernels::ExecutionScope::deadline)
            .map_or(deadline, |outer| deadline.min(outer));
        let checkpoint = || {
            if let Some(scope) = &scope {
                scope.check().map_err(crate::error::scope_error)?;
            }
            if cancel.load(Ordering::Acquire) {
                Err(MathError::Cancelled)
            } else if Instant::now() >= deadline {
                Err(MathError::Limit("implicit regime time"))
            } else {
                Ok(())
            }
        };
        // Resolve the complete original physical union before borrowing proof scope.
        // A numerical rival's failure must not erase its resolved search domain.
        let mut configurations = Vec::with_capacity(self.alternatives.len());
        for regime in &mut self.alternatives {
            checkpoint()?;
            configurations.push(regime.configuration.resolve(
                &mut regime.problem,
                parameters,
                cancel,
                None,
            )?);
        }
        let numerical_key = self.product_key(parameters, &configurations);
        let numerical_hit = self.product_matches(numerical_key);
        let useful_chart = if let Some(chart) = &self.chart {
            let alternatives = self
                .alternatives
                .iter()
                .map(|r| {
                    Ok(super::SelectionAlternative {
                        id: r.problem.id,
                        program: r.isolation.as_ref().ok_or_else(|| {
                            MathError::Contract("missing selection projection".into())
                        })?,
                        residual_identity: r.problem.identity,
                        unknowns: &r.problem.unknowns,
                    })
                })
                .collect::<Result<Vec<_>, MathError>>()?;
            let candidate = self
                .numerical
                .as_ref()
                .map(|p| p.selected.values.clone())
                .unwrap_or_else(|| {
                    chart
                        .existence
                        .iter()
                        .map(|i| i.lower + (i.upper - i.lower) * 0.5)
                        .collect()
                });
            let request = super::SelectionProofRequest {
                selection: self.id,
                alternatives: &alternatives,
                winner: chart.winner,
                parameters,
                candidate: &candidate,
                order: order.max(DerivativeOrder::First),
                time_limit: deadline.saturating_duration_since(Instant::now()),
                cancel,
            };
            self.verifier
                .as_ref()
                .is_some_and(|verifier| chart.validate_scope(&request, verifier.identity()).is_ok())
                .then_some(chart.winner)
        } else {
            None
        };
        let mut proposals = if numerical_hit {
            self.numerical
                .as_ref()
                .map(|p| p.proposals.clone())
                .unwrap_or_default()
        } else {
            vec![None; self.alternatives.len()]
        };
        let mut candidates = Vec::with_capacity(self.alternatives.len());
        let mut failed_proposal = None;
        for (index, (regime, options)) in self
            .alternatives
            .iter_mut()
            .zip(&mut configurations)
            .enumerate()
        {
            checkpoint()?;
            if useful_chart.is_some_and(|winner| winner != index) {
                continue;
            }
            options.time_limit = options
                .time_limit
                .min(deadline.saturating_duration_since(Instant::now()));
            if options.time_limit.is_zero() {
                return Err(MathError::Limit("implicit regime time"));
            }
            let cached = numerical_hit
                .then(|| proposals.get(index).and_then(Clone::clone))
                .flatten();
            if !numerical_hit
                && useful_chart == Some(index)
                && let Some(previous) = &self.numerical
            {
                options.start = previous.selected.values.clone();
            }
            let proposal = cached
                .map(Ok)
                .unwrap_or_else(|| {
                    regime
                        .solver
                        .solve(regime.problem.clone(), parameters, options, cancel)
                })
                .and_then(|point| {
                    checkpoint()?;
                    regime.problem.verify(parameters, &point, options, cancel)?;
                    checkpoint()?;
                    Ok(point)
                });
            let point = match proposal {
                Ok(point) => point,
                Err(error @ MathError::Domain { .. }) => {
                    if failed_proposal.is_none() {
                        failed_proposal = Some(error);
                    }
                    continue;
                }
                Err(error) => return Err(error),
            };
            proposals[index] = Some(point.clone());
            checkpoint()?;
            if let Some((score, tolerance)) =
                regime.assess(parameters, &point, DerivativeOrder::Value, cancel)?
            {
                candidates.push((index, point, score, tolerance));
            }
        }
        checkpoint()?;
        let best = candidates
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.2.total_cmp(&b.2))
            .map(|(index, _)| index)
            .ok_or_else(|| {
                failed_proposal.unwrap_or(MathError::Domain {
                    source_id: self.id,
                    requirement: "no eligible implicit regime",
                })
            })?;
        let score = candidates[best].2;
        let tolerance = candidates[best].3;
        if candidates
            .iter()
            .filter(|candidate| (candidate.2 - score).abs() <= tolerance.max(candidate.3))
            .count()
            != 1
        {
            return Err(MathError::Domain {
                source_id: self.id,
                requirement: "implicit regime selection is tied within physical tolerance",
            });
        }
        let eligible = candidates.len();
        let (winner, point, _, _) = candidates.swap_remove(best);
        let id = self.alternatives[winner].problem.id;
        if let Some(bound) = self.derivative_branch.filter(|branch| *branch != id) {
            return Err(MathError::Provider {
                source_id: self.id,
                provider: self.id,
                cause: pse_kernels::ProviderError::RegimeCrossing {
                    selector: self.id,
                    bound,
                    selected: id,
                },
            });
        }
        let verifier = self.verifier.as_ref().ok_or_else(|| {
            MathError::Contract("nonlinear selection requires its admitted verifier".into())
        })?;
        let alternatives = self
            .alternatives
            .iter()
            .map(|regime| {
                Ok(super::SelectionAlternative {
                    id: regime.problem.id,
                    program: regime.isolation.as_ref().ok_or_else(|| {
                        MathError::Contract("nonlinear selection projection is missing".into())
                    })?,
                    residual_identity: regime.problem.identity,
                    unknowns: &regime.problem.unknowns,
                })
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let request = super::SelectionProofRequest {
            selection: self.id,
            alternatives: &alternatives,
            winner,
            parameters,
            candidate: &point,
            order: order.max(DerivativeOrder::First),
            time_limit: deadline.saturating_duration_since(Instant::now()),
            cancel,
        };
        let mut fresh_chart = None;
        if self
            .chart
            .as_ref()
            .is_none_or(|chart| chart.validate(&request, verifier.identity()).is_err())
        {
            let evidence = match &self.chart {
                Some(chart) if chart.validate_scope(&request, verifier.identity()).is_ok() => {
                    verifier.promote(&request, chart)?
                }
                _ => verifier.certify(&request)?,
            };
            checkpoint()?;
            let mut chart = match evidence {
                super::SelectionEvidence::Unique(chart) => chart,
                super::SelectionEvidence::Multiple => {
                    return Err(MathError::Domain {
                        source_id: self.id,
                        requirement: "nonlinear selection has tied or better competitive roots",
                    });
                }
                super::SelectionEvidence::Incomplete(super::SelectionProofRefusal::Resource) => {
                    return Err(MathError::Limit("nonlinear selection proof resources"));
                }
                super::SelectionEvidence::Incomplete(reason) => {
                    return Err(MathError::Domain {
                        source_id: self.id,
                        requirement: match reason {
                            super::SelectionProofRefusal::Unsupported => {
                                "nonlinear selection projection is unsupported"
                            }
                            super::SelectionProofRefusal::Chart => {
                                "nonlinear selection root chart is not certified"
                            }
                            super::SelectionProofRefusal::Coverage => {
                                "nonlinear selection eligible-domain coverage is incomplete"
                            }
                            super::SelectionProofRefusal::Boundary => {
                                "nonlinear selection chart has no proved guard or boundary margin"
                            }
                            super::SelectionProofRefusal::Resource => {
                                "nonlinear selection proof resources are exhausted"
                            }
                        },
                    });
                }
            };
            chart.validate(&request, verifier.identity())?;
            // Retain the already admitted immutable sources, even if a verifier returned
            // an independently allocated but structurally equal source projection.
            for (scope, alternative) in chart.alternatives.iter_mut().zip(&alternatives) {
                scope.program = alternative.program.clone();
            }
            if retained_chart_bytes(&chart)? > self.chart_bytes {
                return Err(MathError::Limit(
                    "nonlinear selection retained chart extent",
                ));
            }
            fresh_chart = Some(chart);
        }
        checkpoint()?;
        let regime = &mut self.alternatives[winner];
        if regime
            .problem
            .unknowns
            .iter()
            .zip(&point)
            .any(|(unknown, value)| *value <= unknown.lower || *value >= unknown.upper)
        {
            return Err(MathError::Domain {
                source_id: id,
                requirement: "branch-local implicit root is on its admissibility boundary",
            });
        }
        let derivatives = match self.numerical.as_ref().filter(|p| {
            numerical_hit && p.selected.id == id && p.order >= order.max(DerivativeOrder::First)
        }) {
            Some(product) => pse_kernels::ProviderValues {
                values: point.clone(),
                jacobian: product.selected.jacobian.clone(),
                hessians: product.selected.hessians.clone(),
            },
            None => regime.problem.derivatives(
                parameters,
                &point,
                order.max(DerivativeOrder::First),
                &configurations[winner],
                cancel,
            )?,
        };
        checkpoint()?;
        if regime
            .assess(parameters, &point, DerivativeOrder::First, cancel)?
            .is_none()
        {
            return Err(MathError::Domain {
                source_id: id,
                requirement: "certified nonlinear winning proposal is ineligible",
            });
        }
        checkpoint()?;
        let mut chain_proof = None;
        if let Some(chart) = fresh_chart {
            if self.sheet.is_some()
                && self.chart.as_ref().is_some_and(|previous| {
                    previous.selection == chart.selection
                        && previous.winner == chart.winner
                        && previous
                            .alternatives
                            .iter()
                            .zip(&chart.alternatives)
                            .all(|(a, b)| {
                                a.id == b.id
                                    && a.residual_identity == b.residual_identity
                                    && a.program == b.program
                                    && a.unknowns.iter().zip(&b.unknowns).all(|(a, b)| {
                                        a.id == b.id
                                            && a.lower.to_bits() == b.lower.to_bits()
                                            && a.upper.to_bits() == b.upper.to_bits()
                                    })
                            })
                })
            {
                let previous = self
                    .chart
                    .as_ref()
                    .ok_or_else(|| MathError::Contract("missing selected sheet chart".into()))?;
                let mut connected = self.sheet_parameters.len() == chart.parameters.len()
                    && chart
                        .parameters
                        .iter()
                        .zip(&self.sheet_parameters)
                        .all(|(i, p)| i.interior_contains(*p))
                    && previous
                        .existence
                        .iter()
                        .zip(&chart.uniqueness)
                        .all(|(e, u)| u.lower < e.lower && e.upper < u.upper);
                if !connected
                    && let Some(anchor) = common_parameter_anchor(
                        previous,
                        &chart,
                        &self.sheet_parameters,
                        parameters,
                    )
                {
                    let candidate = previous
                        .uniqueness
                        .iter()
                        .zip(&chart.uniqueness)
                        .map(|(a, b)| {
                            let l = a.lower.max(b.lower);
                            let u = a.upper.min(b.upper);
                            l + (u - l) * 0.5
                        })
                        .collect::<Vec<_>>();
                    if previous
                        .uniqueness
                        .iter()
                        .zip(&chart.uniqueness)
                        .all(|(a, b)| a.lower.max(b.lower) < a.upper.min(b.upper))
                    {
                        let alternatives = self
                            .alternatives
                            .iter()
                            .map(|r| {
                                Ok(super::SelectionAlternative {
                                    id: r.problem.id,
                                    program: r.isolation.as_ref().ok_or_else(|| {
                                        MathError::Contract("missing connection projection".into())
                                    })?,
                                    residual_identity: r.problem.identity,
                                    unknowns: &r.problem.unknowns,
                                })
                            })
                            .collect::<Result<Vec<_>, MathError>>()?;
                        let connection = super::SelectionProofRequest {
                            selection: self.id,
                            alternatives: &alternatives,
                            winner,
                            parameters: &anchor,
                            candidate: &candidate,
                            order: order.max(DerivativeOrder::First),
                            time_limit: deadline.saturating_duration_since(Instant::now()),
                            cancel,
                        };
                        connected = verifier.connect(&connection, previous, &chart)?;
                        checkpoint()?;
                    }
                }
                if !connected {
                    let alternatives = self
                        .alternatives
                        .iter()
                        .map(|r| {
                            Ok(super::SelectionAlternative {
                                id: r.problem.id,
                                program: r.isolation.as_ref().ok_or_else(|| {
                                    MathError::Contract("missing chart-chain projection".into())
                                })?,
                                residual_identity: r.problem.identity,
                                unknowns: &r.problem.unknowns,
                            })
                        })
                        .collect::<Result<Vec<_>, MathError>>()?;
                    let endpoint = super::SelectionProofRequest {
                        selection: self.id,
                        alternatives: &alternatives,
                        winner,
                        parameters,
                        candidate: &point,
                        order: order.max(DerivativeOrder::First),
                        time_limit: deadline.saturating_duration_since(Instant::now()),
                        cancel,
                    };
                    let chain = super::ChartChainRequest {
                        endpoint: &endpoint,
                        previous,
                        next: &chart,
                        origin: &self.sheet_parameters,
                        coverage: self.chain_coverage,
                    };
                    chain.validate(verifier.identity())?;
                    let evidence = verifier.connect_chain(&chain)?;
                    let work = evidence.work();
                    self.chain_work = Some(work);
                    self.observed_chain.charts = self
                        .observed_chain
                        .charts
                        .checked_add(work.charts)
                        .ok_or(MathError::Limit("selected chart work observation overflow"))?;
                    self.observed_chain.connections = self
                        .observed_chain
                        .connections
                        .checked_add(work.connections)
                        .ok_or(MathError::Limit(
                            "selected anchor work observation overflow",
                        ))?;
                    self.observed_chain.proof_cells = self
                        .observed_chain
                        .proof_cells
                        .checked_add(work.proof_cells)
                        .ok_or(MathError::Limit("selected proof work observation overflow"))?;
                    match evidence {
                        super::ChartChainEvidence::Connected(proof)
                            if (proof.coverage == self.chain_coverage
                                || proof.coverage
                                    == super::ChartChainCoverage::SelectedFunction)
                                && proof.charts > 0
                                && proof.connections == proof.charts.saturating_add(1)
                                && proof.proof_cells
                                    >= proof.connections.saturating_add(proof.charts) =>
                        {
                            checkpoint()?;
                            connected = true;
                            chain_proof = Some(proof);
                        }
                        super::ChartChainEvidence::Interrupted(_) => {
                            return Err(MathError::Cancelled);
                        }
                        super::ChartChainEvidence::Refused {
                            reason: super::SelectionProofRefusal::Resource,
                            ..
                        }
                        | super::ChartChainEvidence::Incomplete(
                            super::SelectionProofRefusal::Resource,
                        ) => {
                            return Err(MathError::Limit(
                                "selected root chart-chain proof resources",
                            ));
                        }
                        super::ChartChainEvidence::Refused { reason, .. }
                        | super::ChartChainEvidence::Incomplete(reason) => {
                            return Err(MathError::Domain {
                                source_id: self.id,
                                requirement: match reason {
                                    super::SelectionProofRefusal::Boundary => {
                                        "selected root chart-chain guard or boundary margin is not certified"
                                    }
                                    super::SelectionProofRefusal::Unsupported => {
                                        "selected root chart-chain projection is unsupported"
                                    }
                                    super::SelectionProofRefusal::Chart => {
                                        "selected root chart-chain overlap is not certified"
                                    }
                                    super::SelectionProofRefusal::Coverage => {
                                        "selected root chart-chain coverage is not certified"
                                    }
                                    super::SelectionProofRefusal::Resource => {
                                        "selected root chart-chain proof resources"
                                    }
                                },
                            });
                        }
                        _ => {}
                    }
                    checkpoint()?;
                }
                if !connected {
                    return Err(MathError::Domain {
                        source_id: self.id,
                        requirement: "selected root sheet continuity is not certified",
                    });
                }
            }
            self.chart = Some(chart);
            self.last_chain = chain_proof;
        }
        if order > DerivativeOrder::Value {
            self.derivative_branch = Some(id);
        }
        let selected = SelectedRegime {
            id,
            values: point,
            jacobian: if order >= DerivativeOrder::First {
                derivatives.jacobian
            } else {
                vec![]
            },
            hessians: if order >= DerivativeOrder::Second {
                derivatives.hessians
            } else {
                vec![]
            },
            examined: self.alternatives.len(),
            eligible,
        };
        if order > DerivativeOrder::Value
            && self.sheet.is_none()
            && let Some(chart) = &self.chart
        {
            let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::SelectedRootSheetV1);
            h.id(&self.id).id(&id).hash(&chart.verifier_identity);
            for interval in &chart.uniqueness {
                h.f64(interval.lower).f64(interval.upper);
            }
            for p in parameters {
                h.f64(*p);
            }
            self.sheet = Some(h.finish_hash());
        }
        self.sheet_parameters = parameters.to_vec();
        self.retain_product(numerical_key, proposals, selected.clone(), order);
        Ok(selected)
    }

    fn evaluate_operational(
        &mut self,
        parameters: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<SelectedRegime, MathError> {
        let deadline = Instant::now()
            .checked_add(self.time_limit)
            .ok_or(MathError::Limit("implicit regime deadline"))?;
        let scope = self.scope.clone();
        let deadline = scope
            .as_ref()
            .and_then(pse_kernels::ExecutionScope::deadline)
            .map_or(deadline, |outer| deadline.min(outer));
        let checkpoint = || {
            if let Some(scope) = &scope {
                scope.check().map_err(crate::error::scope_error)?;
            }
            if cancel.load(Ordering::Acquire) {
                Err(MathError::Cancelled)
            } else if Instant::now() >= deadline {
                Err(MathError::Limit("implicit regime time"))
            } else {
                Ok(())
            }
        };
        let local = order > DerivativeOrder::Value || self.derivative_branch.is_some();
        let assessment_order = if local {
            DerivativeOrder::First
        } else {
            DerivativeOrder::Value
        };
        let mut configurations = Vec::with_capacity(self.alternatives.len());
        for regime in &mut self.alternatives {
            checkpoint()?;
            configurations.push(regime.configuration.resolve(
                &mut regime.problem,
                parameters,
                cancel,
                None,
            )?);
        }
        let numerical_key = self.product_key(parameters, &configurations);
        let numerical_hit = self.product_matches(numerical_key)
            && self
                .numerical
                .as_ref()
                .is_some_and(|p| p.proposals.iter().all(Option::is_some));
        let mut proposals = vec![None; self.alternatives.len()];
        let mut candidates = Vec::with_capacity(self.alternatives.len());
        for (index, (regime, options)) in self
            .alternatives
            .iter_mut()
            .zip(&mut configurations)
            .enumerate()
        {
            checkpoint()?;
            options.time_limit = options
                .time_limit
                .min(deadline.saturating_duration_since(Instant::now()));
            if options.time_limit.is_zero() {
                return Err(MathError::Limit("implicit regime time"));
            }
            let point = match self
                .numerical
                .as_ref()
                .filter(|_| numerical_hit)
                .and_then(|p| p.proposals[index].clone())
            {
                Some(point) => point,
                None => regime
                    .solver
                    .solve(regime.problem.clone(), parameters, options, cancel)?,
            };
            proposals[index] = Some(point.clone());
            checkpoint()?;
            // The capability may return an iterate; generic original-space validation remains mandatory.
            regime.problem.verify(parameters, &point, options, cancel)?;
            checkpoint()?;
            if local
                && regime
                    .problem
                    .unknowns
                    .iter()
                    .zip(&point)
                    .any(|(u, y)| *y <= u.lower || *y >= u.upper)
            {
                return Err(MathError::Domain {
                    source_id: regime.problem.id,
                    requirement: "branch-local implicit root is on its admissibility boundary",
                });
            }
            // Regularity of every alternative is needed: a failed or singular rival
            // cannot be treated as evidence that the winning branch persists nearby.
            let derivatives = regime.problem.derivatives(
                parameters,
                &point,
                if local {
                    order.max(DerivativeOrder::First)
                } else {
                    DerivativeOrder::Value
                },
                options,
                cancel,
            )?;
            if let Some((score, tolerance)) =
                regime.assess(parameters, &point, assessment_order, cancel)?
            {
                candidates.push((regime.problem.id, point, score, tolerance, derivatives));
            }
        }
        checkpoint()?;
        let best = candidates
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.2.total_cmp(&b.2))
            .map(|(i, _)| i)
            .ok_or(MathError::Domain {
                source_id: self.id,
                requirement: "no eligible implicit regime",
            })?;
        let score = candidates[best].2;
        let tolerance = candidates[best].3;
        if candidates
            .iter()
            .filter(|c| (c.2 - score).abs() <= tolerance.max(c.3))
            .count()
            != 1
        {
            return Err(MathError::Domain {
                source_id: self.id,
                requirement: "implicit regime selection is tied within physical tolerance",
            });
        }
        let eligible = candidates.len();
        let (id, values, _, _, derivatives) = candidates.swap_remove(best);
        if let Some(bound) = self.derivative_branch.filter(|branch| *branch != id) {
            // A typed, recoverable refusal: the outer solve counts it per iteration.
            return Err(MathError::Provider {
                source_id: self.id,
                provider: self.id,
                cause: pse_kernels::ProviderError::RegimeCrossing {
                    selector: self.id,
                    bound,
                    selected: id,
                },
            });
        }
        checkpoint()?;
        if order > DerivativeOrder::Value {
            self.derivative_branch = Some(id);
        }
        let selected = SelectedRegime {
            id,
            values,
            jacobian: if order >= DerivativeOrder::First {
                derivatives.jacobian
            } else {
                vec![]
            },
            hessians: if order >= DerivativeOrder::Second {
                derivatives.hessians
            } else {
                vec![]
            },
            examined: self.alternatives.len(),
            eligible,
        };
        self.retain_product(numerical_key, proposals, selected.clone(), order);
        Ok(selected)
    }
}

// A shared interior anchor on the actual straight parameter segment. Convex boxes
// cover each adjoining subsegment; the root connection still needs verifier proof.
fn common_parameter_anchor(
    previous: &super::SelectionChart,
    next: &super::SelectionChart,
    from: &[f64],
    to: &[f64],
) -> Option<Vec<f64>> {
    if from.len() != to.len()
        || from.len() != previous.parameters.len()
        || from.len() != next.parameters.len()
    {
        return None;
    }
    let mut lower = 0.0f64;
    let mut upper = 1.0f64;
    for (((a, b), from), to) in previous
        .parameters
        .iter()
        .zip(&next.parameters)
        .zip(from)
        .zip(to)
    {
        let l = a.lower.max(b.lower);
        let u = a.upper.min(b.upper);
        if l >= u {
            return None;
        }
        let delta = to - from;
        if delta == 0.0 {
            if *from <= l || *from >= u {
                return None;
            }
        } else {
            let x = (l - from) / delta;
            let y = (u - from) / delta;
            lower = lower.max(x.min(y));
            upper = upper.min(x.max(y));
        }
    }
    if lower >= upper {
        return None;
    }
    let t = lower + (upper - lower) * 0.5;
    let anchor = from
        .iter()
        .zip(to)
        .map(|(from, to)| from + t * (to - from))
        .collect::<Vec<_>>();
    previous
        .parameters
        .iter()
        .zip(&next.parameters)
        .zip(&anchor)
        .all(|((a, b), v)| a.interior_contains(*v) && b.interior_contains(*v))
        .then_some(anchor)
}

// One worker retains at most one chart. Reserve conservative vector capacities;
// the immutable source DAGs already belong to the admitted factory programs.
pub(super) fn certificate_bytes(
    inputs: usize,
    unknowns: impl Iterator<Item = usize>,
) -> Result<usize, MathError> {
    let mut bytes = size_of::<Option<PointRefinement>>()
        + size_of::<u64>()
        + size_of::<Option<super::SelectionChart>>()
        + size_of::<Option<SelectionProduct>>()
        + size_of::<Option<super::ChartChainProof>>()
        + size_of::<super::ChartChainCoverage>()
        + size_of::<Option<super::ChartChainWork>>()
        + size_of::<super::ChartChainWork>()
        + size_of::<u64>();
    let mut winner_width = 0usize;
    for width in unknowns {
        winner_width = winner_width.max(width);
        let refined_vectors = width
            .checked_mul(4 * size_of::<f64>())
            .ok_or(MathError::Limit("selected refined controls extent"))?;
        let metadata = (size_of::<Option<Vec<f64>>>()
            + size_of::<Option<Options>>()
            + size_of::<Option<pse_ids::ContentHash>>()
            + 2 * size_of::<crate::guarded::PreparedSupport>()
            + size_of::<Option<Arc<crate::factorable::RootIsolationProgram>>>())
        .checked_add(refined_vectors)
        .ok_or(MathError::Limit("selected refined controls extent"))?;
        bytes = bytes
            .checked_add(super::numerical_product_bytes(inputs, width)?)
            .and_then(|bytes| bytes.checked_add(metadata))
            .ok_or(MathError::Limit("selected numerical product extent"))?;
        bytes = bytes
            .checked_add(
                size_of::<super::SelectionScope>()
                    .checked_mul(2)
                    .and_then(|scope| {
                        width
                            .checked_mul(size_of::<super::Unknown>())
                            .and_then(|bounds| bounds.checked_mul(2))
                            .and_then(|bounds| scope.checked_add(bounds))
                    })
                    .ok_or(MathError::Limit("nonlinear selection certificate extent"))?,
            )
            .ok_or(MathError::Limit("nonlinear selection certificate extent"))?;
    }
    inputs
        .checked_add(
            winner_width
                .checked_mul(3)
                .ok_or(MathError::Limit("nonlinear selection certificate extent"))?,
        )
        .and_then(|intervals| intervals.checked_mul(size_of::<super::ProofInterval>()))
        .and_then(|intervals| intervals.checked_mul(2))
        .and_then(|intervals| bytes.checked_add(intervals))
        .ok_or(MathError::Limit("nonlinear selection certificate extent"))
}
fn chart_extent(alternatives: &[Regime]) -> Result<usize, MathError> {
    certificate_bytes(
        alternatives
            .first()
            .map_or(0, |regime| regime.problem.inputs),
        alternatives
            .iter()
            .map(|regime| regime.problem.unknowns.len()),
    )
}
fn retained_chart_bytes(chart: &super::SelectionChart) -> Result<usize, MathError> {
    let scopes = chart
        .alternatives
        .capacity()
        .checked_mul(size_of::<super::SelectionScope>())
        .and_then(|scopes| size_of::<Option<super::SelectionChart>>().checked_add(scopes))
        .ok_or(MathError::Limit(
            "nonlinear selection retained chart extent",
        ))?;
    let bounds = chart
        .alternatives
        .iter()
        .try_fold(scopes, |bytes, scope| {
            scope
                .unknowns
                .capacity()
                .checked_mul(size_of::<super::Unknown>())
                .and_then(|extent| bytes.checked_add(extent))
        })
        .ok_or(MathError::Limit(
            "nonlinear selection retained chart extent",
        ))?;
    [&chart.parameters, &chart.existence, &chart.uniqueness]
        .iter()
        .try_fold(bounds, |bytes, intervals| {
            intervals
                .capacity()
                .checked_mul(size_of::<super::ProofInterval>())
                .and_then(|extent| bytes.checked_add(extent))
        })
        .ok_or(MathError::Limit(
            "nonlinear selection retained chart extent",
        ))
}
