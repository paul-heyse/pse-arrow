// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Test-only local native science and explicit exchange values; no product adapter.

use crate::reference_scalar::{self, Inputs};
use pse_backend_native::solve::SolveIntent;
use pse_runtime::{
    CancelSource,
    math::settings::SolveSettings,
    workflow::{DiagnosticOperation, JacobianDiagnosticControls, ModelingAnalysis},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

pub(super) use crate::reference_support::Result;

#[derive(Clone)]
pub(super) struct LinearRow {
    pub(super) name: String,
    pub(super) coefficients: Vec<f64>,
    pub(super) rhs: f64,
}

#[derive(Clone)]
pub(super) struct LinearCase {
    pub(super) id: String,
    pub(super) variables: Vec<String>,
    pub(super) point: Vec<f64>,
    pub(super) rows: Vec<LinearRow>,
}

impl LinearCase {
    pub(super) fn dependent() -> Self {
        Self {
            id: "degenerate-jacobian-five-rows-v1".into(),
            variables: ["a", "b", "c", "d", "e"].map(String::from).to_vec(),
            point: vec![0.5; 5],
            rows: [
                ("first", [1., 1., 0., 0., 0.], 1.),
                ("second", [2., 2., 0., 0., 0.], 2.),
                ("third", [0., 0., 1., 1., 0.], 1.),
                ("fourth", [0., 0., 0., 1., 1.], 1.),
                ("fifth", [0., 0., 1., 2., 1.], 2.),
            ]
            .into_iter()
            .map(|(name, coefficients, rhs)| LinearRow {
                name: name.into(),
                coefficients: coefficients.to_vec(),
                rhs,
            })
            .collect(),
        }
    }

    fn source(&self) -> String {
        // The original parity case, authored from the explicit exchange coefficients.
        let mut source = String::from("package parity { def Dependent { ");
        for (name, value) in self.variables.iter().zip(&self.point) {
            source.push_str(&format!(
                "var {name}:Scalar; annotation start {name}({value}); "
            ));
        }
        for row in &self.rows {
            let terms = row
                .coefficients
                .iter()
                .zip(&self.variables)
                .filter(|(coefficient, _)| **coefficient != 0.)
                .map(|(coefficient, variable)| format!("{coefficient}*{variable}"))
                .collect::<Vec<_>>()
                .join(" + ");
            source.push_str(&format!("eq {}: {terms} == {}; ", row.name, row.rhs));
        }
        source.push_str("} }");
        source
    }
}

#[derive(Clone, Debug)]
pub(super) struct RowSet {
    pub(super) rows: Vec<String>,
    /// Multipliers on original unscaled equations (native weight / physical row nominal).
    pub(super) multipliers: BTreeMap<String, f64>,
    pub(super) irreducible_at_tolerance: bool,
}

#[derive(Clone)]
pub(super) struct CandidateReport {
    pub(super) complete: bool,
    pub(super) unavailable: Vec<String>,
    pub(super) sets: Vec<RowSet>,
}

pub(super) struct Candidate {
    pub(super) id: String,
    pub(super) report: Option<CandidateReport>,
}

pub(super) struct NativeCandidate {
    context: crate::reference_support::NativeContext,
}
impl NativeCandidate {
    pub(super) fn new() -> Result<Self> {
        Ok(Self {
            context: crate::reference_support::NativeContext::new()?,
        })
    }
    pub(super) async fn run(&self, case: &LinearCase) -> Result<(CandidateReport, Inputs)> {
        let source_text = case.source();
        let (package, originals) = self.context.package(&source_text).await?;
        let cancel = CancelSource::new();
        let services = self.context.runtime.execution_services();
        let declaration = package
            .declarations(services.preparation())
            .await?
            .iter()
            .find(|row| row.name == "Dependent")
            .unwrap()
            .declaration_id;
        let analysis = ModelingAnalysis {
            root: declaration,
            instance: pse_modeling::specialize::root_instance(declaration),
            bindings: Default::default(),
            limits: Default::default(),
            case: pse_compiler::workspace::ModelingCaseBindings {
                values: case
                    .variables
                    .iter()
                    .cloned()
                    .zip(case.point.iter().copied())
                    .collect(),
                ..Default::default()
            },
            order: pse_kernels::DerivativeOrder::First,
            compiler: Default::default(),
            solver: SolveSettings {
                intent: SolveIntent::Root,
                ..Default::default()
            }
            .profile()?,
            numerical: Default::default(),
        };
        let prepared = package
            .prepare_diagnostics(services.preparation(), &analysis, &cancel)
            .await?;
        let source_map = self
            .context
            .runtime
            .modeling_diagnostic_source_map(&prepared, 1 << 20, Duration::from_secs(30), &cancel)
            .await?;
        let source = source_map.document();
        source.validate()?;
        let names: BTreeMap<_, _> = source
            .members
            .iter()
            .filter(|member| source.rows.contains(&member.member))
            .map(|member| {
                (
                    member.member,
                    member.path.rsplit('.').next().unwrap().to_owned(),
                )
            })
            .collect();
        assert_eq!(
            names.len(),
            5,
            "every native Jacobian row must retain its authored name"
        );
        assert_eq!(
            names.values().cloned().collect::<BTreeSet<_>>(),
            case.rows.iter().map(|row| row.name.clone()).collect()
        );
        let mut inputs = case
            .variables
            .iter()
            .zip(&case.point)
            .map(|(name, value)| (format!("point:{name}"), *value))
            .collect::<Vec<_>>();
        let mut outputs = Vec::new();
        for row in &case.rows {
            inputs.extend(case.variables.iter().zip(&row.coefficients).map(
                |(name, coefficient)| {
                    (format!("row:{}:coefficient:{name}", row.name), *coefficient)
                },
            ));
            inputs.push((format!("row:{}:rhs", row.name), row.rhs));
            let id = names
                .iter()
                .find(|(_, name)| *name == &row.name)
                .map(|(id, _)| *id)
                .ok_or("original row identity is unavailable")?;
            outputs.push((format!("row:{}", row.name), id));
        }
        let configuration = pse_ids::preimage::hash(&serde_json::to_vec(
            &inputs
                .iter()
                .map(|(key, value)| (key, value.to_bits()))
                .collect::<Vec<_>>(),
        )?);
        let contract = reference_scalar::bind(
            reference_scalar::Case {
                name: &case.id,
                identity: declaration.as_id(),
                source: &source_text,
                configuration,
                inputs,
                outputs,
            },
            package.physical_context(),
            originals,
        )?;
        let nominals: BTreeMap<_, _> = source.row_nominals.iter().copied().collect();
        let values = prepared.model.values.clone();
        let native = package
            .diagnose_jacobian_optimization(
                &services,
                prepared,
                values,
                JacobianDiagnosticControls {
                    maximum_attempts: 16,
                    ..Default::default()
                }
                .policy(),
                Default::default(),
                &DiagnosticOperation::new(&cancel, Duration::from_secs(120), None)?,
            )
            .await?;
        let sets = native
            .evidence
            .degenerate
            .iter()
            .map(|found| {
                let rows = found.rows.iter().map(|id| names[id].clone()).collect();
                let multipliers = found
                    .certificate
                    .weights
                    .iter()
                    .filter(|(_, weight)| *weight != 0.)
                    .map(|(id, weight)| (names[id].clone(), weight / nominals[id]))
                    .collect();
                RowSet {
                    rows,
                    multipliers,
                    irreducible_at_tolerance: found.irreducible_at_tolerance,
                }
            })
            .collect();
        Ok((
            CandidateReport {
                complete: native.evidence.complete,
                unavailable: native.evidence.unavailable.clone(),
                sets,
            },
            contract,
        ))
    }

    pub(super) async fn close(&self) -> Result<()> {
        self.context.close().await
    }
}
