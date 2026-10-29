// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public Rust KKT response in original Scalar coordinates, without Python candidates.

use crate::reference_support::{NativeContext, Result};
use pse_backend_native::solve::{Backend, SolveIntent};
use pse_ids::SemanticId;
use pse_relations::{
    columnar::RelationRow,
    generated::{
        enums::{DerivedQuantity, DualQualification, NumericalSource, NumericalTarget},
        runtime::{local_validity, parametric_sensitivities, resolved_numerics, solve_variables},
    },
};
use pse_runtime::{
    CancelSource,
    math::settings::{SensitivityRequest, SolveSettings},
    workflow::{ModelingAnalysis, RunResult},
};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Serialize)]
pub(super) struct SensitivityCase {
    pub(super) id: String,
    pub(super) nominal: BTreeMap<String, f64>,
    pub(super) perturbed: BTreeMap<String, f64>,
    pub(super) variables: Vec<String>,
    pub(super) unit: String,
    pub(super) initial: f64,
    pub(super) z_bounds: [f64; 2],
    pub(super) coupling: String,
    pub(super) objective: String,
}

impl SensitivityCase {
    pub(super) fn quadratic() -> Self {
        Self {
            id: "active-bound-quadratic-response".into(),
            nominal: BTreeMap::from([("a".into(), 1.0), ("b".into(), 2.0)]),
            perturbed: BTreeMap::from([("a".into(), 1.1), ("b".into(), 2.1)]),
            variables: ["x", "y", "z"].map(String::from).to_vec(),
            unit: "dimensionless Scalar".into(),
            initial: 0.5,
            z_bounds: [0.0, 10.0],
            coupling: "x + y == a".into(),
            objective: "0.5*x*x + y*y - b*x + 0.5*z*z + a*z".into(),
        }
    }

    fn source(&self) -> String {
        format!(
            "package parity {{ def Root {{
             param a: Scalar = {}; param b: Scalar = {};
             var x: Scalar; var y: Scalar; var z: Scalar;
             eq coupling: {}; let cost: Scalar = {};
             annotation objective cost(minimize); annotation bounds z({}, {});
             annotation start x({}); annotation start y({}); annotation start z({}); }} }}",
            self.nominal["a"], self.nominal["b"], self.coupling, self.objective,
            self.z_bounds[0], self.z_bounds[1], self.initial, self.initial, self.initial,
        )
    }
}

#[derive(Clone, Serialize)]
pub(super) struct QuantityReport {
    pub(super) base: f64,
    pub(super) increment: f64,
    pub(super) predicted: f64,
    pub(super) allowance: f64,
    pub(super) coordinate_scale: f64,
    /// dx/dp, in original target units per original parameter unit.
    pub(super) derivative: BTreeMap<String, f64>,
}

#[derive(Clone, Serialize)]
pub(super) struct CandidateReport {
    pub(super) complete: bool,
    pub(super) parameter_scales: BTreeMap<String, f64>,
    pub(super) quantities: BTreeMap<String, QuantityReport>,
}

#[derive(Serialize)]
pub(super) struct Candidate {
    pub(super) id: String,
    pub(super) report: Option<CandidateReport>,
}

fn rows<R: RelationRow>(result: &RunResult, name: &str) -> Vec<R> {
    R::rows(&result.table(name).unwrap()).unwrap()
}

fn real(value: Option<f64>) -> f64 {
    let value = value.expect("required native scientific value");
    assert!(value.is_finite(), "nonfinite native scientific value");
    value
}

pub(super) struct NativeCandidate {
    context: NativeContext,
}

impl NativeCandidate {
    pub(super) fn new() -> Result<Self> {
        Ok(Self { context: NativeContext::new()? })
    }

    pub(super) async fn run(&self, case: &SensitivityCase) -> Result<CandidateReport> {
        let source = case.source();
        let package = self.context.package(&source).await?;
        let services = self.context.runtime.execution_services();
        let cancel = CancelSource::new();
        let declarations = package.declarations(services.preparation()).await?;
        let root = declarations.iter().find(|row| row.name == "Root").unwrap().declaration_id;
        let mut settings = SolveSettings {
            backend: Some(Backend::Ipopt), intent: SolveIntent::Optimize, ..Default::default()
        };
        let mut analysis = ModelingAnalysis {
            root, instance: pse_modeling::specialize::root_instance(root),
            bindings: Default::default(), limits: Default::default(), case: Default::default(),
            order: pse_kernels::DerivativeOrder::Second, compiler: Default::default(),
            solver: settings.clone().profile()?, numerical: Default::default(),
        };
        analysis.bindings.demand = ["a", "b", "x", "y", "z"].map(String::from).to_vec();
        // Identity discovery is public prepared science, with no Python fixture adapter.
        let plain = Box::pin(package.prepare_analysis(services.preparation(), &analysis, &cancel)).await?;
        let paths = &plain.model.model.compiled().model.paths;
        let ids: BTreeMap<&str, SemanticId> = ["a", "b", "x", "y", "z"]
            .into_iter().map(|name| (name, paths[name])).collect();
        let [a, b] = [ids["a"], ids["b"]];
        settings.sensitivity = Some(SensitivityRequest {
            parameters: vec![a, b], reduced_hessian: false, propagation: None,
        });
        assert_eq!(serde_json::to_value(&settings.numerics)?,
            serde_json::to_value(pse_model::numerics::NumericalPolicy::default())?);
        assert_eq!(settings.sensitivity.as_ref().unwrap().parameters, vec![a, b]);
        analysis.solver = settings.clone().profile()?;
        drop(plain);
        let prepared = Box::pin(package.prepare_analysis(services.preparation(), &analysis, &cancel)).await?;
        let handle = prepared.start(&services)?;
        let result = handle.wait().await?;
        assert!(result.usable(), "{:?}", result.report());

        let variables: Vec<solve_variables::Row> = rows(&result, "runtime.solve_variables");
        let indexed: BTreeMap<_, _> = variables.iter().map(|row| (row.symbol_id, row)).collect();
        assert_eq!(indexed.len(), variables.len(), "duplicate native variable identity");
        for (name, parameter) in [("a", a), ("b", b)] {
            let row = indexed[&parameter];
            // These are authored parameter bindings in original units, never KKT scales.
            assert_eq!(real(row.value), case.nominal[name]);
            assert!(row.parameter && row.fixed);
            assert!(row.unit_id.is_some() && row.quantity_id.is_some());
        }
        let assessments = result.assessments();
        let [assessment] = assessments else { panic!("one candidate assessment required: {assessments:?}") };
        assert_eq!(assessment.validated, Some(true));
        assert!(assessment.permits_result && assessment.refusals.is_empty());
        let validity: Vec<local_validity::Row> = rows(&result, "runtime.local_validity");
        let [local] = validity.as_slice() else { panic!("one local validity required: {validity:?}") };
        assert_eq!(local.quantity, DerivedQuantity::ParametricSensitivity);
        let valid = &local.validity;
        assert!(valid.certified && valid.reason.is_none() && !valid.conditional);
        assert_eq!((valid.licq, valid.strict_complementarity, valid.second_order, valid.weakly_active),
            (Some(true), Some(true), Some(true), Some(0)));
        let basis = (local.run_id, local.step);
        assert_eq!((assessment.run_id, assessment.step), basis);
        assert!(variables.iter().all(|row| (row.run_id, row.step) == basis));
        let bound = indexed[&ids["z"]];
        assert_eq!(bound.lower, Some(case.z_bounds[0]));
        assert_eq!(bound.upper, Some(case.z_bounds[1]));
        assert_eq!(bound.dual_qualification, DualQualification::SensitivityCertified);
        assert!((real(bound.value) - real(bound.lower)).abs() <= real(bound.tolerance));
        assert!(real(bound.lower_dual) > 0.0);
        assert!(real(bound.upper_dual) < real(bound.lower_dual));
        // The active lower branch stays regular: unconstrained z=-a is inadmissible.
        assert!(case.nominal["a"] > 0.0 && case.perturbed["a"] > 0.0);

        let numerics: Vec<resolved_numerics::Row> = rows(&result, "runtime.resolved_numerics");
        let mut resolved = BTreeMap::new();
        for row in &numerics {
            assert_eq!((row.run_id, row.step), basis);
            assert!(row.nominal.is_finite() && row.nominal > 0.0);
            assert!(row.coordinate_scale.is_finite() && row.coordinate_scale > 0.0);
            let engineering = row.engineering.as_ref().unwrap();
            assert_eq!(engineering.relative_fraction, settings.numerics.engineering_relative_fraction);
            assert!(row.provenance.iter().all(|item| !item.selected || item.source != NumericalSource::Analysis));
            if row.target_kind == NumericalTarget::Variable {
                assert!(resolved.insert(row.target_id, row).is_none());
                assert_eq!(Some(row.unit_id), indexed[&row.target_id].unit_id);
                assert_eq!(Some(row.quantity_id), indexed[&row.target_id].quantity_id);
                assert!(engineering.budget.is_finite() && engineering.budget > 0.0);
                assert_eq!(engineering.budget, row.budget);
            }
        }
        assert!(ids.values().all(|id| resolved.contains_key(id)));
        let sensitivities: Vec<parametric_sensitivities::Row> = rows(&result, "runtime.parametric_sensitivities");
        assert!(sensitivities.iter().all(|row| (row.run_id, row.step) == basis));
        let parameter_scales = [("a", a), ("b", b)].into_iter()
            .map(|(name, id)| (name.to_owned(), resolved[&id].coordinate_scale)).collect();
        let mut quantities = BTreeMap::new();
        for (name, jacobian) in [("x", [2.0 / 3.0, 1.0 / 3.0]), ("y", [1.0 / 3.0, -1.0 / 3.0]), ("z", [0.0, 0.0])] {
            let target = ids[name];
            let variable = indexed[&target];
            assert_eq!(variable.dual_qualification, DualQualification::SensitivityCertified);
            assert!(!variable.fixed && !variable.parameter);
            let allowance = resolved[&target].engineering.as_ref().unwrap().budget;
            let output_scale = resolved[&target].coordinate_scale;
            let mut derivative = BTreeMap::new();
            for row in sensitivities.iter().filter(|row| row.target_kind == NumericalTarget::Variable && row.target_id == target) {
                let (parameter_name, expected) = if row.parameter_id == a { ("a", jacobian[0]) }
                    else { assert_eq!(row.parameter_id, b); ("b", jacobian[1]) };
                let parameter = indexed[&row.parameter_id];
                assert_eq!(parameter.quantity_id, indexed[&a].quantity_id);
                assert_eq!(variable.quantity_id, indexed[&a].quantity_id);
                assert_eq!(Some(row.parameter_unit_id), parameter.unit_id);
                assert_eq!(Some(row.target_unit_id), variable.unit_id);
                assert_eq!(row.parameter_unit_id, row.target_unit_id);
                let value = real(row.primal);
                assert!(derivative.insert(parameter_name.to_owned(), value).is_none());
                let parameter_scale = resolved[&row.parameter_id].coordinate_scale;
                // J_hat=J*S_p/S_x. This empirical physical allowance is no backward-error proof.
                let normalized = value * parameter_scale / output_scale;
                let wanted = expected * parameter_scale / output_scale;
                assert!((normalized - wanted).abs() <= allowance / output_scale, "{name}/{parameter_name}");
            }
            assert_eq!(derivative.keys().map(String::as_str).collect::<Vec<_>>(), ["a", "b"]);
            let base = real(variable.value);
            let increment = derivative.iter().map(|(parameter, value)|
                value * (case.perturbed[parameter] - case.nominal[parameter])).sum();
            quantities.insert(name.into(), QuantityReport {
                base, increment, predicted: base + increment, allowance,
                coordinate_scale: output_scale, derivative,
            });
        }
        Ok(CandidateReport { complete: true, parameter_scales, quantities })
    }

    pub(super) async fn close(&self) -> Result<()> { self.context.close().await }
}
