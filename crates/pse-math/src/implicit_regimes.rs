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
    pub residual: super::Factory,
    pub eligibility: Arc<CompiledBody>,
    pub criterion: Arc<CompiledBody>,
}
/// Attempt-bound alternative selection using the same injected native root capability.
#[derive(Debug)]
pub struct RegimeFactory {
    pub spec: pse_kernels::ProviderSpec,
    pub alternatives: Vec<RegimeFactoryBranch>,
    pub maximum_regimes: usize,
    pub time_limit: Duration,
    pub cancel: Arc<AtomicBool>,
}
impl pse_kernels::ProviderFactory for RegimeFactory {
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        &self.spec
    }
    fn configuration_key(&self) -> pse_ids::ContentHash {
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::ImplicitRegimeConfigurationV1);
        hash.hash(&self.spec.identity())
            .u64(self.maximum_regimes as u64)
            .u64(self.time_limit.as_secs())
            .u64(self.time_limit.subsec_nanos() as u64)
            .u64(self.alternatives.len() as u64);
        for branch in &self.alternatives {
            hash.hash(&branch.residual.configuration_key());
        }
        hash.finish_hash()
    }
    fn create(&self) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        self.create_scoped(self.cancel.clone())
    }
    fn create_scoped(
        &self,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
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
                    problem: r.residual.problem(cancel.clone())?,
                    solver: r.residual.solver.clone(),
                    configuration: super::ConfigurationWorker::new(
                        r.residual.configuration.clone(),
                        r.residual.hints.as_ref(),
                        r.residual.terms.as_ref(),
                    ),
                    eligibility: r.eligibility.worker(),
                    criterion: r.criterion.worker(),
                })
            })
            .collect::<Result<_, _>>()?;
        let selection = RegimeSelection::new(
            self.spec.id,
            alternatives,
            self.maximum_regimes,
            self.time_limit,
        )
        .map_err(super::provider_error)?;
        Ok(Box::new(SelectedProvider {
            spec: self.spec.clone(),
            selection,
            cancel,
        }))
    }
}
#[derive(Debug)]
struct SelectedProvider {
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
            jacobian: request.outputs.iter().flat_map(|i| {
                let n = inputs.len();
                selected.jacobian.get(i * n..(i + 1) * n).unwrap_or_default().iter().copied()
            }).collect(),
            hessians: request.outputs.iter().flat_map(|i| {
                let n = inputs.len() * inputs.len();
                selected.hessians.get(i * n..(i + 1) * n).unwrap_or_default().iter().copied()
            }).collect(),
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
}

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
    fn identity(&self) -> pse_ids::ContentHash { crate::implicit::solver_identity("test.regime-roots.v1") }
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
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let unit = registry.quantity_type(q).unwrap().canonical_unit;
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
                            unit,
                            pse_quantity::literal::LiteralContext::Explicit { quantity_type: q },
                            id,
                        )
                        .unwrap()
                    };
                    let outputs = match kind {
                        0 => {
                            let root = literal(&mut b, value);
                            vec![b.binary(Binary::Sub, y, root, None, id).unwrap()]
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
                Regime::new(problem, options, body(1), body(2), Arc::new(Roots {fail:None, wrong:false})).unwrap()
            })
            .collect();
        RegimeSelection::new(SemanticId::NIL, alternatives, 2, Duration::from_secs(1)).unwrap()
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
        let jet = selected.evaluate(&[2.], DerivativeOrder::Second, &cancel).unwrap();
        assert_eq!(jet.jacobian, vec![0.0]);
        assert_eq!(jet.hessians, vec![0.0]);
        assert!(matches!(selected.evaluate(&[-2.], DerivativeOrder::First, &cancel),
            Err(MathError::Domain { requirement: "implicit derivative trial crosses the bound regime", .. })));
        assert!(matches!(selected.evaluate(&[-2.], DerivativeOrder::Value, &cancel),
            Err(MathError::Domain { requirement: "implicit derivative trial crosses the bound regime", .. })));
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
        selected.alternatives[1].solver = Arc::new(Roots { fail: Some(failed), wrong: false });
        assert!(matches!(selected.evaluate(&[-2.], DerivativeOrder::Value, &cancel),
            Err(MathError::Domain { requirement: "controlled nonconvergence", .. })));
        selected.alternatives[1].solver = Arc::new(Roots { fail: None, wrong: true });
        assert!(matches!(selected.evaluate(&[-2.], DerivativeOrder::Value, &cancel),
            Err(MathError::Domain { requirement: "inner root did not satisfy original residual budgets", .. })));
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
        })
    }
}
/// The selected values retain the winning semantic regime identity and evidence extent.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedRegime {
    pub id: SemanticId,
    pub values: Vec<f64>,
    /// Local derivatives of the selected regular branch, in parameter order.
    pub jacobian: Vec<f64>,
    /// Local second partials, output-major full symmetric matrices.
    pub hessians: Vec<f64>,
    pub examined: usize,
    pub eligible: usize,
}
/// Alternative solves share a total wall allowance and one outer cancellation owner.
#[derive(Debug)]
pub struct RegimeSelection {
    id: SemanticId,
    alternatives: Vec<Regime>,
    time_limit: Duration,
    derivative_branch: Option<SemanticId>,
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
        Ok(Self {
            id,
            alternatives,
            time_limit,
            derivative_branch: None,
        })
    }
    /// Minimize the authored criterion over all verified eligible alternatives. Ties refuse.
    /// Derivatives additionally verify all alternatives are regular and their eligibility
    /// predicates and scores are locally continuous. The first derivative request binds
    /// this worker to its winning branch; later crossings refuse. No transition or global
    /// stability claim follows from this local contract.
    pub fn evaluate(
        &mut self,
        parameters: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<SelectedRegime, MathError> {
        let deadline = Instant::now()
            .checked_add(self.time_limit)
            .ok_or(MathError::Limit("implicit regime deadline"))?;
        let checkpoint = || {
            if cancel.load(Ordering::Acquire) {
                Err(MathError::Cancelled)
            } else if Instant::now() >= deadline {
                Err(MathError::Limit("implicit regime time"))
            } else {
                Ok(())
            }
        };
        let local = order > DerivativeOrder::Value || self.derivative_branch.is_some();
        let assessment_order = if local { DerivativeOrder::First } else { DerivativeOrder::Value };
        let mut candidates = Vec::with_capacity(self.alternatives.len());
        for regime in &mut self.alternatives {
            checkpoint()?;
            let mut options =
                regime
                    .configuration
                    .resolve(&mut regime.problem, parameters, cancel)?;
            options.time_limit = options
                .time_limit
                .min(deadline.saturating_duration_since(Instant::now()));
            if options.time_limit.is_zero() {
                return Err(MathError::Limit("implicit regime time"));
            }
            let point = regime.solver.solve(regime.problem.clone(), parameters, &options, cancel)?;
            // The capability may return an iterate; generic original-space validation remains mandatory.
            regime
                .problem
                .verify(parameters, &point, &options, cancel)?;
            checkpoint()?;
            if local && regime.problem.unknowns.iter().zip(&point).any(|(u, y)| *y <= u.lower || *y >= u.upper) {
                return Err(MathError::Domain { source_id: regime.problem.id, requirement: "branch-local implicit root is on its admissibility boundary" });
            }
            // Regularity of every alternative is needed: a failed or singular rival
            // cannot be treated as evidence that the winning branch persists nearby.
            let derivatives = regime.problem.derivatives(parameters, &point,
                if local { order.max(DerivativeOrder::First) } else { DerivativeOrder::Value }, &options, cancel)?;
            let inputs = point.iter().chain(parameters).copied().collect::<Vec<_>>();
            let mut providers = regime
                .problem
                .providers
                .lock()
                .map_err(|_| MathError::Library("regime provider lock poisoned".into()))?;
            let eligible = regime.eligibility.evaluate(
                &inputs,
                assessment_order,
                &mut providers,
                cancel,
            )?;
            match eligible.values.as_slice() {
                [value] if *value == 0. => continue,
                [value] if *value == 1. => {}
                _ => {
                    return Err(MathError::Contract(
                        "regime eligibility requires one Boolean indicator".into(),
                    ));
                }
            }
            let criterion = regime.criterion.evaluate(
                &inputs,
                assessment_order,
                &mut providers,
                cancel,
            )?;
            let [score, tolerance] = criterion.values.as_slice() else {
                return Err(MathError::Contract(
                    "regime criterion requires score and tolerance".into(),
                ));
            };
            if !score.is_finite() || !tolerance.is_finite() || *tolerance < 0. {
                return Err(MathError::Domain {
                    source_id: regime.problem.id,
                    requirement: "finite regime score and nonnegative tie tolerance",
                });
            }
            candidates.push((regime.problem.id, point, *score, *tolerance, derivatives));
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
        if self.derivative_branch.is_some_and(|branch| branch != id) {
            return Err(MathError::Domain { source_id: self.id, requirement: "implicit derivative trial crosses the bound regime" });
        }
        if order > DerivativeOrder::Value { self.derivative_branch = Some(id); }
        checkpoint()?;
        Ok(SelectedRegime {
            id,
            values,
            jacobian: if order >= DerivativeOrder::First { derivatives.jacobian } else { vec![] },
            hessians: if order >= DerivativeOrder::Second { derivatives.hessians } else { vec![] },
            examined: self.alternatives.len(),
            eligible,
        })
    }
}
