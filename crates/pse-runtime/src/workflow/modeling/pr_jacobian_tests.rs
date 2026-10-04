// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original PR fixture derivatives against independent evaluations of the same case.
use super::*;
use crate::math::{MathRuntimeError, WorkerBudget};
use pse_backend_native::solve::{Controls, Execution, SolveIntent};
use pse_columnar::flight::FlightCancellation;
use pse_kernels::DerivativeOrder;
use pse_math::{
    assembly::CaseWorker,
    binding::CaseValues,
    diagnostics::{MatrixPolicy, analyze_matrix},
    index::{GlobalCol, GlobalRow},
    normalization::Normalization,
};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroUsize, path::Path, sync::Arc, time::Duration};

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
const DIMENSION: usize = 120;
const RELATIVE_STEP: f64 = 1e-9;
const STABLE_RELATIVE: f64 = 2e-4;
const STABLE_ABSOLUTE: f64 = 1e-6;
const STRONG: f64 = 1e-3;
const AGREEMENT_RELATIVE: f64 = 1e-3;
const AGREEMENT_ABSOLUTE: f64 = 1e-5;

#[derive(Deserialize)]
struct Manifest {
    settings: Settings,
    runs: Vec<SourceRun>,
}
#[derive(Deserialize)]
struct Settings {
    memory_limit_bytes: usize,
    threads: usize,
    math_worker_bytes: usize,
    preparation: crate::workflow::PreparationSettings,
}
#[derive(Deserialize)]
struct SourceRun {
    name: String,
    package: String,
    dependencies: Vec<String>,
    physical: String,
}

/// Load exact document bytes; unlike the acceptance seed helper, no tests are removed.
fn documents(root: &Path) -> TestResult<BTreeMap<String, Vec<u8>>> {
    fn visit(root: &Path, directory: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> TestResult<()> {
        for entry in std::fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_dir() {
                visit(root, &path, out)?;
            } else if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("toml" | "yaml" | "yml" | "pse" | "parquet")
            ) {
                let relative = path
                    .strip_prefix(root)?
                    .to_str()
                    .ok_or_else(|| std::io::Error::other("non-UTF8 reference document path"))?;
                out.insert(relative.replace('\\', "/"), std::fs::read(&path)?);
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out)?;
    Ok(out)
}

async fn package(
    settings: &Settings,
    source: &SourceRun,
    root: &Path,
) -> TestResult<(ModelingPackage, tempfile::TempDir)> {
    let spill = tempfile::tempdir()?;
    let positive = |value| {
        NonZeroUsize::new(value)
            .ok_or_else(|| std::io::Error::other("positive reference execution allowance required"))
    };
    assert_eq!(
        settings.threads, 1,
        "this diagnostic runs native proofs serially"
    );
    let shared = crate::SharedRuntime::build(crate::ResourceBudget {
        memory_limit_bytes: positive(settings.memory_limit_bytes)?,
        spill_dir: spill.path().to_owned(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: positive(16)?,
        threads: pse_engine::ThreadBudget {
            pool_threads: positive(settings.threads)?,
            target_partitions: positive(settings.threads)?,
        },
        execution: Default::default(),
        cache: crate::DeltaCacheBudget::for_memory(settings.memory_limit_bytes),
        math: crate::math::MathPolicy {
            worker_bytes: settings.math_worker_bytes,
            ..Default::default()
        },
        hashing_may_use_pool: false,
    })?;
    let registry = pse_schema::shared_registry()?;
    let sessions = Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
    let runtime = Runtime::from_shared(shared.clone(), registry.clone(), sessions.clone());
    let token = pse_columnar::CancellationToken::new();
    let pool = shared.pool();
    let validation = sessions.validation_context(&registry)?;
    let load = |name: &str| -> TestResult<_> {
        Ok(
            crate::authoring_driver::document::load_package_documents_owned(
                &documents(&root.join(name))?,
                &registry,
                Default::default(),
                &pool,
                &token,
                &validation,
            )?,
        )
    };
    let physical_documents = crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
        vec![load(&source.physical)?],
        &pool,
        &token,
    )?;
    let physical = runtime
        .physical_from_documents(&physical_documents, &token)
        .await?;
    let bundles = std::iter::once(&source.package)
        .chain(&source.dependencies)
        .map(|name| load(name))
        .collect::<TestResult<Vec<_>>>()?;
    let closure = crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
        bundles, &pool, &token,
    )?;
    Ok((runtime.modeling_from_documents(&closure, physical)?, spill))
}

#[derive(Clone, Serialize)]
struct Member {
    id: SemanticId,
    path: String,
    declaration: Option<DeclarationId>,
}
#[derive(Serialize)]
struct Column {
    member: Member,
    initial: f64,
    nominal: f64,
    step: f64,
    central: bool,
    selected_root_input: bool,
}
#[derive(Serialize)]
struct Row {
    member: Member,
    initial: f64,
    nominal: f64,
}
#[derive(Serialize)]
struct Cell {
    row: usize,
    column: usize,
    analytic: f64,
    difference: f64,
    half_difference: f64,
    normalized_error: f64,
    normalized_drift: f64,
    stable: bool,
    strong: bool,
    permitted_error: f64,
}
#[derive(Serialize)]
struct Sample {
    fixture: DeclarationId,
    rows: Vec<Row>,
    columns: Vec<Column>,
    /// Complete raw physical derivatives, row-major, including structural zeros.
    analytic: Vec<f64>,
    difference: Vec<f64>,
    half_difference: Vec<f64>,
    cells: Vec<Cell>,
    stable_strong: usize,
    disagreements: usize,
    noisy: usize,
    selected_root_rows: Vec<usize>,
    selected_root_stable_strong: usize,
    hessians: Vec<HessianSample>,
    rank: usize,
    rank_cutoff: f64,
    singular_values: Vec<f64>,
    row_norms: Vec<f64>,
    column_norms: Vec<f64>,
    worker_bytes: usize,
    elapsed_seconds: f64,
}
#[derive(Serialize)]
struct HessianSample {
    name: &'static str,
    source_rows: Vec<usize>,
    /// Physical row multipliers: distinct dimensionless weights divided by Sr.
    weights: Vec<f64>,
    gradient: Vec<f64>,
    /// Complete raw physical weighted Hessian, row-major, including structural zeros.
    analytic: Vec<f64>,
    difference: Vec<f64>,
    half_difference: Vec<f64>,
    cells: Vec<Cell>,
    stable_strong: usize,
    disagreements: usize,
    noisy: usize,
}

/// The original nested overrides link outer x/y/beta to the selected implicit roots.
fn selected_root_source(member: &Member) -> bool {
    member.declaration.is_some_and(|id| {
        matches!(
            id.to_string().as_str(),
            "01a0eed189fc72d7af4d92b527ee4046"
                | "01a0eed189fc72d7af4d92b62fdcc5d7"
                | "01a0eed189fc72d7af4d92b767fe5b32"
        )
    })
}

/// These are provider inputs, excluding the outer identity x/y/beta coordinates.
fn selected_root_input(row: &Member, column: &Member) -> bool {
    row.path
        .rsplit_once('.')
        .is_some_and(|(parent, _)| column.path.starts_with(&format!("{parent}.closure.")))
}

fn weighted_gradients(
    worker: &mut CaseWorker,
    values: &CaseValues,
    weights: &[Vec<f64>],
) -> Result<Vec<Vec<f64>>, MathRuntimeError> {
    let matrix = worker.jacobian(values)?;
    if weights.iter().any(|w| w.len() != matrix.nrows()) {
        return Err(pse_math::MathError::Contract("weighted PR Jacobian rows".into()).into());
    }
    Ok(weights
        .iter()
        .map(|w| {
            (0..matrix.ncols())
                .map(|column| {
                    matrix
                        .row_idx_of_col(column)
                        .zip(matrix.val_of_col(column))
                        .map(|(row, value)| w[row] * value)
                        .sum()
                })
                .collect()
        })
        .collect())
}

/// Differentiate actual Jacobians, independently of CaseWorker.hessian and its patterns.
fn gradient_difference(
    worker: &mut CaseWorker,
    base: &CaseValues,
    weights: &[Vec<f64>],
    id: SemanticId,
    step: f64,
    central: bool,
) -> Result<Vec<Vec<f64>>, MathRuntimeError> {
    let x = base.scalars[&id];
    let mut plus = base.clone();
    plus.scalars.insert(id, x + step);
    let upper = worker.jacobian(&plus)?.clone();
    let (lower_values, divisor) = if central {
        let mut minus = base.clone();
        minus.scalars.insert(id, x - step);
        (minus, (x + step) - (x - step))
    } else {
        (base.clone(), (x + step) - x)
    };
    let lower = worker.jacobian(&lower_values)?;
    weighted_jacobian_difference(upper.as_ref(), lower.as_ref(), weights, divisor)
}

fn weighted_jacobian_difference(
    upper: faer::sparse::SparseColMatRef<'_, usize, f64>,
    lower: faer::sparse::SparseColMatRef<'_, usize, f64>,
    weights: &[Vec<f64>],
    divisor: f64,
) -> Result<Vec<Vec<f64>>, MathRuntimeError> {
    if divisor == 0.
        || !divisor.is_finite()
        || upper.nrows() != lower.nrows()
        || upper.ncols() != lower.ncols()
        || weights.iter().any(|w| w.len() != upper.nrows())
    {
        return Err(pse_math::MathError::Contract(
            "representable PR gradient difference step".into(),
        )
        .into());
    }
    let mut result = vec![vec![0.; upper.ncols()]; weights.len()];
    let mut differences = vec![0.; upper.nrows()];
    for column in 0..upper.ncols() {
        differences.fill(0.);
        // Align original row identities, including structural zeros. Subtract before
        // weighting: a large unchanged row must not erase another row's small change.
        for (row, value) in upper.row_idx_of_col(column).zip(upper.val_of_col(column)) {
            differences[row] += value;
        }
        for (row, value) in lower.row_idx_of_col(column).zip(lower.val_of_col(column)) {
            differences[row] -= value;
        }
        for (group, weights) in weights.iter().enumerate() {
            result[group][column] = weights
                .iter()
                .zip(&differences)
                .map(|(weight, difference)| weight * difference)
                .sum::<f64>()
                / divisor;
        }
    }
    Ok(result)
}

#[test]
fn weighted_jacobian_difference_preserves_changes_beside_large_unchanged_rows() {
    use faer::sparse::{SparseColMat, Triplet};
    let upper = SparseColMat::try_new_from_triplets(
        4,
        1,
        &[
            Triplet::new(0, 0, 1e20),
            Triplet::new(1, 0, 0.25),
            Triplet::new(2, 0, 0.5),
        ],
    )
    .unwrap();
    let lower = SparseColMat::try_new_from_triplets(
        4,
        1,
        &[
            Triplet::new(0, 0, 1e20),
            Triplet::new(1, 0, 0.125),
            Triplet::new(3, 0, 0.0625),
        ],
    )
    .unwrap();
    let weights = vec![vec![2., 3., 4., 8.]];
    let weighted = |matrix: &SparseColMat<usize, f64>| {
        matrix
            .row_idx_of_col(0)
            .zip(matrix.val_of_col(0))
            .map(|(row, value)| weights[0][row] * value)
            .sum::<f64>()
    };
    assert_eq!(weighted(&upper) - weighted(&lower), 0.);
    for divisor in [0.25, 0.125] {
        let actual =
            weighted_jacobian_difference(upper.as_ref(), lower.as_ref(), &weights, divisor)
                .unwrap();
        assert_eq!(actual, vec![vec![1.875 / divisor]]);
    }
}

/// Finite differences use only Value results; actual representable offsets set the divisor.
fn difference(
    worker: &mut CaseWorker,
    base: &CaseValues,
    initial: &[f64],
    id: SemanticId,
    step: f64,
    central: bool,
) -> Result<Vec<f64>, MathRuntimeError> {
    let x = base.scalars[&id];
    let mut plus = base.clone();
    plus.scalars.insert(id, x + step);
    let upper = worker.constraints(&plus)?;
    let (lower, divisor) = if central {
        let mut minus = base.clone();
        minus.scalars.insert(id, x - step);
        (worker.constraints(&minus)?, (x + step) - (x - step))
    } else {
        (initial.to_vec(), (x + step) - x)
    };
    if divisor == 0. || !divisor.is_finite() || upper.len() != lower.len() {
        return Err(
            pse_math::MathError::Contract("representable PR difference step".into()).into(),
        );
    }
    Ok(upper
        .iter()
        .zip(lower)
        .map(|(a, b)| (a - b) / divisor)
        .collect())
}

#[expect(
    clippy::print_stderr,
    reason = "bounded named native diagnostic evidence accompanies assertions"
)]
fn describe(sample: &Sample, artifact: &Path) {
    eprintln!(
        "original PR Jacobian: {}x{}, stable strong={}, disagreements={}, noisy={}, rank={}, cutoff={:.3e}, elapsed={:.1}s, worker_bytes={}, raw artifact={}",
        sample.rows.len(),
        sample.columns.len(),
        sample.stable_strong,
        sample.disagreements,
        sample.noisy,
        sample.rank,
        sample.rank_cutoff,
        sample.elapsed_seconds,
        sample.worker_bytes,
        artifact.display()
    );
    eprintln!(
        "selected-root Jacobian coverage: source_rows={:?}, stable strong closure-input cells={}",
        sample.selected_root_rows, sample.selected_root_stable_strong
    );
    let mut worst = sample
        .cells
        .iter()
        .filter(|c| c.strong || !c.stable)
        .collect::<Vec<_>>();
    worst.sort_by(|a, b| b.normalized_error.total_cmp(&a.normalized_error));
    for cell in worst.into_iter().take(20) {
        let row = &sample.rows[cell.row];
        let column = &sample.columns[cell.column];
        eprintln!(
            "row={} {} source={:?}; column={} {} source={:?}; F={:.9e} x={:.9e} h={:.3e} J={:.9e} FDh={:.9e} FDh2={:.9e} scaled_error={:.3e} scaled_drift={:.3e} stable={} strong={} allowance={:.3e}",
            row.member.id,
            row.member.path,
            row.member.declaration,
            column.member.id,
            column.member.path,
            column.member.declaration,
            row.initial,
            column.initial,
            column.step,
            cell.analytic,
            cell.difference,
            cell.half_difference,
            cell.normalized_error,
            cell.normalized_drift,
            cell.stable,
            cell.strong,
            cell.permitted_error
        );
    }
    for hessian in &sample.hessians {
        eprintln!(
            "weighted Hessian {}: source_rows={:?}, stable strong={}, disagreements={}, noisy={}",
            hessian.name,
            hessian.source_rows,
            hessian.stable_strong,
            hessian.disagreements,
            hessian.noisy
        );
        let mut worst = hessian
            .cells
            .iter()
            .filter(|c| c.strong || !c.stable)
            .collect::<Vec<_>>();
        worst.sort_by(|a, b| b.normalized_error.total_cmp(&a.normalized_error));
        for cell in worst.into_iter().take(20) {
            let row = &sample.columns[cell.row];
            let column = &sample.columns[cell.column];
            eprintln!(
                "H {}: coordinate={} {} source={:?}; coordinate={} {} source={:?}; h={:.3e} H={:.9e} FDh={:.9e} FDh2={:.9e} scaled_error={:.3e} scaled_drift={:.3e} stable={} strong={} allowance={:.3e}",
                hessian.name,
                row.member.id,
                row.member.path,
                row.member.declaration,
                column.member.id,
                column.member.path,
                column.member.declaration,
                column.step,
                cell.analytic,
                cell.difference,
                cell.half_difference,
                cell.normalized_error,
                cell.normalized_drift,
                cell.stable,
                cell.strong,
                cell.permitted_error
            );
        }
    }
}

#[tokio::test]
async fn original_pr_case_jacobian_matches_stable_value_differences() -> TestResult<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/reference");
    let manifest: Manifest =
        toml::from_str(&std::fs::read_to_string(root.join("conformance.toml"))?)?;
    let source = manifest
        .runs
        .iter()
        .find(|r| r.name == "seed")
        .ok_or_else(|| std::io::Error::other("reference seed closure absent"))?;
    let (package, _spill) = package(&manifest.settings, source, &root).await?;
    let fixture = DeclarationId::from(SemanticId::parse_hex("01a0eecf2935721c9fc3aaab14032e96")?);
    let cancel = crate::CancelSource::new();
    let solver = crate::math::solves::SolverProfile {
        intent: SolveIntent::Root,
        controls: Controls {
            time_limit: Duration::from_secs(1800),
            ..Default::default()
        },
        ..Default::default()
    };
    let declared = package
        .declared_execution(
            fixture,
            manifest.settings.preparation.compiler,
            solver,
            Default::default(),
            manifest.settings.preparation.limits,
            &cancel,
        )
        .await?;
    assert!(matches!(declared.procedure, DeclaredProcedure::Solve));
    let analysis = declared.analysis;
    let resolved = package
        .resolve_case(
            analysis.root,
            analysis.instance,
            analysis.bindings,
            analysis.limits,
            analysis.case,
            DerivativeOrder::Second,
            analysis.compiler,
            analysis.solver,
            analysis.numerical,
            Default::default(),
            false,
            &cancel,
        )
        .await?;
    let plan = &resolved.model.case.compiled().plan;
    let row_ids = plan
        .structure()
        .rows()
        .iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    let column_ids = plan.columns().to_vec();
    assert_eq!((row_ids.len(), column_ids.len()), (DIMENSION, DIMENSION));
    assert_eq!(plan.order(), DerivativeOrder::Second);
    assert_eq!(resolved.model.case.structure().matching.len(), DIMENSION);
    plan.structure().validate_values(&resolved.model.values)?;
    let model = &resolved.model.model.compiled().model;
    let member = |id| {
        let lineage = model.symbols.get(&id).map(|s| &s.lineage).or_else(|| {
            model
                .equations
                .iter()
                .find(|r| r.id == id)
                .map(|r| &r.lineage)
        });
        Member {
            id,
            path: lineage.map_or_else(|| id.to_string(), |l| l.path.clone()),
            declaration: lineage.map(|l| l.declaration),
        }
    };
    let row_members = row_ids.iter().copied().map(&member).collect::<Vec<_>>();
    let column_members = column_ids.iter().copied().map(&member).collect::<Vec<_>>();
    let inlet_temperature = model
        .paths
        .get("root.inlet.T")
        .ok_or_else(|| std::io::Error::other("original inlet temperature absent"))?;
    assert_eq!(resolved.model.values.scalars[inlet_temperature], 368.);
    assert!(
        !column_ids.contains(inlet_temperature),
        "the original inlet remains fixed"
    );
    let bounds = plan
        .structure()
        .variables()
        .iter()
        .map(|v| {
            (
                v.port.id,
                (
                    v.lower.unwrap_or(f64::NEG_INFINITY),
                    v.upper.unwrap_or(f64::INFINITY),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for member in &column_members {
        let x = resolved.model.values.scalars[&member.id];
        let (lower, upper) = bounds[&member.id];
        assert!(
            lower <= x && x <= upper,
            "original start {} outside bounds",
            member.path
        );
    }
    let normalization = Normalization::from_policy(&resolved.numerics, &column_ids, &row_ids)?;
    let base = resolved.model.values.clone();
    let controls = resolved.solver.controls.clone();
    let service = package.runtime.shared.math();
    let executable = service.assemble(resolved.model.case.clone()).await?;
    let worker_allowance = package.runtime.shared.budget().math.worker_bytes;
    // The native source/workspace leases are retained by registrations. This job owns
    // one numeric worker and the bounded matrix inspection, never one worker per column.
    // Three full derivative comparisons retain 43,200 classified cells and nine
    // 120x120 arrays, alongside the bounded SVD workspace and original identity maps.
    let diagnostic_bytes = 8 << 20;
    let _diagnostic_owner = service.reserve("math:pr-jacobian-diagnostic", diagnostic_bytes)?;
    let budget = WorkerBudget::new(worker_allowance);
    let job_service = service.clone();
    let providers = resolved.providers;
    let sample = service
        .job(
            1,
            worker_allowance,
            FlightCancellation::default(),
            move |flag| {
                let execution = Execution::new(flag.clone(), &controls);
                let scope = execution.scope()?;
                let checkpoint = || {
                    scope
                        .check()
                        .map_err(pse_backend_native::ProblemError::Provider)
                };
                let mut owned =
                    job_service.worker(executable, &providers, scope.clone(), &budget)?;
                let worker = owned.worker();
                let worker_bytes = worker.assembly().numeric_worker_bytes();
                // First binds each mathematical selector before perturbed Value calls.
                let matrix = worker.jacobian(&base)?;
                checkpoint()?;
                let mut analytic = vec![0.; DIMENSION * DIMENSION];
                for column in 0..DIMENSION {
                    for (row, value) in matrix.row_idx_of_col(column).zip(matrix.val_of_col(column))
                    {
                        analytic[row * DIMENSION + column] = *value;
                    }
                }
                let row_scales = normalization
                    .rows
                    .iter()
                    .map(|v| 1. / v)
                    .collect::<Vec<_>>();
                let column_scales = normalization
                    .variables
                    .iter()
                    .map(|v| 1. / v)
                    .collect::<Vec<_>>();
                let evidence = analyze_matrix::<GlobalRow, GlobalCol>(
                    matrix.as_ref(),
                    &row_scales,
                    &column_scales,
                    MatrixPolicy {
                        dense_entries: 3 * DIMENSION * DIMENSION,
                        findings: DIMENSION * DIMENSION,
                        parallel_tolerance: 1e-8,
                        rank_absolute: 1e-12,
                        rank_relative: 1e-8,
                        singular_vector: 0.1,
                    },
                    &flag,
                )?;
                checkpoint()?;
                let initial = worker.constraints(&base)?;
                assert_eq!(initial.len(), DIMENSION);
                let selected_root_rows = row_members
                    .iter()
                    .enumerate()
                    .filter_map(|(i, member)| selected_root_source(member).then_some(i))
                    .collect::<Vec<_>>();
                assert_eq!(
                    selected_root_rows.len(),
                    10,
                    "two original five-row nested root links"
                );
                let rows: Vec<Row> = row_members
                    .into_iter()
                    .enumerate()
                    .map(|(i, member)| Row {
                        member,
                        initial: initial[i],
                        nominal: normalization.rows[i],
                    })
                    .collect();
                // Nonuniform positive weights make each normalized constraint contribute
                // differently; the second comparison isolates the original selected-root links.
                let all_weights = normalization
                    .rows
                    .iter()
                    .enumerate()
                    .map(|(row, nominal)| {
                        (1. + (row + 1) as f64 / (DIMENSION + 1) as f64) / nominal
                    })
                    .collect::<Vec<_>>();
                let root_weights = all_weights
                    .iter()
                    .enumerate()
                    .map(|(row, weight)| {
                        if selected_root_rows.contains(&row) {
                            *weight
                        } else {
                            0.
                        }
                    })
                    .collect::<Vec<_>>();
                let weights = vec![all_weights, root_weights];
                let initial_gradients = weighted_gradients(worker, &base, &weights)?;
                let mut hessians = Vec::with_capacity(weights.len());
                for (group, weights) in weights.iter().enumerate() {
                    checkpoint()?;
                    let matrix = worker.hessian(&base, 0., weights)?;
                    let mut full = vec![0.; DIMENSION * DIMENSION];
                    for column in 0..DIMENSION {
                        for (row, value) in
                            matrix.row_idx_of_col(column).zip(matrix.val_of_col(column))
                        {
                            full[row * DIMENSION + column] = *value;
                            full[column * DIMENSION + row] = *value;
                        }
                    }
                    hessians.push(HessianSample {
                        name: if group == 0 {
                            "all_original_rows"
                        } else {
                            "selected_root_links"
                        },
                        source_rows: if group == 0 {
                            (0..DIMENSION).collect()
                        } else {
                            selected_root_rows.clone()
                        },
                        weights: weights.clone(),
                        gradient: initial_gradients[group].clone(),
                        analytic: full,
                        difference: vec![0.; DIMENSION * DIMENSION],
                        half_difference: vec![0.; DIMENSION * DIMENSION],
                        cells: vec![],
                        stable_strong: 0,
                        disagreements: 0,
                        noisy: 0,
                    });
                }
                checkpoint()?;
                let mut columns = Vec::with_capacity(DIMENSION);
                let mut full_difference = vec![0.; analytic.len()];
                let mut half_difference = full_difference.clone();
                for (column, member) in column_members.into_iter().enumerate() {
                    checkpoint()?;
                    let x = base.scalars[&member.id];
                    let (lower, upper) = bounds[&member.id];
                    assert!(
                        lower <= x && x <= upper,
                        "original start {} outside bounds",
                        member.path
                    );
                    let selected_root_input = selected_root_rows
                        .iter()
                        .any(|row| selected_root_input(&rows[*row].member, &member));
                    // Central differences balance O(h^2) truncation against O(epsilon/h)
                    // roundoff at cbrt(epsilon). Tiny steps can leave weighted-gradient changes
                    // only a few ULPs above a large affine baseline; even equal h/h2 estimates
                    // then fail to establish resolution. Actual selector inputs retain the
                    // smaller step inside the observed 1e-8 relative certified chart; all other
                    // coordinates leave those inputs unchanged. Chart validation still governs
                    // every perturbed evaluation, including any narrower chart in this fixture.
                    let relative_step = if selected_root_input {
                        RELATIVE_STEP
                    } else {
                        f64::EPSILON.cbrt()
                    };
                    let magnitude = relative_step * x.abs().max(1.);
                    let central = x - magnitude >= lower && x + magnitude <= upper;
                    let step = if x + magnitude <= upper {
                        magnitude
                    } else {
                        -magnitude
                    };
                    assert!(
                        x + step >= lower && x + step <= upper && x + step != x,
                        "no legal representable FD direction for {}",
                        member.path
                    );
                    let coarse = difference(worker, &base, &initial, member.id, step, central)?;
                    let fine = difference(worker, &base, &initial, member.id, step / 2., central)?;
                    for row in 0..DIMENSION {
                        full_difference[row * DIMENSION + column] = coarse[row];
                        half_difference[row * DIMENSION + column] = fine[row];
                    }
                    let coarse_gradients =
                        gradient_difference(worker, &base, &weights, member.id, step, central)?;
                    let fine_gradients = gradient_difference(
                        worker,
                        &base,
                        &weights,
                        member.id,
                        step / 2.,
                        central,
                    )?;
                    for (group, hessian) in hessians.iter_mut().enumerate() {
                        for row in 0..DIMENSION {
                            hessian.difference[row * DIMENSION + column] =
                                coarse_gradients[group][row];
                            hessian.half_difference[row * DIMENSION + column] =
                                fine_gradients[group][row];
                        }
                    }
                    columns.push(Column {
                        member,
                        initial: x,
                        nominal: normalization.variables[column],
                        step,
                        central,
                        selected_root_input,
                    });
                }
                checkpoint()?;
                let mut cells = Vec::with_capacity(analytic.len());
                let (mut stable_strong, mut disagreements, mut noisy) = (0, 0, 0);
                for row in 0..DIMENSION {
                    for column in 0..DIMENSION {
                        let index = row * DIMENSION + column;
                        let scale = normalization.variables[column] / normalization.rows[row];
                        let a = analytic[index] * scale;
                        let coarse = full_difference[index] * scale;
                        let fine = half_difference[index] * scale;
                        assert!(a.is_finite() && coarse.is_finite() && fine.is_finite());
                        let drift = (coarse - fine).abs();
                        let stable = drift
                            <= STABLE_ABSOLUTE + STABLE_RELATIVE * coarse.abs().max(fine.abs());
                        let strong = a.abs().max(fine.abs()) >= STRONG;
                        let error = (a - fine).abs();
                        let permitted = AGREEMENT_ABSOLUTE
                            + AGREEMENT_RELATIVE * a.abs().max(fine.abs())
                            + 4. * drift;
                        stable_strong += usize::from(stable && strong);
                        disagreements += usize::from(stable && strong && error > permitted);
                        noisy += usize::from(!stable);
                        cells.push(Cell {
                            row,
                            column,
                            analytic: analytic[index],
                            difference: full_difference[index],
                            half_difference: half_difference[index],
                            normalized_error: error,
                            normalized_drift: drift,
                            stable,
                            strong,
                            permitted_error: permitted,
                        });
                    }
                }
                let selected_root_stable_strong = cells
                    .iter()
                    .filter(|cell| {
                        cell.stable
                            && cell.strong
                            && selected_root_source(&rows[cell.row].member)
                            && selected_root_input(
                                &rows[cell.row].member,
                                &columns[cell.column].member,
                            )
                    })
                    .count();
                for hessian in &mut hessians {
                    hessian.cells = Vec::with_capacity(analytic.len());
                    for row in 0..DIMENSION {
                        for column in 0..DIMENSION {
                            let index = row * DIMENSION + column;
                            let scale =
                                normalization.variables[row] * normalization.variables[column];
                            let a = hessian.analytic[index] * scale;
                            let coarse = hessian.difference[index] * scale;
                            let fine = hessian.half_difference[index] * scale;
                            assert!(a.is_finite() && coarse.is_finite() && fine.is_finite());
                            let drift = (coarse - fine).abs();
                            let stable = drift
                                <= STABLE_ABSOLUTE + STABLE_RELATIVE * coarse.abs().max(fine.abs());
                            let strong = a.abs().max(fine.abs()) >= STRONG;
                            let error = (a - fine).abs();
                            let permitted = AGREEMENT_ABSOLUTE
                                + AGREEMENT_RELATIVE * a.abs().max(fine.abs())
                                + 4. * drift;
                            hessian.stable_strong += usize::from(stable && strong);
                            hessian.disagreements +=
                                usize::from(stable && strong && error > permitted);
                            hessian.noisy += usize::from(!stable);
                            hessian.cells.push(Cell {
                                row,
                                column,
                                analytic: hessian.analytic[index],
                                difference: hessian.difference[index],
                                half_difference: hessian.half_difference[index],
                                normalized_error: error,
                                normalized_drift: drift,
                                stable,
                                strong,
                                permitted_error: permitted,
                            });
                        }
                    }
                }
                checkpoint()?;
                Ok(Sample {
                    fixture,
                    rows,
                    columns,
                    analytic,
                    difference: full_difference,
                    half_difference,
                    cells,
                    stable_strong,
                    disagreements,
                    noisy,
                    selected_root_rows,
                    selected_root_stable_strong,
                    hessians,
                    rank: evidence.rank,
                    rank_cutoff: evidence.cutoff,
                    singular_values: evidence.modes.into_iter().map(|m| m.value).collect(),
                    row_norms: evidence.row_norms,
                    column_norms: evidence.column_norms,
                    worker_bytes,
                    elapsed_seconds: execution.started.elapsed().as_secs_f64(),
                })
            },
        )
        .await?;
    // Retain complete raw matrices and original identity maps for diagnosis without
    // flooding native test output with 14,400 entries.
    let mut artifact = tempfile::Builder::new()
        .prefix("pse-original-pr-jacobian-")
        .suffix(".json")
        .tempfile()?;
    serde_json::to_writer(artifact.as_file_mut(), &sample)?;
    let (_file, artifact_path) = artifact.keep()?;
    describe(&sample, &artifact_path);
    assert_eq!(sample.cells.len(), DIMENSION * DIMENSION);
    assert!(
        sample.stable_strong >= DIMENSION,
        "too few stable strong derivatives: {}; raw diagnostic {}",
        sample.stable_strong,
        artifact_path.display()
    );
    assert_eq!(
        sample.disagreements,
        0,
        "stable strong analytic/Value disagreements; raw diagnostic {}",
        artifact_path.display()
    );
    assert!(
        sample.selected_root_stable_strong > 0,
        "no stable strong selected-root closure-input Jacobian cells; raw diagnostic {}",
        artifact_path.display()
    );
    for hessian in &sample.hessians {
        assert_eq!(hessian.cells.len(), DIMENSION * DIMENSION);
        assert!(
            hessian.stable_strong > 0,
            "no stable material Hessian subset for {}; raw diagnostic {}",
            hessian.name,
            artifact_path.display()
        );
        assert_eq!(
            hessian.disagreements,
            0,
            "stable strong Hessian/Jacobian-FD disagreements for {}; raw diagnostic {}",
            hessian.name,
            artifact_path.display()
        );
    }
    Ok(())
}
