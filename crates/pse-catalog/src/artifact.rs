// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Named native outputs and one coherent Delta publication. There is no scheduler,
//! pass record, content-addressed stage restore, or execution callback here.
mod checkpoint;
mod consumption;
pub(crate) mod dependencies;
mod descriptor;
use crate::delta::publication_plan::{self, Member, MemberWrite};
pub use checkpoint::prepare_checkpoint;
use datafusion::{common::ResolvedTableReference, logical_expr::LogicalPlan};
use pse_columnar::CancellationToken;
use pse_engine::{
    EngineError,
    session::{EngineSession, PreparedComputation},
};
use pse_ids::SemanticId;
use pse_relations::generated::runtime::publications;
use pse_schema::model::provider::{OperationPurpose, ProviderScope};
use std::{collections::BTreeMap, sync::Arc};

/// One declared output of a native composition. Its entire producer graph remains
/// a native plan; shared subexpressions may use the invoking session's cache factory.
#[derive(Clone, Debug)]
pub struct RelationOutput {
    /// Registered output contract, including the exact recursive Arrow fields.
    pub relation_id: SemanticId,
    /// Native producer with actual source providers and implementations.
    pub plan: LogicalPlan,
}

/// The logical name and Delta location of the control table being published.
#[derive(Clone, Debug)]
pub struct PublicationTarget {
    /// Name whose catalog/schema/table policies govern this write.
    pub reference: ResolvedTableReference,
    /// Actual Delta control-table location.
    pub location: url::Url,
}

/// Complete named outputs over one retained native provider generation.
#[derive(Clone, Debug)]
pub struct ArtifactPlan {
    session: EngineSession,
    outputs: BTreeMap<ResolvedTableReference, RelationOutput>,
    // Identifies the retained immutable implementations, not their names or a plan hash.
    operation_id: SemanticId,
    fresh_observations: bool,
    publication: PublicationSelection,
}
#[derive(Clone, Debug)]
enum PublicationSelection {
    Relations,
    Inspection,
    Product(Arc<pse_model::artifact::ArtifactDescriptor>),
}
impl ArtifactPlan {
    /// Bind explicit output declarations without executing any source or algorithm.
    /// # Errors
    /// Empty output set, absent declaration, foreign source or incompatible fields.
    pub fn new(
        session: EngineSession,
        outputs: BTreeMap<ResolvedTableReference, RelationOutput>,
        cancel: &CancellationToken,
    ) -> Result<Self, EngineError> {
        if outputs.is_empty() {
            return Err(invalid("a native artifact requires named outputs"));
        }
        let plans = session.derive_plan_fields_many(
            &outputs
                .values()
                .map(|output| output.plan.clone())
                .collect::<Vec<_>>(),
            cancel,
        )?;
        let outputs: BTreeMap<ResolvedTableReference, RelationOutput> = outputs
            .into_iter()
            .zip(plans)
            .map(|((name, output), plan)| {
                cancel.checkpoint()?;
                if name.catalog.is_empty() || name.schema.is_empty() || name.table.is_empty() {
                    return Err(invalid(
                        "artifact output names must be fully qualified and nonempty",
                    ));
                }
                let spec = session
                    .registry()
                    .relation_by_id(output.relation_id)
                    .ok_or_else(|| invalid("artifact output declaration absent"))?;
                let plan = pse_engine::session::output::declare_relation_output(
                    plan,
                    session.registry(),
                    spec,
                )
                .map_err(pse_engine::session::engine)?;
                Ok((
                    name,
                    RelationOutput {
                        relation_id: output.relation_id,
                        plan,
                    },
                ))
            })
            .collect::<Result<_, EngineError>>()?;
        let mut fresh_observations = session.bindings().iter().any(|(_, binding)| {
            binding
                .effects
                .iter()
                .any(|effect| *effect != pse_schema::model::provider::OperationEffect::Read)
        });
        fresh_observations |= session
            .plans_require_fresh(outputs.values().map(|output| &output.plan), cancel)
            .map_err(pse_engine::session::engine)?;
        Ok(Self {
            session,
            outputs,
            operation_id: SemanticId::from_bytes(*uuid::Uuid::now_v7().as_bytes()),
            fresh_observations,
            publication: PublicationSelection::Relations,
        })
    }
    /// Restrict this composition to inspection. Partial value demand does not
    /// establish a complete publication's member or obligation inventory.
    #[must_use]
    pub fn inspection(mut self) -> Self {
        self.publication = PublicationSelection::Inspection;
        self
    }
    /// Rebind the same retained composition to new exact selected native inputs.
    /// Opaque providers and executable implementations must remain the actual owners.
    /// No cold plan reconstruction obtains the retained composition identity.
    /// # Errors
    /// Different implementations, source schemas, opaque providers or effects.
    pub fn rebind_inputs(
        &self,
        session: EngineSession,
        cancel: &CancellationToken,
    ) -> Result<Self, EngineError> {
        use datafusion::common::tree_node::Transformed;
        if self.session.implementation_generation() != session.implementation_generation()
            || self.session.registry().fingerprint() != session.registry().fingerprint()
        {
            return Err(invalid(
                "rebind requires the retained engine and registry generation",
            ));
        }
        let mut outputs = BTreeMap::new();
        for (name, output) in &self.outputs {
            let plan = output
                .plan
                .clone()
                .transform_up_with_subqueries(|node| {
                    let LogicalPlan::TableScan(mut scan) = node else {
                        return Ok(Transformed::no(node));
                    };
                    let provider = datafusion::datasource::source_as_provider(&scan.source)?;
                    let originals: Vec<_> = self
                        .session
                        .bindings()
                        .iter()
                        .filter_map(|(_, binding)| {
                            (Arc::ptr_eq(&binding.provider, &provider)
                                || (binding.witness.is_some()
                                    && binding.dependencies.len() == 1
                                    && Arc::ptr_eq(&binding.dependencies[0], &provider)))
                            .then_some(binding)
                        })
                        .map(|binding| (binding.reference.clone(), binding))
                        .collect::<BTreeMap<_, _>>()
                        .into_values()
                        .collect();
                    let [original] = originals.as_slice() else {
                        return Err(datafusion::common::DataFusionError::Plan(
                            "rebind requires one unambiguous native source binding".into(),
                        ));
                    };
                    let replacement = session
                        .bindings()
                        .iter()
                        .find_map(|(_, binding)| {
                            (binding.reference == original.reference).then_some(binding)
                        })
                        .ok_or_else(|| {
                            datafusion::common::DataFusionError::Plan("rebind source absent".into())
                        })?;
                    if original.provider.schema() != replacement.provider.schema()
                        || original.effects != replacement.effects
                        || (original.witness.is_none()
                            && !Arc::ptr_eq(&original.provider, &replacement.provider))
                        || original.witness.is_some() != replacement.witness.is_some()
                    {
                        return Err(datafusion::common::DataFusionError::Plan(
                            "rebind changed an opaque source or source contract".into(),
                        ));
                    }
                    let source = if Arc::ptr_eq(&original.provider, &provider) {
                        replacement.provider.clone()
                    } else {
                        let [source] = replacement.dependencies.as_slice() else {
                            return Err(datafusion::common::DataFusionError::Plan(
                                "rebind changed the selected view's native source topology".into(),
                            ));
                        };
                        if source.schema() != provider.schema() {
                            return Err(datafusion::common::DataFusionError::Plan(
                                "rebind changed the selected view's storage schema".into(),
                            ));
                        }
                        Arc::clone(source)
                    };
                    scan.source = datafusion::datasource::provider_as_source(source);
                    Ok(Transformed::yes(LogicalPlan::TableScan(scan)))
                })
                .map_err(pse_engine::session::engine)?
                .data;
            outputs.insert(
                name.clone(),
                RelationOutput {
                    relation_id: output.relation_id,
                    plan,
                },
            );
        }
        let mut rebound = Self::new(session, outputs, cancel)?;
        rebound.operation_id = self.operation_id;
        rebound.publication = self.publication.clone();
        Ok(rebound)
    }
    /// Retained native source, function, configuration and resource owners.
    pub fn session(&self) -> &EngineSession {
        &self.session
    }
    /// Exact output contracts and the native graphs that produce them.
    pub fn outputs(&self) -> &BTreeMap<ResolvedTableReference, RelationOutput> {
        &self.outputs
    }
    /// Complete typed dependencies of this actual retained composition. Opaque
    /// implementations carry its generation identity; a cold reconstruction must
    /// execute freshly even when every display name is unchanged.
    /// # Errors
    /// Conflicting source selections or incomplete dependency declarations.
    pub fn dependencies(
        &self,
        cancel: &CancellationToken,
    ) -> Result<Vec<pse_relations::generated::runtime::native_dependencies::Row>, EngineError> {
        dependencies::capture(self, cancel)
    }
    /// Prepare a native exact-dependency difference query. An empty result admits
    /// equality only for this retained implementation generation and immutable
    /// observations. Supplied duplicate rows and explicit NULL changes are visible.
    /// # Errors
    /// Varying observations, invalid facts, native policy or planning failure.
    pub async fn prepare_dependency_difference(
        &self,
        previous: &[pse_relations::generated::runtime::native_dependencies::Row],
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        if self.fresh_observations {
            return Err(invalid(
                "observed or unknown native inputs require fresh execution",
            ));
        }
        let (session, plan) = consumption::comparison(
            self.session.clone(),
            self.dependencies(cancel)?,
            previous,
            cancel,
        )
        .await?;
        session.prepare(plan, cancel)
    }

    /// Reuse one exact published output only after the native dependency query
    /// establishes equivalence to this retained composition. The output scan is a
    /// real child and cannot run before that requirement succeeds. Cold or changed
    /// implementation generations, missing evidence and varying observations refuse.
    /// # Errors
    /// Unavailable output, incompatible contracts, non-reusable observations or policy.
    pub async fn prepare_reuse(
        &self,
        publication: &crate::delta::publication::Publication,
        reference: &ResolvedTableReference,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        if self.fresh_observations {
            return Err(invalid(
                "observed or unknown native inputs require fresh execution",
            ));
        }
        let output = self
            .outputs
            .get(reference)
            .ok_or_else(|| invalid("artifact output absent"))?;
        let member = publication.member(reference)?;
        if member.relation_id != output.relation_id
            || publication.session().registry().fingerprint()
                != self.session.registry().fingerprint()
        {
            return Err(invalid(
                "reused output contract differs from the retained composition",
            ));
        }
        // Bind the selected output into the requesting environment, preserving its
        // current resource limits, policies, requirements and function implementations.
        let mut session = self.session.clone();
        let source = publication
            .session()
            .bindings()
            .iter()
            .find_map(|(_, binding)| {
                (binding.reference
                    == datafusion::common::TableReference::full(
                        reference.catalog.clone(),
                        reference.schema.clone(),
                        reference.table.clone(),
                    ))
                .then_some(binding)
            })
            .ok_or_else(|| invalid("reused output source absent"))?;
        let alias = ResolvedTableReference {
            catalog: "reuse".into(),
            schema: "outputs".into(),
            table: "selected".into(),
        };
        let native_alias = datafusion::common::TableReference::full(
            alias.catalog.clone(),
            alias.schema.clone(),
            alias.table.clone(),
        );
        let mut binding = source.as_ref().clone();
        binding.reference = native_alias.clone();
        session.bind_target(scope(reference));
        session
            .bind_source(
                pse_engine::provider::binding::BindingKey::Native(native_alias),
                binding,
            )
            .map_err(pse_engine::session::engine)?;
        let (dependency_session, actual) = crate::delta::dependencies::with_member_dependencies(
            &session,
            &alias,
            "stored_dependencies",
            cancel,
        )?;
        // Dependency receipts are a native data child, read through common admission.
        // Their exact selections parameterize the native consumed-value comparisons.
        let spec = pse_relations::generated::runtime::native_dependencies::spec(
            dependency_session.registry(),
        )?;
        let completed = dependency_session
            .prepare(actual.plan().clone(), cancel)?
            .execute(cancel)
            .await?;
        let checked =
            completed.into_checked_relation(dependency_session.registry(), spec, cancel)?;
        let batch =
            pse_relations::generated::runtime::native_dependencies::View::from_checked(&checked)?;
        let previous = batch.rows()?;
        let (session, violations) =
            consumption::comparison(session, self.dependencies(cancel)?, &previous, cancel).await?;
        let value = session.relation_plan(&alias)?;
        let guarded = pse_engine::session::contract::ExecutionContract::plan(
            value.plan().clone(),
            Some(violations),
            [pse_schema::model::provider::OperationEffect::Read]
                .into_iter()
                .collect(),
        );
        session.prepare(guarded, cancel)
    }
    /// Execute this artifact's sibling outputs within one invocation. Explicit
    /// native cache nodes share completed producers only for this operation;
    /// retained physical owners keep their cells alive until all outputs finish.
    /// A later invocation always receives a fresh cache scope.
    /// # Errors
    /// Native preparation, execution, policy, resource or cancellation failure.
    pub async fn execute_outputs(
        &self,
        cancel: &CancellationToken,
    ) -> Result<
        BTreeMap<ResolvedTableReference, pse_engine::session::CompletedComputation>,
        EngineError,
    > {
        let prepared = self.session.prepare_many(
            &self
                .outputs
                .values()
                .map(|output| output.plan.clone())
                .collect::<Vec<_>>(),
            cancel,
        )?;
        let completed =
            PreparedComputation::execute_group(prepared.into_iter().map(Ok), cancel).await?;
        Ok(self.outputs.keys().cloned().zip(completed).collect())
    }

    /// Prepare one named result for owned streaming through common admission.
    /// # Errors
    /// Unknown output, native analysis, policy or cancellation failure.
    pub fn prepare(
        &self,
        reference: &ResolvedTableReference,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        let output = self
            .outputs
            .get(reference)
            .ok_or_else(|| invalid("artifact output name absent"))?;
        self.session.prepare_rule_plan(output.plan.clone(), cancel)
    }
    /// Compose exactly these outputs and explicit unchanged members into one native
    /// publication command. Every requested output has exactly one Delta destination.
    /// The caller's exact input vector is checked before writes can start.
    /// # Errors
    /// Different destination inventory, different input selections, invalid members,
    /// output contract, native planning, scoped policy or cancellation failure.
    pub fn prepare_publication(
        &self,
        target: PublicationTarget,
        mut header: publications::Row,
        mut destinations: BTreeMap<ResolvedTableReference, url::Url>,
        retained: Vec<publications::RuntimePublicationsFieldMembersItem>,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        cancel.checkpoint()?;
        if matches!(self.publication, PublicationSelection::Inspection) {
            return Err(invalid(
                "partial inspection cannot publish a complete artifact",
            ));
        }
        self.validate_product_header(&header, cancel)?;
        if self.outputs.keys().ne(destinations.keys()) {
            return Err(invalid(
                "publication destinations differ from the complete artifact output set",
            ));
        }
        self.check_publication_inputs(&mut header, &retained, cancel)?;
        let mut session = self.session.with_purpose(OperationPurpose::Publish);
        session.bind_target(scope(&target.reference));
        let mut members = Vec::with_capacity(self.outputs.len() + retained.len());
        for (reference, output) in &self.outputs {
            session.bind_target(scope(reference));
            members.push(Member::Write(MemberWrite {
                reference: reference.clone(),
                relation_id: output.relation_id,
                table: crate::delta::provider::table_builder(
                    destinations
                        .remove(reference)
                        .ok_or_else(|| invalid("publication destination absent"))?,
                    &session.bound_state()?,
                )
                .map_err(pse_engine::session::engine)?
                .build()
                .map_err(|error| {
                    pse_engine::session::engine(datafusion::common::DataFusionError::External(
                        Box::new(error),
                    ))
                })?,
                input: output.plan.clone(),
            }));
        }
        members.extend(retained.into_iter().map(Member::Retained));
        let plan = publication_plan::plan_bound(
            target.location,
            header,
            members,
            Arc::clone(session.registry()),
            if self.fresh_observations {
                SemanticId::from_bytes(*uuid::Uuid::now_v7().as_bytes())
            } else {
                self.operation_id
            },
            self.dependencies(cancel)?,
        )
        .map_err(pse_engine::session::engine)?;
        session.prepare(plan, cancel)
    }
    fn check_publication_inputs(
        &self,
        header: &mut publications::Row,
        retained: &[publications::RuntimePublicationsFieldMembersItem],
        cancel: &CancellationToken,
    ) -> Result<(), EngineError> {
        let mut selected = BTreeMap::new();
        for output in self.outputs.values() {
            for member in
                crate::selection::selected_dependencies(&self.session, &output.plan, cancel)?
            {
                selected.insert(
                    (
                        member.catalog_name.clone(),
                        member.schema_name.clone(),
                        member.table_name.clone(),
                    ),
                    member.clone(),
                );
            }
        }
        for member in retained {
            let name = (
                member.catalog_name.clone(),
                member.schema_name.clone(),
                member.table_name.clone(),
            );
            let reference = ResolvedTableReference {
                catalog: name.0.clone().into(),
                schema: name.1.clone().into(),
                table: name.2.clone().into(),
            };
            if crate::selection::selected_member(&self.session, &reference)? != *member {
                return Err(invalid(
                    "retained output is not an exact selected native input",
                ));
            }
            if selected
                .insert(name, member.clone())
                .is_some_and(|previous| previous != *member)
            {
                return Err(invalid(
                    "retained output conflicts with a selected native input",
                ));
            }
        }
        header.inputs.sort_by(|left, right| {
            (&left.catalog_name, &left.schema_name, &left.table_name).cmp(&(
                &right.catalog_name,
                &right.schema_name,
                &right.table_name,
            ))
        });
        let mut observed = header.clone();
        observed.members = selected.into_values().collect();
        let mut builder = publications::Builder::new()?;
        builder.push(observed)?;
        let observed = builder.finish()?.into_batch();
        let inputs = observed
            .column_by_name("inputs")
            .ok_or_else(|| invalid("publication input contract absent"))?;
        let selected = observed
            .column_by_name("members")
            .ok_or_else(|| invalid("publication member contract absent"))?;
        if inputs != selected {
            return Err(invalid(
                "publication input vector differs from the exact bound selections",
            ));
        }
        Ok(())
    }
}
fn scope(name: &ResolvedTableReference) -> ProviderScope {
    ProviderScope::Table(
        name.catalog.to_string(),
        name.schema.to_string(),
        name.table.to_string(),
    )
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Admission {
        path: "artifact.plan".into(),
        reason: reason.into(),
    }
}
