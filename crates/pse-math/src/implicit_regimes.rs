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
                    problem: r.residual.problem(scope.clone())?,
                    solver: r.residual.solver.clone(),
                    configuration: super::ConfigurationWorker::new(
                        r.residual.configuration.clone(),
                        r.residual.hints.as_ref(),
                        r.residual.terms.as_ref(),
                    ),
                    eligibility: r.eligibility.worker_scoped(scope.clone()),
                    criterion: r.criterion.worker_scoped(scope.clone()),
                    isolation: r.isolation.clone(),
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
            crate::implicit::solver_identity("test.regime-roots.v1")
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
                        lower: value - 0.25,
                        upper: value + 0.25,
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
        assert_eq!(calls.load(Ordering::Relaxed), 2);
        selected
            .evaluate(&[-1.5], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 3);
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
        assert_eq!(calls.load(Ordering::Relaxed), 4);
        let program = selected.alternatives[1].isolation.as_ref().unwrap();
        let mut changed = program.as_ref().clone();
        changed.nodes.push(crate::factorable::Node::Const(
            crate::factorable::Constant::Float(0.),
        ));
        selected.alternatives[1].isolation = Some(Arc::new(changed));
        selected
            .evaluate(&[-1.5], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 5);
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
        assert_eq!(calls.load(Ordering::Relaxed), 5);
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
        assert_eq!(calls.load(Ordering::Relaxed), 5);
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            selected.evaluate(&[-1.5], DerivativeOrder::Second, &cancel),
            Err(MathError::Cancelled)
        ));
        assert_eq!(calls.load(Ordering::Relaxed), 5);
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
        })
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
        let mut candidates = Vec::with_capacity(self.alternatives.len());
        let mut failed_proposal = None;
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
            let proposal = regime
                .solver
                .solve(regime.problem.clone(), parameters, options, cancel)
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
            self.chart = None;
            let evidence = verifier.certify(&request)?;
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
        let derivatives = regime.problem.derivatives(
            parameters,
            &point,
            order.max(DerivativeOrder::First),
            &configurations[winner],
            cancel,
        )?;
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
        if let Some(chart) = fresh_chart {
            self.chart = Some(chart);
        }
        if order > DerivativeOrder::Value {
            self.derivative_branch = Some(id);
        }
        Ok(SelectedRegime {
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
        })
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
        let mut candidates = Vec::with_capacity(self.alternatives.len());
        for regime in &mut self.alternatives {
            checkpoint()?;
            let mut options =
                regime
                    .configuration
                    .resolve(&mut regime.problem, parameters, cancel, None)?;
            options.time_limit = options
                .time_limit
                .min(deadline.saturating_duration_since(Instant::now()));
            if options.time_limit.is_zero() {
                return Err(MathError::Limit("implicit regime time"));
            }
            let point =
                regime
                    .solver
                    .solve(regime.problem.clone(), parameters, &options, cancel)?;
            checkpoint()?;
            // The capability may return an iterate; generic original-space validation remains mandatory.
            regime
                .problem
                .verify(parameters, &point, &options, cancel)?;
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
                &options,
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
        Ok(SelectedRegime {
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
        })
    }
}

// One worker retains at most one chart. Reserve conservative vector capacities;
// the immutable source DAGs already belong to the admitted factory programs.
pub(super) fn certificate_bytes(
    inputs: usize,
    unknowns: impl Iterator<Item = usize>,
) -> Result<usize, MathError> {
    let mut bytes = size_of::<Option<super::SelectionChart>>();
    let mut winner_width = 0usize;
    for width in unknowns {
        winner_width = winner_width.max(width);
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
                .checked_mul(2)
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
