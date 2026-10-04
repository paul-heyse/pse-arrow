// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Optional path curvature is prepared by the original package's compiler owner.
use super::*;
use pse_kernels::{DerivativeOrder, ExecutionScope};

impl ModelingPackage {
    /// Prepare an actual Second parametric program for an explicitly requested event
    /// probe. Ordinary root response and limited-memory correction still require only
    /// First; absence of this optional product never promotes their support.
    ///
    /// # Errors
    /// The preparation belongs to another source, has no scalar parameter request,
    /// or compiler support, source validation or the original scope refuses it.
    pub async fn prepare_path_curvature(
        &self,
        original: &ModelingSolvePreparation,
        scope: ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<crate::math::solves::paths::PathCurvature, WorkflowError> {
        scope
            .check()
            .map_err(pse_backend_native::ProblemError::Provider)
            .map_err(crate::math::MathRuntimeError::from)?;
        if self.revision.identity() != original.source.revision.identity()
            || self.physical.key != original.source.physical.key
        {
            return Err(contract(
                "path curvature must belong to the original package revision and physical context",
            ));
        }
        let request = original
            .profile
            .sensitivity
            .as_ref()
            .filter(|request| request.parameters.len() == 1)
            .ok_or_else(|| {
                contract("path curvature requires the actual scalar parametric request")
            })?;
        let program = self
            .parametric_program(
                &original.model.model,
                &original.model,
                &request.parameters,
                DerivativeOrder::Second,
                original.compiler,
                driver,
            )
            .await?;
        scope
            .check()
            .map_err(pse_backend_native::ProblemError::Provider)
            .map_err(crate::math::MathRuntimeError::from)?;
        Ok(self
            .runtime
            .native()
            .prepare_path_curvature(original.solve.clone(), program, scope, driver)
            .await?)
    }
}
