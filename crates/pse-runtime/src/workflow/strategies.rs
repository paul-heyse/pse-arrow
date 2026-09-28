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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
    /// Explicit sum-of-squares witness factors, one row per weight.
    pub gram_factors: Vec<Vec<f64>>,
    /// Nonnegative Gram weights, verified exactly against Q.
    pub gram_weights: Vec<f64>,
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
        let bytes = serde_json::to_vec(&request).map_err(|e| contract(e.to_string()))?;
        if bytes.len() > self.shared.budget().math.workspace_bytes / 4 {
            return Err(contract("cone request exceeds workspace allowance"));
        }
        // Identity of the pse-owned request encoding, independent of any library's serde.
        let identity = native::identity::of(pse_ids::Frame::ExplicitConicV3, &request)
            .map_err(MathRuntimeError::from)?;
        let target = |p: &AnalysisPort, kind| pse_math::numerics::TargetSpec {
            id: p.symbol_id,
            kind,
            quantity: p.quantity_id.into(),
            unit: p.unit_id.into(),
            integer: false,
            declared_tolerance: None,
        };
        if request.objective_port.symbol_id != SemanticId::NIL {
            return Err(contract("cone objective identity must be NIL"));
        }
        let targets: Vec<_> = request
            .variables
            .iter()
            .map(|p| target(p, NumericalTarget::Variable))
            .chain(request.rows.iter().map(|p| target(p, NumericalTarget::Row)))
            .chain([target(&request.objective_port, NumericalTarget::Objective)])
            .collect();
        let numerics = Arc::new(
            pse_math::numerics::resolve(&physical.quantities, &targets, &[], &profile.numerics)
                .map_err(super::math)?,
        );
        let request = Arc::new(request);
        let source = request.clone();
        let allowance = self.shared.budget().math.workspace_bytes;
        let ((problem, proof), owner) = self
            .native()
            .submit(1, allowance, move |flag, _| {
                if flag.load(std::sync::atomic::Ordering::Relaxed) {
                    return Err(MathRuntimeError::Cancelled);
                }
                let n = source.variables.len();
                if source.gram_weights.len() != source.gram_factors.len()
                    || source.gram_factors.iter().any(|r| r.len() != n)
                {
                    return Err(
                        native::ProblemError::Internal("Gram witness dimensions".into()).into(),
                    );
                }
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
                let work = n
                    .checked_mul(n)
                    .and_then(|v| v.checked_mul(source.gram_factors.len().max(1)))
                    .ok_or(MathRuntimeError::Limit("Gram work extent"))?;
                if work > allowance / 16 {
                    return Err(MathRuntimeError::Limit("Gram witness allowance"));
                }
                pse_math::initialize()?;
                let factors = faer::Mat::from_fn(source.gram_factors.len(), n, |r, c| {
                    source.gram_factors[r][c]
                });
                let proof = native::GramCertificate::new(
                    &problem.full_quadratic()?,
                    1.0,
                    &factors,
                    &source.gram_weights,
                    allowance / 16,
                )?;
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
                Ok(((Arc::new(problem), Arc::new(proof)), retained))
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

#[cfg(feature = "solver-kinsol")]
mod conditional;
#[cfg(feature = "solver-kinsol")]
pub use conditional::{
    CausalUnitRequest, PreparedInitializationStrategy, PreparedRecycle, RecycleRequest,
};

#[cfg(all(test, feature = "solver-kinsol"))]
mod start_tests;
#[cfg(test)]
mod tests;
