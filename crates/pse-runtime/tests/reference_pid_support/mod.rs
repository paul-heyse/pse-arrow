// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored control-library PI loop, executed through public local Rust science.

use futures_util::future::BoxFuture;
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_backend_native::solve::SolveIntent;
use pse_model::generated::enums::{ModelingConformanceStatus, TrajectoryTermination};
use pse_relations::{columnar::RelationRow, generated::runtime::simulation_samples};
use pse_runtime::{
    CancelSource,
    math::settings::SolveSettings,
    workflow::{
        ConformanceControls, ExecutionServices, ModelingConformanceReport, ModelingPackage,
        PhysicalContext,
    },
};
use serde::Serialize;
use std::collections::BTreeMap;

use crate::reference_support::NativeContext;
pub(super) use crate::reference_support::{Result, root};

#[derive(Clone, Serialize)]
pub(super) struct LoopCase {
    pub(super) id: String,
    pub(super) tau_seconds: f64,
    pub(super) kp: f64,
    pub(super) ti_seconds: f64,
    pub(super) bias: f64,
    pub(super) setpoint: f64,
    pub(super) initial_process: f64,
    pub(super) initial_integral_seconds: f64,
    pub(super) derivative_seconds: f64,
    pub(super) lower: f64,
    pub(super) upper: f64,
    pub(super) samples_seconds: Vec<f64>,
    pub(super) engineering_fraction: f64,
    pub(super) initial_step_seconds: f64,
}

impl LoopCase {
    pub(super) fn original() -> Self {
        Self {
            id: "closed-loop-pi-first-order-v1".into(),
            tau_seconds: 5.,
            kp: 0.5,
            ti_seconds: 2.,
            bias: 0.5,
            setpoint: 1.,
            initial_process: 0.5,
            initial_integral_seconds: 0.,
            derivative_seconds: 0.,
            lower: -1000.,
            upper: 1000.,
            samples_seconds: vec![0., 1., 2., 5., 10., 20.],
            // Same owner as the old generated Python NumericalPolicy default.
            engineering_fraction: pse_model::numerics::NumericalPolicy::default()
                .engineering_relative_fraction,
            initial_step_seconds: 1e-4,
        }
    }

    fn source(&self) -> String {
        let samples = self
            .samples_seconds
            .iter()
            .map(|t| format!("{t}{{s}}"))
            .collect::<Vec<_>>()
            .join(",");
        let end = self.samples_seconds.last().unwrap();
        format!(
            r#"package pid_parity {{
 use control @"1.0.0";
 def Loop {{
  domain t:Time from 0{{s}} to {end}{{s}};
  discretize grid on t using integrated(elements=1,order=1);
  child regulator:control.PID=control.PID(timeline=t,first=0{{s}},Kp={kp},Ti={ti}{{s}},
   Td={td}{{s}},bias={bias},lower={lower},upper={upper},initial_integral={integral}{{s}});
  var y[i in t]:Scalar;
  eq plant[i in t]:d(y[i])/di==(regulator.output[i]-y[i])/{tau}{{s}};
  eq plant_initial:y[0{{s}}]=={start};
  eq measured[i in t]:regulator.measurement[i]==y[i];
  eq target[i in t]:regulator.setpoint[i]=={setpoint};
  annotation start y({start});
 }}
 test pi_loop fixture {{dof 0; route integrated; procedure integrate;
  integrate samples({samples}) relative({accuracy}) normalized_absolute({accuracy}) step({step}{{s}});
 }} {{ child root:Loop=Loop(); }}
}}"#,
            kp = self.kp,
            ti = self.ti_seconds,
            td = self.derivative_seconds,
            bias = self.bias,
            lower = self.lower,
            upper = self.upper,
            integral = self.initial_integral_seconds,
            tau = self.tau_seconds,
            start = self.initial_process,
            setpoint = self.setpoint,
            accuracy = self.engineering_fraction,
            step = self.initial_step_seconds
        )
    }
}

#[derive(Clone, Serialize)]
pub(super) struct CandidateReport {
    pub(super) complete: bool,
    pub(super) accepted: bool,
    pub(super) termination: String,
    pub(super) fixture_passed: bool,
    pub(super) samples_seconds: Vec<f64>,
    pub(super) process: Vec<f64>,
    pub(super) measurement: Vec<f64>,
}

#[derive(Serialize)]
pub(super) struct Candidate {
    pub(super) id: String,
    pub(super) report: Option<CandidateReport>,
}

fn admit_physical<'a>(
    context: &'a NativeContext,
    documents: &'a OwnedDocumentSet,
    token: &'a pse_columnar::CancellationToken,
) -> BoxFuture<'a, Result<PhysicalContext>> {
    Box::pin(async move {
        Ok(context
            .runtime
            .physical_from_documents(documents, token)
            .await?)
    })
}

fn modeling<'a>(
    context: &'a NativeContext,
    documents: &'a OwnedDocumentSet,
    physical: PhysicalContext,
    cancel: &'a CancelSource,
) -> BoxFuture<'a, Result<ModelingPackage>> {
    Box::pin(async move {
        Ok(context
            .runtime
            .modeling_from_documents(documents, physical, cancel)
            .await?)
    })
}

fn conform<'a>(
    package: &'a ModelingPackage,
    services: &'a ExecutionServices,
    fixture: pse_model::generated::identities::DeclarationId,
    cancel: &'a CancelSource,
) -> BoxFuture<'a, Result<ModelingConformanceReport>> {
    Box::pin(async move {
        Ok(package
            .conform_request(
                services,
                &Default::default(),
                SolveSettings {
                    intent: SolveIntent::Root,
                    ..Default::default()
                },
                ConformanceControls {
                    fixtures: Some(vec![fixture]),
                    ..Default::default()
                },
                cancel,
            )
            .await?)
    })
}

fn package(context: &NativeContext, source: String) -> BoxFuture<'_, Result<ModelingPackage>> {
    Box::pin(async move {
        let physical = crate::scientific_inputs::package_documents("packages/reference/physical");
        let mut manifest: toml::Value = toml::from_str(&std::fs::read_to_string(
            root().join("tests/fixtures/packages/minimal_explicit/package.toml"),
        )?)?;
        manifest["package"]["id_policy"] = "named".into();
        manifest["package"]["name"] = "pid.reference".into();
        // Derive dependency identities and versions from the captured declarations.
        let mut dependencies = Vec::new();
        for path in [
            "packages/reference/physical",
            "packages/reference/process",
            "packages/reference/methods",
        ] {
            let documents = crate::scientific_inputs::package_documents(path);
            let header: toml::Value =
                toml::from_str(std::str::from_utf8(&documents["package.toml"])?)?;
            let version: Vec<i64> = header["package"]["version"]
                .as_str()
                .unwrap()
                .split('.')
                .map(|part| part.parse().unwrap())
                .collect();
            assert_eq!(version.len(), 3);
            let mut requirement = toml::Table::new();
            requirement.insert("operator".into(), "exact".into());
            for (key, value) in ["major", "minor", "patch"].into_iter().zip(version) {
                requirement.insert(key.into(), value.into());
            }
            let mut dependency = toml::Table::new();
            dependency.insert("package_id".into(), header["package"]["package_id"].clone());
            dependency.insert("version_req".into(), requirement.into());
            dependencies.push(dependency.into());
        }
        manifest["package"]["dependencies"] = toml::Value::Array(dependencies);
        let mut bundles = vec![BTreeMap::from([
            (
                "package.toml".into(),
                toml::to_string(&manifest)?.into_bytes(),
            ),
            ("models/loop.pse".into(), source.into_bytes()),
        ])];
        for (_, documents) in crate::scientific_inputs::family_packages("parity") {
            bundles.push(documents);
        }
        let pool = context.shared.pool();
        let token = pse_columnar::CancellationToken::new();
        let validation = context.runtime.validation_context();
        let load = |documents: &BTreeMap<String, Vec<u8>>| -> Result<_> {
            Ok(load_package_documents_owned(
                documents,
                context.runtime.registry(),
                Default::default(),
                &pool,
                &token,
                &validation,
            )?)
        };
        let physical_documents =
            OwnedDocumentSet::try_from_bundles(vec![load(&physical)?], &pool, &token)?;
        let physical = admit_physical(context, &physical_documents, &token).await?;
        let documents = OwnedDocumentSet::try_from_bundles(
            bundles.iter().map(load).collect::<Result<Vec<_>>>()?,
            &pool,
            &token,
        )?;
        modeling(context, &documents, physical, &CancelSource::new()).await
    })
}

pub(super) fn run<'a>(
    context: &'a NativeContext,
    case: &'a LoopCase,
) -> BoxFuture<'a, Result<CandidateReport>> {
    Box::pin(async move {
        let package = package(context, case.source()).await?;
        let services = context.runtime.execution_services();
        let fixture = package
            .declarations(services.preparation())
            .await?
            .iter()
            .find(|declaration| declaration.name == "pi_loop")
            .unwrap()
            .declaration_id;
        let cancel = CancelSource::new();
        let conformance = conform(&package, &services, fixture, &cancel).await?;
        assert!(conformance.complete);
        assert_eq!(
            conformance.fixture_statuses[&fixture],
            ModelingConformanceStatus::Passed,
            "{:?}",
            conformance.failures
        );
        let trajectory = &conformance.trajectories[&fixture];
        assert!(trajectory.accepted(), "{:?}", trajectory.validation_error());
        assert_eq!(
            trajectory.report().termination,
            TrajectoryTermination::Completed
        );
        let rows = simulation_samples::Row::rows(&trajectory.table("runtime.simulation_samples")?)?;
        let mut series: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for row in rows {
            series.entry(row.symbol_id).or_default().push(row);
        }
        // Preserve the original independent trajectory selection: plant state and
        // controller measurement are the only series starting at the process value.
        let mut matching = Vec::new();
        for mut rows in series.into_values() {
            rows.sort_by(|a, b| a.time.total_cmp(&b.time));
            if rows
                .first()
                .is_some_and(|row| row.time == 0. && row.value == case.initial_process)
            {
                assert_eq!(rows.len(), case.samples_seconds.len());
                for (row, time) in rows.iter().zip(&case.samples_seconds) {
                    assert_eq!(row.time, *time);
                    assert_eq!(row.quantity_id, rows[0].quantity_id);
                    assert_eq!(row.unit_id, rows[0].unit_id);
                    assert!(row.value.is_finite());
                }
                matching.push(rows);
            }
        }
        assert_eq!(
            matching.len(),
            2,
            "plant and measurement must both be retained"
        );
        assert_eq!(matching[0][0].quantity_id, matching[1][0].quantity_id);
        assert_eq!(matching[0][0].unit_id, matching[1][0].unit_id);
        let values: Vec<Vec<f64>> = matching
            .iter()
            .map(|rows| rows.iter().map(|row| row.value).collect())
            .collect();
        for (process, measured) in values[0].iter().zip(&values[1]) {
            assert!((process - measured).abs() <= case.engineering_fraction);
        }
        Ok(CandidateReport {
            complete: trajectory.checks_complete(),
            accepted: trajectory.accepted(),
            termination: trajectory.report().termination.as_str().into(),
            fixture_passed: true,
            samples_seconds: case.samples_seconds.clone(),
            process: values[0].clone(),
            measurement: values[1].clone(),
        })
    })
}
