// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit analysis requests use the shared compiler and native completion owners.
use super::{PhysicalContext, Runtime, WorkflowError, contract};
use crate::math::{
    MathRuntimeError,
    solves::{PreparedSolve, SolveHandle, SolverProfile},
};
use pse_backend_native as native;
#[cfg(any(feature = "solver-kinsol", test))]
use pse_backend_native::solve::*;
#[cfg(feature = "solver-kinsol")]
use pse_ids::FramedHasher;
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use std::sync::Arc;

/// Explicit physical coordinate of a native analysis request.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisPort {
    /// Stable source identity, never a solver column index.
    pub symbol_id: SemanticId,
    /// Admitted physical quantity contract.
    pub quantity_id: SemanticId,
    /// Unit of the supplied coordinate.
    pub unit_id: SemanticId,
}

/// Explicit continuous cone analysis. The request declares geometry; it is never inferred from NLP rows.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConicRequest {
    /// Complete source coordinates; all bounds are explicit cone rows.
    pub variables: Vec<AnalysisPort>,
    /// Cone row coordinates, in native block order.
    pub rows: Vec<AnalysisPort>,
    /// Physical objective coordinate, with NIL identity.
    pub objective_port: AnalysisPort,
    /// Upper triangular Q in 1/2 x'Qx + c'x + constant.
    pub quadratic: native::conic::SparseMatrix,
    /// Objective coefficients.
    pub objective: Vec<f64>,
    /// A in Ax+s=b.
    pub constraints: native::conic::SparseMatrix,
    /// Exact authored b.
    pub rhs: Vec<f64>,
    /// Cone blocks in the pse-owned vocabulary, mapped to the library only by its adapter.
    pub cones: Vec<native::conic::Cone>,
    /// Original objective constant.
    pub objective_constant: f64,
}
impl ConicRequest {
    /// Actual copied source arrays, native validation and exact Gram containers.
    /// The existing four-MiB opaque certificate allowance remains separate from
    /// these known populations and is not a measured foreign-heap bound.
    fn construction_allocation_bound(&self) -> Result<usize, MathRuntimeError> {
        let extent = || -> Option<usize> {
            let n = self.variables.len();
            let ports = n.checked_add(self.rows.len())?.checked_add(1)?;
            let sparse =
                [&self.quadratic, &self.constraints]
                    .iter()
                    .try_fold(0usize, |bytes, matrix| {
                        bytes
                            .checked_add(
                                matrix
                                    .column_starts
                                    .len()
                                    .checked_add(matrix.row_indices.len())?
                                    .checked_mul(size_of::<usize>())?,
                            )?
                            .checked_add(matrix.values.len().checked_mul(size_of::<f64>())?)
                    })?;
            let alpha = self.cones.iter().try_fold(0usize, |count, cone| {
                count.checked_add(match cone {
                    native::conic::Cone::GeneralizedPower { alpha, .. } => alpha.len(),
                    _ => 0,
                })
            })?;
            let source = sparse
                .checked_add(
                    self.objective
                        .len()
                        .checked_add(self.rhs.len())?
                        .checked_add(alpha)?
                        .checked_mul(size_of::<f64>())?,
                )?
                .checked_add(
                    self.cones
                        .len()
                        .checked_mul(size_of::<native::conic::Cone>())?,
                )?;
            let symmetric = self
                .quadratic
                .values
                .len()
                .checked_mul(16 * (size_of::<usize>() + size_of::<f64>()))?
                .checked_add(n.checked_add(1)?.checked_mul(4 * size_of::<usize>())?)?;
            source
                .checked_mul(2)?
                .checked_add(ports.checked_mul(1024)?)?
                .checked_add(symmetric)?
                .checked_add(native::GramCertificate::construction_allocation_bound(n).ok()?)?
                .checked_add(4 << 20)
        };
        extent().ok_or(MathRuntimeError::Limit("explicit cone construction extent"))
    }
}

/// Prepared explicit cone analysis, retaining the declared request and resolved policy.
#[derive(Clone, Debug)]
pub struct PreparedConic {
    runtime: Runtime,
    request: Arc<ConicRequest>,
    solve: PreparedSolve,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PreparedConic {
    /// Exact explicit representation supplied by the caller.
    pub fn request(&self) -> &ConicRequest {
        &self.request
    }
    /// Inspect the same resolved route and numerical contract used for execution.
    pub fn solve(&self) -> &PreparedSolve {
        &self.solve
    }
    /// Submit to the existing finite solve lifecycle, including native destruction and join.
    pub fn start(&self) -> Result<SolveHandle, WorkflowError> {
        Ok(self.runtime.native().solve(self.solve.clone())?)
    }
}
impl Runtime {
    /// Prepare a concrete native cone request against the admitted quantity registry.
    pub async fn prepare_conic(
        &self,
        request: ConicRequest,
        physical: &PhysicalContext,
        profile: SolverProfile,
    ) -> Result<PreparedConic, WorkflowError> {
        use pse_model::generated::enums::NumericalTarget;
        // Preserve the original encoded-size ceiling without first allocating a
        // full request encoding. Identity still uses the canonical serde framer.
        struct EncodingExtent {
            bytes: usize,
            limit: usize,
        }
        impl std::io::Write for EncodingExtent {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.bytes = self
                    .bytes
                    .checked_add(bytes.len())
                    .filter(|n| *n <= self.limit)
                    .ok_or_else(|| {
                        std::io::Error::other("cone request exceeds workspace allowance")
                    })?;
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut encoding = EncodingExtent {
            bytes: 0,
            limit: self.shared.budget().math.workspace_bytes / 4,
        };
        serde_json::to_writer(&mut encoding, &request).map_err(|e| contract(e.to_string()))?;
        let mut policy_encoding = EncodingExtent {
            bytes: 0,
            limit: usize::MAX,
        };
        serde_json::to_writer(&mut policy_encoding, &profile.numerics)
            .map_err(|e| contract(e.to_string()))?;
        // Resolution can retain policy-owned provenance in each selected target
        // alongside its validation copies. Count its actual encoded population,
        // using the existing conservative scientific decoding envelope.
        let policy_extent = request
            .variables
            .len()
            .checked_add(request.rows.len())
            .and_then(|n| n.checked_add(2))
            .and_then(|n| n.checked_mul(policy_encoding.bytes))
            .and_then(|n| n.checked_mul(16))
            .ok_or(MathRuntimeError::Limit(
                "cone numerical policy construction extent",
            ))?;
        let demand = request
            .construction_allocation_bound()?
            .checked_add(policy_extent)
            .ok_or(MathRuntimeError::Limit("explicit cone construction extent"))?;
        if demand > self.shared.budget().math.worker_bytes {
            return Err(MathRuntimeError::Limit("explicit cone construction capacity").into());
        }
        // Identity of the pse-owned request encoding, independent of any library's serde.
        let identity = pse_ids::document::of(pse_ids::Frame::ExplicitConicV5, &request)
            .map_err(|e| contract(e.to_string()))?;
        if request.objective_port.symbol_id != SemanticId::NIL {
            return Err(contract("cone objective identity must be NIL"));
        }
        let quantities = physical.quantities.clone();
        let preconditions = physical.preconditions.clone();
        let request = Arc::new(request);
        let source = request.clone();
        let allowance = self.shared.budget().math.workspace_bytes;
        let ((problem, proof, numerics, profile), owner) = self
            .native()
            .submit(1, demand, move |flag, _| {
                if flag.load(std::sync::atomic::Ordering::Relaxed) {
                    return Err(MathRuntimeError::Cancelled);
                }
                let target = |p: &AnalysisPort, kind| pse_math::numerics::TargetSpec {
                    id: p.symbol_id,
                    kind,
                    quantity: p.quantity_id.into(),
                    unit: p.unit_id.into(),
                    integer: false,
                    declared_tolerance: None,
                };
                let targets: Vec<_> = source
                    .variables
                    .iter()
                    .map(|p| target(p, NumericalTarget::Variable))
                    .chain(source.rows.iter().map(|p| target(p, NumericalTarget::Row)))
                    .chain([target(&source.objective_port, NumericalTarget::Objective)])
                    .collect();
                let numerics = Arc::new(pse_math::numerics::resolve(
                    &quantities,
                    &preconditions,
                    &targets,
                    &[],
                    &profile.numerics,
                )?);
                let n = source.variables.len();
                let problem = native::ConicProblem {
                    contract: native::OracleContract {
                        identity,
                        variables: source
                            .variables
                            .iter()
                            .map(|p| native::Variable {
                                id: p.symbol_id,
                                lower: f64::NEG_INFINITY,
                                upper: f64::INFINITY,
                            })
                            .collect(),
                        rows: source.rows.iter().map(|p| p.symbol_id).collect(),
                        derivatives: DerivativeOrder::Second,
                        smoothness: DerivativeOrder::Second,
                    },
                    quadratic: source.quadratic.clone(),
                    objective: source.objective.clone(),
                    constraints: source.constraints.clone(),
                    rhs: source.rhs.clone(),
                    cones: source.cones.clone(),
                    objective_constant: source.objective_constant,
                };
                problem.quadratic.validate()?;
                if problem.quadratic.rows != n || problem.quadratic.columns != n {
                    return Err(
                        native::ProblemError::Contract("cone quadratic extent".into()).into(),
                    );
                }
                // The quadratic's convexity is decided exactly (ADR-0121 Outcome 2): an
                // indefinite or undecided quadratic is refused, never solved.
                pse_math::initialize()?;
                let proof = match native::GramCertificate::certify(
                    &problem.full_quadratic()?,
                    1.0,
                    allowance / 16,
                    &flag,
                )? {
                    pse_math::convexity::Definiteness::Psd(proof) => proof,
                    pse_math::convexity::Definiteness::Indefinite => {
                        return Err(native::ProblemError::Contract(
                            "the cone objective quadratic is not positive semidefinite".into(),
                        )
                        .into());
                    }
                    pse_math::convexity::Definiteness::Inconclusive => {
                        return Err(MathRuntimeError::Limit("exact Gram certificate allowance"));
                    }
                };
                let sparse = [&problem.quadratic, &problem.constraints]
                    .iter()
                    .map(|a| {
                        (a.column_starts.capacity() + a.row_indices.capacity()) * size_of::<usize>()
                            + a.values.capacity() * size_of::<f64>()
                    })
                    .sum::<usize>();
                // Keep known conic buffers and a distinct opaque certificate allowance.
                let retained = sparse
                    .checked_add(
                        (problem.objective.capacity() + problem.rhs.capacity()) * size_of::<f64>(),
                    )
                    .and_then(|n| n.checked_add(4 << 20))
                    .ok_or(MathRuntimeError::Limit("conic retained extent"))?;
                Ok((
                    (Arc::new(problem), Arc::new(proof), numerics, profile),
                    retained.max(demand),
                ))
            })?
            .finish()
            .await?;
        let solve = self
            .native()
            .prepare_conic(problem, proof, profile, numerics)
            .await?;
        Ok(PreparedConic {
            runtime: self.clone(),
            request,
            solve,
            _owner: owner,
        })
    }
}

mod requests;
pub use requests::{CausalUnitRealization, CausalUnitRequest, RecycleRequest};
#[cfg(feature = "solver-kinsol")]
mod conditional;
#[cfg(feature = "solver-kinsol")]
pub use conditional::{PreparedInitializationStrategy, PreparedRecycle};

#[cfg(test)]
#[cfg(feature = "solver-kinsol")]
mod start_tests;
#[cfg(test)]
mod tests;

/// Immutable temporary-stage evidence from initialization, without changing specification bindings.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InitializationDocument {
    #[doc = "Original specification values."]
    pub original: std::collections::BTreeMap<SemanticId, f64>,
    #[doc = "Candidate values for solved unknowns."]
    pub solved_unknowns: std::collections::BTreeMap<SemanticId, f64>,
    #[doc = "Number of completed temporary stages."]
    pub completed_stages: usize,
    #[doc = "Whether the original specification bindings were restored."]
    pub original_bindings_restored: bool,
    #[doc = "Whether initialization was cancelled."]
    pub cancelled: bool,
    #[doc = "Retained temporary-stage evidence."]
    pub stages: Vec<InitializationStageDocument>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "Retained evidence from one temporary initialization stage."]
pub struct InitializationStageDocument {
    #[doc = "Temporary stage index."]
    pub stage: usize,
    #[doc = "Whether this temporary stage completed."]
    pub completed: bool,
    #[doc = "Temporary values applied only to this stage."]
    pub overlay: std::collections::BTreeMap<SemanticId, f64>,
    #[doc = "Values returned by this stage."]
    pub candidate: std::collections::BTreeMap<SemanticId, f64>,
}
#[cfg(feature = "solver-kinsol")]
impl From<&crate::math::initialization::InitializationReport> for InitializationDocument {
    fn from(report: &crate::math::initialization::InitializationReport) -> Self {
        Self {
            original: report.original.scalars.clone(),
            solved_unknowns: report.values.scalars.clone(),
            completed_stages: report.completed_stages,
            original_bindings_restored: report.original_bindings_restored,
            cancelled: report.cancelled,
            stages: report
                .stages
                .iter()
                .map(|stage| InitializationStageDocument {
                    stage: stage.stage,
                    completed: stage.completed,
                    overlay: stage.overlay.clone(),
                    candidate: stage.candidate.scalars.clone(),
                })
                .collect(),
        }
    }
}
