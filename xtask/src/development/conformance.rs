// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Manifest-driven native conformance. Applications own runtime construction and shutdown.
use anyhow::{Context, Result, ensure};
use pse_admission::authoring_driver::document::{
    CapturedPackage, OwnedDocumentBundle, OwnedDocumentSet, capture_package,
    load_package_documents_owned,
};
use pse_model::generated::{
    enums::{ConformanceExecution, ConformancePresolve},
    identities::DeclarationId,
};
use pse_runtime::{
    CancelSource, ResourceBudget,
    math::settings::SolveSettings,
    workflow::{
        ConformanceControls, ConformanceManifest, ConformanceRun, ModelingConformanceReport,
        ModelingDiagnosticPolicy, PreparationSettings, PureConformanceControls, RunSettings,
        Runtime, conform_pure_request,
    },
};
use pse_schema::Registry;
use std::{
    collections::BTreeMap,
    io::Read,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    time::Duration,
};

mod progress;
pub use progress::{CampaignProgress, ProcessInterrupt};

/// Explicit application capability selection; declarations alone load no native library.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum NativeProviders {
    /// Ordinary source-only capability context.
    #[default]
    None,
    /// Authenticated CoolProp 8 homogeneous water branches.
    #[value(name = "coolprop8-water")]
    CoolpropWater,
}
impl NativeProviders {
    /// Refuse an unavailable selected capability before constructing execution services.
    ///
    /// # Errors
    /// Returns an error when the selected provider is absent from this build.
    pub fn validate(self) -> Result<()> {
        ensure!(
            self == Self::None || cfg!(feature = "property-coolprop"),
            "selected coolprop8-water capability requires the property-coolprop build feature"
        );
        Ok(())
    }
}

/// An admitted campaign selection. All run selection is resolved before execution.
#[derive(Debug)]
pub struct Campaign {
    manifest: ConformanceManifest,
    parent: PathBuf,
    runs: Vec<usize>,
    fixtures: Option<Vec<DeclarationId>>,
}

impl Campaign {
    /// Admit the manifest and select one named run or its entire ordered inventory.
    /// An explicit fixture selection requires exactly one run.
    pub fn load(
        path: &Path,
        run: Option<&str>,
        fixtures: Option<Vec<DeclarationId>>,
    ) -> Result<Self> {
        let path = path
            .canonicalize()
            .with_context(|| format!("conformance manifest {}", path.display()))?;
        let manifest = ConformanceManifest::load(&path)?;
        let runs: Vec<_> = manifest
            .runs
            .iter()
            .enumerate()
            .filter(|(_, candidate)| run.is_none_or(|name| candidate.name == name))
            .map(|(index, _)| index)
            .collect();
        ensure!(
            !runs.is_empty(),
            "{}: unknown conformance run {run:?}",
            path.display()
        );
        ensure!(
            fixtures.is_none() || runs.len() == 1,
            "a fixture selection requires exactly one selected conformance run"
        );
        // The same public native control owns duplicate occurrence admission.
        PureConformanceControls {
            fixtures: fixtures.clone(),
            ..Default::default()
        }
        .selection()?;
        Ok(Self {
            manifest,
            parent: path.parent().unwrap_or_else(|| Path::new(".")).to_owned(),
            runs,
            fixtures,
        })
    }

    /// Shared admitted resource/scientific settings; no second settings document is introduced.
    pub fn settings(&self) -> &RunSettings {
        &self.manifest.settings
    }

    /// Selected runs in authored manifest order.
    pub fn runs(&self) -> impl Iterator<Item = &ConformanceRun> {
        self.runs.iter().map(|index| &self.manifest.runs[*index])
    }

    fn roots(&self, run: &ConformanceRun) -> Result<NativeSourceSelection> {
        NativeSourceSelection::resolve(&self.parent, run)
    }

    /// Refuse missing source roots or unavailable providers before any runtime effects.
    pub fn preflight(&self, providers: NativeProviders) -> Result<()> {
        providers.validate()?;
        for run in self.runs() {
            self.roots(run)?;
        }
        Ok(())
    }

    /// Check authored source roots before preparing the executor or creating output.
    /// Fixture binding and provider registration remain execution-time decisions.
    pub fn readiness(&self, providers: NativeProviders) -> Result<serde_json::Value> {
        providers.validate()?;
        let runs = self.runs().map(|run| {
            let roots = self.roots(run)?;
            Ok(serde_json::json!({
                "run": run.name,
                "execution": run.execution,
                "physical": roots.physical,
                "modeling": roots.modeling,
            }))
        }).collect::<Result<Vec<_>>>()?;
        Ok(serde_json::json!({
            "version": 1,
            "runs": runs,
            "requested_fixtures": self.fixtures,
            "provider": match providers {
                NativeProviders::None => "none",
                NativeProviders::CoolpropWater => "coolprop8-water",
            },
            "fixture_admission": "requires_runtime",
            "provider_registration": "requires_runtime",
        }))
    }

    /// Execute one selected native run using the application's existing runtime owner.
    /// The application constructs that runtime using the campaign budget and settles it
    /// after all successes or failures. No canonical store is opened implicitly.
    pub async fn run_native(
        &self,
        run: &ConformanceRun,
        runtime: &Runtime,
        providers: NativeProviders,
        cancel: &CancelSource,
    ) -> Result<RunReport> {
        self.require_selected(run, ConformanceExecution::Native)?;
        providers.validate()?;
        cancel.checkpoint()?;
        let selection = self.roots(run)?;
        cancel.checkpoint()?;
        let original = selection.capture_physical(runtime.registry())?;
        cancel.checkpoint()?;
        ensure!(
            selection == self.roots(run)?,
            "conformance package roots changed during capture"
        );
        let source = runtime.source_services();
        let pool = source.preparation().pool();
        let validation = runtime.validation_context();
        let token = cancel.token();
        let load = |documents: &BTreeMap<String, Vec<u8>>| -> Result<_> {
            Ok(load_package_documents_owned(
                documents,
                runtime.registry(),
                Default::default(),
                &pool,
                &token,
                &validation,
            )?)
        };
        let physical_bundle = load(&original.files)?;
        drop(original);
        let physical =
            OwnedDocumentSet::try_from_bundles(vec![physical_bundle.clone()], &pool, &token)?;
        let physical = runtime.physical_from_documents(&physical, &token).await?;
        cancel.checkpoint()?;
        let registrations = match providers {
            NativeProviders::None => BTreeMap::new(),
            NativeProviders::CoolpropWater => {
                #[cfg(feature = "property-coolprop")]
                {
                    use pse_backend_native::coolprop::{self, WaterRegime};
                    [
                        WaterRegime::Liquid,
                        WaterRegime::Vapor,
                        WaterRegime::Supercritical,
                    ]
                    .into_iter()
                    .map(|regime| {
                        cancel.checkpoint()?;
                        Ok((
                            regime.implementation().to_owned(),
                            coolprop::registration(physical.quantities(), regime)?,
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>>>()?
                }
                #[cfg(not(feature = "property-coolprop"))]
                anyhow::bail!("selected coolprop8-water capability is unprepared in this build")
            }
        };
        cancel.checkpoint()?;
        let bundles = selection.modeling_bundles(&physical_bundle, runtime.registry(), load)?;
        let documents = OwnedDocumentSet::try_from_bundles(bundles, &pool, &token)?;
        let package = runtime
            .modeling_from_documents_registered(&documents, physical, registrations, cancel)
            .await?;
        let settings = &self.manifest.settings;
        let mut solve = SolveSettings {
            intent: settings.intent,
            backend: settings.backend,
            presolve: match settings.presolve {
                ConformancePresolve::Auto => pse_backend_native::presolve::PolicyKind::Auto,
                ConformancePresolve::Off => pse_backend_native::presolve::PolicyKind::Off,
            },
            ..Default::default()
        };
        ensure!(
            settings.time_limit_seconds.is_finite() && settings.time_limit_seconds > 0.,
            "conformance time allowance must be positive and finite"
        );
        solve.controls.time_limit = Duration::try_from_secs_f64(settings.time_limit_seconds)?;
        let diagnostics = settings.diagnostics.as_deref().map(|path| -> Result<ModelingDiagnosticPolicy> {
            let policy: ModelingDiagnosticPolicy = <ModelingDiagnosticPolicy as pse_runtime::boundary_documents::BoundaryDocument>::WIRE.decode_slice(&bounded_file(Path::new(path))?)?;
            policy.validate()?;
            Ok(policy)
        }).transpose()?;
        let report = package
            .conform_request(
                &runtime.execution_services(),
                &preparation(settings),
                solve,
                ConformanceControls {
                    maximum_fixtures: settings.maximum_fixtures,
                    maximum_checks: settings.maximum_checks,
                    derivative_cells: settings.derivative_cells,
                    derivative_step: settings.derivative_step,
                    derivative_tolerance: settings.derivative_tolerance,
                    fixtures: self.fixtures.clone(),
                    diagnostics,
                },
                cancel,
            )
            .await?;
        drop(source);
        Ok(RunReport {
            name: run.name.clone(),
            report,
        })
    }

    /// Pure conformance builds only the native standalone scientific owner.
    pub async fn run_pure(
        &self,
        run: &ConformanceRun,
        spill: &Path,
        cancel: &CancelSource,
    ) -> Result<RunReport> {
        self.require_selected(run, ConformanceExecution::Pure)?;
        let documents = std::iter::once(&run.package)
            .chain(&run.dependencies)
            .map(|path| {
                cancel.checkpoint()?;
                documents(&self.parent.join(path))
            })
            .collect::<Result<Vec<_>>>()?;
        cancel.checkpoint()?;
        let settings = &self.manifest.settings;
        let report = conform_pure_request(
            documents,
            self::documents(&self.parent.join(&run.physical))?,
            budget(settings, spill)?,
            PureConformanceControls {
                maximum_fixtures: settings.maximum_fixtures,
                maximum_checks: settings.maximum_checks,
                fixtures: self.fixtures.clone(),
            },
            preparation(settings),
            cancel,
        )
        .await?;
        Ok(RunReport {
            name: run.name.clone(),
            report,
        })
    }

    fn require_selected(
        &self,
        run: &ConformanceRun,
        execution: ConformanceExecution,
    ) -> Result<()> {
        ensure!(
            self.runs().any(|selected| std::ptr::eq(selected, run)),
            "run must be borrowed from this admitted campaign selection"
        );
        ensure!(
            run.execution == execution,
            "conformance execution route differs from the authored run"
        );
        Ok(())
    }
}

/// Resolved source occurrences for one native run, in authored modeling order.
#[derive(Debug, PartialEq, Eq)]
struct NativeSourceSelection {
    physical: PathBuf,
    modeling: Vec<PathBuf>,
}

impl NativeSourceSelection {
    fn resolve(parent: &Path, run: &ConformanceRun) -> Result<Self> {
        let resolve = |path: &str| {
            let source = parent.join(path);
            let resolved = source
                .canonicalize()
                .with_context(|| format!("conformance package root {}", source.display()))?;
            ensure!(
                resolved.is_dir() && resolved.join("package.toml").is_file(),
                "conformance package root {} has no package.toml",
                resolved.display()
            );
            Ok(resolved)
        };
        let physical = resolve(&run.physical)?;
        let modeling = std::iter::once(&run.package)
            .chain(&run.dependencies)
            .map(|path| resolve(path))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { physical, modeling })
    }

    fn capture_physical(&self, registry: &Registry) -> Result<CapturedPackage> {
        let original = capture_package(&self.physical, registry, Default::default())?;
        original.verify(&self.physical, registry)?;
        Ok(original)
    }

    fn modeling_bundles(
        &self,
        physical: &OwnedDocumentBundle,
        registry: &Registry,
        load: impl Fn(&BTreeMap<String, Vec<u8>>) -> Result<OwnedDocumentBundle>,
    ) -> Result<Vec<OwnedDocumentBundle>> {
        self.modeling
            .iter()
            .map(|path| {
                if *path == self.physical {
                    // The actual root was resolved before capture. The clone retains
                    // its original lease, parser results and exact document bytes.
                    Ok(physical.clone())
                } else {
                    load(&capture_package(path, registry, Default::default())?.files)
                }
            })
            .collect()
    }
}

/// Original native report and its authored run name. Tables keep the report's own custody.
#[derive(Debug)]
pub struct RunReport {
    /// Manifest run identity.
    pub name: String,
    /// Complete native report, including failed, missing and unattempted fixtures.
    pub report: ModelingConformanceReport,
}
impl RunReport {
    /// A complete pass; selected fixture execution does not imply package coverage.
    pub fn passed_completely(&self) -> bool {
        self.report.complete && self.report.passed()
    }

    /// Publish the six retained IPC report relations, refusing existing destinations.
    /// Each file is published only after its IPC footer completes. An error does not
    /// claim the entire report set was published; earlier complete files remain explicit.
    pub fn export(&self, path: &Path) -> Result<Vec<PathBuf>> {
        self.export_observed(path, |_| {})
    }

    /// Publish the same reports and notify the owner after each successful file publication.
    pub fn export_observed(
        &self,
        path: &Path,
        mut published_file: impl FnMut(&Path),
    ) -> Result<Vec<PathBuf>> {
        let mut tables = vec![
            (path.to_owned(), self.report.table()?),
            (
                path.with_extension("fixtures.arrow"),
                self.report.fixture_statuses_table()?,
            ),
            (
                path.with_extension("findings.arrow"),
                self.report.findings_table()?,
            ),
            (
                path.with_extension("parity.arrow"),
                self.report.parity_table()?,
            ),
        ];
        let admissions = self.report.admission_tables()?;
        for (relation, suffix) in [
            (
                pse_relations::generated::runtime::route_decisions::RELATION_ID,
                "routes.arrow",
            ),
            (
                pse_relations::generated::runtime::structural_assessments::RELATION_ID,
                "structure.arrow",
            ),
        ] {
            let table = admissions
                .get(&relation)
                .context("native conformance admission relation missing")?;
            tables.push((path.with_extension(suffix), table.clone()));
        }
        let mut published = Vec::new();
        for (destination, table) in tables {
            let parent = destination
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let mut staged = tempfile::NamedTempFile::new_in(parent)?;
            {
                let mut writer = datafusion::arrow::ipc::writer::FileWriter::try_new(
                    staged.as_file_mut(),
                    table.batch().schema().as_ref(),
                )?;
                writer.write(table.batch())?;
                writer.finish()?;
            }
            staged.persist_noclobber(&destination)?;
            published_file(&destination);
            published.push(destination);
        }
        Ok(published)
    }
}

/// Construct the explicitly admitted campaign resource selection for a native caller.
pub fn budget(settings: &RunSettings, spill: &Path) -> Result<ResourceBudget> {
    let positive =
        |value| NonZeroUsize::new(value).context("conformance resource selection must be positive");
    let threads = positive(settings.threads)?;
    let mut math = pse_runtime::math::MathPolicy::for_pool(settings.memory_limit_bytes);
    if let Some(value) = settings.math_jobs {
        math.jobs = value;
    }
    if let Some(value) = settings.math_worker_bytes {
        math.worker_bytes = value;
    }
    if let Some(value) = settings.math_admission_wait_ms {
        math.admission_wait = Duration::from_millis(value);
    }
    if let Some(value) = settings.math_preparation_limit_ms {
        math.preparation_limit = Duration::from_millis(value);
    }
    Ok(ResourceBudget {
        memory_limit_bytes: positive(settings.memory_limit_bytes)?,
        spill_dir: spill.to_owned(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: positive(16)?,
        threads: pse_engine::ThreadBudget {
            pool_threads: threads,
            target_partitions: threads,
        },
        execution: Default::default(),
        cache: pse_runtime::CacheBudget::disabled(1024),
        math,
    })
}

fn preparation(settings: &RunSettings) -> PreparationSettings {
    if let Some(preparation) = &settings.preparation {
        return preparation.clone();
    }
    let mut preparation = PreparationSettings::default();
    if let Some(value) = settings.expansion_items {
        preparation.limits.items = value;
    }
    if let Some(value) = settings.expansion_members {
        preparation.limits.members = value;
    }
    if let Some(value) = settings.expansion_depth {
        preparation.limits.depth = value;
    }
    preparation.limits.body_occurrences = settings.body_occurrences;
    preparation.limits.body_slots = settings.body_slots;
    preparation
}

/// Capture a package through its owning production loader rather than reimplementing
/// filesystem extension filters. Shared with native campaign consumers.
pub fn documents(root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    Ok(capture_package(
        root,
        pse_schema::shared_registry()?.as_ref(),
        Default::default(),
    )?
    .files)
}

fn bounded_file(path: &Path) -> Result<Vec<u8>> {
    const MAXIMUM: u64 = 16 << 20;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAXIMUM + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAXIMUM,
        "{}: conformance input exceeds extent",
        path.display()
    );
    Ok(bytes)
}

#[cfg(test)]
#[path = "conformance_tests.rs"]
mod tests;
