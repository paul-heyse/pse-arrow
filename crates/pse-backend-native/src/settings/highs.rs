// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The HiGHS settings document: the LP method, the MIP node budget, opt-in native
//! diagnostics and a partial MIP start.
use pse_ids::SemanticId;
use pse_model::scalars::FiniteBound;
use std::collections::BTreeMap;

/// Explicit native LP method, a registry vocabulary (ADR-0115 Outcome 3); automatic
/// remains a native class-specific decision.
pub use pse_model::generated::enums::HighsMethod as Method;

include!("highs/declarations.rs");
impl Settings {
    /// Admit declared controls without creating a native model or reading native options.
    pub(crate) fn admit_controls(
        &self,
        controls: &crate::solve::Controls,
    ) -> Result<(), crate::ProblemError> {
        use crate::{
            ProblemError,
            solve::{OptionValue, reject_reserved},
        };
        controls.validate()?;
        if self.nodes == Some(0) || self.nodes.is_some_and(|nodes| nodes > i32::MAX as u32) {
            return Err(ProblemError::Contract(
                "a HiGHS node budget must be positive and within the native range".into(),
            ));
        }
        reject_reserved(
            &controls.options,
            &[
                "threads",
                "parallel",
                "time_limit",
                "solver",
                "infinite_bound",
                "infinite_cost",
                "small_matrix_value",
                "large_matrix_value",
                "user_bound_scale",
                "user_cost_scale",
                "solve_relaxation",
                "mip_max_nodes",
                "simplex_iteration_limit",
                "ipm_iteration_limit",
                "pdlp_iteration_limit",
                "qp_iteration_limit",
                "qp_regularization_value",
                "primal_feasibility_tolerance",
                "dual_feasibility_tolerance",
                "mip_feasibility_tolerance",
                "mip_abs_gap",
                "mip_rel_gap",
                "simplex_scale_strategy",
                "log_file",
                "blend_multi_objectives",
            ],
        )?;
        if controls
            .options
            .values()
            .any(|v| matches!(v,OptionValue::Text(t)if t.len()>=512))
        {
            return Err(ProblemError::Unsupported(
                "HiGHS text option exceeds native readback capacity".into(),
            ));
        }
        Ok(())
    }
    /// Conditional class/method/scaling admission over the actual represented model.
    pub(crate) fn admit_model(
        &self,
        p: &crate::CoefficientProblem,
        accuracy: &crate::solve::ResolvedAccuracy,
    ) -> Result<(), crate::ProblemError> {
        use crate::ProblemError;
        use pse_model::generated::enums::ModelingVariableDomain;
        let method = self.method;
        let discrete = p
            .domains
            .iter()
            .any(|d| *d != ModelingVariableDomain::Continuous);
        let quadratic = p
            .hessian
            .as_ref()
            .is_some_and(|q| q.val().iter().any(|v| *v != 0.0));
        if !accuracy.native_scaling && (method != Method::Simplex || discrete || quadratic) {
            return Err(ProblemError::Unsupported("disabling all HiGHS algorithmic scaling is qualified only for explicit continuous simplex LP".into()));
        }
        if discrete && method != Method::Choose {
            return Err(ProblemError::Unsupported(
                "explicit LP method cannot relax a mixed-integer model".into(),
            ));
        }
        // HiGHS solves a continuous QP with its active-set QP solver whatever `solver`
        // says, and HiPO (the QP interior point) is not built: an explicit LP method on a
        // quadratic objective would be ignored or refused natively (F04).
        if quadratic && method != Method::Choose {
            return Err(ProblemError::Unsupported(
                "explicit LP method cannot solve a quadratic objective".into(),
            ));
        }
        if let Some(start) = &self.sparse_start
            && start.iter().any(|(id, value)| {
                !value.is_finite()
                    || !p
                        .contract
                        .variables
                        .iter()
                        .any(|variable| variable.id == *id)
            })
        {
            return Err(ProblemError::Contract(
                "HiGHS sparse start has an unknown coordinate or nonfinite value".into(),
            ));
        }
        Ok(())
    }
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            method: Method::Choose,
            nodes: None,
            diagnostics: Request::default(),
            sparse_start: None,
        }
    }
}

impl Request {
    /// Any diagnostic work was requested.
    pub fn any(&self) -> bool {
        self.rays
            || self.iis
            || self.ranging
            || self.relaxation.is_some()
            || self.fixed_lp
            || self.basis_inverse.is_some()
            || self.presolve
            || self.cut_pool
    }
}
