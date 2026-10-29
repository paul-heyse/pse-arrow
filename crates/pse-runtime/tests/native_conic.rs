// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public native cone construction with independent objective and feasibility answers.
#![recursion_limit = "512"]
#![allow(clippy::unwrap_used, clippy::panic, reason = "downstream scientific assertions")]
#[path = "reference_support/mod.rs"]
mod support;
use futures_util::FutureExt;
use pse_backend_native::{conic::{Cone, SparseMatrix}, solve::SolveIntent};
use pse_runtime::{math::solves::{Outcome, SolverProfile}, workflow::{AnalysisPort, ConicRequest}};
use support::{NativeContext, Result};
use std::panic::AssertUnwindSafe;

#[tokio::test]
async fn public_cone_preserves_original_objective_and_refuses_invalid_geometry() -> Result<()> {
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        let package = context.package("package cone { def Root { param p:Scalar=1; } }").await?;
        let physical = package.physical_context();
        let quantity = physical.quantities().neutral_dimensionless().unwrap();
        let unit = physical.quantities().quantity_type(quantity).unwrap().canonical_unit;
        let port = |symbol_id| AnalysisPort { symbol_id, quantity_id: quantity.as_id(), unit_id: unit.as_id() };
        // min x+3 subject to x>=2. Both optimum x=2 and objective 5 are independent answers.
        let request = ConicRequest {
            variables: vec![port(pse_ids::SemanticId::from_bytes([1;16]))],
            rows: vec![port(pse_ids::SemanticId::from_bytes([2;16]))],
            objective_port: port(pse_ids::SemanticId::NIL),
            quadratic: SparseMatrix::zeros(1,1), objective: vec![1.0],
            constraints: SparseMatrix::new(1,1,vec![0,1],vec![0],vec![-1.0]),
            rhs: vec![-2.0], cones: vec![Cone::Nonnegative { dimension:1 }], objective_constant:3.0,
        };
        let mut invalid = request.clone(); invalid.rhs.clear();
        assert!(context.runtime.prepare_conic(invalid, physical, SolverProfile::default()).await.is_err());
        let prepared = context.runtime.prepare_conic(request, physical,
            SolverProfile { intent: SolveIntent::Optimize, ..Default::default() }).await?;
        let report = prepared.start(&context.runtime.preparation_services())?.finish().await?;
        let Outcome::Native(report) = &report.outcome else { panic!("native cone report required"); };
        let candidate = report.candidate.as_ref().unwrap();
        assert!((candidate.primal[0]-2.0).abs()<1e-6);
        assert!((candidate.objective.unwrap()-5.0).abs()<1e-6);
        assert!(report.quality.as_ref().unwrap().feasible());
        assert_eq!(report.qualification, pse_backend_native::solve::Qualification::OptimalWithinTolerance);
        context.close().await?;
        // Independently owned answers remain accessible after caller and native lane closure.
        assert!((candidate.primal[0]-2.0).abs()<1e-6);
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    }).catch_unwind().await;
    let cleanup = context.close().await;
    match body { Ok(result) => { result?; cleanup }, Err(panic) => { let _ = cleanup; std::panic::resume_unwind(panic) } }
}
