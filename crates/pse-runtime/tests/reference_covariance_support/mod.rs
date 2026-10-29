// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Test-only weighted fit through public Rust scientific APIs.

use futures_util::future::BoxFuture;
use pse_backend_native::solve::{Backend, SolveIntent};
use pse_ids::SemanticId;
use pse_model::generated::enums::NumericalTarget;
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{
        fit_observations, fit_parameters, parameter_covariances, resolved_numerics,
    },
};
use pse_runtime::{
    CancelSource,
    math::settings::SolveSettings,
    workflow::{
        FitDeclarations, FitPreparationDocument, FitProfile, ModelingPackage, PreparationServices,
        PreparedFit, SourceServices,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::reference_support::NativeContext;
pub(super) use crate::reference_support::{Result, root};

#[derive(Clone, Serialize)]
pub(super) struct LineCase {
    pub(super) id: String,
    pub(super) x: Vec<f64>,
    pub(super) y: Vec<f64>,
    pub(super) sigma: f64,
    pub(super) parameters: Vec<String>,
    pub(super) response_unit: String,
    pub(super) parameter_units: Vec<String>,
    pub(super) covariance_convention: String,
}

impl LineCase {
    pub(super) fn weighted() -> Self {
        Self {
            id: "weighted-line-known-sigma-v1".into(),
            x: vec![1., 2., 3., 4., 5.],
            y: vec![2.1, 3.9, 6.2, 7.8, 10.1],
            sigma: 0.5,
            parameters: vec!["a".into(), "b".into()],
            // The original case uses Scalar for x, y, a and b. Covariance is
            // physical parameter-unit products, without residual-variance scaling.
            response_unit: "1".into(),
            parameter_units: vec!["1".into(), "1".into()],
            covariance_convention: "inverse_information_known_sigma".into(),
        }
    }

    fn source(&self) -> String {
        let mut source =
            String::from("package parity { def Line { param a:Scalar=1; param b:Scalar=1; ");
        for (k, x) in self.x.iter().enumerate() {
            source.push_str(&format!("let y{k}:Scalar=a+b*{x}; "));
        }
        source.push_str("} entity kind origin provenance {attribute title:Text;} ");
        source.push_str("entity origin experiment {title=\"weighted analytic line\"} ");
        source.push_str("enum role {measured facets(measured)} ");
        source.push_str("entity kind reading {attribute value:Scalar?;attribute sigma:Scalar?;} ");
        for (k, y) in self.y.iter().enumerate() {
            let observation = format!("{:02x}", 160 + k).repeat(16);
            source.push_str(&format!(
                "@id(\"{observation}\") entity reading measured{k} provenance(experiment,role.measured) {{value={y},sigma={}}} ",
                self.sigma,
            ));
        }
        source.push('}');
        source
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Covariance {
    pub(super) parameters: Vec<String>,
    pub(super) parameter_units: Vec<String>,
    pub(super) parameter_scales: Vec<f64>,
    pub(super) coordinates: String,
    pub(super) layout: String,
    pub(super) convention: String,
    pub(super) values: Vec<f64>,
}

#[derive(Clone, Serialize)]
pub(super) struct CandidateReport {
    pub(super) complete: bool,
    pub(super) estimates: BTreeMap<String, f64>,
    pub(super) estimate_allowances: BTreeMap<String, f64>,
    pub(super) response_unit: String,
    pub(super) covariance: Covariance,
}

#[derive(Serialize)]
pub(super) struct Candidate {
    pub(super) id: String,
    pub(super) report: Option<CandidateReport>,
}

fn id(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}

// Erase futures at actual public source/preparation/execution stage boundaries.
fn attach<'a>(
    package: ModelingPackage,
    services: &'a SourceServices,
    declarations: FitDeclarations,
) -> BoxFuture<'a, Result<ModelingPackage>> {
    Box::pin(async move {
        Ok(package
            .with_fit_declarations(services, declarations)
            .await?)
    })
}

fn prepare<'a>(
    package: &'a ModelingPackage,
    services: &'a PreparationServices,
    profile: FitProfile,
    cancel: &'a CancelSource,
) -> BoxFuture<'a, Result<PreparedFit>> {
    Box::pin(async move {
        Ok(package
            .prepare_fit(
                services,
                id(158).into(),
                profile,
                Default::default(),
                Default::default(),
                cancel,
            )
            .await?)
    })
}

pub(super) fn run<'a>(
    native: &'a NativeContext,
    case: &'a LineCase,
) -> BoxFuture<'a, Result<CandidateReport>> {
    Box::pin(async move {
        let source = case.source();
        let package = native.package(&source).await?;
        let services = native.runtime.execution_services();
        let declarations = package.declarations(services.preparation()).await?;
        let line = declarations
            .iter()
            .find(|row| row.name == "Line")
            .unwrap()
            .declaration_id;
        let names = BTreeMap::from([(id(151), "a"), (id(152), "b")]);
        let parameters: Vec<_> = names
            .keys()
            .map(|parameter| {
                serde_json::json!({
                    "symbol_id": parameter, "fixed": false, "value": 1.,
                    "lower": -100., "upper": 100., "scale": 1.,
                })
            })
            .collect();
        let bindings: Vec<_> = names
            .iter()
            .map(|(parameter, name)| {
                serde_json::json!({
                    "parameter_id": parameter, "path": name,
                })
            })
            .collect();
        let observations: Vec<_> = (0..case.y.len())
            .map(|k| {
                serde_json::json!({
                    "observation_id": id(160 + u8::try_from(k).unwrap()),
                    "value_attribute": "value", "standard_deviation_attribute": "sigma",
                    "experiment_id": id(159), "output_path": format!("y{k}"),
                    "time": null, "time_basis": null, "time_unit_id": null,
                    "included": true, "importance": 1.,
                })
            })
            .collect();
        let mut fits = FitDeclarations::default();
        fits.fits.push(serde_json::from_value(serde_json::json!({
            "fit_id": id(158), "parameters": parameters,
            "experiments": [{"experiment_id": id(159), "case_id": line,
                "route": "steady", "bindings": bindings}],
            "observations": observations,
        }))?);
        let package = attach(package, &native.runtime.source_services(), fits).await?;
        let settings = SolveSettings {
            intent: SolveIntent::Optimize,
            backend: Some(Backend::Ipopt),
            ..Default::default()
        };
        let fraction = settings.numerics.engineering_relative_fraction;
        let document: FitPreparationDocument = serde_json::from_value(serde_json::json!({
            "optimizer": "native_nlp", "solver": settings,
        }))?;
        let cancel = CancelSource::new();
        let prepared = prepare(
            &package,
            services.preparation(),
            document.profile()?,
            &cancel,
        )
        .await?;
        assert_eq!(
            prepared.numerics().policy.engineering_relative_fraction,
            fraction
        );
        let handle = prepared.start(&services)?;
        let waiting: BoxFuture<'_, _> = Box::pin(handle.wait());
        let result = waiting.await?;
        assert!(
            result.usable(),
            "native fitting must publish a usable result: {:?}",
            result.report()
        );

        // Verify unit identities against the admitted physical registry before
        // giving their dimensionless convention a portable exchange spelling.
        let quantities = package.physical_context().quantities();
        let scalar = quantities.quantity_type(quantities.neutral_dimensionless().unwrap())?;
        let unit = quantities.unit(scalar.canonical_unit)?;
        assert!(unit.dimension.is_dimensionless());
        assert_eq!(unit.scale_to_canonical, 1.);
        assert_eq!(unit.offset_to_canonical, 0.);
        let expected_unit = scalar.canonical_unit.as_id();

        let rows = fit_parameters::Row::rows(&result.table("runtime.fit_parameters")?)?;
        assert_eq!(rows.len(), 2);
        let mut estimates = BTreeMap::new();
        for row in rows {
            assert_eq!(row.run_id, result.run_id);
            assert!(!row.fixed);
            assert_eq!(row.unit_id, expected_unit);
            assert_eq!(row.scale, 1.);
            let value = row.value.expect("fit estimate missing");
            assert!(value.is_finite());
            assert!(
                estimates
                    .insert(names[&row.parameter_id].into(), value)
                    .is_none()
            );
        }
        let mut estimate_allowances = BTreeMap::new();
        for row in resolved_numerics::Row::rows(&result.table("runtime.resolved_numerics")?)? {
            if row.target_kind != NumericalTarget::Variable || !names.contains_key(&row.target_id) {
                continue;
            }
            assert_eq!(row.unit_id, expected_unit);
            let engineering = row
                .engineering
                .expect("frozen engineering allowance missing");
            assert_eq!(engineering.relative_fraction, fraction);
            assert!(engineering.budget.is_finite() && engineering.budget > 0.);
            assert_eq!(engineering.budget, row.budget);
            assert!(
                estimate_allowances
                    .insert(names[&row.target_id].into(), engineering.budget)
                    .is_none()
            );
        }
        assert_eq!(estimate_allowances.len(), 2);
        let measured = fit_observations::Row::rows(&result.table("runtime.fit_observations")?)?;
        assert_eq!(measured.len(), case.y.len());
        assert_eq!(
            measured
                .iter()
                .map(|row| row.observation_id)
                .collect::<BTreeSet<_>>(),
            (160..165).map(id).collect()
        );
        assert!(measured.iter().all(|row| row.included
            && row.unit_id == expected_unit
            && row.prediction.is_some_and(f64::is_finite)));
        let covariance =
            parameter_covariances::Row::rows(&result.table("runtime.parameter_covariances")?)?;
        assert_eq!(
            covariance.len(),
            1,
            "one complete published covariance required"
        );
        let covariance = &covariance[0];
        assert_eq!(covariance.run_id, result.run_id);
        assert_eq!(
            covariance
                .parameters
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
            names.keys().copied().collect()
        );
        assert_eq!(covariance.parameters.len(), 2);
        assert_eq!(covariance.parameter_units, vec![expected_unit; 2]);
        assert_eq!(covariance.parameter_quantities, vec![scalar.id.as_id(); 2]);
        assert_eq!(covariance.parameter_scales, vec![1.; 2]);
        assert_eq!(covariance.values.len(), 4);
        assert!(covariance.values.iter().all(|value| value.is_finite()));
        Ok(CandidateReport {
            complete: true,
            estimates,
            estimate_allowances,
            response_unit: "1".into(),
            covariance: Covariance {
                parameters: covariance
                    .parameters
                    .iter()
                    .map(|parameter| names[parameter].into())
                    .collect(),
                parameter_units: vec!["1".into(); 2],
                parameter_scales: covariance.parameter_scales.clone(),
                coordinates: "physical".into(),
                layout: "row_major".into(),
                convention: "inverse_information_known_sigma".into(),
                values: covariance.values.clone(),
            },
        })
    })
}
