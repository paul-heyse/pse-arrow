// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared maps retain their physical coordinates, original residual and frozen scaling.
//! They do not pass through equation normalization, warm transport or retained sessions.
use crate::{
    ProblemError,
    kinsol::{Function, Session, Settings},
    quality::{self, Tolerances},
    solve::{Backend, Compatibility, Controls, Execution, ResolvedAccuracy, SolveReport},
};

/// Inputs of one declared fixed-point or Picard application.
#[derive(Debug)]
pub struct MapInput<'a> {
    /// Explicit physical map/splitting, scales, authored start and authorized restarts.
    pub problem: super::Problem<'a>,
    /// Finite controls of this application.
    pub controls: &'a Controls,
    /// Original acceptance budgets resolved by the caller.
    pub accuracy: &'a ResolvedAccuracy,
    /// Original cancellation, deadline, work and storage allowance.
    pub execution: Execution,
    /// Original physical acceptance tolerances.
    pub tolerances: &'a Tolerances,
    /// Original source/profile/data identities, used for evidence rather than reuse.
    pub compatibility: Compatibility,
}
/// Select the actual declared map route without constructing a native session.
/// # Errors
/// The binary does not link map execution or the selected method is not a map.
pub fn map_route(settings: &Settings) -> Result<crate::routing::Route, ProblemError> {
    let adapter = super::adapter(Backend::Kinsol);
    if !adapter.linked() {
        return Err(super::unlinked(Backend::Kinsol));
    }
    let capability = adapter.capability_for(&super::BackendSettings::Kinsol(settings.method))?;
    if capability.derivatives != crate::solve::DerivativeCapability::ValueOnly {
        return Err(ProblemError::Contract(
            "declared map method is not value-only".into(),
        ));
    }
    Ok(crate::routing::Route::Native(adapter.backend()))
}
/// Execute a declared map in a fresh native session, qualify its original residual and
/// destroy all native state before returning. The caller owns the admitted worker scope.
/// # Errors
/// An equation was supplied as a map, settings/source differ, or native execution fails.
pub fn map(input: MapInput<'_>) -> Result<SolveReport, ProblemError> {
    super::adapter(Backend::Kinsol).execute_map(input)
}
pub(super) fn execute(input: MapInput<'_>) -> Result<SolveReport, ProblemError> {
    let _original = super::original_scope(&input.execution);
    input.execution.check()?;
    let (function, initial, settings, restarts) = match input.problem {
        super::Problem::FixedPoint {
            oracle,
            initial,
            settings,
            restarts,
        } => (Function::FixedPoint(oracle), initial, settings, restarts),
        super::Problem::Picard {
            oracle,
            linear,
            initial,
            settings,
            restarts,
        } => (
            Function::Picard { oracle, linear },
            initial,
            settings,
            restarts,
        ),
        _ => return Err(super::representation(Backend::Kinsol)),
    };
    if input.compatibility.backend != Backend::Kinsol {
        return Err(ProblemError::Contract(
            "declared map native identity differs".into(),
        ));
    }
    let adapter = super::adapter(Backend::Kinsol);
    let selected = super::BackendSettings::Kinsol(settings.method);
    let capability = adapter.capability_for(&selected)?;
    if capability.derivatives != crate::solve::DerivativeCapability::ValueOnly {
        return Err(ProblemError::Contract(
            "declared map method is not value-only".into(),
        ));
    }
    // Validate before allocation and retain source-owned structure and original checks.
    input.controls.validate()?;
    if input.controls.threads != 1 || !input.controls.options.is_empty() {
        return Err(ProblemError::Unsupported(
            "declared maps require one thread and typed settings".into(),
        ));
    }
    if input.controls.reuse == crate::solve::ReusePolicy::RequireReuse {
        return Err(ProblemError::Unsupported(
            "declared maps require a fresh native session".into(),
        ));
    }
    if input.controls.start == crate::solve::StartPolicy::PreviousAccepted {
        return Err(ProblemError::Unsupported(
            "declared maps consume the authored physical start".into(),
        ));
    }
    if initial.len() != function.contract().variables.len()
        || initial.iter().any(|value| !value.is_finite())
    {
        return Err(ProblemError::Contract(
            "declared map physical start dimensions/values".into(),
        ));
    }
    settings.validate(&function)?;
    input
        .tolerances
        .validate(initial.len(), function.contract().rows.len())?;
    if let Some(schedule) = restarts {
        if !matches!(function, Function::FixedPoint(_)) {
            return Err(ProblemError::Contract(
                "declared restarts require a frozen fixed-point map".into(),
            ));
        }
        schedule.validate(function.contract(), input.controls)?;
    }
    let mut session = Session::new(
        function,
        settings.clone(),
        input.execution.clone(),
        input.compatibility,
    )?;
    let mut report = session.solve_with_restarts(
        initial,
        input.controls,
        input.accuracy,
        input.execution.clone(),
        input.tolerances,
        None,
        restarts,
    )?;
    // A map iterate is diagnostic/result evidence, never an automatic future root seed.
    report.warm_start = None;
    quality::qualify(&mut report, input.accuracy);
    drop(session);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        NleOracle, OracleContract,
        kinsol::{FixedPointOracle, Method, Strategy},
        solver_tests::{Polynomial, execution, stamp},
    };
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Debug)]
    struct OriginalMap {
        source: Polynomial,
        starts: Arc<Mutex<Vec<f64>>>,
        drops: Arc<AtomicUsize>,
        called: bool,
        residual_bias: f64,
    }
    impl Drop for OriginalMap {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    impl FixedPointOracle for OriginalMap {
        fn contract(&self) -> &OracleContract {
            &self.source.c
        }
        fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.source.matrix.symbolic()
        }
        fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            if !self.called {
                self.starts.lock().unwrap().push(x[0]);
                self.called = true;
            }
            out[0] = 1.;
            Ok(())
        }
        fn original_residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            NleOracle::residual(&mut self.source, x, out)?;
            out[0] += self.residual_bias;
            Ok(())
        }
    }
    #[test]
    fn maps_keep_physical_starts_and_original_residual_with_fresh_owned_sessions() {
        let starts = Arc::new(Mutex::new(Vec::new()));
        let drops = Arc::new(AtomicUsize::new(0));
        let settings = Settings {
            method: Method {
                strategy: Strategy::FixedPoint,
                ..Default::default()
            },
            variable_scales: vec![37.],
            residual_scales: vec![19.],
            step_tolerance: 1e-10,
        };
        let controls = Controls::default();
        let accuracy = ResolvedAccuracy::verification();
        let tolerances = Tolerances {
            variables: vec![1e-8],
            rows: vec![1e-8],
            integrality: 1e-8,
        };
        for (initial, residual_bias) in [(9., 0.), (2., 1.)] {
            let mut source = Polynomial::new();
            source.c.derivatives = pse_kernels::DerivativeOrder::Value;
            let report = map(MapInput {
                problem: super::super::Problem::FixedPoint {
                    oracle: Box::new(OriginalMap {
                        source,
                        starts: starts.clone(),
                        drops: drops.clone(),
                        called: false,
                        residual_bias,
                    }),
                    initial: &[initial],
                    settings: &settings,
                    restarts: None,
                },
                controls: &controls,
                accuracy: &accuracy,
                execution: execution(),
                tolerances: &tolerances,
                compatibility: stamp(Backend::Kinsol),
            })
            .unwrap();
            assert_eq!(report.candidate.as_ref().unwrap().primal, vec![1.]);
            assert!(report.warm_start.is_none());
            if residual_bias == 0. {
                assert_eq!(
                    report.termination.category,
                    crate::solve::Termination::Success
                );
            } else {
                assert!(report.quality.as_ref().unwrap().normalized_max > 1.);
            }
        }
        assert_eq!(*starts.lock().unwrap(), vec![9., 2.]);
        assert_eq!(
            drops.load(Ordering::SeqCst),
            2,
            "native session releases each original oracle before returning"
        );
        assert_eq!(settings.variable_scales[0].to_bits(), 37_f64.to_bits());
        assert_eq!(settings.residual_scales[0].to_bits(), 19_f64.to_bits());
    }
    #[test]
    fn map_seam_preserves_authorized_restart_budget_and_rejects_hidden_start_reuse() {
        #[derive(Debug)]
        struct AffineMap {
            source: Polynomial,
            calls: Arc<Mutex<Vec<f64>>>,
        }
        impl FixedPointOracle for AffineMap {
            fn contract(&self) -> &OracleContract {
                &self.source.c
            }
            fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
                self.source.matrix.symbolic()
            }
            fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
                self.calls.lock().unwrap().push(x[0]);
                out[0] = 0.5 * x[0] + 0.5;
                Ok(())
            }
            fn original_residual(
                &mut self,
                x: &[f64],
                out: &mut [f64],
            ) -> Result<(), ProblemError> {
                out[0] = x[0] - 1.;
                Ok(())
            }
        }
        let settings = Settings {
            method: Method {
                strategy: Strategy::FixedPoint,
                ..Default::default()
            },
            variable_scales: vec![37.],
            residual_scales: vec![19.],
            step_tolerance: 1e-10,
        };
        let schedule = crate::kinsol::RestartSchedule {
            starts: vec![vec![1.], vec![42.]],
            iterations: 1,
        };
        let calls = Arc::new(Mutex::new(Vec::new()));
        for hidden_start in [false, true] {
            calls.lock().unwrap().clear();
            let controls = Controls {
                iterations: 2,
                history: 4,
                start: if hidden_start {
                    crate::solve::StartPolicy::PreviousAccepted
                } else {
                    crate::solve::StartPolicy::NoPriorStart
                },
                ..Default::default()
            };
            let mut source = Polynomial::new();
            source.c.derivatives = pse_kernels::DerivativeOrder::Value;
            let report = map(MapInput {
                problem: super::super::Problem::FixedPoint {
                    oracle: Box::new(AffineMap {
                        source,
                        calls: calls.clone(),
                    }),
                    initial: &[0.],
                    settings: &settings,
                    restarts: Some(&schedule),
                },
                controls: &controls,
                accuracy: &ResolvedAccuracy::verification(),
                execution: execution(),
                tolerances: &Tolerances {
                    variables: vec![1e-8],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
                compatibility: stamp(Backend::Kinsol),
            });
            if hidden_start {
                assert!(matches!(report, Err(ProblemError::Unsupported(_))));
                assert!(
                    calls.lock().unwrap().is_empty(),
                    "no native evaluation before start admission"
                );
                continue;
            }
            let report = report.unwrap();
            assert_eq!(
                report.metrics["recycle.native_attempts"],
                crate::solve::Metric::Integer(2)
            );
            assert!(report.evidence.work.iterations.unwrap() <= 2);
            assert!(report.quality.as_ref().unwrap().feasible());
            assert_eq!(report.candidate.unwrap().primal, vec![1.]);
            assert!(report.warm_start.is_none());
            let calls = calls.lock().unwrap();
            assert_eq!(calls[0], 0.);
            assert!(
                calls.contains(&1.),
                "the explicitly authorized restart was consumed"
            );
            assert!(
                !calls.contains(&42.),
                "accepted restart must not consume a successor start"
            );
        }
    }
    #[test]
    fn picard_executes_constant_splitting_without_requesting_original_jacobian_values() {
        #[derive(Debug)]
        struct ResidualOnly(Polynomial);
        impl NleOracle for ResidualOnly {
            fn contract(&self) -> &OracleContract {
                &self.0.c
            }
            fn operations(&self) -> crate::RootOperations {
                crate::RootOperations {
                    jacobian: false,
                    jacobian_product: false,
                }
            }
            fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
                out[0] = x[0] - 1.;
                Ok(())
            }
            fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
                self.0.matrix.symbolic()
            }
            fn jacobian(&mut self, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
                panic!("Picard must consume its declared splitting, never original Jacobian values")
            }
            fn jacobian_product(
                &mut self,
                _: &[f64],
                _: &[f64],
                _: &mut [f64],
            ) -> Result<(), ProblemError> {
                panic!(
                    "Picard must consume its declared splitting, never an original Jacobian action"
                )
            }
        }
        let mut source = Polynomial::new();
        source.c.derivatives = pse_kernels::DerivativeOrder::Value;
        let linear = faer::sparse::SparseColMat::try_new_from_triplets(
            1,
            1,
            &[faer::sparse::Triplet::new(0, 0, 1.)],
        )
        .unwrap();
        let settings = Settings {
            method: Method {
                strategy: Strategy::Picard,
                ..Default::default()
            },
            variable_scales: vec![7.],
            residual_scales: vec![11.],
            step_tolerance: 1e-10,
        };
        let report = map(MapInput {
            problem: super::super::Problem::Picard {
                oracle: Box::new(ResidualOnly(source)),
                linear,
                initial: &[9.],
                settings: &settings,
                restarts: None,
            },
            controls: &Controls::default(),
            accuracy: &ResolvedAccuracy::verification(),
            execution: execution(),
            tolerances: &Tolerances {
                variables: vec![1e-8],
                rows: vec![1e-8],
                integrality: 1e-8,
            },
            compatibility: stamp(Backend::Kinsol),
        })
        .unwrap();
        assert_eq!(
            report.termination.category,
            crate::solve::Termination::Success,
            "{report:?}"
        );
        assert!((report.candidate.unwrap().primal[0] - 1.).abs() < 1e-8);
        assert!(report.warm_start.is_none());
    }
}
