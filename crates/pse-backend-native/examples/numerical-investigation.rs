// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite investigation inputs through the actual dynamic derivative diagnostic.
#![allow(
    clippy::print_stdout,
    reason = "an explicit diagnostic example exports JSON"
)]
#![allow(
    unused_crate_dependencies,
    reason = "this diagnostic example selects only the public dynamic sampling surface"
)]
use faer::sparse::{SparseColMat, Triplet};
use pse_backend_native::{
    ProblemError,
    derivative_diagnostics::{DynamicSample, Policy, analyze_dynamic},
    dynamics::{self, Function},
    solve::{Controls, Execution},
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_math::normalization::Normalization;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug)]
struct FunctionOracle {
    contract: dynamics::Contract,
    logarithmic: bool,
    wrong: bool,
}
impl dynamics::Oracle for FunctionOracle {
    fn contract(&self) -> &dynamics::Contract {
        &self.contract
    }
    fn support(&self, _: usize, _: Function) -> Vec<dynamics::SupportEntry> {
        vec![dynamics::SupportEntry::new(
            pse_math::index::OriginalRow::new(0),
            pse_math::index::OriginalCol::new(0),
        )]
    }
    fn evaluate(
        &mut self,
        _: usize,
        _: Function,
        _: f64,
        state: &[f64],
        _: &[f64],
        derivatives: bool,
    ) -> Result<dynamics::Evaluation, ProblemError> {
        let x = state[0];
        if self.logarithmic && x <= -1. {
            return Err(ProblemError::Math(pse_math::MathError::OutsideRange {
                source_id: self.contract.outputs[0],
                target: self.contract.states[0],
                value: x,
                lower: Some(-1.),
                upper: None,
            }));
        }
        let (value, derivative) = if self.logarithmic {
            (x.ln_1p(), 1. / (1. + x))
        } else {
            (x * x * x, 3. * x * x)
        };
        let jacobian = if derivatives {
            Some(
                Arc::new(
                    SparseColMat::try_new_from_triplets(
                        1,
                        1,
                        &[Triplet::new(
                            0,
                            0,
                            if self.wrong {
                                derivative * 2.
                            } else {
                                derivative
                            },
                        )],
                    )
                    .map_err(|error| ProblemError::Contract(error.to_string()))?,
                )
                .into(),
            )
        } else {
            None
        };
        Ok(dynamics::Evaluation {
            values: vec![value],
            jacobian,
        })
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut samples = Vec::new();
    for (name, logarithmic, point, bounds, scale, wrong) in [
        ("analytic-cubic", false, 2., (-10., 10.), 4., false),
        ("wrong-derivative", false, 2., (-10., 10.), 4., true),
        ("log-central", true, -0.9999, (-1., 10.), 1., false),
        ("log-inward", true, -0.9999, (-0.9999, 10.), 1., false),
        (
            "log-refused",
            true,
            -0.9999,
            (f64::NEG_INFINITY, f64::INFINITY),
            1.,
            false,
        ),
    ] {
        for exponent in 1..=17 {
            let perturbation = 10_f64.powi(-exponent);
            let id = |n| SemanticId::from_bytes([n; 16]);
            let oracle = FunctionOracle {
                contract: dynamics::Contract {
                    identity: ContentHash::from_bytes([52; 32]),
                    states: vec![id(1)],
                    differential: vec![true],
                    parameters: vec![],
                    outputs: vec![id(2)],
                    events: vec![vec![]],
                    signs: vec![],
                    quadratures: vec![],
                    balances: vec![],
                    derivatives: DerivativeOrder::First,
                },
                logarithmic,
                wrong,
            };
            let report = analyze_dynamic(
                Box::new(oracle),
                DynamicSample {
                    mode: 0,
                    function: Function::Output,
                    time: 0.,
                    state: vec![point],
                    parameters: vec![],
                    bounds: vec![bounds],
                },
                Normalization {
                    variables: vec![scale],
                    rows: vec![1.],
                    objective: 1.,
                },
                Policy {
                    perturbation,
                    relative_tolerance: 1e-6,
                    maximum_cells: 2,
                },
                Execution::new(Default::default(), &Controls::default()),
            )?;
            samples.push(json!({"case":name,"point":point,"coordinate":"state[0]","coordinate_unit":"dimensionless","output_unit":"dimensionless","coordinate_scale":scale,"row_scale":1.,"perturbation":perturbation,"complete":report.complete,"passed":report.passed(),"rejected_evaluations":report.rejected_evaluations,"terminal":format!("{:?}",report.terminal),"failure":report.failure.as_ref().map(ToString::to_string),"comparisons":report.comparisons,"summary":report.summary()}));
        }
    }
    println!(
        "{}",
        serde_json::to_string(
            &json!({"schema":"numerical-investigation-v1","scope":"actual dynamic derivative diagnostics at fixed time and mode; local derivative evidence only","samples":samples})
        )?
    );
    Ok(())
}
