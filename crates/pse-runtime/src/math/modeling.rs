// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite generic preparation uses the existing worker, cancellation and allocation owner.
pub(crate) use super::preparation::{BasisKey, OwnedModelingFrontier, PreparedBasis};
use super::{MathRuntimeError, MathService, Workspace};
use pse_authoring::language::Declaration;
use pse_columnar::{AllocationLease, flight::FlightCancellation};
use pse_compiler::workspace::PreparedModeling;
use pse_ids::SemanticId;
use pse_model::HeapUsage;
use pse_model::lineage::Solved;
use pse_modeling::{Bindings, DeclarationId, InstanceId, Limits};
use std::{collections::BTreeMap, sync::Arc};
/// A source revision retains its reservation through all dependent jobs.
#[derive(Clone, Debug)]
pub struct ModelingRevision {
    pub(super) admitted: Arc<pse_compiler::workspace::ModelingRevision>,
    identity: pse_ids::roles::SourceRevisionHash,
    _lease: Arc<AllocationLease>,
}
impl ModelingRevision {
    pub(crate) fn retained_bytes(&self) -> usize {
        self.admitted.retained_bytes()
    }
    #[cfg(test)]
    pub(crate) fn same_admission(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.admitted, &other.admitted)
    }
    /// What specializing `root` as `instance` solves (`pse_model::lineage`).
    pub(super) fn solved(
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
    pub(crate) fn identity(&self) -> pse_ids::roles::SourceRevisionHash {
        self.identity
    }
    pub(crate) fn declarations(&self) -> &[Declaration] {
        self.admitted.declarations()
    }
    /// The package data documents the declarations were admitted with (ADR-0125).
    #[cfg(test)]
    pub(crate) fn documents(&self) -> &Arc<pse_modeling::document::DocumentInventory> {
        self.admitted.documents()
    }
    /// The admitted package: checked declarations, entities and tables.
    pub(crate) fn checked(&self) -> &pse_modeling::CheckedPackage {
        self.admitted.checked()
    }
    /// The source entity a test names as the source of its expected values (ADR-0123
    /// Outcome 5).
    pub(crate) fn oracle(&self, test: DeclarationId) -> Option<DeclarationId> {
        self.admitted.oracle(test)
    }
    /// The release an oracle's values come from (Plan 23 H6).
    pub(crate) fn release_of(&self, oracle: DeclarationId) -> Option<DeclarationId> {
        self.admitted.release_of(oracle)
    }
}
/// The identity of a modeling source revision (ADR-0123 Outcome 8): the structured
/// declaration rows in order; each package data document's identity and the hash of its
/// exact bytes, in identity order (ADR-0125), so one changed data byte is another revision;
/// the identity of the physical inventory the rows are admitted against; and the physical
/// name bindings admission resolves them with.
pub(crate) fn source_revision(
    rows: &[Declaration],
    documents: &pse_modeling::document::DocumentInventory,
    physical: &pse_ids::ContentHash,
    names: &BTreeMap<String, SemanticId>,
) -> pse_ids::roles::SourceRevisionHash {
    use pse_model::SemanticFrame;
    let mut source = pse_ids::FramedHasher::new(pse_ids::Frame::ModelingSourceRevisionV4);
    source.u64(rows.len() as u64);
    for row in rows {
        row.frame(&mut source);
    }
    source.u64(documents.documents.len() as u64);
    for (id, document) in &documents.documents {
        source.id(id).hash(&document.content_hash);
    }
    source.hash(physical);
    source.u64(names.len() as u64);
    for (name, id) in names {
        source.str(name).id(id);
    }
    pse_ids::roles::SourceRevisionHash::from_id(source.finish_hash())
}
/// Kernel products retain memory after the workspace generation rotates.
#[derive(Clone, Debug)]
pub struct ModelingPreparation {
    pub(super) product: PreparedModeling,
    pub(super) solved: Solved,
    pub(super) consumed_sources: Arc<BTreeMap<String, String>>,
    pub(super) _owner: Arc<super::products::ProductOwner>,
    pub(super) _source_owner: Option<Arc<AllocationLease>>,
}
impl ModelingPreparation {
    pub(crate) fn with_consumed_source_versions(
        mut self,
        versions: Arc<BTreeMap<String, String>>,
        owner: Arc<AllocationLease>,
    ) -> Self {
        self.consumed_sources = versions;
        self._source_owner = Some(owner);
        self
    }
    /// Exact immutable source objects consumed before scientific preparation.
    pub(crate) fn consumed_source_versions(&self) -> &BTreeMap<String, String> {
        &self.consumed_sources
    }
    /// Source lineage, original values, typed mathematics and structural evidence.
    pub fn compiled(&self) -> &PreparedModeling {
        &self.product
    }
    /// The admitted selected-supplier equation view, retaining the same product owner.
    pub(crate) fn original_equations(&self) -> Option<Self> {
        Some(Self {
            product: self.product.original_equations()?,
            ..self.clone()
        })
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
    pub admitted: pse_math::SharedAllocation<pse_compiler::workspace::AdmittedImplicit>,
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
    fn minimum_order(&self) -> pse_kernels::DerivativeOrder {
        pse_kernels::DerivativeOrder::First
    }
    fn solve(
        &self,
        _: Arc<pse_math::implicit::Problem>,
        _: &[f64],
        _: &pse_math::implicit::Options,
        _: &Arc<std::sync::atomic::AtomicBool>,
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
    pub async fn modeling_point(
        self: &Arc<Self>,
        workspace: Workspace,
        revision: ModelingRevision,
        root: DeclarationId,
        bindings: Bindings,
        limits: Limits,
        profile: pse_compiler::workspace::Profile,
        driver: &crate::CancelSource,
    ) -> Result<pse_compiler::workspace::ModelingPointChecks, MathRuntimeError> {
        let control = FlightCancellation::default();
        let operation = self.job_scoped(
            1,
            super::WITHIN_WORKSPACE,
            control.clone(),
            driver.deadline(),
            move |flag| {
                let _lease = workspace.lease;
                let mut compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                compiler.publish_modeling_revision(revision.admitted.clone())?;
                Ok(compiler.check_modeling_point(
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
            },
        );
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
        mut provider_demands: BTreeMap<pse_kernels::ProviderKey, pse_kernels::DerivativeOrder>,
        profile: pse_compiler::workspace::Profile,
        driver: &crate::CancelSource,
    ) -> Result<BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>, MathRuntimeError>
    {
        if inner.is_empty() {
            return Ok(external);
        }
        let demand_entry =
            size_of::<(pse_kernels::ProviderKey, pse_kernels::DerivativeOrder)>() + 128;
        let mut analysis =
            provider_demands
                .len()
                .checked_mul(demand_entry)
                .and_then(|bytes| {
                    bytes.checked_add(inner.len().checked_mul(
                        size_of::<pse_kernels::DerivativeRequirements>() + demand_entry,
                    )?)
                })
                .ok_or(MathRuntimeError::Limit("inner analysis extent"))?;
        for item in &inner {
            let Some(bound) = item.admitted.requirements_allocation_bound()? else {
                analysis = self.policy.worker_bytes;
                break;
            };
            analysis = analysis
                .checked_add(bound)
                .ok_or(MathRuntimeError::Limit("inner analysis extent"))?;
        }
        if analysis > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit("inner analysis capacity"));
        }
        let control = FlightCancellation::default();
        let operation = async {
            #[cfg(feature = "solver-kinsol")]
            let solver: Arc<dyn pse_math::implicit::InnerSolver> =
                Arc::new(pse_backend_native::implicit::Kinsol);
            #[cfg(not(feature = "solver-kinsol"))]
            let solver: Arc<dyn pse_math::implicit::InnerSolver> = Arc::new(MissingInnerSolver);
            #[cfg(feature = "solver-root-isolation")]
            let verifier: Option<Arc<dyn pse_math::implicit::SelectionVerifier>> =
                Some(Arc::new(pse_backend_native::root_isolation::Ibex));
            #[cfg(not(feature = "solver-root-isolation"))]
            let verifier: Option<Arc<dyn pse_math::implicit::SelectionVerifier>> = None;
            // Capability and dependency analysis can construct affine/accelerator
            // support. Admit its working set separately from the source products
            // before selecting the actual factory compilation population.
            let worker_bytes = self.policy.worker_bytes;
            let (
                (inner, solver, verifier, accelerators, provider_demands, demand),
                _analysis_owner,
            ) = self
                .job_retained_scoped(
                    1,
                    analysis,
                    control.clone(),
                    driver.deadline(),
                    move |flag| {
                        // Inputs are in dependency order. Propagate the actual consumer demands
                        // backwards before compiling any provider, including the inner adapter's
                        // residual minimum and explicitly authored partial derivatives.
                        let mut resolved = Vec::with_capacity(inner.len());
                        for item in inner.iter().rev() {
                            let key = item.admitted.descriptor.spec().key();
                            let requested = provider_demands
                                .get(&key)
                                .copied()
                                .unwrap_or(pse_kernels::DerivativeOrder::Value);
                            let requirements = item.admitted.requirements(
                                requested,
                                solver.minimum_order(),
                                verifier.as_deref(),
                                &accelerators,
                                &flag,
                                profile.evaluation,
                            )?;
                            resolved.push(requirements);
                            for (dependency, required) in
                                item.admitted.provider_demands(requirements)?
                            {
                                provider_demands
                                    .entry(dependency)
                                    .and_modify(|order| *order = (*order).max(required))
                                    .or_insert(required);
                            }
                        }
                        resolved.reverse();
                        let mut demand = 0usize;
                        for (item, requirements) in inner.iter().zip(resolved) {
                            let Some(construction) =
                                item.admitted.reconstruction_allocation_bound(
                                    &item.configurations,
                                    requirements,
                                    profile.evaluation,
                                )?
                            else {
                                // Unknown control/provider populations retain conservative admission.
                                demand = worker_bytes;
                                break;
                            };
                            demand = demand
                                .checked_add(construction)
                                .ok_or(MathRuntimeError::Limit("inner construction extent"))?;
                        }
                        if demand > worker_bytes {
                            return Err(MathRuntimeError::Limit("inner construction capacity"));
                        }
                        // Keep the propagated demand map admitted across the phase boundary.
                        let retained = provider_demands
                            .len()
                            .checked_mul(
                                size_of::<(pse_kernels::ProviderKey, pse_kernels::DerivativeOrder)>(
                                ) + 128,
                            )
                            .ok_or(MathRuntimeError::Limit("inner demand extent"))?;
                        Ok((
                            (
                                inner,
                                solver,
                                verifier,
                                accelerators,
                                provider_demands,
                                demand,
                            ),
                            retained,
                        ))
                    },
                )
                .await?;
            // Opaque native Rational/optimizer construction retains the separate
            // foreign allowance; the known payload follows actual scientific demand.
            self.job_retained_scoped(1, demand, control.clone(), driver.deadline(), move |flag| {
                let mut factories = Vec::new();
                let mut retained = 0usize;
                for item in inner {
                    let source_key = item.admitted.descriptor.spec().key();
                    let requested_output = provider_demands
                        .get(&source_key)
                        .copied()
                        .unwrap_or(pse_kernels::DerivativeOrder::Value);
                    #[cfg(not(feature = "solver-kinsol"))]
                    if item.admitted.algorithm == pse_compiler::workspace::ImplicitAlgorithm::Native
                    {
                        return Err(MathRuntimeError::Infrastructure(
                            "nested realization requires the KINSOL capability".into(),
                        ));
                    }
                    let factory = item.admitted.reconstruction_factory(
                        item.configurations,
                        pse_compiler::workspace::ImplicitCapabilities {
                            solver: solver.clone(),
                            verifier: verifier.clone(),
                            accelerators: &accelerators,
                        },
                        requested_output,
                        flag.clone(),
                        profile.evaluation,
                    )?;
                    retained = retained
                        .checked_add(factory.retained_bytes()?)
                        .ok_or(MathRuntimeError::Limit("inner program extent"))?;
                    let dependencies = item
                        .admitted
                        .bodies()
                        .flat_map(|b| b.math().providers())
                        .map(pse_kernels::ProviderSpec::key)
                        .collect::<Vec<_>>();
                    let descriptor = item
                        .admitted
                        .descriptor
                        .restrict_order(requested_output)
                        .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?;
                    factories.push((source_key, descriptor, factory, dependencies));
                }
                Ok((factories, retained))
            })
            .await
        };
        tokio::pin!(operation);
        let (factories, owner) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        let mut registrations = external;
        for (source_key, descriptor, mut factory, dependencies) in factories {
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
            let registration = pse_kernels::Registration::bind(descriptor, Arc::new(factory))
                .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?;
            if registrations.insert(source_key, registration).is_some() {
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
        // Charge construction, simultaneous earlier reports and the transported input.
        // Summing per-row peaks is conservative: actual findings share one work allowance.
        let bytes = terms.values().try_fold(0usize, |bytes, values| {
            let peak = pse_math::diagnostics::term_allocation_bound(values, policy)?;
            bytes
                .checked_add(peak)
                .and_then(|bytes| {
                    bytes.checked_add(values.capacity().checked_mul(size_of::<f64>())?)
                })
                .and_then(|bytes| bytes.checked_add(256))
                .ok_or(MathRuntimeError::Limit(
                    "term diagnostic construction extent",
                ))
        })?;
        if bytes > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit(
                "term diagnostic construction extent",
            ));
        }
        let operation = self.job_retained(1, bytes, control.clone(), move |flag| {
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
                let bytes = prepared.plan.retained_bytes();
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
        order: pse_kernels::DerivativeOrder,
        profile: pse_compiler::workspace::Profile,
        driver: &crate::CancelSource,
    ) -> Result<Arc<super::ExecutableCase>, MathRuntimeError> {
        let control = FlightCancellation::default();
        let operation =
            self.job_retained(1, super::WITHIN_WORKSPACE, control.clone(), move |flag| {
                let _lease = workspace.lease;
                let compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                let prepared = compiler.prepare_modeling_parametric(
                    view.compiled(),
                    &parameters,
                    order,
                    profile,
                    &flag,
                )?;
                let bytes = prepared.plan.retained_bytes();
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
                let bytes = product.plan.retained_bytes();
                Ok((product, bytes))
            });
        tokio::pin!(operation);
        let (product, lease) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        self.count(|p| &p.observations);
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
                let bytes = product.retained_bytes();
                Ok((product, bytes))
            });
        tokio::pin!(operation);
        let owned = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        self.count(|p| &p.views);
        self.own_preparation(owned)
    }
    /// Value-only rebind of a prepared view (A6). No value-dependent product is rebuilt
    /// when its consumed values and derived realization parameters are unchanged.
    /// Comparison metadata and changed products use bounded admission; the structure, its
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
        self.rebind_scoped(prepared, values, driver, None).await
    }
    async fn rebind_scoped(
        self: &Arc<Self>,
        prepared: &super::Preparation,
        values: pse_math::binding::CaseValues,
        driver: &crate::CancelSource,
        deadline: Option<std::time::Instant>,
    ) -> Result<super::Preparation, MathRuntimeError> {
        enum Comparison {
            Unchanged,
            Sharing(Box<pse_compiler::workspace::PreparedCase>),
            Rebuild(pse_math::binding::CaseValues),
        }
        // Both phases share the enclosing clock or one admission-only cutoff.
        // A comparison needing Derived::complete or frozen-value validation copies
        // metadata only after bounded population/CPU/pool admission.
        let cutoff = self.admission_deadline(deadline)?;
        let compiled = prepared.prepared.clone();
        let binding_demand = compiled.rebind_binding_allocation_bound(&values)?;
        if binding_demand > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit(
                "rebind binding construction capacity",
            ));
        }
        let control = FlightCancellation::default();
        let entry = self.admit_entry(1, binding_demand, &control, deadline, Some(cutoff))?;
        let comparison = self.job_retained_on_entry(
            1,
            binding_demand,
            control.clone(),
            deadline,
            entry,
            move |flag| {
                if compiled.values_match(&values) {
                    let completed = compiled.derived.complete(&values);
                    if compiled.coefficient_values.iter().all(|(id, bits)| {
                        completed.scalars.get(id).map(|v| v.to_bits()) == Some(*bits)
                    }) {
                        compiled
                            .plan
                            .structure()
                            .validate_frozen_values(&completed)?;
                        return Ok((Comparison::Unchanged, 0));
                    }
                    let rebound = compiled.rebind(&values, &flag)?;
                    let bytes = rebound.rebind_allocation_bytes(&compiled);
                    return Ok((Comparison::Sharing(Box::new(rebound)), bytes));
                }
                Ok((Comparison::Rebuild(values), 0))
            },
        );
        tokio::pin!(comparison);
        let (comparison, lease) = tokio::select! {result=&mut comparison=>result?,()=driver.cancelled()=>{control.cancel();let _=comparison.await;return Err(MathRuntimeError::Cancelled);}};
        let values = match comparison {
            Comparison::Unchanged => {
                self.count(|p| &p.shared);
                return Ok(prepared.clone());
            }
            Comparison::Sharing(rebound) => {
                self.count(|p| &p.shared);
                // Release the temporary box before own_rebind allocates its two
                // retained PreparedCase owners under the same binding lease.
                let rebound = {
                    let boxed = rebound;
                    *boxed
                };
                return Ok(Self::own_rebind(prepared, rebound, lease));
            }
            Comparison::Rebuild(values) => values,
        };
        drop(lease);
        let compiled = prepared.prepared.clone();
        let demand = compiled
            .rebind_allocation_bound(&values)?
            .unwrap_or(self.policy.worker_bytes);
        if demand > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit("rebind construction capacity"));
        }
        let control = FlightCancellation::default();
        let entry = self.admit_entry(1, demand, &control, deadline, Some(cutoff))?;
        let operation =
            self.job_retained_on_entry(1, demand, control.clone(), deadline, entry, move |flag| {
                let rebound = compiled.rebind(&values, &flag)?;
                let bytes = rebound.rebind_allocation_bytes(&compiled);
                Ok((rebound, bytes))
            });
        tokio::pin!(operation);
        let (rebound, lease) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        self.count(|p| &p.rebuilt);
        Ok(Self::own_rebind(prepared, rebound, lease))
    }
    /// Rebind inside an existing attempt, including CPU waiting and late completion.
    #[cfg(feature = "solver-kinsol")]
    pub(crate) async fn rebind_within(
        self: &Arc<Self>,
        prepared: &super::Preparation,
        values: pse_math::binding::CaseValues,
        driver: &crate::CancelSource,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<super::Preparation, MathRuntimeError> {
        // This direct job owns its original deadline and completion drain. A
        // shared-loader waiter may abandon a loader; a rebind must await its own
        // admitted operation so deadline refusal has released the population ticket.
        scope
            .check()
            .map_err(pse_backend_native::ProblemError::Provider)?;
        let result = self
            .rebind_scoped(prepared, values, driver, scope.deadline())
            .await?;
        scope
            .check()
            .map_err(pse_backend_native::ProblemError::Provider)?;
        Ok(result)
    }
    /// Own immutable generated declarations and their package data documents under the
    /// deployment pool.
    pub fn modeling_revision(
        &self,
        workspace: &Workspace,
        rows: Vec<Declaration>,
        scope: pse_modeling::PhysicalScope,
        documents: Arc<pse_modeling::document::DocumentInventory>,
        physical: &pse_ids::ContentHash,
    ) -> Result<ModelingRevision, MathRuntimeError> {
        let bytes = rows
            .owned_bytes()
            .saturating_add(
                scope
                    .documents
                    .as_ref()
                    .map_or(0, |d| d.len() * (size_of::<SemanticId>() + 32)),
            )
            .saturating_add(documents.retained_bytes());
        if bytes > self.policy.workspace_bytes / 2 {
            return Err(MathRuntimeError::Limit("modeling source bytes"));
        }
        // The source rows are charged before building checked state inside the
        // workspace's lease; the retained lease then takes the revision's actual extent.
        let reservation =
            pse_columnar::MemoryConsumer::new("modeling:source-revision").register(&self.pool);
        reservation.try_grow(bytes)?;
        let admitted = workspace
            .compiler
            .lock()
            .map_err(|_| MathRuntimeError::Infrastructure("compiler lock poisoned".into()))?
            .publish_modeling_with(rows, scope, documents)?;
        let identity = source_revision(
            admitted.declarations(),
            admitted.documents(),
            physical,
            &admitted.physical_bindings(),
        );
        if admitted.retained_bytes() > self.policy.workspace_bytes / 2 {
            return Err(MathRuntimeError::Limit("modeling admitted source bytes"));
        }
        reservation.try_resize(admitted.retained_bytes())?;
        Ok(ModelingRevision {
            identity,
            admitted,
            _lease: AllocationLease::new(reservation),
        })
    }
    /// Publish the selected revision and prepare it while holding the single compiler writer.
    #[expect(
        clippy::too_many_arguments,
        reason = "semantic preparation shares the compiler specialization boundary"
    )]
    pub async fn prepare_semantic_modeling_revision(
        self: &Arc<Self>,
        workspace: Workspace,
        revision: ModelingRevision,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        driver: &crate::CancelSource,
    ) -> Result<pse_compiler::workspace::SemanticModeling, MathRuntimeError> {
        let control = FlightCancellation::default();
        let operation = self.job_retained_scoped(
            1,
            super::WITHIN_WORKSPACE,
            control.clone(),
            driver.deadline(),
            move |flag| {
                let _workspace_lease = workspace.lease;
                let mut compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                compiler.publish_modeling_revision(revision.admitted.clone())?;
                let product = compiler.prepare_semantic_modeling_cancellable(
                    root, instance, bindings, limits, flag,
                )?;
                let bytes = product.retained_bytes();
                Ok((product, bytes))
            },
        );
        tokio::pin!(operation);
        let (product, lease) = tokio::select! { result=&mut operation => result?, ()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        let owner = self.shared_product(
            vec![32, product.model.allocation_identity()],
            Arc::new(product.clone()),
            lease,
            Vec::new(),
        )?;
        Ok(product.with_owner(owner))
    }
    /// Publish the selected revision and prepare its selected numerical outputs.
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
        let frontier = self
            .plan_frontier(
                workspace.clone(),
                revision,
                root,
                instance,
                bindings,
                limits,
                driver,
            )
            .await?;
        self.complete_frontier(workspace, Arc::new(frontier), driver)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::source_revision;
    use pse_ids::{ContentHash, SemanticId};
    use pse_modeling::document::{DataDocument, DocumentInventory, RowSet};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn rows() -> Vec<pse_authoring::language::Declaration> {
        pse_authoring::language::parse(
            "package p { entity kind item {} table t[j: item]: Temperature missing optional; fn f(x: Temperature)->Temperature^2 = x*x; }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap()
    }
    fn names(id: u8) -> BTreeMap<String, SemanticId> {
        BTreeMap::from([("p.Temperature".to_owned(), SemanticId::from_bytes([id; 16]))])
    }
    /// One data document whose exact bytes are `bytes`.
    fn documents(bytes: &[u8]) -> DocumentInventory {
        let id = pse_ids::named_id(SemanticId::from_bytes([5; 16]), "data/t.parquet");
        DocumentInventory {
            field_spans: BTreeMap::new(),
            packages: BTreeMap::new(),
            documents: BTreeMap::from([(
                id,
                Arc::new(DataDocument {
                    allocation_owner: None,
                    id,
                    path: "data/t.parquet".into(),
                    content_hash: pse_ids::encoding_checksum(bytes).content_hash(),
                    rows: RowSet::new(Vec::new()).unwrap(),
                }),
            )]),
        }
    }

    /// ADR-0123 Outcome 8: the same rows, data documents, physical inventory and name
    /// bindings reproduce the identity; the frozen vector pins the preimage layout of
    /// `pse.modeling.source-revision.v4` over no rows, no documents, a zero inventory and no
    /// names.
    #[test]
    fn unchanged_inputs_reproduce_the_source_revision() {
        let physical = ContentHash::from_bytes([3; 32]);
        let data = documents(b"PAR1");
        assert_eq!(
            source_revision(&rows(), &data, &physical, &names(1)),
            source_revision(&rows(), &data, &physical, &names(1))
        );
        assert_eq!(
            source_revision(
                &[],
                &DocumentInventory::default(),
                &ContentHash::from_bytes([0; 32]),
                &BTreeMap::new()
            )
            .as_id()
            .to_hex(),
            "30df31c5f268df111f7875c912d9c492bca249e249b07cbd27dc49b719372401"
        );
    }

    /// Binding one physical name to another quantity type, or admitting the same rows
    /// against another physical inventory, is another source revision.
    #[test]
    fn source_revision_changes_with_a_physical_name_binding() {
        let physical = ContentHash::from_bytes([3; 32]);
        let data = DocumentInventory::default();
        let base = source_revision(&rows(), &data, &physical, &names(1));
        assert_ne!(base, source_revision(&rows(), &data, &physical, &names(2)));
        assert_ne!(
            base,
            source_revision(&rows(), &data, &ContentHash::from_bytes([4; 32]), &names(1))
        );
        assert_ne!(
            base,
            source_revision(&rows()[..1], &data, &physical, &names(1))
        );
    }

    /// ADR-0123 Outcome 8, ADR-0125: one changed byte of a data document is another source
    /// revision, and so is a document added.
    #[test]
    fn source_revision_changes_with_one_data_byte() {
        let physical = ContentHash::from_bytes([3; 32]);
        let base = source_revision(&rows(), &documents(b"PAR1"), &physical, &names(1));
        assert_ne!(
            base,
            source_revision(&rows(), &documents(b"PAR2"), &physical, &names(1))
        );
        assert_ne!(
            base,
            source_revision(&rows(), &DocumentInventory::default(), &physical, &names(1))
        );
    }
}
