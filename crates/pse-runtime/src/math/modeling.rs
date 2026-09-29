// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite generic preparation uses the existing worker, cancellation and allocation owner.
use super::{MathRuntimeError, MathService, Workspace};
use pse_authoring::language::Declaration;
use pse_columnar::{AllocationLease, flight::FlightCancellation};
use pse_compiler::workspace::PreparedModeling;
use pse_ids::SemanticId;
use pse_model::HeapUsage;
use pse_model::lineage::Solved;
use pse_modeling::{Bindings, DeclarationId, InstanceId, Limits};
use pse_quantity::QuantityTypeId;
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};
/// A source revision retains its reservation through all dependent jobs.
#[derive(Clone, Debug)]
pub struct ModelingRevision {
    admitted: Arc<pse_compiler::workspace::ModelingRevision>,
    identity: pse_ids::ContentHash,
    _lease: Arc<AllocationLease>,
}
impl ModelingRevision {
    pub(crate) fn quantity_names(&self) -> &BTreeMap<String, QuantityTypeId> {
        self.admitted.quantity_names()
    }
    /// What specializing `root` as `instance` solves (`pse_model::lineage`).
    fn solved(
        &self,
        root: DeclarationId,
        instance: InstanceId,
    ) -> Result<Solved, MathRuntimeError> {
        self.admitted.solved(root, instance).ok_or_else(|| {
            MathRuntimeError::Infrastructure(format!(
                "prepared root {root} is not an admitted declaration"
            ))
        })
    }
    pub(crate) fn identity(&self) -> pse_ids::ContentHash {
        self.identity
    }
    pub(crate) fn declarations(&self) -> &[Declaration] {
        self.admitted.declarations()
    }
}
/// Kernel products retain memory after the workspace generation rotates.
#[derive(Clone, Debug)]
pub struct ModelingPreparation {
    product: PreparedModeling,
    solved: Solved,
    _lease: Arc<AllocationLease>,
}
impl ModelingPreparation {
    /// Source lineage, original values, typed mathematics and structural evidence.
    pub fn compiled(&self) -> &PreparedModeling {
        &self.product
    }
    /// The model, case and instance this preparation specialized: what its lineage,
    /// solve rows and numerical requirements name.
    pub const fn solved(&self) -> Solved {
        self.solved
    }
}
/// One model view and its solver projection share the admitted product reservation.
#[derive(Clone, Debug)]
pub struct ModelingCasePreparation {
    /// The specialized model view.
    pub model: ModelingPreparation,
    /// The case's compiled solver projection.
    pub case: super::Preparation,
    /// The case's resolved input values.
    pub values: pse_math::binding::CaseValues,
    /// Inward bound tightenings admission applied to this case's specifications
    /// (ADR-0103 item 4); the solver projection carries the tightened bounds.
    pub tightenings: Vec<pse_modeling::DomainTightening>,
}
/// Bounded original-term evidence retains the scheduler's allocation allowance.
#[derive(Clone, Debug)]
pub struct ModelingTermEvidence {
    /// Term reports of the rows examined, by row.
    pub results: BTreeMap<SemanticId, pse_math::diagnostics::TermReport>,
    /// Rows left unexamined once the shared budget ran out.
    pub unattempted: usize,
    _owner: Arc<AllocationLease>,
}
/// Fully resolved physical input to a nested provider; iteration never acquires another worker.
#[derive(Clone, Debug)]
pub struct ModelingInner {
    /// The checked implicit system.
    pub admitted: Arc<pse_compiler::workspace::AdmittedImplicit>,
    /// Numerical configuration of each residual alternative, by residual.
    pub configurations: BTreeMap<SemanticId, pse_math::implicit::Configuration>,
}
#[cfg(not(feature = "solver-kinsol"))]
#[derive(Debug)]
struct MissingInnerSolver;
#[cfg(not(feature = "solver-kinsol"))]
impl pse_math::implicit::InnerSolver for MissingInnerSolver {
    fn identity(&self) -> pse_ids::ContentHash {
        pse_math::implicit::solver_identity("missing.inner-solver.v1")
    }
    fn solve(
        &self,
        _: Arc<pse_math::implicit::Problem>,
        _: &[f64],
        _: &pse_math::implicit::Options,
        _: &Arc<AtomicBool>,
    ) -> Result<Vec<f64>, pse_math::MathError> {
        Err(pse_math::MathError::Contract(
            "nested realization requires the KINSOL capability".into(),
        ))
    }
}
impl MathService {
    /// Run authored pure expectations through the compiler without acquiring a solver.
    #[expect(
        clippy::too_many_arguments,
        reason = "one compiler job receives the workspace and revision with the specialization request, profile and cancellation driver"
    )]
    pub async fn modeling_expectations(
        self: &Arc<Self>,
        workspace: Workspace,
        revision: ModelingRevision,
        root: DeclarationId,
        bindings: Bindings,
        limits: Limits,
        profile: pse_compiler::workspace::Profile,
        driver: &crate::CancelSource,
    ) -> Result<Vec<pse_compiler::workspace::ModelingExpectationResult>, MathRuntimeError> {
        let control = FlightCancellation::default();
        let operation = self.job(1, super::WITHIN_WORKSPACE, control.clone(), move |flag| {
            let _lease = workspace.lease;
            let mut compiler = workspace
                .compiler
                .lock()
                .map_err(|_| MathRuntimeError::Infrastructure("compiler lock poisoned".into()))?;
            compiler.publish_modeling_revision(revision.admitted.clone())?;
            Ok(compiler.check_modeling_expectations(
                root,
                pse_modeling::specialize::root_instance(root),
                bindings,
                limits,
                &pse_math::binding::CaseValues {
                    scalars: BTreeMap::new(),
                },
                profile,
                flag,
            )?)
        });
        tokio::pin!(operation);
        tokio::select! { result = &mut operation => result, () = driver.cancelled() => {
            control.cancel(); let _ = operation.await; Err(MathRuntimeError::Cancelled)
        }}
    }
    /// Compile immutable inner programs under the same admission and allocation policy as outer artifacts.
    pub async fn modeling_inner_providers(
        self: &Arc<Self>,
        inner: Vec<ModelingInner>,
        accelerators: Arc<pse_math::implicit::accelerators::Accelerators>,
        external: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: pse_compiler::workspace::Profile,
        driver: &crate::CancelSource,
    ) -> Result<BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>, MathRuntimeError>
    {
        if inner.is_empty() {
            return Ok(external);
        }
        let control = FlightCancellation::default();
        let foreign = self.policy.foreign_bytes;
        let operation =
            self.job_retained(1, self.policy.worker_bytes, control.clone(), move |flag| {
                #[cfg(feature = "solver-kinsol")]
                let solver: Arc<dyn pse_math::implicit::InnerSolver> =
                    Arc::new(pse_backend_native::implicit::Kinsol);
                #[cfg(not(feature = "solver-kinsol"))]
                let solver: Arc<dyn pse_math::implicit::InnerSolver> = Arc::new(MissingInnerSolver);
                let mut factories = Vec::new();
                let mut retained = foreign;
                for item in inner {
                    #[cfg(not(feature = "solver-kinsol"))]
                    if item.admitted.algorithm == pse_compiler::workspace::ImplicitAlgorithm::Native
                    {
                        return Err(MathRuntimeError::Infrastructure(
                            "nested realization requires the KINSOL capability".into(),
                        ));
                    }
                    let factory = item.admitted.factory(
                        item.configurations,
                        solver.clone(),
                        &accelerators,
                        flag.clone(),
                        profile.evaluation,
                    )?;
                    retained = retained
                        .checked_add(factory.retained_numeric_bytes()?)
                        .ok_or(MathRuntimeError::Limit("inner program extent"))?;
                    let dependencies = item
                        .admitted
                        .bodies()
                        .flat_map(|b| b.math.providers())
                        .map(pse_kernels::ProviderSpec::key)
                        .collect::<Vec<_>>();
                    factories.push((item.admitted.descriptor.clone(), factory, dependencies));
                }
                Ok((factories, retained))
            });
        tokio::pin!(operation);
        let (factories, owner) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        let mut registrations = external;
        for (descriptor, mut factory, dependencies) in factories {
            let providers = dependencies
                .into_iter()
                .map(|key| {
                    registrations
                        .get(&key)
                        .cloned()
                        .map(|r| (key, r))
                        .ok_or_else(|| {
                            MathRuntimeError::Infrastructure(
                                "implicit provider dependency is not admitted before its consumer"
                                    .into(),
                            )
                        })
                })
                .collect::<Result<_, _>>()?;
            factory.set_providers(providers);
            factory.retain(owner.clone());
            let key = descriptor.spec().key();
            let registration = pse_kernels::Registration::bind(descriptor, Arc::new(factory))
                .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?;
            if registrations.insert(key, registration).is_some() {
                return Err(MathRuntimeError::Infrastructure(
                    "duplicate inner provider identity".into(),
                ));
            }
        }
        Ok(registrations)
    }
    /// Execute combinatorial term inspection on an admitted math worker, with a
    /// single work budget across every row rather than resetting it for each equation.
    pub async fn modeling_term_diagnostics(
        self: &Arc<Self>,
        terms: BTreeMap<SemanticId, Vec<f64>>,
        policy: pse_math::diagnostics::TermPolicy,
        driver: &crate::CancelSource,
    ) -> Result<ModelingTermEvidence, MathRuntimeError> {
        let control = FlightCancellation::default();
        let bytes = self.policy.worker_bytes;
        let operation = self.job_retained(1, bytes, control.clone(), move |flag| {
            let input_bytes = terms
                .values()
                .try_fold(0usize, |n, v| n.checked_add(v.len().checked_mul(32)?))
                .ok_or(MathRuntimeError::Limit("term diagnostic input extent"))?;
            if input_bytes > bytes / 2 {
                return Err(MathRuntimeError::Limit("term diagnostic input extent"));
            }
            let count = terms.len();
            let mut remaining = policy.combinations;
            let mut findings = policy.findings;
            let mut results = BTreeMap::new();
            for (id, values) in terms {
                if remaining == 0 || findings == 0 {
                    break;
                }
                let p = pse_math::diagnostics::TermPolicy {
                    combinations: remaining,
                    findings,
                    ..policy
                };
                let report = pse_math::diagnostics::analyze_terms(&values, p, &flag)?;
                remaining = remaining.saturating_sub(report.examined);
                findings = findings.saturating_sub(report.cancellations.len());
                results.insert(id, report);
            }
            let retained = results
                .values()
                .try_fold(0usize, |n, r| {
                    n.checked_add(
                        r.cancellations
                            .iter()
                            .map(|v| v.capacity() * size_of::<usize>())
                            .sum::<usize>()
                            + r.cancellations.capacity() * size_of::<Vec<usize>>()
                            + r.mismatched.capacity() * size_of::<usize>()
                            + 256,
                    )
                })
                .ok_or(MathRuntimeError::Limit("term diagnostic output extent"))?;
            let unattempted = count - results.len();
            Ok(((results, unattempted), retained))
        });
        tokio::pin!(operation);
        let ((results, unattempted), owner) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        Ok(ModelingTermEvidence {
            results,
            unattempted,
            _owner: owner,
        })
    }
    pub(crate) async fn validate_modeling_partition(
        self: &Arc<Self>,
        workspace: Workspace,
        model: ModelingPreparation,
        rows: Vec<SemanticId>,
        columns: Vec<SemanticId>,
        profile: pse_compiler::workspace::Profile,
        driver: &crate::CancelSource,
    ) -> Result<(), MathRuntimeError> {
        if rows.is_empty() && columns.is_empty() {
            return Ok(());
        }
        let control = FlightCancellation::default();
        let operation = self.job(
            1,
            pse_structural::incidence::MATCHING_STACK,
            control.clone(),
            move |flag| {
                let _lease = workspace.lease;
                let compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                let analysis = compiler.analyze_modeling_partition(
                    model.compiled(),
                    rows,
                    columns,
                    profile,
                    &flag,
                )?;
                pse_backend_native::structural::admit(
                    &analysis,
                    pse_backend_native::structural::Mode::Roots,
                )?;
                Ok(())
            },
        );
        tokio::pin!(operation);
        tokio::select! {r=&mut operation=>r,()=driver.cancelled()=>{control.cancel();let _=operation.await;Err(MathRuntimeError::Cancelled)}}
    }
    /// Compile function roles with ordered state/parameter derivatives using the
    /// ordinary artifact cache; source ownership remains in the modeling revision.
    #[expect(
        clippy::too_many_arguments,
        reason = "one compiler job receives the workspace and model with the selected rows, coordinates, order, profile and cancellation driver"
    )]
    pub async fn prepare_modeling_functions(
        self: &Arc<Self>,
        workspace: Workspace,
        model: ModelingPreparation,
        rows: Vec<SemanticId>,
        coordinates: Vec<SemanticId>,
        order: pse_kernels::DerivativeOrder,
        profile: pse_compiler::workspace::Profile,
        driver: &crate::CancelSource,
    ) -> Result<Arc<super::ExecutableCase>, MathRuntimeError> {
        let control = FlightCancellation::default();
        let foreign = self.policy.foreign_bytes;
        let operation =
            self.job_retained(1, super::WITHIN_WORKSPACE, control.clone(), move |flag| {
                let _lease = workspace.lease;
                let compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                let prepared = compiler.prepare_modeling_functions(
                    model.compiled(),
                    rows,
                    coordinates,
                    order,
                    profile,
                    &flag,
                )?;
                let bytes = prepared
                    .plan
                    .retained_bytes()
                    .checked_add(foreign)
                    .ok_or(MathRuntimeError::Limit("modeling function extent"))?;
                Ok((prepared, bytes))
            });
        tokio::pin!(operation);
        let (prepared, lease) = tokio::select! {r=&mut operation=>r?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        self.assemble_functions(prepared, lease, driver).await
    }
    /// Compile the parametric program of a prepared solver view over `parameters`, in
    /// request order (Plan 22 S1), through the compiler and artifact cache. It depends on no
    /// value, so one program serves every value rebind of the view (A6).
    pub async fn prepare_modeling_parametric(
        self: &Arc<Self>,
        workspace: Workspace,
        view: super::Preparation,
        parameters: Vec<SemanticId>,
        profile: pse_compiler::workspace::Profile,
        driver: &crate::CancelSource,
    ) -> Result<Arc<super::ExecutableCase>, MathRuntimeError> {
        let control = FlightCancellation::default();
        let foreign = self.policy.foreign_bytes;
        let operation =
            self.job_retained(1, super::WITHIN_WORKSPACE, control.clone(), move |flag| {
                let _lease = workspace.lease;
                let compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                let prepared = compiler.prepare_modeling_parametric(
                    view.compiled(),
                    &parameters,
                    profile,
                    &flag,
                )?;
                let bytes = prepared
                    .plan
                    .retained_bytes()
                    .checked_add(foreign)
                    .ok_or(MathRuntimeError::Limit("parametric program extent"))?;
                Ok((prepared, bytes))
            });
        tokio::pin!(operation);
        let (prepared, lease) = tokio::select! {r=&mut operation=>r?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        self.assemble_functions(prepared, lease, driver).await
    }
    /// Compile a value-independent observation program for `rows` through the compiler
    /// and artifact cache. Every evaluation binds its own values (A6).
    pub async fn prepare_modeling_observations(
        self: &Arc<Self>,
        workspace: Workspace,
        model: ModelingPreparation,
        rows: std::collections::BTreeSet<SemanticId>,
        profile: pse_compiler::workspace::Profile,
        driver: &crate::CancelSource,
    ) -> Result<Arc<super::ExecutableCase>, MathRuntimeError> {
        let control = FlightCancellation::default();
        let foreign = self.policy.foreign_bytes;
        let operation =
            self.job_retained(1, super::WITHIN_WORKSPACE, control.clone(), move |flag| {
                let _lease = workspace.lease;
                let compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                let product = compiler.prepare_modeling_observations(
                    model.compiled(),
                    &rows,
                    profile,
                    &flag,
                )?;
                let bytes = product
                    .plan
                    .retained_bytes()
                    .checked_add(foreign)
                    .ok_or(MathRuntimeError::Limit("observation extent"))?;
                Ok((product, bytes))
            });
        tokio::pin!(operation);
        let (product, lease) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        self.preparations
            .observations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.assemble_functions(product, lease, driver).await
    }
    /// Prepare the solver view of a bound structure and bind its first values: one
    /// structural preparation (A6). Later values rebind it ([`Self::rebind`]).
    #[expect(
        clippy::too_many_arguments,
        reason = "one compiler job receives the workspace and model with the structure, values, order, profile and cancellation driver"
    )]
    pub async fn prepare_modeling_view(
        self: &Arc<Self>,
        workspace: Workspace,
        model: ModelingPreparation,
        structure: Arc<pse_math::binding::CaseStructure>,
        values: pse_math::binding::CaseValues,
        order: pse_kernels::DerivativeOrder,
        profile: pse_compiler::workspace::Profile,
        driver: &crate::CancelSource,
    ) -> Result<super::Preparation, MathRuntimeError> {
        let control = FlightCancellation::default();
        let foreign = self.policy.foreign_bytes;
        let operation =
            self.job_retained(1, super::WITHIN_WORKSPACE, control.clone(), move |flag| {
                let _lease = workspace.lease;
                let compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                let product = compiler.prepare_modeling_view(
                    model.compiled(),
                    structure,
                    &values,
                    order,
                    profile,
                    &flag,
                )?;
                let bytes = product
                    .retained_bytes()
                    .checked_add(foreign)
                    .ok_or(MathRuntimeError::Limit("bound case extent"))?;
                Ok((product, bytes))
            });
        tokio::pin!(operation);
        let owned = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        self.preparations
            .views
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.own_preparation(owned)
    }
    /// Value-only rebind of a prepared view (A6). Nothing runs when no value the derived
    /// realization parameters (ADR-0104) or the value-dependent products consumed changed;
    /// otherwise only those are rebuilt, on an admitted worker, and the structure, its
    /// derivation rules and its programs stay shared.
    ///
    /// # Errors
    /// Values that do not bind the structure, a refused derived parameter, a failed
    /// projection or cancellation.
    pub async fn rebind(
        self: &Arc<Self>,
        prepared: &super::Preparation,
        values: pse_math::binding::CaseValues,
        driver: &crate::CancelSource,
    ) -> Result<super::Preparation, MathRuntimeError> {
        use std::sync::atomic::Ordering::Relaxed;
        let compiled = prepared.prepared.clone();
        if compiled.values_match(&values) {
            self.preparations.shared.fetch_add(1, Relaxed);
            // Derived realization parameters are unchanged (ADR-0104); they complete the
            // values the recorded assumptions are compared with.
            let completed = compiled.derived.complete(&values);
            if compiled
                .coefficient_values
                .iter()
                .all(|(id, bits)| completed.scalars.get(id).map(|v| v.to_bits()) == Some(*bits))
            {
                compiled
                    .plan
                    .structure()
                    .validate_frozen_values(&completed)?;
                return Ok(prepared.clone());
            }
            // Every product is shared; only the recorded fixed and parameter values follow
            // the new values, which needs no worker.
            let rebound = compiled.rebind(&values, &Arc::new(AtomicBool::new(false)))?;
            return Ok(Self::own_rebind(
                prepared,
                rebound,
                self.reserve("math:rebind", 0)?,
            ));
        }
        let control = FlightCancellation::default();
        let operation =
            self.job_retained(1, self.policy.worker_bytes, control.clone(), move |flag| {
                let rebound = compiled.rebind(&values, &flag)?;
                let bytes = rebound.presolve.bytes()
                    + rebound
                        .coefficients
                        .as_ref()
                        .map_or(0, |c| c.retained_bytes())
                    + rebound.derived.retained_bytes();
                Ok((rebound, bytes))
            });
        tokio::pin!(operation);
        let (rebound, lease) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        self.preparations.rebuilt.fetch_add(1, Relaxed);
        Ok(Self::own_rebind(prepared, rebound, lease))
    }
    /// Own immutable generated declarations under the deployment pool.
    pub fn modeling_revision(
        &self,
        workspace: &Workspace,
        rows: Vec<Declaration>,
        names: BTreeMap<String, QuantityTypeId>,
    ) -> Result<ModelingRevision, MathRuntimeError> {
        let bytes = rows
            .owned_bytes()
            .saturating_add(names.keys().map(|n| n.capacity() + 128).sum::<usize>());
        if bytes > self.policy.workspace_bytes / 2 {
            return Err(MathRuntimeError::Limit("modeling source bytes"));
        }
        // The source rows are charged before building checked state inside the
        // workspace's lease; the retained lease then takes the revision's actual extent.
        let reservation =
            pse_columnar::MemoryConsumer::new("modeling:source-revision").register(&self.pool);
        reservation.try_grow(bytes)?;
        use pse_model::SemanticFrame;
        let mut source = pse_ids::FramedHasher::new(pse_ids::Frame::ModelingSourceRevisionV1);
        source.u64(rows.len() as u64);
        for row in &rows {
            row.frame(&mut source);
        }
        source.u64(names.len() as u64);
        for (name, quantity) in &names {
            source.str(name).id(&quantity.as_id());
        }
        let admitted = workspace
            .compiler
            .lock()
            .map_err(|_| MathRuntimeError::Infrastructure("compiler lock poisoned".into()))?
            .publish_modeling(rows, names)?;
        if admitted.retained_bytes() > self.policy.workspace_bytes / 2 {
            return Err(MathRuntimeError::Limit("modeling admitted source bytes"));
        }
        reservation.try_resize(admitted.retained_bytes())?;
        Ok(ModelingRevision {
            identity: source.finish_hash(),
            admitted,
            _lease: AllocationLease::new(reservation),
        })
    }
    /// Publish the selected revision and prepare it while holding the single compiler writer.
    #[expect(
        clippy::too_many_arguments,
        reason = "one compiler job receives the workspace and revision with the specialization request and cancellation driver"
    )]
    pub async fn prepare_modeling_revision(
        self: &Arc<Self>,
        workspace: Workspace,
        revision: ModelingRevision,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        driver: &crate::CancelSource,
    ) -> Result<ModelingPreparation, MathRuntimeError> {
        let lineage = revision.clone();
        let control = FlightCancellation::default();
        let foreign = self.policy.foreign_bytes;
        let operation =
            self.job_retained(1, super::WITHIN_WORKSPACE, control.clone(), move |flag| {
                let _workspace_lease = workspace.lease;
                let mut compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                if flag.load(std::sync::atomic::Ordering::Acquire) {
                    return Err(MathRuntimeError::Cancelled);
                }
                compiler.publish_modeling_revision(revision.admitted.clone())?;
                let product = compiler
                    .prepare_modeling_cancellable(root, instance, bindings, limits, flag)?;
                let bytes = product
                    .retained_bytes()
                    .checked_add(foreign)
                    .ok_or(MathRuntimeError::Limit("modeling product extent"))?;
                Ok((product, bytes))
            });
        tokio::pin!(operation);
        let (product, lease) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        let solved = lineage.solved(root, instance)?;
        Ok(ModelingPreparation {
            product,
            solved,
            _lease: lease,
        })
    }
}
