// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original PR assembly and implicit derivative conditions on the production accuracy basis.
use super::*;
use crate::math::WorkerBudget;
use pse_backend_native::solve::{Controls, Execution, SolveIntent};
use pse_columnar::flight::FlightCancellation;
use pse_kernels::{
    DerivativeOrder, EvaluationContext, ExecutionScope, Provider, ProviderError, ProviderFactory,
    ProviderKey, ProviderRequest, ProviderSpec, ProviderValues, Registration,
};
use pse_math::{
    diagnostics::{MatrixPolicy, analyze_matrix},
    guarded::{CompiledBody, Evaluation, PreparedBody},
    implicit::{Configuration, Factory, ImplicitFactory, Options},
    index::{GlobalCol, GlobalRow},
    normalization::Normalization,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    num::NonZeroUsize,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
const DIMENSION: usize = 120;
#[derive(Deserialize)]
struct Manifest {
    settings: Settings,
    runs: Vec<SourceRun>,
}
#[derive(Deserialize)]
struct Settings {
    memory_limit_bytes: usize,
    threads: usize,
    math_jobs: usize,
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
        settings.preparation.compiler.optimization.cores, 1,
        "each diagnostic native proof uses one optimizer core"
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
        cache: crate::CacheBudget::for_memory(settings.memory_limit_bytes),
        math: crate::math::MathPolicy {
            worker_bytes: settings.math_worker_bytes,
            jobs: settings.math_jobs,
            ..Default::default()
        },
        hashing_may_use_pool: false,
    })?;
    let registry = pse_schema::shared_registry()?;
    let sessions = Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
    let runtime = Runtime::from_shared(
        shared.clone(),
        registry.clone(),
        sessions.clone(),
        crate::workflow::tests::canonical_deployment(),
    );
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
    Ok((
        runtime
            .modeling_from_documents(&closure, physical, &crate::CancelSource::new())
            .await?,
        spill,
    ))
}

/// The current authored seed closure, shared with focused initialization controls.
pub(super) async fn reference_seed_package() -> TestResult<(
    ModelingPackage,
    tempfile::TempDir,
    crate::workflow::PreparationSettings,
)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/reference");
    let manifest: Manifest =
        toml::from_str(&std::fs::read_to_string(root.join("conformance.toml"))?)?;
    let source = manifest
        .runs
        .iter()
        .find(|r| r.name == "seed")
        .ok_or_else(|| std::io::Error::other("reference seed closure absent"))?;
    let (package, spill) = package(&manifest.settings, source, &root).await?;
    Ok((package, spill, manifest.settings.preparation))
}

#[derive(Clone, Serialize)]
struct Member {
    id: SemanticId,
    path: String,
    declaration: Option<DeclarationId>,
}
#[derive(Serialize)]
struct Coordinate {
    member: Member,
    initial: f64,
    nominal: f64,
}
#[derive(Clone, Debug)]
struct ObservedJet {
    inputs: Vec<f64>,
    order: DerivativeOrder,
    values: ProviderValues,
}
type Observations = Arc<Mutex<BTreeMap<ProviderKey, ObservedJet>>>;

/// Observe the existing factory, without changing its scope, configuration or math.
#[derive(Debug)]
struct ObservedFactory {
    original: Registration,
    observations: Observations,
    scope: ExecutionScope,
    source_key: ProviderKey,
}
impl ProviderFactory for ObservedFactory {
    fn spec(&self) -> &ProviderSpec {
        self.original.spec()
    }
    fn configuration_key(&self) -> pse_ids::ContentHash {
        self.original.configuration_key()
    }
    fn envelope(&self) -> Option<Vec<(f64, f64)>> {
        self.original.envelope().map(<[_]>::to_vec)
    }
    fn create(&self) -> Result<Box<dyn Provider>, ProviderError> {
        self.create_scoped(self.scope.clone())
    }
    fn create_scoped(&self, scope: ExecutionScope) -> Result<Box<dyn Provider>, ProviderError> {
        assert!(Arc::ptr_eq(scope.cancellation(), self.scope.cancellation()));
        assert_eq!(scope.deadline(), self.scope.deadline());
        Ok(Box::new(ObservedProvider {
            inner: self.original.worker_scoped(scope)?,
            observations: self.observations.clone(),
            source_key: self.source_key,
        }))
    }
}
#[derive(Debug)]
struct ObservedProvider {
    inner: Box<dyn Provider>,
    observations: Observations,
    source_key: ProviderKey,
}
impl Provider for ObservedProvider {
    fn spec(&self) -> &ProviderSpec {
        self.inner.spec()
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        request.validate(self.spec(), context)?;
        // The implicit factory already computes the full root jet before projecting.
        // Requesting all outputs observes that jet; it adds no perturbed inner solve.
        let all = ProviderRequest::all(self.spec(), request.order);
        all.validate(self.spec(), context)?;
        let full = self.inner.evaluate(inputs, &all, context)?;
        full.validate(self.spec(), &all)?;
        let mut observations = self
            .observations
            .lock()
            .map_err(|_| ProviderError::Contract("PR observation lock poisoned".into()))?;
        let key = self.source_key;
        if observations
            .get(&key)
            .is_none_or(|previous| previous.order <= request.order)
        {
            observations.insert(
                key,
                ObservedJet {
                    inputs: inputs.to_vec(),
                    order: request.order,
                    values: full.clone(),
                },
            );
        }
        drop(observations);
        let n = self.spec().inputs.len();
        let mut projected = ProviderValues {
            values: Vec::new(),
            jacobian: Vec::new(),
            hessians: Vec::new(),
        };
        for &output in &request.outputs {
            projected.values.push(full.values[output]);
            if request.order >= DerivativeOrder::First {
                projected
                    .jacobian
                    .extend_from_slice(&full.jacobian[output * n..(output + 1) * n]);
            }
            if request.order >= DerivativeOrder::Second {
                projected
                    .hessians
                    .extend_from_slice(&full.hessians[output * n * n..(output + 1) * n * n]);
            }
        }
        projected.validate(self.spec(), request)?;
        Ok(projected)
    }
}

#[derive(Clone)]
struct RootCase {
    key: ProviderKey,
    spec: ProviderSpec,
    unknowns: Vec<SemanticId>,
    rows: Vec<SemanticId>,
    residual: Arc<PreparedBody>,
    eligibility: Arc<PreparedBody>,
    factory: Factory,
    fraction: usize,
}
#[derive(Serialize)]
struct RootEvidence {
    provider: SemanticId,
    unknowns: Vec<SemanticId>,
    rows: Vec<SemanticId>,
    parameters: Vec<SemanticId>,
    inputs: Vec<f64>,
    point: Vec<f64>,
    physical_variable_allowances: Vec<f64>,
    physical_row_allowances: Vec<f64>,
    residuals: Vec<f64>,
    first_backward_errors: Vec<f64>,
    /// Canonical assembled linear systems, one per symmetric input pair.
    second_backward_errors: Vec<f64>,
    /// Independently expanded defining equations, including curvature term scale.
    second_condition_backward_errors: Vec<f64>,
    jacobian: Vec<f64>,
    hessians: Vec<f64>,
}
#[derive(Serialize)]
struct MaterialResponse {
    provider: SemanticId,
    input_delta: Vec<f64>,
    observed: Vec<f64>,
    predicted: Vec<f64>,
    allowances: Vec<f64>,
}
#[derive(Clone, Serialize)]
struct RootLink {
    row: usize,
    identity_column: usize,
    provider: SemanticId,
    output: usize,
    input_columns: Vec<usize>,
}
#[derive(Serialize)]
struct Sample {
    #[serde(skip)]
    _allocation: Arc<pse_columnar::AllocationLease>,
    fixture: DeclarationId,
    /// Original admitted case start, not a completed outer steady-state solve.
    rows: Vec<Coordinate>,
    columns: Vec<Coordinate>,
    analytic: Vec<f64>,
    all_original_weighted_hessian: Vec<f64>,
    selected_root_weighted_hessian: Vec<f64>,
    selected_root_weights: Vec<f64>,
    selected_root_links: Vec<RootLink>,
    roots: Vec<RootEvidence>,
    material_response: MaterialResponse,
    linear_backward_error: f64,
    /// Retained production supplier action authority; not a Second certificate.
    supplier_action_accuracy: f64,
    rank: usize,
    rank_cutoff: f64,
    singular_values: Vec<f64>,
    worker_bytes: usize,
    elapsed_seconds: f64,
}

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

/// Same componentwise linear backward error used by the production implicit solver.
/// Published row/unknown coordinate substitutions cancel componentwise in this
/// ratio; parameter scales cancel for each First or Second right-hand side.
fn backward_error(actual: f64, rhs: f64, absolute_terms: f64) -> f64 {
    assert!(actual.is_finite() && rhs.is_finite() && absolute_terms.is_finite());
    let denominator = rhs.abs() + absolute_terms;
    if denominator == 0. {
        assert_eq!(actual, rhs, "a structural zero must remain zero");
        0.
    } else {
        (actual - rhs).abs() / denominator
    }
}

fn evaluate_compact(
    body: &CompiledBody,
    formals: &[f64],
    order: DerivativeOrder,
    scope: &ExecutionScope,
) -> Result<Evaluation, pse_math::MathError> {
    let mut worker = body.worker_scoped(scope.clone());
    let inputs = worker
        .input_formals()
        .iter()
        .map(|&formal| {
            formals.get(formal).copied().ok_or_else(|| {
                pse_math::MathError::Contract("PR primitive compact formal missing".into())
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    // This fixture's primitive residual and hint programs are explicit. Any hidden
    // provider dependency is a capability gap, rather than another nested oracle.
    worker.evaluate(&inputs, order, &mut BTreeMap::new(), scope.cancellation())
}

/// Invoke the retained production resolver on its actual hints and original terms.
/// Nominals condition the equations; variable_tolerance supplies physical accuracy.
fn production_options(
    root: &RootCase,
    inputs: &[f64],
    scope: &ExecutionScope,
) -> Result<Options, pse_math::MathError> {
    match &root.factory.configuration {
        Configuration::Fixed(_, options) => Ok(options.clone()),
        Configuration::Hints(resolver) => {
            let formals = std::iter::repeat_n(0., root.unknowns.len())
                .chain(inputs.iter().copied())
                .collect::<Vec<_>>();
            let hints = if let Some(body) = &root.factory.hints {
                assert!(
                    body.input_formals()
                        .iter()
                        .all(|&f| f >= root.unknowns.len())
                );
                evaluate_compact(body, &formals, DerivativeOrder::Value, scope)?.values
            } else {
                Vec::new()
            };
            let (_, initial) = resolver.resolve(&hints, None, None)?;
            if let Some(body) = &root.factory.terms {
                let nominal = initial
                    .variable_nominals
                    .iter()
                    .copied()
                    .chain(inputs.iter().copied())
                    .collect::<Vec<_>>();
                let terms = evaluate_compact(body, &nominal, DerivativeOrder::Value, scope)?.values;
                Ok(resolver.resolve(&hints, Some(&terms), None)?.1)
            } else {
                Ok(initial)
            }
        }
    }
}

fn primitive(
    root: &RootCase,
    scope: &ExecutionScope,
    optimization: pse_math::library::Optimization,
    limits: pse_math::jets::EvaluationLimits,
) -> Result<(CompiledBody, CompiledBody), pse_math::MathError> {
    assert!(
        root.residual.providers().is_empty(),
        "explicit PR residual required"
    );
    assert!(
        root.eligibility.providers().is_empty(),
        "explicit PR guard required"
    );
    let width = root.unknowns.len() + root.spec.inputs.len();
    let residual = root.residual.compile(
        &(0..root.rows.len()).collect::<Vec<_>>(),
        &(0..width).collect::<Vec<_>>(),
        DerivativeOrder::Second,
        optimization,
        limits,
        scope.cancellation(),
    )?;
    assert_eq!(residual.coordinates(), &(0..width).collect::<Vec<_>>());
    let eligibility = root.eligibility.compile(
        &[0],
        &[],
        DerivativeOrder::Value,
        optimization,
        limits,
        scope.cancellation(),
    )?;
    Ok((residual, eligibility))
}

#[allow(
    clippy::too_many_arguments,
    reason = "independent point oracle receives the exact root, compiled residual and eligibility, physical tolerances, derivative order and scope"
)]
fn qualified_point(
    root: &RootCase,
    residual: &CompiledBody,
    eligibility: &CompiledBody,
    inputs: &[f64],
    point: &[f64],
    options: &Options,
    order: DerivativeOrder,
    scope: &ExecutionScope,
) -> Result<Evaluation, pse_math::MathError> {
    assert_eq!(point.len(), root.unknowns.len());
    assert_eq!(options.variable_tolerance.len(), point.len());
    assert_eq!(options.residual_tolerance.len(), root.rows.len());
    let formals = point
        .iter()
        .copied()
        .chain(inputs.iter().copied())
        .collect::<Vec<_>>();
    let guard = evaluate_compact(eligibility, &formals, DerivativeOrder::Value, scope)?;
    assert_eq!(
        guard.values,
        vec![1.],
        "authored two-phase regime must actually be eligible"
    );
    let fraction = point[root.fraction];
    let allowance = options.variable_tolerance[root.fraction];
    assert!(
        fraction > allowance && fraction < 1. - allowance,
        "selected fraction must be interior beyond production physical allowance"
    );
    let evaluation = evaluate_compact(residual, &formals, order, scope)?;
    for (row, (&value, &allowance)) in evaluation
        .values
        .iter()
        .zip(&options.residual_tolerance)
        .enumerate()
    {
        assert!(
            allowance.is_finite() && allowance > 0. && value.abs() <= allowance,
            "actual inner PR row {} outside production budget: {} > {}",
            root.rows[row],
            value.abs(),
            allowance
        );
    }
    Ok(evaluation)
}

fn root_conditions(
    root: &RootCase,
    jet: &ObservedJet,
    primitive: &Evaluation,
    options: &Options,
    tolerance: f64,
) -> RootEvidence {
    let m = root.unknowns.len();
    let n = root.spec.inputs.len();
    let width = m + n;
    assert_eq!(jet.order, DerivativeOrder::Second);
    assert_eq!(options.derivative_tolerance, tolerance);
    assert_eq!(primitive.jacobian.len(), m * width);
    assert_eq!(primitive.hessians.len(), m * width * width);
    let mut first = Vec::with_capacity(m * n);
    let mut second = Vec::with_capacity(m * n * (n + 1) / 2);
    let mut second_conditions = Vec::with_capacity(m * n * n);
    for row in 0..m {
        for a in 0..n {
            let terms = (0..m)
                .map(|y| primitive.jacobian[row * width + y] * jet.values.jacobian[y * n + a])
                .collect::<Vec<_>>();
            first.push(backward_error(
                terms.iter().sum(),
                -primitive.jacobian[row * width + m + a],
                terms.iter().map(|t| t.abs()).sum(),
            ));
            // The production owner assembles only a <= b, then mirrors the
            // solved jet. Re-summing the reversed pair invents a different
            // cancellation-sensitive RHS which the owner never solved.
            for b in a..n {
                let mut rhs = 0.;
                for i in 0..width {
                    let va = if i < m {
                        jet.values.jacobian[i * n + a]
                    } else {
                        if i == m + a { 1. } else { 0. }
                    };
                    for j in 0..width {
                        let vb = if j < m {
                            jet.values.jacobian[j * n + b]
                        } else {
                            if j == m + b { 1. } else { 0. }
                        };
                        rhs -= primitive.hessians[row * width * width + i * width + j] * va * vb;
                    }
                }
                let terms = (0..m)
                    .map(|y| {
                        primitive.jacobian[row * width + y]
                            * jet.values.hessians[y * n * n + a * n + b]
                    })
                    .collect::<Vec<_>>();
                for y in 0..m {
                    assert_eq!(
                        jet.values.hessians[y * n * n + a * n + b].to_bits(),
                        jet.values.hessians[y * n * n + b * n + a].to_bits(),
                        "production Second jet must retain its mirrored canonical solve"
                    );
                }
                let error =
                    backward_error(terms.iter().sum(), rhs, terms.iter().map(|t| t.abs()).sum());
                assert!(
                    error <= tolerance,
                    "PR canonical Second provider={} row={} inputs=({}, {}) error={error} budget={tolerance}",
                    root.spec.id,
                    root.rows[row],
                    root.spec.inputs[a].id,
                    root.spec.inputs[b].id
                );
                second.push(error);
            }
            for b in 0..n {
                let mut curvature = 0.;
                let mut magnitude = 0.;
                for i in 0..width {
                    let va = if i < m {
                        jet.values.jacobian[i * n + a]
                    } else {
                        f64::from(i == m + a)
                    };
                    for j in 0..width {
                        let vb = if j < m {
                            jet.values.jacobian[j * n + b]
                        } else {
                            f64::from(j == m + b)
                        };
                        let term =
                            va * primitive.hessians[row * width * width + i * width + j] * vb;
                        curvature += term;
                        magnitude += term.abs();
                    }
                }
                let mut linear = 0.;
                for y in 0..m {
                    let term = primitive.jacobian[row * width + y]
                        * jet.values.hessians[y * n * n + a * n + b];
                    linear += term;
                    magnitude += term.abs();
                }
                // This expanded equation has no fixed assembled RHS. Include
                // all contraction terms before cancellation in its denominator.
                assert!(linear.is_finite() && curvature.is_finite() && magnitude.is_finite());
                let error = if magnitude == 0. {
                    assert_eq!(linear + curvature, 0., "structural Second condition zero");
                    0.
                } else {
                    (linear + curvature).abs() / magnitude
                };
                assert!(
                    error <= tolerance,
                    "PR expanded Second provider={} row={} inputs=({}, {}) error={error} budget={tolerance}",
                    root.spec.id,
                    root.rows[row],
                    root.spec.inputs[a].id,
                    root.spec.inputs[b].id
                );
                second_conditions.push(error);
            }
        }
    }
    for (order, errors) in [("First", &first), ("Second", &second)] {
        assert!(
            errors.iter().all(|e| *e <= tolerance),
            "PR {order} implicit equations exceed production backward budget {tolerance}: {errors:?}"
        );
    }
    RootEvidence {
        provider: root.spec.id,
        unknowns: root.unknowns.clone(),
        rows: root.rows.clone(),
        parameters: root.spec.inputs.iter().map(|p| p.id).collect(),
        inputs: jet.inputs.clone(),
        point: jet.values.values.clone(),
        physical_variable_allowances: options.variable_tolerance.clone(),
        physical_row_allowances: options.residual_tolerance.clone(),
        residuals: primitive.values.clone(),
        first_backward_errors: first,
        second_backward_errors: second,
        second_condition_backward_errors: second_conditions,
        jacobian: jet.values.jacobian.clone(),
        hessians: jet.values.hessians.clone(),
    }
}

#[allow(
    clippy::panic,
    reason = "selected-root fixture links must be direct authored aliases; other expressions invalidate the oracle setup"
)]
fn path_symbol(
    expression: &pse_authoring::dsl::Expr,
    names: &BTreeMap<String, SemanticId>,
) -> SemanticId {
    let pse_authoring::dsl::ExprKind::Path(path) = &expression.kind else {
        panic!("original selected-root links require direct authored aliases");
    };
    assert_eq!(path.segments.len(), 1);
    assert!(path.segments[0].indices.is_empty());
    names[&path.segments[0].name]
}

#[expect(
    clippy::print_stderr,
    reason = "bounded named native diagnostic evidence"
)]
fn describe(sample: &Sample, artifact: &Path) {
    eprintln!(
        "original PR production-basis conditions: {}x{} at authored outer start; {} accepted inner roots, {} original links; backward budget={}, elapsed={:.1}s, worker_bytes={}, raw artifact={}",
        sample.rows.len(),
        sample.columns.len(),
        sample.roots.len(),
        sample.selected_root_links.len(),
        sample.linear_backward_error,
        sample.elapsed_seconds,
        sample.worker_bytes,
        artifact.display()
    );
}

#[tokio::test]
async fn original_pr_case_derivatives_satisfy_production_basis_conditions() -> TestResult<()> {
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
    let numerics = resolved.numerics.clone();
    let backward_budget = numerics.policy.linear_backward_error;
    let two_phase = DeclarationId::from(SemanticId::parse_hex("01a0eeced80374ab83c081e54c29cda5")?);
    let fraction_declaration =
        DeclarationId::from(SemanticId::parse_hex("01a0eeced80374ab83c081db14994aa2")?);
    let mut roots = Vec::new();
    for inner in resolved.model.model.compiled().implicit_order()? {
        let alternatives = inner
            .residuals
            .iter()
            .filter(|residual| {
                model
                    .regimes
                    .values()
                    .flat_map(|selection| &selection.alternatives)
                    .any(|regime| regime.id == residual.id && regime.declaration == two_phase)
            })
            .collect::<Vec<_>>();
        if alternatives.is_empty() {
            continue;
        }
        assert_eq!(
            alternatives.len(),
            1,
            "one authored PR two-phase regime per selector"
        );
        let residual = alternatives[0];
        let key = inner.descriptor.spec().key();
        let registration = resolved
            .providers
            .get(&key)
            .ok_or_else(|| std::io::Error::other("actual PR provider registration absent"))?;
        let source = registration
            .source::<pse_math::implicit::reconstruction::ReconstructionFactory>()
            .ok_or_else(|| {
                std::io::Error::other(
                    "actual PR factory source unavailable for production configuration",
                )
            })?;
        let ImplicitFactory::Regimes(factory) = source.factory() else {
            return Err(std::io::Error::other("authored PR regime factory required").into());
        };
        let branch = factory
            .alternatives
            .iter()
            .find(|branch| branch.residual.spec.id == residual.id)
            .ok_or_else(|| std::io::Error::other("actual authored two-phase factory absent"))?;
        assert_eq!(branch.residual.rows, residual.rows);
        assert_eq!(
            inner.unknowns,
            registration
                .spec()
                .outputs
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            branch
                .residual
                .unknowns
                .iter()
                .map(|u| u.id)
                .collect::<Vec<_>>(),
            inner.unknowns
        );
        let fraction = inner
            .unknowns
            .iter()
            .position(|id| model.symbols[id].lineage.declaration == fraction_declaration)
            .ok_or_else(|| std::io::Error::other("authored PR phase fraction absent"))?;
        let assessment = residual
            .assessment
            .as_ref()
            .ok_or_else(|| std::io::Error::other("authored PR two-phase eligibility absent"))?;
        roots.push(RootCase {
            key,
            spec: registration.spec().clone(),
            unknowns: inner.unknowns.clone(),
            rows: residual.rows.clone(),
            residual: residual.body.math().clone(),
            eligibility: assessment.eligibility.math().clone(),
            factory: branch.residual.clone(),
            fraction,
        });
    }
    assert_eq!(roots.len(), 2, "original inlet and outlet PR selectors");
    for root in &roots {
        assert_eq!((root.unknowns.len(), root.spec.inputs.len()), (7, 4));
        for port in &root.spec.inputs {
            assert!(
                column_ids.contains(&port.id),
                "original physical provider input must retain its column identity"
            );
            assert!(resolved.model.values.scalars[&port.id].is_finite());
        }
    }
    let names = model
        .symbols
        .keys()
        .map(|id| (pse_modeling::specialize::symbol_name(*id), *id))
        .collect::<BTreeMap<_, _>>();
    let mut links = Vec::new();
    for (row, member) in row_members
        .iter()
        .enumerate()
        .filter(|(_, m)| selected_root_source(m))
    {
        let equation = &model
            .equations
            .iter()
            .find(|e| e.id == member.id)
            .ok_or_else(|| std::io::Error::other("original selected-root equation absent"))?
            .equation;
        let pse_authoring::dsl::EquationKind::Relation {
            lhs,
            sense: pse_authoring::dsl::EquationSense::Eq,
            rhs,
        } = &equation.kind
        else {
            return Err(std::io::Error::other("original selected-root equality required").into());
        };
        let lhs = path_symbol(lhs, &names);
        let mut rhs = path_symbol(rhs, &names);
        let mut visited = BTreeSet::new();
        while !roots.iter().any(|root| root.unknowns.contains(&rhs)) {
            assert!(visited.insert(rhs), "selected-root alias cycle");
            rhs = path_symbol(
                model.symbols[&rhs].expression.as_ref().ok_or_else(|| {
                    std::io::Error::other("selected-root direct alias unavailable")
                })?,
                &names,
            );
        }
        let root = roots
            .iter()
            .find(|root| root.unknowns.contains(&rhs))
            .unwrap();
        // The admitted fixture uses canonical direct link contributions. Verify that
        // exact interpretation before independently composing the provider derivatives.
        let contributions = plan
            .structure()
            .instances()
            .iter()
            .flat_map(|binding| {
                binding
                    .contributions
                    .iter()
                    .filter(move |c| c.target == pse_math::binding::Target::Row(member.id))
                    .map(move |c| (binding, c))
            })
            .collect::<Vec<_>>();
        assert_eq!(contributions.len(), 1);
        let (binding, contribution) = contributions[0];
        assert_eq!(contribution.scale, 1.);
        for slot in binding.slots.iter() {
            assert_eq!(
                (slot.scale(), slot.offset()),
                (1., 0.),
                "canonical original PR link inputs"
            );
        }
        links.push(RootLink {
            row,
            identity_column: column_ids
                .iter()
                .position(|id| *id == lhs)
                .ok_or_else(|| std::io::Error::other("original link identity coordinate absent"))?,
            provider: root.spec.id,
            output: root.unknowns.iter().position(|id| *id == rhs).unwrap(),
            input_columns: root
                .spec
                .inputs
                .iter()
                .map(|p| column_ids.iter().position(|id| *id == p.id).unwrap())
                .collect(),
        });
    }
    assert_eq!(links.len(), 10, "two original five-row nested root links");
    assert_eq!(
        links
            .iter()
            .map(|link| row_members[link.row].declaration.unwrap())
            .collect::<BTreeSet<_>>()
            .len(),
        3
    );
    for root in &roots {
        assert_eq!(
            links
                .iter()
                .filter(|link| link.provider == root.spec.id)
                .count(),
            5
        );
    }
    let composition_axes = roots
        .iter()
        .map(|root| {
            root.spec
                .inputs
                .iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    model.symbols[&p.id]
                        .lineage
                        .path
                        .ends_with(".z")
                        .then_some(i)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for axes in &composition_axes {
        assert_eq!(axes.len(), 2);
    }
    let base = resolved.model.values.clone();
    let controls = resolved.solver.controls.clone();
    let service = package.runtime.shared.math();
    let executable = service.assemble(resolved.model.case.clone()).await?;
    let worker_allowance = package.runtime.shared.budget().math.worker_bytes;
    let compiler = manifest.settings.preparation.compiler;
    // One ordinary assembly worker, two observed root jets, primitive programs and
    // three raw 120x120 matrices are bounded by the deployment's numeric allowance.
    // This is no longer a 43,200-cell finite-difference qualification campaign.
    let diagnostic_bytes = compiler.evaluation.scratch_bytes;
    let diagnostic_owner =
        service.reserve("math:pr-production-basis-diagnostic", diagnostic_bytes)?;
    let job_diagnostic_owner = diagnostic_owner.clone();
    let budget = WorkerBudget::new(worker_allowance);
    let job_service = service.clone();
    let original_providers = resolved.providers;
    let sample = service.job(1, worker_allowance, FlightCancellation::default(), move |flag| {
        // Cancelling the async waiter does not join the native worker. Its clone
        // keeps the pre-admitted diagnostic storage owned until worker teardown;
        // the waiter's original owner also covers the returned Sample buffers.
        let diagnostic_owner = job_diagnostic_owner;
        let execution = Execution::new(flag.clone(), &controls);
        let scope = execution.scope()?;
        let checkpoint = || scope.check().map_err(pse_backend_native::ProblemError::Provider);
        let observations: Observations = Arc::new(Mutex::new(BTreeMap::new()));
        let mut providers = original_providers.clone();
        for root in &roots {
            let original = original_providers[&root.key].clone();
            // Registration already frames a declared envelope into its configuration
            // key. These actual implicit factories declare none, so delegation is exact.
            assert!(original.envelope().is_none());
            let observed = Registration::bind(original.descriptor(), Arc::new(ObservedFactory {
                original: original.clone(), observations: observations.clone(), scope: scope.clone(), source_key: root.key,
            })).map_err(pse_backend_native::ProblemError::Provider)?;
            assert_eq!(observed.spec(), original.spec());
            assert_eq!(observed.configuration_key(), original.configuration_key());
            providers.insert(root.key, observed);
        }
        let mut owned = job_service.worker(executable, &providers, scope.clone(), &budget)?;
        let worker = owned.worker();
        let worker_bytes = worker.assembly().numeric_worker_bytes();
        let matrix = worker.jacobian(&base).inspect_err(|error| eprintln!("original PR Jacobian: {error}"))?;
        let mut analytic = vec![0.; DIMENSION*DIMENSION];
        for column in 0..DIMENSION {
            for (row,value) in matrix.row_idx_of_col(column).zip(matrix.val_of_col(column)) {
                analytic[row*DIMENSION+column] = *value;
            }
        }
        let evidence = analyze_matrix::<GlobalRow, GlobalCol>(matrix.as_ref(),
            &normalization.rows.iter().map(|v|1./v).collect::<Vec<_>>(),
            &normalization.variables.iter().map(|v|1./v).collect::<Vec<_>>(),
            MatrixPolicy { dense_entries: 3*DIMENSION*DIMENSION, findings: DIMENSION*DIMENSION,
                // Rank interpretation is separate from physical convergence accuracy.
                parallel_tolerance: 1e-8, rank_absolute: 1e-12, rank_relative: 1e-8, singular_vector: 0.1 }, &flag)?;
        checkpoint()?;
        let initial = worker.constraints(&base)?;
        assert_eq!(initial.len(), DIMENSION);
        let all_weights = normalization.rows.iter().enumerate()
            .map(|(row,nominal)| (1.+(row+1) as f64/(DIMENSION+1) as f64)/nominal).collect::<Vec<_>>();
        let root_weights = all_weights.iter().enumerate().map(|(row,w)|
            if links.iter().any(|link|link.row == row) { *w } else { 0. }).collect::<Vec<_>>();
        let mut hessians = Vec::new();
        for weights in [&all_weights, &root_weights] {
            checkpoint()?;
            let matrix = worker.hessian(&base, 0., weights).inspect_err(|error| eprintln!("original PR Hessian: {error}"))?;
            let mut full = vec![0.; DIMENSION*DIMENSION];
            for column in 0..DIMENSION {
                for (row,value) in matrix.row_idx_of_col(column).zip(matrix.val_of_col(column)) {
                    full[row*DIMENSION+column] = *value;
                    full[column*DIMENSION+row] = *value;
                }
            }
            assert!(full.iter().all(|v|v.is_finite()));
            hessians.push(full);
        }
        let jets = observations.lock().map_err(|_| pse_math::MathError::Contract("PR observation lock poisoned".into()))?.clone();
        assert_eq!(jets.len(), 2);
        let mut root_evidence = Vec::new();
        let mut compiled_bytes = 0;
        let mut material_response = None;
        for (index, root) in roots.iter().enumerate() {
            checkpoint()?;
            let jet = &jets[&root.key];
            assert_eq!(jet.inputs, root.spec.inputs.iter().map(|p|base.scalars[&p.id]).collect::<Vec<_>>());
            let (residual, eligibility) = primitive(root, &scope, compiler.optimization, compiler.evaluation)
                .inspect_err(|error| eprintln!("independent PR primitive {:?}: {error}", root.key))?;
            let bytes = residual.retained_bytes()+residual.worker_bytes()+eligibility.retained_bytes()+eligibility.worker_bytes();
            compiled_bytes += bytes;
            let raw_matrices = 3 * DIMENSION * DIMENSION * size_of::<f64>();
            assert!(compiled_bytes + raw_matrices <= diagnostic_bytes,
                "primitive storage and raw matrices retain their single admitted owner");
            let options = production_options(root, &jet.inputs, &scope)?;
            let actual = qualified_point(root, &residual, &eligibility, &jet.inputs,
                &jet.values.values, &options, DerivativeOrder::Second, &scope)?;
            root_evidence.push(root_conditions(root, jet, &actual, &options, backward_budget));
            if index == 0 {
                // One material physical composition change, keeping total composition
                // fixed, replaces hundreds of sub-budget perturbed nested evaluations.
                let axes = &composition_axes[index];
                let input_budget = axes.iter().map(|&axis| crate::workflow::tests::engineering_target(
                    &numerics, pse_relations::generated::enums::NumericalTarget::Variable,
                    root.spec.inputs[axis].id).budget).fold(f64::INFINITY, f64::min);
                let step = 10.*input_budget;
                let mut changed = jet.inputs.clone();
                changed[axes[0]] += step;
                changed[axes[1]] -= step;
                for &axis in axes {
                    let (lower,upper) = bounds[&root.spec.inputs[axis].id];
                    assert!(changed[axis] >= lower && changed[axis] <= upper);
                    assert!(changed[axis] > 0. && changed[axis] < 1.);
                }
                let mut supplier = original_providers[&root.key].worker_scoped(scope.clone())
                    .map_err(pse_backend_native::ProblemError::Provider)?;
                let request = ProviderRequest::all(&root.spec, DerivativeOrder::Value);
                let context = EvaluationContext { cancelled: &flag, max_result_bytes: diagnostic_bytes };
                let changed_values = supplier.evaluate(&changed, &request, &context)
                    .map_err(pse_backend_native::ProblemError::Provider)?;
                changed_values.validate(&root.spec, &request)
                    .map_err(pse_backend_native::ProblemError::Provider)?;
                let changed_options = production_options(root, &changed, &scope)?;
                qualified_point(root, &residual, &eligibility, &changed, &changed_values.values,
                    &changed_options, DerivativeOrder::Value, &scope)?;
                let delta = changed.iter().zip(&jet.inputs).map(|(a,b)|a-b).collect::<Vec<_>>();
                let observed = changed_values.values.iter().zip(&jet.values.values).map(|(a,b)|a-b).collect::<Vec<_>>();
                let predicted = (0..root.unknowns.len()).map(|y| delta.iter().enumerate()
                    .map(|(a,d)|jet.values.jacobian[y*delta.len()+a]*d).sum::<f64>()).collect::<Vec<_>>();
                let allowances = options.variable_tolerance.iter().zip(&changed_options.variable_tolerance)
                    .map(|(a,b)|a+b).collect::<Vec<_>>();
                assert!(observed[root.fraction].abs() > allowances[root.fraction],
                    "composition response must be resolved beyond production accuracy");
                // Empirical response at this step; this is neither a certified Taylor
                // remainder nor a claim that a First action enclosure certifies Second.
                for y in 0..root.unknowns.len() {
                    assert!((observed[y]-predicted[y]).abs() <= allowances[y],
                        "material PR response {} exceeds actual physical production budgets", root.unknowns[y]);
                }
                material_response = Some(MaterialResponse { provider: root.spec.id,
                    input_delta: delta, observed, predicted, allowances });
            }
        }
        let mut expected_hessian = vec![0.; DIMENSION*DIMENSION];
        let mut absolute_hessian_terms = expected_hessian.clone();
        for link in &links {
            let root = roots.iter().find(|root|root.spec.id == link.provider).unwrap();
            let jet = &jets[&root.key];
            let n = root.spec.inputs.len();
            let lhs = base.scalars[&column_ids[link.identity_column]];
            let rhs = jet.values.values[link.output];
            assert!(backward_error(initial[link.row], lhs-rhs, lhs.abs()+rhs.abs()) <= backward_budget);
            for column in 0..DIMENSION {
                let mut expected: f64 = if column == link.identity_column { 1. } else { 0. };
                let mut magnitude = expected.abs();
                for (axis,&input_column) in link.input_columns.iter().enumerate() {
                    if column == input_column {
                        let term = jet.values.jacobian[link.output*n+axis];
                        expected -= term;
                        magnitude += term.abs();
                    }
                }
                assert!(backward_error(analytic[link.row*DIMENSION+column], expected, magnitude) <= backward_budget,
                    "original PR link Jacobian row {} column {}", row_ids[link.row], column_ids[column]);
            }
            for (a,&column_a) in link.input_columns.iter().enumerate() {
                for (b,&column_b) in link.input_columns.iter().enumerate() {
                    let term = -root_weights[link.row]*jet.values.hessians[link.output*n*n+a*n+b];
                    let entry = column_a*DIMENSION+column_b;
                    expected_hessian[entry] += term;
                    absolute_hessian_terms[entry] += term.abs();
                }
            }
        }
        for entry in 0..expected_hessian.len() {
            assert!(backward_error(hessians[1][entry], expected_hessian[entry], absolute_hessian_terms[entry]) <= backward_budget,
                "original PR selected-root weighted Hessian entry {entry}");
        }
        checkpoint()?;
        Ok(Sample {
            _allocation: diagnostic_owner,
            fixture,
            rows: row_members.into_iter().enumerate().map(|(i,member)| Coordinate {
                member, initial: initial[i], nominal: normalization.rows[i] }).collect(),
            columns: column_members.into_iter().enumerate().map(|(i,member)| Coordinate {
                initial: base.scalars[&member.id], member, nominal: normalization.variables[i] }).collect(),
            analytic, all_original_weighted_hessian: hessians.remove(0),
            selected_root_weighted_hessian: hessians.remove(0), selected_root_weights: root_weights,
            selected_root_links: links, roots: root_evidence, material_response: material_response.unwrap(),
            linear_backward_error: backward_budget, supplier_action_accuracy: numerics.policy.supplier_action_accuracy, rank: evidence.rank, rank_cutoff: evidence.cutoff,
            singular_values: evidence.modes.into_iter().map(|m|m.value).collect(),
            worker_bytes, elapsed_seconds: execution.started.elapsed().as_secs_f64(),
        })
    }).await?;
    let mut artifact = tempfile::Builder::new()
        .prefix("pse-original-pr-production-basis-")
        .suffix(".json")
        .tempfile()?;
    serde_json::to_writer(artifact.as_file_mut(), &sample)?;
    let (_file, artifact_path) = artifact.keep()?;
    describe(&sample, &artifact_path);
    assert_eq!(sample.analytic.len(), DIMENSION * DIMENSION);
    assert_eq!(sample.selected_root_links.len(), 10);
    assert_eq!(sample.roots.len(), 2);
    for root in &sample.roots {
        assert_eq!(root.first_backward_errors.len(), 7 * 4);
        assert_eq!(root.second_backward_errors.len(), 7 * 4 * 5 / 2);
        assert_eq!(root.second_condition_backward_errors.len(), 7 * 4 * 4);
    }
    Ok(())
}
