// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Paired frozen-feed measurements over the existing native study owner.
use super::conformance::documents;
use anyhow::{Context, Result, ensure};
use datafusion::arrow::{
    array::{Array, Float64Array, Int64Array},
    record_batch::RecordBatch,
};
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_model::{
    diagnostic::BoundaryDiagnostic,
    generated::{
        enums::{ModelingAnalysisRoute, NativeBackend, NativeSolveIntent, NativeTermination},
        identities::DeclarationId,
    },
    study::{OccurrenceKey, StartPolicy},
};
use pse_relations::columnar::FieldCheckedBatch;
use pse_runtime::{
    CancelSource,
    math::settings::SolveSettings,
    workflow::{
        BindingAssignment, BindingQuantity, BindingTarget, CaseOperation, Completion,
        ConformanceManifest, ConformanceRun, LocalStudy, ModelingPackage, OperationRequest,
        PhysicalContext, PointOverlay, Runtime, StudyInputs, StudyOccurrenceResult,
        StudyOutputDemand, StudyPoint, StudyPointPolicy, StudyReport, TerminationDetail,
        WorkflowError,
    },
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// Frozen paired-feed population; rows are never filtered by scientific outcomes.
pub const FEED_COUNT: usize = 200;
/// Independent value-only occurrences of the selected value-sweep case.
pub const SWEEP_COUNT: usize = 1000;

/// One original feed coordinate with the declared K, Pa and mole-fraction units.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Feed {
    /// Original sequential occurrence coordinate.
    pub index: usize,
    /// Absolute temperature in kelvins.
    pub temperature: f64,
    /// Absolute pressure in pascals.
    pub pressure: f64,
    /// Benzene mole fraction; the toluene fraction is its complement.
    pub benzene: f64,
}

/// One admitted complete file; identity and decoding consume the same frozen bytes.
#[derive(Debug)]
pub struct Feeds {
    rows: Vec<Feed>,
    /// SHA-256 of the actual decoded frozen file bytes.
    pub sha256: String,
    /// Original generator metadata carried by that file.
    pub generator: String,
}
impl Feeds {
    /// Admit the complete ordered 200-feed Parquet population before execution.
    pub fn load(path: &Path) -> Result<Self> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take((8 << 20) + 1)
            .read_to_end(&mut bytes)?;
        ensure!(bytes.len() <= 8 << 20, "campaign feed file exceeds 8 MiB");
        let sha256 = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let builder = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(
            bytes::Bytes::from(bytes),
        )?;
        let schema = builder.schema();
        ensure!(
            schema.fields().len() == 4,
            "campaign requires exactly four declared feed fields"
        );
        let generator = schema
            .metadata()
            .get("generator")
            .context("campaign feed generator metadata absent")?
            .clone();
        ensure!(
            !generator.is_empty(),
            "campaign feed generator metadata empty"
        );
        for (name, unit) in [("temperature", "K"), ("pressure", "Pa"), ("benzene", "1")] {
            ensure!(
                schema
                    .field_with_name(name)?
                    .metadata()
                    .get("unit")
                    .map(String::as_str)
                    == Some(unit),
                "campaign feed {name} requires unit {unit}"
            );
        }
        let mut rows = Vec::with_capacity(FEED_COUNT);
        for batch in builder.with_batch_size(FEED_COUNT + 1).build()? {
            let batch = batch?;
            let index = batch
                .column_by_name("index")
                .and_then(|array| array.as_any().downcast_ref::<Int64Array>())
                .context("campaign feed index must be int64")?;
            let values = ["temperature", "pressure", "benzene"].map(|name| {
                batch
                    .column_by_name(name)
                    .and_then(|array| array.as_any().downcast_ref::<Float64Array>())
                    .context("campaign feed coordinates must be float64")
            });
            let [temperature, pressure, benzene] = values;
            let (temperature, pressure, benzene) = (temperature?, pressure?, benzene?);
            ensure!(
                [
                    index.null_count(),
                    temperature.null_count(),
                    pressure.null_count(),
                    benzene.null_count()
                ]
                .iter()
                .all(|count| *count == 0),
                "campaign feed coordinates must be complete"
            );
            for row in 0..batch.num_rows() {
                ensure!(
                    rows.len() < FEED_COUNT && index.value(row) == i64::try_from(rows.len())?,
                    "complete ordered frozen 200-feed population required"
                );
                let feed = Feed {
                    index: rows.len(),
                    temperature: temperature.value(row),
                    pressure: pressure.value(row),
                    benzene: benzene.value(row),
                };
                ensure!(
                    (350.0..=400.0).contains(&feed.temperature)
                        && (80000.0..=120000.0).contains(&feed.pressure)
                        && (0.05..=0.95).contains(&feed.benzene),
                    "campaign feed outside declared physical sampling bounds"
                );
                rows.push(feed);
            }
        }
        ensure!(
            rows.len() == FEED_COUNT,
            "complete ordered frozen 200-feed population required"
        );
        Ok(Self {
            rows,
            sha256,
            generator,
        })
    }
    /// Original coordinates in their admitted occurrence order.
    pub fn rows(&self) -> &[Feed] {
        &self.rows
    }
}

/// Selected existing campaign; paired flashes always use the same admitted feeds.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Both flash formulations over the same 200 feeds.
    Flash,
    /// One independent thousand-point temperature sweep.
    Study,
}
impl Phase {
    /// Selected authored cases, with no automatic phase suppression after failure.
    pub fn cases(self) -> &'static [&'static str] {
        match self {
            Self::Flash => &["measurement_smooth", "measurement_nested"],
            Self::Study => &["measurement_value_sweep"],
        }
    }
    /// Entire original occurrence population for each selected case.
    pub fn count(self) -> usize {
        match self {
            Self::Flash => FEED_COUNT,
            Self::Study => SWEEP_COUNT,
        }
    }
}
/// Explicit execution selection; recording requires a caller-configured canonical runtime.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Execution {
    /// Explicitly ephemeral native study execution.
    Local,
    /// Original recording handoff and managed canonical execution.
    Recorded,
}

/// Admitted source selection and complete immutable feed inventory.
#[derive(Debug)]
pub struct Campaign {
    parent: PathBuf,
    run: ConformanceRun,
    /// Complete original paired feed population.
    pub feeds: Feeds,
    /// Admitted native manifest resource and preparation controls.
    pub settings: pse_runtime::workflow::RunSettings,
    /// Actual per-occurrence native solve request.
    pub solve: SolveSettings,
}
impl Campaign {
    /// Select one manifest run through its native owner, never a Python DTO or parser.
    pub fn load(manifest: &Path, selected_run: &str, feeds: &Path) -> Result<Self> {
        let path = manifest.canonicalize()?;
        let manifest = ConformanceManifest::load(&path)?;
        let run = manifest
            .runs
            .into_iter()
            .find(|run| run.name == selected_run)
            .context("campaign conformance run absent")?;
        ensure!(
            manifest.settings.preparation.is_some(),
            "campaign requires explicit native preparation settings"
        );
        let mut solve = SolveSettings {
            backend: Some(NativeBackend::Ipopt),
            intent: NativeSolveIntent::FeasiblePoint,
            ..Default::default()
        };
        solve.controls.time_limit = Duration::from_secs(600);
        solve.controls.threads = 1;
        solve.controls.history = 4096;
        Ok(Self {
            parent: path
                .parent()
                .context("campaign manifest parent")?
                .to_owned(),
            run,
            feeds: Feeds::load(feeds)?,
            settings: manifest.settings,
            solve,
        })
    }
    /// Admit sources using the caller's original registry, validation and allocation owner.
    pub async fn package(
        &self,
        runtime: &Runtime,
        cancel: &CancelSource,
    ) -> Result<(PhysicalContext, ModelingPackage)> {
        let source = runtime.source_services();
        let pool = source.preparation().pool();
        let validation = runtime.validation_context();
        let token = pse_columnar::CancellationToken::new();
        let load = |path: &Path| -> Result<_> {
            Ok(load_package_documents_owned(
                &documents(path)?,
                runtime.registry(),
                Default::default(),
                &pool,
                &token,
                &validation,
            )?)
        };
        let physical = OwnedDocumentSet::try_from_bundles(
            vec![load(&self.parent.join(&self.run.physical))?],
            &pool,
            &token,
        )?;
        let physical = runtime.physical_from_documents(&physical, &token).await?;
        let bundles = std::iter::once(&self.run.package)
            .chain(&self.run.dependencies)
            .map(|path| load(&self.parent.join(path)))
            .collect::<Result<Vec<_>>>()?;
        let modeling = OwnedDocumentSet::try_from_bundles(bundles, &pool, &token)?;
        let package = runtime
            .modeling_from_documents(&modeling, physical.clone(), cancel)
            .await?;
        Ok((physical, package))
    }
    /// Construct independent, full-detail occurrences; all assignments use native physical IDs.
    pub fn points(
        &self,
        phase: Phase,
        case: DeclarationId,
        physical: &PhysicalContext,
    ) -> Result<Vec<StudyPoint>> {
        let preparation = self
            .settings
            .preparation
            .as_ref()
            .context("campaign preparation settings absent")?;
        let assignment = |target: &str, magnitude: f64, quantity: &str, unit: &str| -> Result<_> {
            Ok(BindingAssignment {
                target: BindingTarget::Path(target.into()),
                value: BindingQuantity {
                    magnitude: pse_model::scalars::FiniteBound::try_new(magnitude)?,
                    quantity: physical.resolve_quantity(quantity)?,
                    unit: physical.resolve_unit(unit)?,
                },
            })
        };
        (0..phase.count())
            .map(|index| {
                let assignments = match phase {
                    Phase::Flash => {
                        let row = &self.feeds.rows[index];
                        vec![
                            assignment("root.inlet.T", row.temperature, "Temperature", "K")?,
                            assignment("root.inlet.pressure", row.pressure, "Pressure", "Pa")?,
                            assignment(
                                "root.inlet.z[chem.benzene]",
                                row.benzene,
                                "MoleFraction",
                                "1",
                            )?,
                            assignment(
                                "root.inlet.z[chem.toluene]",
                                1.0 - row.benzene,
                                "MoleFraction",
                                "1",
                            )?,
                        ]
                    }
                    Phase::Study => vec![assignment(
                        "root.inlet.T",
                        sweep_temperature(index),
                        "Temperature",
                        "K",
                    )?],
                };
                Ok(StudyPoint {
                    operation: OperationRequest::DeclaredCase(CaseOperation {
                        case,
                        route: ModelingAnalysisRoute::Steady,
                        settings: self.solve.clone(),
                    }),
                    preparation: preparation.clone(),
                    overlay: PointOverlay { assignments },
                    policy: StudyPointPolicy {
                        key: OccurrenceKey(u32::try_from(index)?),
                        dependencies: vec![],
                        start: StartPolicy::Fresh,
                        attempt_limit: 1,
                    },
                })
            })
            .collect()
    }
    /// Execute the selected complete populations. Every phase retains its own original
    /// report or refusal; a failed first phase never filters the paired second phase.
    /// Runtime construction, managed-worker attachment and actual shutdown remain caller-owned.
    pub async fn run(
        &self,
        runtime: &Runtime,
        phase: Phase,
        execution: Execution,
        cancel: &CancelSource,
    ) -> Result<CampaignReport> {
        let (physical, package) = self.package(runtime, cancel).await?;
        let services = runtime.execution_services();
        let declarations = package.declarations(services.preparation()).await?;
        let mut phases = Vec::new();
        for name in phase.cases() {
            let started = Instant::now();
            let run = async {
                let ids: Vec<_> = declarations
                    .iter()
                    .filter(|row| row.name == *name)
                    .map(|row| row.declaration_id)
                    .collect();
                ensure!(ids.len() == 1, "campaign requires one authored {name} case");
                let points = self.points(phase, ids[0], &physical)?;
                let local: LocalStudy = package
                    .admit_study(
                        &services,
                        &StudyInputs::Explicit(points),
                        &StudyOutputDemand::Full {},
                        cancel,
                    )
                    .await?;
                let report = match execution {
                    Execution::Local => local.execute(&services, phase.count(), cancel).await?,
                    Execution::Recorded => {
                        let recorded = runtime.record_study(&local, cancel).await?;
                        runtime
                            .execute_study(&recorded, phase.count(), cancel)
                            .await?
                    }
                };
                Ok::<_, anyhow::Error>(report)
            }
            .await;
            let seconds = started.elapsed().as_secs_f64();
            phases.push(match run {
                Ok(report) => PhaseResult::Completed(Box::new(
                    FrozenPhase::capture(name, report, phase, &self.feeds, seconds).await,
                )),
                Err(error) => PhaseResult::Refused {
                    name: (*name).into(),
                    seconds,
                    error,
                },
            });
        }
        Ok(CampaignReport {
            phase,
            execution,
            source: self.run.clone(),
            settings: self.settings.clone(),
            solve: self.solve.clone(),
            resources: None,
            feed_sha256: self.feeds.sha256.clone(),
            feed_generator: self.feeds.generator.clone(),
            phases,
        })
    }
}

fn sweep_temperature(index: usize) -> f64 {
    360.0 + 10.0 * index as f64 / (SWEEP_COUNT - 1) as f64
}

/// Original failure detail, including structured diagnostic when the native owner has one.
#[derive(Debug, Serialize)]
pub struct Failure {
    /// Original cause text.
    pub message: String,
    /// Native structured cause when its owner supplies one.
    pub diagnostic: Option<BoundaryDiagnostic>,
}
impl Failure {
    fn workflow(error: &WorkflowError) -> Self {
        Self {
            message: error.to_string(),
            diagnostic: Some(error.boundary_diagnostic()),
        }
    }
}
/// Feed attribution for every occurrence, including failed and unattempted points.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Coordinate {
    /// Exact original frozen flash coordinate.
    Flash(Feed),
    /// Original value-only temperature coordinate.
    Sweep {
        /// Absolute temperature in kelvins.
        temperature: f64,
    },
}
/// Original per-occurrence outcome; absence of completion or tables remains explicit.
#[derive(Debug, Serialize)]
pub struct Occurrence {
    /// Original ordinal, including failed or unattempted occurrences.
    pub index: usize,
    /// Original physical input coordinate.
    pub feed: Coordinate,
    /// Original operation-owned scientific usability.
    pub usable: bool,
    /// Every observed native solve succeeded or was acceptable, with nonempty coverage.
    pub native_success: bool,
    /// Original lifecycle, scientific permission, starts and attempt history.
    pub outcome: pse_model::study::PointOutcome,
    /// Original completion, when available.
    pub completion: Option<Completion>,
    /// Original retained terminal cause, when execution was recorded.
    pub termination: Option<TerminationDetail>,
    /// Explicit failures to obtain requested evidence, independently of science.
    pub unavailable: BTreeMap<String, Failure>,
    /// Retained exact table names are encoded separately as Arrow files, without JSON scalar coercion.
    #[serde(skip)]
    pub tables: BTreeMap<String, FrozenTable>,
}
/// Exact relation schema and all selected checked batches, including an empty relation.
#[derive(Debug)]
pub struct FrozenTable {
    /// Exact original native relation schema, even when no rows exist.
    pub schema: datafusion::arrow::datatypes::SchemaRef,
    /// All selected original checked batches with allocation custody.
    pub batches: Vec<FieldCheckedBatch>,
}
/// An actual original study report and frozen exports that survive caller shutdown.
#[derive(Debug)]
pub struct FrozenPhase {
    /// Authored campaign case.
    pub name: String,
    /// Actual caller-observed execution duration, excluding export.
    pub seconds: f64,
    /// Original complete native study owner.
    pub report: StudyReport,
    /// Original ordered occurrences, including every failure.
    pub occurrences: Vec<Occurrence>,
    /// Frozen original outcome relation or its original projection failure.
    pub table: Result<FieldCheckedBatch, WorkflowError>,
    /// Frozen original findings relation or its original projection failure.
    pub findings: Result<FieldCheckedBatch, WorkflowError>,
}
impl FrozenPhase {
    async fn capture(
        name: &str,
        report: StudyReport,
        phase: Phase,
        feeds: &Feeds,
        seconds: f64,
    ) -> Self {
        let table = report.table();
        let findings = report.findings_table();
        let mut occurrences = Vec::with_capacity(report.outcomes.len());
        for (index, outcome) in report.outcomes.iter().enumerate() {
            let mut observation = Occurrence {
                index,
                feed: match phase {
                    Phase::Flash => Coordinate::Flash(feeds.rows[index].clone()),
                    Phase::Study => Coordinate::Sweep {
                        temperature: sweep_temperature(index),
                    },
                },
                usable: outcome.scientific.usable,
                native_success: false,
                outcome: outcome.clone(),
                completion: None,
                termination: None,
                unavailable: BTreeMap::new(),
                tables: BTreeMap::new(),
            };
            if let Some(result) = report.results.get(index).and_then(Option::as_ref) {
                match result {
                    StudyOccurrenceResult::Retained { run, attempt, .. } => {
                        let retained = report.services().retained_data_services();
                        match retained.stored_completion(run, attempt).await {
                            Ok(stored) => {
                                observation.completion = stored.completion.clone();
                                observation.termination = Some(stored.termination.clone());
                            }
                            Err(error) => {
                                observation
                                    .unavailable
                                    .insert("completion".into(), Failure::workflow(&error));
                            }
                        }
                        for name in ["runtime.solve_runs", "runtime.solve_metrics"] {
                            let read = async {
                                let mut reader = retained
                                    .results(
                                        run,
                                        attempt,
                                        name,
                                        0,
                                        u64::MAX,
                                        pse_columnar::CancellationToken::new(),
                                    )
                                    .await?;
                                let schema = reader.schema();
                                let mut batches = Vec::new();
                                while let Some(batch) = reader.next_relation_batch().await? {
                                    batches.push(batch);
                                }
                                Ok::<_, WorkflowError>(FrozenTable { schema, batches })
                            }
                            .await;
                            match read {
                                Ok(batches) => {
                                    observation.tables.insert(name.into(), batches);
                                }
                                Err(error) => {
                                    observation
                                        .unavailable
                                        .insert(name.into(), Failure::workflow(&error));
                                }
                            }
                        }
                    }
                    StudyOccurrenceResult::Ephemeral(result) => {
                        match result.completion() {
                            Ok(completion) => observation.completion = Some(completion.clone()),
                            Err(error) => {
                                observation
                                    .unavailable
                                    .insert("completion".into(), Failure::workflow(error));
                            }
                        }
                        for name in ["runtime.solve_runs", "runtime.solve_metrics"] {
                            match result.table(name) {
                                Ok(table) => {
                                    observation.tables.insert(
                                        name.into(),
                                        FrozenTable {
                                            schema: table.batch().schema(),
                                            batches: vec![table],
                                        },
                                    );
                                }
                                Err(error) => {
                                    observation
                                        .unavailable
                                        .insert(name.into(), Failure::workflow(&error));
                                }
                            }
                        }
                    }
                    StudyOccurrenceResult::Compact(_) => {
                        observation.unavailable.insert("full_result".into(), Failure { message: "campaign selected full occurrence output but received compact output".into(), diagnostic: None });
                    }
                }
            }
            observation.native_success =
                observation.completion.as_ref().is_some_and(|completion| {
                    !completion.solves.is_empty()
                        && completion.solves.iter().all(|solve| {
                            matches!(
                                solve.termination,
                                Some(NativeTermination::Success | NativeTermination::Acceptable)
                            )
                        })
                });
            occurrences.push(observation);
        }
        Self {
            name: name.into(),
            seconds,
            report,
            occurrences,
            table,
            findings,
        }
    }
}
/// One selected phase is retained even when it refuses before producing a study.
#[derive(Debug)]
pub enum PhaseResult {
    /// Actual native study completion, independently of scientific success.
    Completed(Box<FrozenPhase>),
    /// Setup or admission refused before a study report existed.
    Refused {
        /// Original selected authored case.
        name: String,
        /// Actual caller-observed duration before refusal.
        seconds: f64,
        /// Original cause and its typed source chain.
        error: anyhow::Error,
    },
}
/// Complete selected campaign inventory. Scientific success and execution are separate facts.
#[derive(Debug)]
pub struct CampaignReport {
    /// Complete selected case population.
    pub phase: Phase,
    /// Actual requested native execution mechanism.
    pub execution: Execution,
    /// Original native manifest run and exact preparation/resource selection.
    pub source: ConformanceRun,
    /// Exact original resource and preparation controls.
    pub settings: pse_runtime::workflow::RunSettings,
    /// Actual campaign solve request, independently of conformance fixture defaults.
    pub solve: SolveSettings,
    /// Optional observation supplied by the caller's actual shared runtime owner.
    pub resources: Option<pse_runtime::ResourceReport>,
    /// Identity of the exact decoded feed bytes.
    pub feed_sha256: String,
    /// Original feed generator metadata.
    pub feed_generator: String,
    /// Every selected phase in original authored order.
    pub phases: Vec<PhaseResult>,
}
impl CampaignReport {
    /// Attach a real caller observation. These shared-runtime and process-lifetime
    /// high-water marks are neither per-operation peaks nor primary-worker measurements.
    pub fn with_resources(mut self, resources: pse_runtime::ResourceReport) -> Self {
        self.resources = Some(resources);
        self
    }
    /// Complete scientific and evidence availability across every selected occurrence.
    pub fn complete_success(&self) -> bool {
        self.phases.len() == self.phase.cases().len()
            && self.phases.iter().all(|phase| match phase {
                PhaseResult::Completed(phase) => {
                    phase.occurrences.len() == self.phase.count()
                        && phase.table.is_ok()
                        && phase.findings.is_ok()
                        && phase.occurrences.iter().all(|point| {
                            point.usable && point.native_success && point.unavailable.is_empty()
                        })
                }
                PhaseResult::Refused { .. } => false,
            })
    }
    /// Publish frozen owned reports to a fresh directory. Failed science is exported
    /// with every original occurrence; it never becomes a filtered passing population.
    pub fn export(&self, directory: &Path) -> Result<()> {
        std::fs::create_dir(directory)?;
        let mut summaries = Vec::new();
        for phase in &self.phases {
            match phase {
                PhaseResult::Refused { name, seconds, error } => summaries.push(serde_json::json!({"case":name,"seconds":seconds,"refusal":format!("{error:#}"),"diagnostic":error.downcast_ref::<WorkflowError>().map(WorkflowError::boundary_diagnostic)})),
                PhaseResult::Completed(phase) => {
                    let mut records = std::io::BufWriter::new(std::fs::OpenOptions::new().create_new(true).write(true).open(directory.join(format!("{}.jsonl",phase.name)))?);
                    for point in &phase.occurrences {
                        let mut record = serde_json::to_value(point)?;
                        let mut files = BTreeMap::new();
                        for (name, table) in &point.tables {
                            let filename = format!("{}.{}.{}.arrow",phase.name,point.index,name);
                            write_ipc(&directory.join(&filename), table.schema.as_ref(), table.batches.iter().map(FieldCheckedBatch::batch))?;
                            files.insert(name, filename);
                        }
                        record["tables"] = serde_json::to_value(files)?;
                        serde_json::to_writer(&mut records, &record)?; records.write_all(b"\n")?;
                    }
                    records.flush()?;
                    if let Ok(table) = &phase.table { let path = directory.join(format!("{}.parquet",phase.name)); let mut writer = parquet::arrow::ArrowWriter::try_new(std::fs::OpenOptions::new().create_new(true).write(true).open(path)?, table.batch().schema(), None)?; writer.write(table.batch())?; writer.close()?; }
                    if let Ok(findings) = &phase.findings { write_ipc(&directory.join(format!("{}.findings.arrow",phase.name)), findings.batch().schema().as_ref(), [findings.batch()])?; }
                    summaries.push(serde_json::json!({"case":phase.name,"run_id":phase.report.run_id,"source_revision":phase.report.definition.modeling_revision,"count":phase.occurrences.len(),"unattempted":phase.occurrences.iter().filter(|point|point.outcome.attempts.is_empty()).count(),"native_success":phase.occurrences.iter().filter(|point|point.native_success).count(),"usable":phase.occurrences.iter().filter(|point|point.usable).count(),"conclusion":phase.report.decision.conclusion,"seconds":phase.seconds,"preparations":phase.report.preparations,"outcome_table_error":phase.table.as_ref().err().map(ToString::to_string),"findings_error":phase.findings.as_ref().err().map(ToString::to_string)}));
                }
            }
        }
        serde_json::to_writer_pretty(
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(directory.join("summary.json"))?,
            &serde_json::json!({"scope":self.phase,"execution":self.execution,"source_run":self.source,"settings":self.settings,"solve":self.solve,"memory":self.resources,"memory_scope":"caller-supplied shared-runtime and process-lifetime high-water marks; excludes recorded primary worker","feed_sha256":self.feed_sha256,"feed_generator":self.feed_generator,"preparations_scope":"caller process; recorded execution excludes primary worker","start":"no_prior_start","predecessors":"none","outcomes":summaries}),
        )?;
        Ok(())
    }
}
fn write_ipc<'a>(
    path: &Path,
    schema: &datafusion::arrow::datatypes::Schema,
    batches: impl IntoIterator<Item = &'a RecordBatch>,
) -> Result<()> {
    let mut writer = datafusion::arrow::ipc::writer::FileWriter::try_new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)?,
        schema,
    )?;
    for batch in batches {
        writer.write(batch)?;
    }
    writer.finish()?;
    Ok(())
}

#[cfg(test)]
#[path = "thermodynamic_campaign_tests.rs"]
mod tests;
