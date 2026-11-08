// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public Rust KKT response in original Scalar coordinates, without Python candidates.

use crate::reference_support::{NativeContext, Result};
use crate::reference_scalar::{self, Inputs};
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
use std::collections::BTreeMap;

#[derive(Clone)]
pub(super) struct SensitivityCase {
    pub(super) id: String,
    pub(super) nominal: BTreeMap<String, f64>,
    pub(super) perturbed: BTreeMap<String, f64>,
    pub(super) variables: Vec<String>,
    pub(super) unit: String,
    pub(super) initial: f64,
    pub(super) z_bounds: [f64; 2],
    pub(super) coupling: [f64; 2],
    pub(super) objective: [f64; 5],
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
            coupling: [1., 1.],
            objective: [0.5, 1., 1., 0.5, 1.],
        }
    }

    fn source(&self) -> String {
        let [coupling_x, coupling_y] = self.coupling;
        let [square_x, square_y, product_b_x, square_z, product_a_z] = self.objective;
        format!(
            "package parity {{ def Root {{
             param a: Scalar = {}; param b: Scalar = {};
             var x: Scalar; var y: Scalar; var z: Scalar;
             eq coupling: {coupling_x}*x + {coupling_y}*y == a;
             let cost: Scalar = {square_x}*x*x + {square_y}*y*y - {product_b_x}*b*x + {square_z}*z*z + {product_a_z}*a*z;
             annotation objective cost(minimize); annotation bounds z({}, {});
             annotation start x({}); annotation start y({}); annotation start z({}); }} }}",
            self.nominal["a"],
            self.nominal["b"],
            self.z_bounds[0],
            self.z_bounds[1],
            self.initial,
            self.initial,
            self.initial,
        )
    }
}

#[derive(Clone)]
pub(super) struct QuantityReport {
    pub(super) base: f64,
    pub(super) increment: f64,
    pub(super) predicted: f64,
    pub(super) allowance: f64,
    pub(super) coordinate_scale: f64,
    /// dx/dp, in original target units per original parameter unit.
    pub(super) derivative: BTreeMap<String, f64>,
}

#[derive(Clone)]
pub(super) struct CandidateReport {
    pub(super) complete: bool,
    pub(super) unit: String,
    pub(super) coordinates: String,
    pub(super) parameter_scales: BTreeMap<String, f64>,
    pub(super) quantities: BTreeMap<String, QuantityReport>,
}

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
        Ok(Self {
            context: NativeContext::new()?,
        })
    }

    pub(super) fn pool(&self) -> std::sync::Arc<dyn pse_columnar::MemoryPool> {
        self.context.shared.pool()
    }

    pub(super) async fn run(&self, case: &SensitivityCase) -> Result<(CandidateReport, Inputs)> {
        let source = case.source();
        let (package, originals) = self.context.package(&source).await?;
        let services = self.context.runtime.execution_services();
        let cancel = CancelSource::new();
        let declarations = package.declarations(services.preparation()).await?;
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let inputs = [
            ("nominal:a", case.nominal["a"]), ("nominal:b", case.nominal["b"]),
            ("perturbed:a", case.perturbed["a"]), ("perturbed:b", case.perturbed["b"]),
            ("initial", case.initial), ("z:lower", case.z_bounds[0]), ("z:upper", case.z_bounds[1]),
            ("coupling:x", case.coupling[0]), ("coupling:y", case.coupling[1]),
            ("objective:x*x", case.objective[0]), ("objective:y*y", case.objective[1]),
            ("objective:-b*x", case.objective[2]), ("objective:z*z", case.objective[3]),
            ("objective:a*z", case.objective[4]),
        ];
        let mut outputs = Vec::new();
        for stage in ["sipopt:nominal", "sipopt:prediction", "exact:nominal", "exact:prediction"] {
            outputs.extend(case.variables.iter().map(|name| format!("{stage}:{name}")));
        }
        for name in &case.variables {
            outputs.extend(["a", "b"].map(|parameter| format!("derivative:{name}:{parameter}")));
        }
        let configuration = pse_ids::preimage::hash(&serde_json::to_vec(&(
            &case.variables, &case.unit, inputs.map(|(key, value)| (key, value.to_bits())),
        ))?);
        let contract = reference_scalar::bind(reference_scalar::Case {
            name: &case.id,
            identity: root.as_id(),
            source: &source,
            configuration,
            inputs: inputs.into_iter().map(|(key, value)| (key.into(), value)).collect(),
            outputs: outputs.into_iter().map(|key| (key.clone(), pse_ids::named_id(root.as_id(), &key))).collect(),
        }, package.physical_context(), originals)?;
        let mut settings = SolveSettings {
            backend: Some(Backend::Ipopt),
            intent: SolveIntent::Optimize,
            numerics: pse_model::numerics::NumericalPolicy {
                // This fixed active-bound case compares a local derivative and a
                // physical prediction at a prescribed point. Keep the native
                // complementarity error well inside the oracle's point budget.
                kkt: pse_model::numerics::KktTolerances {
                    stationarity: 1e-9,
                    complementarity: 1e-9,
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let mut analysis = ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Default::default(),
            limits: Default::default(),
            case: Default::default(),
            order: pse_kernels::DerivativeOrder::Second,
            compiler: Default::default(),
            solver: settings.clone().profile()?,
            numerical: Default::default(),
        };
        analysis.bindings.demand = ["a", "b", "x", "y", "z"].map(String::from).to_vec();
        // Identity discovery is public prepared science, with no Python fixture adapter.
        let plain =
            Box::pin(package.prepare_analysis(services.preparation(), &analysis, &cancel)).await?;
        let paths = &plain.model.model.compiled().model.paths;
        let ids: BTreeMap<&str, SemanticId> = ["a", "b", "x", "y", "z"]
            .into_iter()
            .map(|name| (name, paths[name]))
            .collect();
        let [a, b] = [ids["a"], ids["b"]];
        settings.sensitivity = Some(SensitivityRequest {
            parameters: vec![a, b],
            reduced_hessian: false,
            propagation: None,
        });
        assert_eq!(
            settings.sensitivity.as_ref().unwrap().parameters,
            vec![a, b]
        );
        analysis.solver = settings.clone().profile()?;
        drop(plain);
        let prepared =
            Box::pin(package.prepare_analysis(services.preparation(), &analysis, &cancel)).await?;
        let handle = prepared.start(&services)?;
        let result = handle.wait().await?;
        assert!(result.usable(), "{:?}", result.report());

        let variables: Vec<solve_variables::Row> = rows(&result, "runtime.solve_variables");
        let indexed: BTreeMap<_, _> = variables.iter().map(|row| (row.symbol_id, row)).collect();
        assert_eq!(
            indexed.len(),
            variables.len(),
            "duplicate native variable identity"
        );
        for (name, parameter) in [("a", a), ("b", b)] {
            let row = indexed[&parameter];
            // These are authored parameter bindings in original units, never KKT scales.
            assert_eq!(real(row.value), case.nominal[name]);
            assert!(row.parameter && row.fixed);
            assert!(row.unit_id.is_some() && row.quantity_id.is_some());
        }
        let assessments = result.assessments();
        let [assessment] = assessments else {
            panic!("one candidate assessment required: {assessments:?}")
        };
        assert_eq!(assessment.validated, Some(true));
        assert!(assessment.permits_result && assessment.refusals.is_empty());
        let validity: Vec<local_validity::Row> = rows(&result, "runtime.local_validity");
        let [local] = validity.as_slice() else {
            panic!("one local validity required: {validity:?}")
        };
        assert_eq!(local.quantity, DerivedQuantity::ParametricSensitivity);
        let valid = &local.validity;
        assert!(valid.certified && valid.reason.is_none() && !valid.conditional);
        assert_eq!(
            (
                valid.licq,
                valid.strict_complementarity,
                valid.second_order,
                valid.weakly_active
            ),
            (Some(true), Some(true), Some(true), Some(0))
        );
        let basis = (local.run_id, local.step);
        assert_eq!((assessment.run_id, assessment.step), basis);
        assert!(variables.iter().all(|row| (row.run_id, row.step) == basis));
        let bound = indexed[&ids["z"]];
        assert_eq!(bound.lower, Some(case.z_bounds[0]));
        assert_eq!(bound.upper, Some(case.z_bounds[1]));
        assert_eq!(
            bound.dual_qualification,
            DualQualification::SensitivityCertified
        );
        assert!((real(bound.value) - real(bound.lower)).abs() <= real(bound.tolerance));
        assert!(real(bound.lower_dual) > 0.0);
        assert!(real(bound.upper_dual) < real(bound.lower_dual));
        // The active lower branch stays regular: unconstrained z=-a is inadmissible.
        assert!(case.nominal["a"] > 0.0 && case.perturbed["a"] > 0.0);

        let numerics: Vec<resolved_numerics::Row> = rows(&result, "runtime.resolved_numerics");
        let engineering_fraction =
            pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
        let mut resolved = BTreeMap::new();
        for row in &numerics {
            assert_eq!((row.run_id, row.step), basis);
            assert!(row.nominal.is_finite() && row.nominal > 0.0);
            assert!(row.coordinate_scale.is_finite() && row.coordinate_scale > 0.0);
            let engineering = row.engineering.as_ref().unwrap();
            assert_eq!(engineering.relative_fraction, engineering_fraction);
            assert!(
                row.provenance
                    .iter()
                    .all(|item| !item.selected || item.source != NumericalSource::Analysis)
            );
            if row.target_kind == NumericalTarget::Variable {
                assert!(resolved.insert(row.target_id, row).is_none());
                assert_eq!(Some(row.unit_id), indexed[&row.target_id].unit_id);
                assert_eq!(Some(row.quantity_id), indexed[&row.target_id].quantity_id);
                assert!(engineering.budget.is_finite() && engineering.budget > 0.0);
                assert_eq!(engineering.budget, row.budget);
            }
        }
        assert!(ids.values().all(|id| resolved.contains_key(id)));
        let sensitivities: Vec<parametric_sensitivities::Row> =
            rows(&result, "runtime.parametric_sensitivities");
        assert!(
            sensitivities
                .iter()
                .all(|row| (row.run_id, row.step) == basis)
        );
        let parameter_scales = [("a", a), ("b", b)]
            .into_iter()
            .map(|(name, id)| (name.to_owned(), resolved[&id].coordinate_scale))
            .collect();
        let mut quantities = BTreeMap::new();
        for name in ["x", "y", "z"] {
            let target = ids[name];
            let variable = indexed[&target];
            assert_eq!(
                variable.dual_qualification,
                DualQualification::SensitivityCertified
            );
            assert!(!variable.fixed && !variable.parameter);
            let allowance = resolved[&target].engineering.as_ref().unwrap().budget;
            let output_scale = resolved[&target].coordinate_scale;
            let mut derivative = BTreeMap::new();
            for row in sensitivities.iter().filter(|row| {
                row.target_kind == NumericalTarget::Variable && row.target_id == target
            }) {
                let parameter_name = if row.parameter_id == a {
                    "a"
                } else {
                    assert_eq!(row.parameter_id, b);
                    "b"
                };
                let parameter = indexed[&row.parameter_id];
                assert_eq!(parameter.quantity_id, indexed[&a].quantity_id);
                assert_eq!(variable.quantity_id, indexed[&a].quantity_id);
                assert_eq!(Some(row.parameter_unit_id), parameter.unit_id);
                assert_eq!(Some(row.target_unit_id), variable.unit_id);
                assert_eq!(row.parameter_unit_id, row.target_unit_id);
                let value = real(row.primal);
                assert!(
                    derivative
                        .insert(parameter_name.to_owned(), value)
                        .is_none()
                );
            }
            assert_eq!(
                derivative.keys().map(String::as_str).collect::<Vec<_>>(),
                ["a", "b"]
            );
            let base = real(variable.value);
            let increment = derivative
                .iter()
                .map(|(parameter, value)| {
                    value * (case.perturbed[parameter] - case.nominal[parameter])
                })
                .sum();
            quantities.insert(
                name.into(),
                QuantityReport {
                    base,
                    increment,
                    predicted: base + increment,
                    allowance,
                    coordinate_scale: output_scale,
                    derivative,
                },
            );
        }
        Ok((CandidateReport {
            complete: true,
            unit: case.unit.clone(),
            coordinates: "physical".into(),
            parameter_scales,
            quantities,
        }, contract))
    }

    pub(super) async fn close(&self) -> Result<()> {
        self.context.close().await
    }
}
