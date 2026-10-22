// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Propagation of a parameter covariance to outputs, `Σ_y = J·Σ_θ·Jᵀ` (Plan 22 S4; ADR-0118
//! items 1, 10 and 11; PS-12): the counterpart of IDAES `sens.py`.
//!
//! `J` is either a modeling step's parametric sensitivities (S1), whose parameters are
//! matched to the covariance's by identity, or a fit's own response derivatives. The
//! result is a first-order statement that holds while both inputs do, so its validity is
//! the conjunction of theirs: when either is withheld, so is the propagated covariance,
//! with `upstream_withheld` and the upstream reason in its detail.
//!
//! A caller propagates a fit's covariance through a later solve by naming it in the solve
//! settings (`SensitivityRequest.propagation`, built from the fit's
//! `runtime.parameter_covariances` row or [`RunResult::parameter_covariance`]); a fit
//! propagates to its own predictions on request (`FitUncertainty.predictions`). Both
//! publish `runtime.propagated_covariances` with a `local_validity` row.
use super::{RunResult, WorkflowError, contract};
pub use crate::math::settings::{CovarianceOrigin, FitCovarianceRecord, ParameterCovariance};
use pse_ids::SemanticId;
use pse_model::scalars::FiniteBound;
use pse_relations::generated::{
    enums::{DerivedQuantity, WithheldReason},
    identities::RunId,
};
mod psd;
mod sources;
pub use psd::CovariancePsdAssessment;

/// Output derivatives with respect to parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Jacobian {
    /// The outputs, by identity: the rows.
    pub outputs: Vec<SemanticId>,
    /// The parameters, by identity: the columns.
    pub parameters: Vec<SemanticId>,
    /// `J` row-major, in output unit per parameter unit.
    pub values: Vec<f64>,
}
/// An input of the propagation that was withheld, and why (PS-12).
#[derive(Clone, Debug, PartialEq)]
pub struct Upstream {
    /// The withheld quantity.
    pub quantity: DerivedQuantity,
    /// Its registry reason.
    pub reason: WithheldReason,
    /// Its typed cause.
    pub detail: String,
}
impl std::fmt::Display for Upstream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the {} it is computed from was withheld ({}): {}",
            self.quantity.as_str(),
            self.reason.as_str(),
            self.detail
        )
    }
}
/// A certified propagated covariance.
#[derive(Clone, Debug, PartialEq)]
pub struct Propagated {
    /// The fit run whose covariance was propagated.
    pub covariance_run: Option<RunId>,
    /// Statistical origin remains explicit after successful mathematical propagation.
    pub origin: CovarianceOrigin,
    /// Actual fit source interpretation; absent for external statistical assumptions.
    pub source: Option<pse_ids::ContentHash>,
    /// Actual fit physical interpretation; absent for external statistical assumptions.
    pub physical: Option<pse_ids::ContentHash>,
    /// Parameter scales used by the dimensionless mathematical check.
    pub scales: Vec<f64>,
    /// The parameters propagated, in the covariance's order.
    pub parameters: Vec<SemanticId>,
    /// The outputs: the order of every matrix.
    pub outputs: Vec<SemanticId>,
    /// `Σ_y` row-major, in row-output unit × column-output unit.
    pub values: Vec<f64>,
    /// Independent mathematical check; it does not establish statistical origin.
    pub psd: CovariancePsdAssessment,
}
impl Propagated {
    pub(super) fn into_row(
        self,
        run_id: RunId,
        step: i64,
        output_units: Vec<SemanticId>,
    ) -> pse_relations::generated::runtime::propagated_covariances::Row {
        use pse_relations::generated::enums::CovarianceInputOrigin;
        let (origin, label, assumptions) = match self.origin {
            CovarianceOrigin::QualifiedFit { .. } => (
                CovarianceInputOrigin::QualifiedFit,
                "qualified fit".to_owned(),
                Vec::new(),
            ),
            CovarianceOrigin::External { label, assumptions } => {
                (CovarianceInputOrigin::External, label, assumptions)
            }
        };
        pse_relations::generated::runtime::propagated_covariances::Row {
            run_id,
            step,
            covariance_run_id: self.covariance_run,
            covariance_origin: origin,
            covariance_label: label,
            covariance_assumptions: assumptions,
            covariance_source_identity: self.source,
            covariance_physical_identity: self.physical,
            parameter_scales: self.scales,
            psd_norm_inf: self.psd.normalized_norm_inf,
            psd_tolerance: self.psd.tolerance,
            psd_minimum_eigenvalue: self.psd.minimum_eigenvalue,
            parameters: self.parameters,
            outputs: self.outputs,
            output_units,
            values: self.values,
        }
    }
}

/// `Σ_y = J·Σ_θ·Jᵀ` over the covariance's parameters, each matched by identity to a column
/// of `J`; a parameter of `J` the covariance does not name is held fixed. Withheld, with
/// the first withheld input, when an input is withheld.
///
/// # Errors
/// A covariance that is not square over distinct parameters, a `J` whose dimensions do not
/// match its identities, or a covariance parameter `J` does not differentiate.
pub fn propagate(
    covariance: Result<&ParameterCovariance, Upstream>,
    jacobian: Result<&Jacobian, Upstream>,
    pool: &std::sync::Arc<dyn pse_columnar::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<
    (
        Result<Propagated, Upstream>,
        Option<pse_columnar::MemoryReservation>,
    ),
    WorkflowError,
> {
    cancel
        .checkpoint()
        .map_err(pse_relations::RelationError::from)
        .map_err(super::relation)?;
    let working = if let (Ok(c), Ok(j)) = (&covariance, &jacobian) {
        let (m, p) = (j.outputs.len(), c.parameters.len());
        let mp = m
            .checked_mul(p)
            .ok_or_else(|| contract("propagation product extent"))?;
        let mm = m
            .checked_mul(m)
            .ok_or_else(|| contract("propagation covariance extent"))?;
        let origin_bytes = origin_extent(&c.origin)?;
        Some(super::result_export::working(
            pool,
            "result:propagation-working",
            &[
                (mp, 8),
                (mm, 8),
                (j.parameters.len(), 128),
                (m, 144),
                (p, 144),
                (origin_bytes, 1),
            ],
        )?)
    } else {
        None
    };
    Ok((
        propagate_checked(covariance, jacobian, pool, cancel)?,
        working,
    ))
}
fn origin_extent(origin: &CovarianceOrigin) -> Result<usize, WorkflowError> {
    let overflow = || contract("covariance origin extent");
    match origin {
        CovarianceOrigin::QualifiedFit {
            record: Some(record),
            ..
        } => record
            .run
            .capacity()
            .checked_add(record.attempt.capacity())
            .ok_or_else(overflow),
        CovarianceOrigin::QualifiedFit { record: None, .. } => Ok(0),
        CovarianceOrigin::External { label, assumptions } => {
            let headers = assumptions
                .capacity()
                .checked_mul(size_of::<String>())
                .ok_or_else(overflow)?;
            let bytes = label.capacity().checked_add(headers).ok_or_else(overflow)?;
            assumptions.iter().try_fold(bytes, |bytes, value| {
                bytes.checked_add(value.capacity()).ok_or_else(overflow)
            })
        }
    }
}
fn propagate_checked(
    covariance: Result<&ParameterCovariance, Upstream>,
    jacobian: Result<&Jacobian, Upstream>,
    pool: &std::sync::Arc<dyn pse_columnar::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<Result<Propagated, Upstream>, WorkflowError> {
    let observation = pse_diagnostics::observation::current();
    let _phase = observation.as_ref().map(|context| {
        context.count("uncertainty.propagation.requests", "count", 1);
        context.phase("uncertainty.propagation")
    });
    let result = (|| {
        let checkpoint = || {
            cancel
                .checkpoint()
                .map_err(pse_relations::RelationError::from)
                .map_err(super::relation)
        };
        checkpoint()?;
        let (covariance, jacobian) = match (covariance, jacobian) {
            (Ok(c), Ok(j)) => (c, j),
            (Err(upstream), _) | (_, Err(upstream)) => return Ok(Err(upstream)),
        };
        covariance
            .admit()
            .map_err(|e| contract(format!("propagated covariance: {e}")))?;
        let verified = covariance
            .verified_fit()
            .map_err(|error| contract(error.to_string()))?;
        let p = covariance.parameters.len();
        let _input_copy = super::result_export::working(
            pool,
            "uncertainty:psd-input",
            &[(covariance.values.len(), 8), (p, 8)],
        )?;
        let input_values: Vec<_> = covariance
            .values
            .iter()
            .map(|value| value.into_inner())
            .collect();
        let scales: Vec<_> = covariance
            .scales
            .iter()
            .map(|value| value.into_inner())
            .collect();
        let psd = psd::admit(p, &input_values, &scales, pool, cancel)?;
        let (m, n) = (jacobian.outputs.len(), jacobian.parameters.len());
        if let Some(context) = &observation {
            context.integer("uncertainty.propagation.outputs", "count", m as i64);
            context.integer("uncertainty.propagation.parameters", "count", n as i64);
        }
        if jacobian.values.len()
            != m.checked_mul(n)
                .ok_or_else(|| contract("Jacobian extent"))?
        {
            return Err(contract(
                "a Jacobian's values match its outputs and parameters",
            ));
        }
        let parameter_access =
            pse_math::index::CheckedInventory::new(&jacobian.parameters, |id| *id)
                .map_err(super::math)?;
        pse_math::index::CheckedInventory::new(&jacobian.outputs, |id| *id).map_err(super::math)?;
        let columns = covariance
            .parameters
            .iter()
            .map(|id| {
                parameter_access
                    .position(id)
                    .ok_or_else(|| contract(format!("the Jacobian does not differentiate {id}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let sigma = |k: usize, l: usize| covariance.values[k * p + l].into_inner();
        let j = |i: usize, k: usize| jacobian.values[i * n + columns[k]];
        // J·Σ_θ, then (J·Σ_θ)·Jᵀ.
        let product: Vec<f64> = (0..m
            .checked_mul(p)
            .ok_or_else(|| contract("propagation product extent"))?)
            .map(|index| {
                let (i, l) = (index / p, index % p);
                let mut sum = -0.0;
                for k in 0..p {
                    if k % 1024 == 0 {
                        checkpoint()?;
                    }
                    sum += j(i, k) * sigma(k, l);
                }
                Ok::<_, WorkflowError>(sum)
            })
            .collect::<Result<_, _>>()?;
        let values: Vec<f64> = (0..m
            .checked_mul(m)
            .ok_or_else(|| contract("propagation covariance extent"))?)
            .map(|index| {
                let (i, r) = (index / m, index % m);
                let mut sum = -0.0;
                for l in 0..p {
                    if l % 1024 == 0 {
                        checkpoint()?;
                    }
                    sum += product[i * p + l] * j(r, l);
                }
                Ok::<_, WorkflowError>(sum)
            })
            .collect::<Result<_, _>>()?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(contract("the propagated covariance overflowed"));
        }
        if (0..m).any(|row| values[row * m + row] < 0.0) {
            return Ok(Err(Upstream {
                quantity: DerivedQuantity::PropagatedCovariance,
                reason: WithheldReason::BacksolveFailed,
                detail: "a propagated variance is negative; the covariance is numerically unresolved and was not clamped".into(),
            }));
        }
        Ok(Ok(Propagated {
            covariance_run: match &covariance.origin {
                CovarianceOrigin::QualifiedFit { run_id, .. } => Some(*run_id),
                CovarianceOrigin::External { .. } => None,
            },
            origin: covariance.origin.clone(),
            source: verified.map(|proof| proof.source),
            physical: verified.map(|proof| proof.physical),
            scales,
            parameters: covariance.parameters.clone(),
            outputs: jacobian.outputs.clone(),
            values,
            psd,
        }))
    })();
    if let Some(context) = observation {
        context.boolean("uncertainty.propagation.returned", result.is_ok());
        match &result {
            Ok(Ok(_)) => {
                context.boolean("uncertainty.propagation.available", true);
                context.text("uncertainty.propagation.withheld", "none");
                context.text("uncertainty.propagation.upstream", "none");
                context.text("uncertainty.propagation.failure", "none");
            }
            Ok(Err(upstream)) => {
                context.boolean("uncertainty.propagation.available", false);
                context.text("uncertainty.propagation.withheld", upstream.reason.as_str());
                context.text(
                    "uncertainty.propagation.upstream",
                    upstream.quantity.as_str(),
                );
                context.text("uncertainty.propagation.failure", "none");
            }
            Err(error) => {
                context.boolean("uncertainty.propagation.available", false);
                context.text("uncertainty.propagation.withheld", "not_applicable");
                context.text("uncertainty.propagation.upstream", "not_applicable");
                context.text("uncertainty.propagation.failure", &error.to_string());
            }
        }
    }
    result
}

impl RunResult {
    /// Restore statistical authority after a document round trip using this actual
    /// retained fit. Run identity alone does not establish content or validity.
    ///
    /// # Errors
    /// The source is not a qualified fit covariance, or any submitted covariance
    /// value, parameter, unit, scale, origin or interpretation differs.
    pub fn reconcile_parameter_covariance(
        &self,
        input: &mut ParameterCovariance,
    ) -> Result<(), WorkflowError> {
        input.admit().map_err(|error| contract(error.to_string()))?;
        let actual = self
            .parameter_covariance()
            .ok_or_else(|| contract("the retained result has no fit covariance"))?
            .map_err(|error| contract(error.to_string()))?;
        actual
            .verified_fit()
            .map_err(|error| contract(error.to_string()))?;
        if input
            .content()
            .map_err(|error| contract(error.to_string()))?
            != actual
                .content()
                .map_err(|error| contract(error.to_string()))?
        {
            return Err(contract(
                "submitted covariance differs from the actual retained qualified fit",
            ));
        }
        input.verified = actual.verified;
        Ok(())
    }

    /// This fit run's parameter covariance as a propagation input naming the run, or why it
    /// is withheld; `None` for a run that is not a fit with free parameters.
    pub fn parameter_covariance(&self) -> Option<Result<ParameterCovariance, Upstream>> {
        let super::RunOutcome::Fit {
            request,
            result: Ok(report),
        } = &self.outcome
        else {
            return None;
        };
        report.covariance.as_ref().map(|covariance| {
            let record = match &self.durability {
                super::RunDurability::Durable(record) => record
                    .run
                    .as_ref()
                    .zip(record.attempt_key.as_ref())
                    .map(|(run, attempt)| FitCovarianceRecord {
                        run: run.key.clone(),
                        attempt: attempt.clone(),
                    }),
                super::RunDurability::Ephemeral => None,
            };
            let point = report
                .solve
                .as_ref()
                .and_then(|solve| solve.evidence.local.as_ref())
                .and_then(|point| point.as_ref().ok());
            let validity = super::local_analysis::record(
                covariance
                    .values
                    .as_ref()
                    .map(|_| &())
                    .map_err(|withheld| (withheld.reason(), withheld.to_string())),
                point,
                false,
            );
            covariance.propagation_input(
                CovarianceOrigin::QualifiedFit {
                    run_id: self.run_id,
                    record,
                },
                request.problem.source_identity,
                request.source.physical.identity(),
                validity,
                &self.runtime.shared.pool(),
            )
        })
    }
}
impl super::Covariance {
    /// The covariance as a propagation input naming `run`, or why it is withheld.
    pub(crate) fn propagation_input(
        &self,
        origin: CovarianceOrigin,
        source: pse_ids::ContentHash,
        physical: pse_ids::ContentHash,
        validity: pse_model::generated::structures::LocalValidity,
        pool: &std::sync::Arc<dyn pse_columnar::MemoryPool>,
    ) -> Result<ParameterCovariance, Upstream> {
        if let Err(withheld) = &self.values {
            return Err(Upstream {
                quantity: DerivedQuantity::ParameterCovariance,
                reason: withheld.reason(),
                detail: withheld.to_string(),
            });
        }
        let failed = |detail: String| Upstream {
            quantity: DerivedQuantity::ParameterCovariance,
            reason: WithheldReason::BacksolveFailed,
            detail,
        };
        let reservation = super::result_export::working(
            pool,
            "uncertainty:qualified-covariance",
            &[
                (self.values.as_ref().map_or(0, Vec::len), 8),
                (self.parameters.len(), 80),
                (1, 1024),
            ],
        )
        .map_err(|error| failed(error.to_string()))?;
        let owner = pse_columnar::AllocationLease::new(reservation);
        match &self.values {
            Ok(values) => {
                let mut covariance = ParameterCovariance {
                    version: pse_model::document::Version,
                    origin,
                    parameters: self.parameters.clone(),
                    parameter_quantities: self.parameter_quantities.clone(),
                    parameter_units: self.parameter_units.clone(),
                    verified: None,
                    values: values
                        .iter()
                        .map(|v| FiniteBound::try_new(*v))
                        .collect::<Result<_, _>>()
                        .map_err(|e| Upstream {
                            quantity: DerivedQuantity::ParameterCovariance,
                            reason: WithheldReason::BacksolveFailed,
                            detail: e.to_string(),
                        })?,
                    scales: self
                        .scales
                        .iter()
                        .map(|scale| FiniteBound::try_new(*scale))
                        .collect::<Result<_, _>>()
                        .map_err(|error| Upstream {
                            quantity: DerivedQuantity::ParameterCovariance,
                            reason: WithheldReason::BacksolveFailed,
                            detail: error.to_string(),
                        })?,
                };
                let content = covariance
                    .content()
                    .map_err(|error| failed(error.to_string()))?;
                covariance.verified = Some(std::sync::Arc::new(
                    crate::math::settings::VerifiedFitCovariance {
                        content,
                        source,
                        physical,
                        validity,
                        _owner: owner,
                    },
                ));
                Ok(covariance)
            }
            Err(withheld) => Err(Upstream {
                quantity: DerivedQuantity::ParameterCovariance,
                reason: withheld.reason(),
                detail: withheld.to_string(),
            }),
        }
    }
}

#[cfg(test)]
fn propagate_for_test(
    covariance: Result<&ParameterCovariance, Upstream>,
    jacobian: Result<&Jacobian, Upstream>,
) -> Result<Result<Propagated, Upstream>, WorkflowError> {
    let pool: std::sync::Arc<dyn pse_columnar::MemoryPool> = std::sync::Arc::new(
        datafusion::execution::memory_pool::GreedyMemoryPool::new(64 << 20),
    );
    propagate(
        covariance,
        jacobian,
        &pool,
        &pse_columnar::CancellationToken::new(),
    )
    .map(|(result, _working)| result)
}

#[cfg(test)]
#[cfg(all(feature = "solver-ipopt", feature = "solver-highs"))]
#[path = "uncertainty_tests.rs"]
mod tests;

#[cfg(test)]
mod projection_tests {
    use super::*;
    #[test]
    fn fit_covariance_content_and_decode_cannot_inherit_an_unverified_run_claim() {
        let id = |value| SemanticId::from_bytes([value; 16]);
        let retained = super::super::Covariance {
            approximation: pse_relations::generated::enums::CovarianceApproximation::GaussNewton,
            parameters: vec![id(1), id(2)],
            scales: vec![2.0, 3.0],
            parameter_quantities: vec![id(11), id(12)],
            parameter_units: vec![id(21), id(22)],
            values: Ok(vec![4.0, 0.0, 0.0, 9.0]),
        };
        let pool: std::sync::Arc<dyn pse_columnar::MemoryPool> = std::sync::Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(1 << 20),
        );
        let input = retained
            .propagation_input(
                CovarianceOrigin::QualifiedFit {
                    run_id: id(9).into(),
                    record: None,
                },
                pse_ids::ContentHash::from_bytes([1; 32]),
                pse_ids::ContentHash::from_bytes([2; 32]),
                super::super::local_analysis::record(Ok(&()), None, false),
                &pool,
            )
            .unwrap();
        assert!(input.verified_fit().unwrap().is_some());
        for field in 0..5 {
            let mut altered = input.clone();
            match field {
                0 => altered.values[0] = FiniteBound::try_new(8.0).unwrap(),
                1 => altered.parameters.swap(0, 1),
                2 => altered.parameter_units[0] = id(23),
                3 => altered.parameter_quantities[0] = id(13),
                _ => altered.scales[0] = FiniteBound::try_new(4.0).unwrap(),
            }
            assert!(altered.verified_fit().is_err());
        }
        let decoded: ParameterCovariance =
            serde_json::from_slice(&serde_json::to_vec(&input).unwrap()).unwrap();
        assert_eq!(
            decoded, input,
            "the payload round-trips without inventing process-local evidence"
        );
        assert!(decoded.verified_fit().is_err());
        drop(input);
        drop(decoded);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn near_tolerance_negative_variance_is_withheld_and_external_origin_stays_external() {
        let id = |value| SemanticId::from_bytes([value; 16]);
        let mut covariance = ParameterCovariance::external(
            vec![id(1), id(2)],
            vec![id(11), id(12)],
            vec![id(21), id(22)],
            [1.0, 0.0, 0.0, -64.0 * f64::EPSILON]
                .into_iter()
                .map(|value| FiniteBound::try_new(value).unwrap())
                .collect(),
            vec![FiniteBound::try_new(1.0).unwrap(); 2],
            "external control".into(),
            vec!["caller-declared covariance".into()],
        );
        let jacobian = Jacobian {
            outputs: vec![id(3)],
            parameters: vec![id(1), id(2)],
            values: vec![0.0, 1.0],
        };
        assert!(
            propagate_for_test(Ok(&covariance), Ok(&jacobian))
                .unwrap()
                .is_err()
        );
        covariance.values[3] = FiniteBound::try_new(0.0).unwrap();
        let product = propagate_for_test(Ok(&covariance), Ok(&jacobian))
            .unwrap()
            .unwrap();
        assert_eq!(product.covariance_run, None);
        assert!(matches!(product.origin, CovarianceOrigin::External { .. }));
    }
    #[test]
    fn selected_propagation_reserves_before_work_and_observes_cancellation() {
        use datafusion::execution::memory_pool::GreedyMemoryPool;
        use std::sync::Arc;
        let id = |value| SemanticId::from_bytes([value; 16]);
        let covariance = ParameterCovariance {
            version: pse_model::document::Version,
            origin: CovarianceOrigin::External {
                label: "projection control".into(),
                assumptions: vec!["declared independent errors".into()],
            },
            parameters: vec![id(1), id(2)],
            parameter_quantities: vec![id(11), id(12)],
            parameter_units: vec![id(21), id(22)],
            verified: None,
            scales: vec![FiniteBound::try_new(1.0).unwrap(); 2],
            values: [4.0, 0.0, 0.0, 9.0]
                .into_iter()
                .map(|v| FiniteBound::try_new(v).unwrap())
                .collect(),
        };
        let jacobian = Jacobian {
            outputs: vec![id(3), id(4)],
            parameters: vec![id(1), id(2)],
            values: vec![2.0, 3.0, 1.0, 0.0],
        };
        let tiny: Arc<dyn pse_columnar::MemoryPool> = Arc::new(GreedyMemoryPool::new(64));
        let cancel = pse_columnar::CancellationToken::new();
        assert!(propagate(Ok(&covariance), Ok(&jacobian), &tiny, &cancel).is_err());
        assert_eq!(tiny.reserved(), 0);
        let pool: Arc<dyn pse_columnar::MemoryPool> = Arc::new(GreedyMemoryPool::new(8 << 20));
        let expected = propagate_for_test(Ok(&covariance), Ok(&jacobian))
            .unwrap()
            .unwrap();
        let (result, working) = propagate(Ok(&covariance), Ok(&jacobian), &pool, &cancel).unwrap();
        assert_eq!(result.unwrap(), expected);
        assert!(
            pool.reserved() > 0,
            "derived buffers remain charged through caller publication"
        );
        drop(working);
        assert_eq!(pool.reserved(), 0);
        cancel.cancel();
        assert!(propagate(Ok(&covariance), Ok(&jacobian), &pool, &cancel).is_err());
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn propagation_preserves_scalar_sum_bits_and_upstream_partiality() {
        let id = |value| SemanticId::from_bytes([value; 16]);
        let covariance = ParameterCovariance {
            version: pse_model::document::Version,
            origin: CovarianceOrigin::External {
                label: "zero-bit control".into(),
                assumptions: vec!["declared independent errors".into()],
            },
            parameters: vec![id(1)],
            parameter_quantities: vec![id(11)],
            parameter_units: vec![id(21)],
            verified: None,
            scales: vec![FiniteBound::try_new(1.0).unwrap()],
            values: vec![FiniteBound::try_new(1.0).unwrap()],
        };
        let jacobian = Jacobian {
            outputs: vec![id(3)],
            parameters: vec![id(1)],
            values: vec![-0.0],
        };
        let product: f64 = (0..1)
            .map(|k| jacobian.values[k] * covariance.values[k].into_inner())
            .sum();
        let expected: f64 = (0..1).map(|k| product * jacobian.values[k]).sum();
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("propagation quantity");
        let _entered = operation.context().enter();
        assert_eq!(
            propagate_for_test(Ok(&covariance), Ok(&jacobian))
                .unwrap()
                .unwrap()
                .values[0]
                .to_bits(),
            expected.to_bits()
        );
        let upstream = Upstream {
            quantity: DerivedQuantity::ParametricSensitivity,
            reason: WithheldReason::ResponsesUnavailable,
            detail: "response refused".into(),
        };
        assert!(
            propagate_for_test(Ok(&covariance), Err(upstream))
                .unwrap()
                .is_err()
        );
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let metric = |name| {
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .unwrap()
        };
        assert_eq!(
            metric("uncertainty.propagation.requests").cumulative,
            pse_diagnostics::observation::Value::Count(2)
        );
        assert_eq!(
            metric("uncertainty.propagation.available").latest,
            pse_diagnostics::observation::Value::Boolean(false)
        );
        assert_eq!(
            metric("uncertainty.propagation.withheld").latest,
            pse_diagnostics::observation::Value::Text(
                WithheldReason::ResponsesUnavailable.as_str().into()
            )
        );
        propagate_for_test(Ok(&covariance), Ok(&jacobian))
            .unwrap()
            .unwrap();
        let restored = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        assert_eq!(
            restored
                .metrics
                .iter()
                .find(|metric| metric.name == "uncertainty.propagation.upstream")
                .unwrap()
                .latest,
            pse_diagnostics::observation::Value::Text("none".into())
        );
        let invalid = Jacobian {
            values: vec![],
            ..jacobian
        };
        assert!(propagate_for_test(Ok(&covariance), Ok(&invalid)).is_err());
        let failed = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        for name in [
            "uncertainty.propagation.upstream",
            "uncertainty.propagation.withheld",
        ] {
            assert_eq!(
                failed
                    .metrics
                    .iter()
                    .find(|metric| metric.name == name)
                    .unwrap()
                    .latest,
                pse_diagnostics::observation::Value::Text("not_applicable".into())
            );
        }
    }

    #[test]
    fn covariance_projection_reorders_parameters_and_refuses_ambiguous_axes() {
        let id = |value| SemanticId::from_bytes([value; 16]);
        let covariance = ParameterCovariance {
            version: pse_model::document::Version,
            origin: CovarianceOrigin::External {
                label: "alignment control".into(),
                assumptions: vec!["declared independent errors".into()],
            },
            parameters: vec![id(2), id(1)],
            parameter_quantities: vec![id(12), id(11)],
            parameter_units: vec![id(22), id(21)],
            verified: None,
            scales: vec![FiniteBound::try_new(1.0).unwrap(); 2],
            values: [4.0, 0.0, 0.0, 9.0]
                .into_iter()
                .map(|v| FiniteBound::try_new(v).unwrap())
                .collect(),
        };
        let mut jacobian = Jacobian {
            outputs: vec![id(3)],
            parameters: vec![id(1), id(2)],
            values: vec![2.0, 3.0],
        };
        assert_eq!(
            propagate_for_test(Ok(&covariance), Ok(&jacobian))
                .unwrap()
                .unwrap()
                .values,
            [72.0]
        );
        jacobian.parameters = vec![id(1), id(1)];
        assert!(propagate_for_test(Ok(&covariance), Ok(&jacobian)).is_err());
        jacobian.parameters = vec![id(1), id(4)];
        assert!(propagate_for_test(Ok(&covariance), Ok(&jacobian)).is_err());
        jacobian.parameters = vec![id(1), id(2)];
        jacobian.outputs = vec![id(3), id(3)];
        jacobian.values = vec![2.0, 3.0, 2.0, 3.0];
        assert!(propagate_for_test(Ok(&covariance), Ok(&jacobian)).is_err());
    }
}
