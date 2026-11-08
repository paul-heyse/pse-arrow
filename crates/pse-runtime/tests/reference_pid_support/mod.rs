// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored control-library PI loop, executed through public local Rust science.

use futures_util::future::BoxFuture;
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_backend_native::solve::SolveIntent;
use pse_model::generated::enums::{ModelingConformanceStatus, TrajectoryTermination};
use pse_model::reference_exchange::ReferenceMeaning;
use pse_relations::{columnar::RelationRow, generated::runtime::simulation_samples};
use pse_runtime::{
    CancelSource,
    math::settings::SolveSettings,
    reference_exchange::{
        ReferenceCaseContract, ReferenceFixedInput, ReferenceInputArtifact,
        ReferenceObservableContract,
    },
    workflow::{
        ConformanceControls, ExecutionServices, ModelingConformanceReport, ModelingPackage,
        PhysicalContext, PhysicalInspectionDocument, PhysicalInspectionQuery,
        PhysicalInspectionRequest, PhysicalQuantityDescription,
    },
};
use sha2::{Digest as _, Sha256};
use std::{collections::BTreeMap, sync::Arc};

use crate::reference_support::NativeContext;
pub(super) use crate::reference_support::{Result, root};

#[derive(Clone)]
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

#[derive(Clone)]
pub(super) struct CandidateReport {
    pub(super) complete: bool,
    pub(super) accepted: bool,
    pub(super) termination: TrajectoryTermination,
    pub(super) fixture_passed: bool,
    pub(super) samples_seconds: Vec<f64>,
    pub(super) process: Vec<f64>,
    pub(super) measurement: Vec<f64>,
}

pub(super) struct Candidate {
    pub(super) id: String,
    pub(super) report: Option<CandidateReport>,
}

pub(super) fn sample_key(role: &str, time: f64) -> String {
    format!("{role}:{:016x}", time.to_bits())
}

pub(super) struct ReferenceInputs {
    pub(super) contract: ReferenceCaseContract,
    pub(super) _owners: [Arc<datafusion::execution::memory_pool::MemoryReservation>; 2],
}

fn physical(
    context: &PhysicalContext,
    name: &str,
) -> Result<(
    PhysicalQuantityDescription,
    Arc<datafusion::execution::memory_pool::MemoryReservation>,
)> {
    let PhysicalInspectionDocument::Page {
        mut quantities,
        owner,
        ..
    } = context.describe(&PhysicalInspectionRequest {
        query: PhysicalInspectionQuery::QuantityNamed { name: name.into() },
        offset: 0,
        limit: 1,
    })?
    else {
        return Err("PID quantity description is unavailable".into());
    };
    if quantities.len() != 1 {
        return Err("PID quantity description is absent or ambiguous".into());
    }
    Ok((
        quantities.remove(0),
        owner.ok_or("PID quantity description lost its allocation owner")?,
    ))
}

fn reference_contract(
    case: &LoopCase,
    authored: &str,
    fixture: pse_model::generated::identities::DeclarationId,
    context: &PhysicalContext,
) -> Result<ReferenceInputs> {
    const ARTIFACTS: &[&str] = &[
        "packages/reference/methods/package.toml",
        "packages/reference/physical/package.toml",
        "packages/reference/process/models/control.pse",
        "packages/reference/process/package.toml",
    ];
    let capture = crate::scientific_inputs::family("parity");
    let files = capture.files();
    let artifacts = ARTIFACTS
        .iter()
        .map(|path| {
            let bytes = files
                .get(*path)
                .ok_or("original PID input artifact absent")?;
            Ok(ReferenceInputArtifact {
                path: (*path).into(),
                sha256: Sha256::digest(bytes).to_vec().into(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let (scalar, scalar_owner) = physical(context, "Scalar")?;
    let (time, time_owner) = physical(context, "Time")?;
    let fixed = |key: &str, value, physical: &PhysicalQuantityDescription| ReferenceFixedInput {
        key: key.into(),
        value,
        physical: physical.clone(),
    };
    let mut specification = vec![
        fixed("tau_seconds", case.tau_seconds, &time),
        fixed("kp", case.kp, &scalar),
        fixed("ti_seconds", case.ti_seconds, &time),
        fixed("bias", case.bias, &scalar),
        fixed("setpoint", case.setpoint, &scalar),
        fixed("initial_process", case.initial_process, &scalar),
        fixed(
            "initial_integral_seconds",
            case.initial_integral_seconds,
            &time,
        ),
        fixed("derivative_seconds", case.derivative_seconds, &time),
        fixed("lower", case.lower, &scalar),
        fixed("upper", case.upper, &scalar),
        fixed("engineering_fraction", case.engineering_fraction, &scalar),
        fixed("initial_step_seconds", case.initial_step_seconds, &time),
    ];
    specification.extend(
        case.samples_seconds
            .iter()
            .map(|value| fixed(&sample_key("time", *value), *value, &time)),
    );
    let observable =
        |key: String, physical: &PhysicalQuantityDescription| ReferenceObservableContract {
            output: pse_ids::named_id(fixture.as_id(), &key),
            key,
            physical: physical.clone(),
        };
    let mut observables = Vec::new();
    for role in ["exact_process", "petsc_process"] {
        observables.extend(
            case.samples_seconds
                .iter()
                .map(|value| observable(sample_key(role, *value), &scalar)),
        );
    }
    observables.push(observable("steady_process".into(), &scalar));
    observables.push(observable("steady_integral_seconds".into(), &time));
    let source = pse_ids::preimage::hash(&serde_json::to_vec(&(capture.identity(), authored))?);
    let configuration = pse_ids::preimage::hash(&serde_json::to_vec(
        &specification
            .iter()
            .map(|input| {
                (
                    &input.key,
                    input.value.to_bits(),
                    input.physical.quantity.id,
                )
            })
            .collect::<Vec<_>>(),
    )?);
    Ok(ReferenceInputs {
        contract: ReferenceCaseContract {
            version: Default::default(),
            case_name: case.id.clone(),
            meaning: ReferenceMeaning {
                case: fixture.as_id(),
                source,
                formulation: pse_ids::preimage::hash(authored.as_bytes()),
                physical: context.identity(),
                configuration,
            },
            observables,
            specification,
            artifacts,
        },
        _owners: [scalar_owner, time_owner],
    })
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
        let selected = crate::scientific_inputs::family("parity");
        let physical = selected
            .package("packages/reference/physical")
            .expect("PID physical package");
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
            let header = &selected
                .package(path)
                .expect("PID dependency package")
                .header;
            let version: Vec<i64> = header
                .version
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
            dependency.insert("package_id".into(), header.package_id.to_string().into());
            dependency.insert("version_req".into(), requirement.into());
            dependencies.push(dependency.into());
        }
        manifest["package"]["dependencies"] = toml::Value::Array(dependencies);
        let local = BTreeMap::from([
            (
                "package.toml".into(),
                toml::to_string(&manifest)?.into_bytes(),
            ),
            ("models/loop.pse".into(), source.into_bytes()),
        ]);
        let bundles =
            std::iter::once(&local).chain(selected.packages().map(|(_, package)| &package.files));
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
            OwnedDocumentSet::try_from_bundles(vec![load(&physical.files)?], &pool, &token)?;
        let physical = admit_physical(context, &physical_documents, &token).await?;
        let documents = OwnedDocumentSet::try_from_bundles(
            bundles.map(load).collect::<Result<Vec<_>>>()?,
            &pool,
            &token,
        )?;
        modeling(context, &documents, physical, &CancelSource::new()).await
    })
}

pub(super) fn run<'a>(
    context: &'a NativeContext,
    case: &'a LoopCase,
) -> BoxFuture<'a, Result<(CandidateReport, ReferenceInputs)>> {
    Box::pin(async move {
        let authored = case.source();
        let package = package(context, authored.clone()).await?;
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
        let input = reference_contract(case, &authored, fixture, package.physical_context())?;
        let model = &trajectory.prepared().model().compiled().model;
        let scalar = &input.contract.observables[0].physical;
        let mut original = |path: &str| -> Result<Vec<f64>> {
            let mut symbols = model.symbols.values().filter(|symbol| {
                let lineage = symbol.lineage.path.as_str();
                lineage == path
                    || lineage
                        .strip_suffix(path)
                        .is_some_and(|prefix| prefix.ends_with('.'))
            });
            let symbol = symbols.next().ok_or("PID authored output is absent")?;
            assert!(
                symbols.next().is_none(),
                "PID authored output path is ambiguous: {path}"
            );
            let mut rows = series
                .remove(&symbol.id)
                .ok_or("PID authored output series is absent")?;
            rows.sort_by(|a, b| a.time.total_cmp(&b.time));
            assert_eq!(rows.len(), case.samples_seconds.len());
            for (row, time) in rows.iter().zip(&case.samples_seconds) {
                assert_eq!(row.time.to_bits(), time.to_bits());
                assert_eq!(row.quantity_id, scalar.quantity.id.as_id());
                assert_eq!(row.unit_id, scalar.representation_unit.id.as_id());
                assert!(row.value.is_finite());
            }
            Ok(rows.into_iter().map(|row| row.value).collect())
        };
        let process = original("root.y")?;
        let measurement = original("root.regulator.measurement")?;
        for (process, measured) in process.iter().zip(&measurement) {
            assert!((process - measured).abs() <= case.engineering_fraction);
        }
        Ok((
            CandidateReport {
                complete: trajectory.checks_complete(),
                accepted: trajectory.accepted(),
                termination: trajectory.report().termination,
                fixture_passed: true,
                samples_seconds: case.samples_seconds.clone(),
                process,
                measurement,
            },
            input,
        ))
    })
}
