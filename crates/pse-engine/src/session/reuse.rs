// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual semantic assembly ownership for preparation reuse, never authorization.
use super::{EngineRules, EngineSession, functions::Functions};
use crate::EngineError;
use std::sync::Arc;

pub(crate) struct Witness {
    generation: pse_ids::SemanticId,
    registry: Arc<pse_schema::Registry>,
    functions: Arc<Functions>,
    rules: Arc<EngineRules>,
    bindings: Vec<(
        crate::provider::binding::BindingKey,
        Arc<crate::provider::binding::TableBinding>,
    )>,
    requirement_planner: Option<Arc<dyn super::policy::RequirementPlanner>>,
    purpose: pse_schema::model::provider::OperationPurpose,
    sources: Vec<(
        Arc<dyn datafusion::catalog::TableProvider>,
        datafusion::arrow::datatypes::SchemaRef,
    )>,
    selection: Option<Arc<super::assembly::Selection>>,
    selected_functions: Option<Functions>,
    relevant: bool,
    native: Arc<super::assembly::ModelAssembly>,
    policies: Vec<pse_schema::model::provider::ProviderPolicy>,
}
impl Witness {
    /// Only actual immutable owners can certify completed value reuse. The
    /// captured binding set is dependency-scoped for transparent producers and
    /// complete for opaque computations and completed metadata resolutions.
    pub(crate) fn retention_eligible(&self) -> bool {
        self.bindings.iter().all(|(_, binding)| {
            binding.mutation.is_none()
                && (binding.checked.is_some() || binding.witness.is_some())
                && binding
                    .effects
                    .iter()
                    .all(|effect| *effect == pse_schema::model::provider::OperationEffect::Read)
        }) && self.sources.iter().all(|(source, _)| {
            self.bindings.iter().any(|(_, binding)| {
                Arc::ptr_eq(source, &binding.provider)
                    || binding
                        .dependencies
                        .iter()
                        .any(|dependency| Arc::ptr_eq(source, dependency))
            })
        })
    }
    pub(crate) fn capture(session: &EngineSession) -> Result<Self, EngineError> {
        Ok(Self {
            generation: session.implementation_generation(),
            bindings: session
                .bindings
                .iter()
                .map(|(key, binding)| (key.clone(), binding))
                .collect(),
            requirement_planner: session.requirement_planner.clone(),
            purpose: session.purpose,
            sources: session
                .bindings
                .providers()
                .into_iter()
                .map(|provider| {
                    let schema = provider.schema();
                    (provider, schema)
                })
                .collect(),
            registry: session.registry.clone(),
            functions: session.native.function_bindings.clone(),
            rules: session.rules.clone(),
            selection: Some(session.selection()?),
            selected_functions: None,
            relevant: false,
            native: session.native.clone(),
            policies: session
                .policies
                .iter()
                .map(super::config::semantic_policy)
                .collect(),
        })
    }
    /// Closed relational producers expose their exact provider dependencies.
    /// Opaque algorithms and metadata resolution retain complete selection proof.
    pub(crate) fn capture_plan(
        session: &EngineSession,
        input: &datafusion::logical_expr::LogicalPlan,
        sources: &[(
            Arc<dyn datafusion::catalog::TableProvider>,
            datafusion::arrow::datatypes::SchemaRef,
        )],
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, EngineError> {
        use datafusion::{common::tree_node::TreeNodeRecursion, logical_expr::LogicalPlan};
        let mut witness = Self::capture(session)?;
        let mut opaque = session.bindings.resolutions().next().is_some()
            || !session.effective_policy()?.requirements.is_empty();
        super::traversal::visit(
            input,
            super::traversal::Purpose::Evidence,
            &session.pool,
            cancel,
            |node, _| {
                if opaque {
                    return Ok(TreeNodeRecursion::Stop);
                }
                if let LogicalPlan::Extension(extension) = node {
                    opaque |= !extension
                        .node
                        .as_any()
                        .is::<super::contract::ExecutionContract>()
                        && super::cache::evidence_input(node).is_none();
                }
                Ok(if opaque {
                    TreeNodeRecursion::Stop
                } else {
                    TreeNodeRecursion::Continue
                })
            },
        )
        .map_err(super::engine)?;
        if !opaque {
            witness.bindings.retain(|(_, binding)| {
                sources.iter().any(|(source, _)| {
                    Arc::ptr_eq(source, &binding.provider)
                        || binding
                            .dependencies
                            .iter()
                            .any(|dependency| Arc::ptr_eq(source, dependency))
                })
            });
            witness.sources = sources.to_vec();
            witness.relevant = true;
            witness.selection = None;
            witness.selected_functions = Some(
                witness
                    .functions
                    .selected(input, &session.pool, cancel)
                    .map_err(super::engine)?,
            );
        }
        Ok(witness)
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.bindings.capacity()
                * size_of::<(
                    crate::provider::binding::BindingKey,
                    Arc<crate::provider::binding::TableBinding>,
                )>()
            + self.sources.capacity()
                * size_of::<(
                    Arc<dyn datafusion::catalog::TableProvider>,
                    datafusion::arrow::datatypes::SchemaRef,
                )>()
            + self
                .native
                .settings
                .iter()
                .map(|(key, value)| {
                    key.capacity() + value.as_ref().map_or(0, String::capacity) + 128
                })
                .sum::<usize>()
            + self
                .policies
                .iter()
                .map(|policy| {
                    use pse_schema::model::provider::ProviderScope;
                    let names = match &policy.scope {
                        ProviderScope::Root | ProviderScope::Invocation => 0,
                        ProviderScope::Catalog(catalog) => catalog.capacity(),
                        ProviderScope::Schema(catalog, schema) => {
                            catalog.capacity() + schema.capacity()
                        }
                        ProviderScope::Table(catalog, schema, table) => {
                            catalog.capacity() + schema.capacity() + table.capacity()
                        }
                    };
                    size_of_val(policy)
                        + names
                        + (policy.requirements.len() + policy.effects.len()) * 128
                        + policy
                            .defaults
                            .iter()
                            .chain(&policy.required_settings)
                            .map(|(key, value)| key.capacity() + value.capacity() + 128)
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
    pub(crate) fn matches(&self, session: &EngineSession) -> bool {
        self.matches_kind(session, true)
    }
    /// Semantic context for field admission. The caller separately checks its
    /// consumed providers and schemas; mutable rows still require fresh execution.
    pub(crate) fn matches_structure(&self, session: &EngineSession) -> bool {
        let matches = self.matches_kind(session, false);
        if !matches {
            tracing::debug!(target: "pse::admission",
                purpose = self.purpose == session.purpose,
                settings = self.native.settings == session.native.settings,
                generation = self.relevant || self.generation == session.implementation_generation(),
                planning = Arc::ptr_eq(&self.native.planning, &session.native.planning),
                registry = Arc::ptr_eq(&self.registry, &session.registry),
                functions = self.selected_functions.as_ref().map_or_else(
                    || Arc::ptr_eq(&self.functions, &session.native.function_bindings),
                    |selected| selected.remains_in(&session.native.function_bindings)),
                rules = Arc::ptr_eq(&self.rules, &session.rules),
                "field context mismatch");
        }
        matches
    }
    fn matches_kind(&self, session: &EngineSession, values: bool) -> bool {
        (!values
            || self.relevant
            || self
                .selection
                .as_ref()
                .is_some_and(|selection| selection.matches_owners(session)))
            && self.purpose == session.purpose
            && (!values || self.relevant || self.bindings.len() == session.bindings.iter().count())
            && self.native.settings == session.native.settings
            && (!values
                || self.bindings.iter().all(|(key, a)| {
                    session.bindings.get(key).is_some_and(|b| {
                        Arc::ptr_eq(a, &b)
                            && !b
                                .effects
                                .contains(&pse_schema::model::provider::OperationEffect::Observe)
                            && b.mutation.is_none()
                            && !b.provider.is::<datafusion::datasource::MemTable>()
                    })
                }))
            && match (&self.requirement_planner, &session.requirement_planner) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            }
            && (!values
                || self
                    .sources
                    .iter()
                    .all(|(provider, schema)| provider.schema() == *schema))
            && (self.relevant || self.generation == session.implementation_generation())
            && Arc::ptr_eq(&self.native.planning, &session.native.planning)
            && Arc::ptr_eq(&self.registry, &session.registry)
            && self.selected_functions.as_ref().map_or_else(
                || Arc::ptr_eq(&self.functions, &session.native.function_bindings),
                |selected| selected.remains_in(&session.native.function_bindings),
            )
            && Arc::ptr_eq(&self.rules, &session.rules)
            && self.policies.len() == session.policies.len()
            && self
                .policies
                .iter()
                .zip(session.policies.iter())
                .all(|(a, b)| super::config::policy_semantics_equal(a, b))
    }
}

#[cfg(test)]
mod scoped_reuse_unit {
    use super::*;
    use datafusion::{
        arrow::datatypes::{DataType, Field, Schema},
        datasource::MemTable,
        execution::{context::SessionContext, session_state::SessionStateBuilder},
    };
    use std::collections::BTreeMap;
    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "paired consumed and unrelated binding controls share exact fixture owners"
    )]
    fn relevant_empty_sources_and_function_owners_survive_only_unrelated_revisions() {
        use datafusion::{
            catalog::TableProvider,
            logical_expr::{LogicalPlanBuilder, col},
        };
        let context = SessionContext::new();
        let cancel = pse_columnar::CancellationToken::new();
        let factory = crate::session::EngineFactory::from_builder(
            context.runtime_env(),
            Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20)),
            "relevant-unit",
            SessionStateBuilder::from(context.state()),
        );
        let base = factory
            .candidate_checked(
                BTreeMap::new(),
                Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
                &cancel,
            )
            .unwrap();
        let schema = Arc::new(Schema::new(vec![Field::new("n", DataType::Int64, false)]));
        let provider = || -> Arc<dyn TableProvider> {
            Arc::new(super::super::materialized::ImmutableTable {
                defaults: BTreeMap::new(),
                schema: schema.clone(),
                data: Vec::<pse_columnar::owned_buffer::OwnedRecordBatch>::new(),
                constraints: None,
            })
        };
        let source = provider();
        let mut session = base;
        session
            .bindings
            .insert(
                crate::provider::binding::BindingKey::Native(
                    datafusion::common::TableReference::full("datafusion", "public", "used"),
                ),
                crate::provider::binding::TableBinding::new(
                    datafusion::common::TableReference::full("datafusion", "public", "used"),
                    source.clone(),
                    None,
                    None,
                ),
            )
            .unwrap();
        let scan = LogicalPlanBuilder::scan(
            "used",
            datafusion::datasource::provider_as_source(source.clone()),
            None,
        )
        .unwrap()
        .build()
        .unwrap();
        let sources = [(source, schema.clone())];
        let proof = Witness::capture_plan(&session, &scan, &sources, &cancel).unwrap();
        let mut changed = session.clone();
        changed
            .bindings
            .insert(
                crate::provider::binding::BindingKey::Native(
                    datafusion::common::TableReference::full("datafusion", "public", "unused"),
                ),
                crate::provider::binding::TableBinding::new(
                    datafusion::common::TableReference::full("datafusion", "public", "unused"),
                    provider(),
                    None,
                    None,
                ),
            )
            .unwrap();
        assert!(
            proof.matches(&changed),
            "unrelated immutable bindings do not invalidate an exact producer"
        );
        let mut replaced = changed.clone();
        replaced
            .bindings
            .replace_table(
                &datafusion::common::TableReference::full("datafusion", "public", "used"),
                &crate::provider::binding::TableBinding::new(
                    datafusion::common::TableReference::full("datafusion", "public", "used"),
                    provider(),
                    None,
                    None,
                ),
            )
            .unwrap();
        assert!(
            !proof.matches(&replaced),
            "an empty consumed source is still a dependency"
        );
        let mut functions_changed = session.clone();
        Arc::make_mut(&mut functions_changed.native).function_bindings =
            Functions::from_state(&SessionStateBuilder::new().build());
        assert!(
            proof.matches(&functions_changed),
            "no function dependency was consumed"
        );
        let used = session.scalar_function("abs").unwrap();
        let plan = LogicalPlanBuilder::from(scan)
            .project([used.call(vec![col("n")])])
            .unwrap()
            .build()
            .unwrap();
        let proof = Witness::capture_plan(&session, &plan, &sources, &cancel).unwrap();
        assert!(
            !proof.matches(&functions_changed),
            "replacing a consumed function invalidates its plan"
        );
    }

    #[test]
    fn actual_owners_and_settings_are_witnesses_not_display_names() {
        let context = SessionContext::new();
        let cancel = pse_columnar::CancellationToken::new();
        let factory = crate::session::EngineFactory::from_builder(
            context.runtime_env(),
            Arc::new(pse_columnar::GreedyMemoryPool::new(8 << 20)),
            "same-display",
            SessionStateBuilder::from(context.state()),
        );
        let session = factory
            .candidate_checked(
                BTreeMap::new(),
                Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
                &cancel,
            )
            .unwrap();
        let proof = Witness::capture(&session).unwrap();
        assert!(proof.matches(&session.clone()));
        let mut changed = session.clone();
        changed.rules = Arc::new((*session.rules).clone());
        assert!(!proof.matches(&changed));
        let schema = Arc::new(Schema::new(vec![Field::new("n", DataType::Int64, false)]));
        let a = session
            .with_provider(
                "numbers".into(),
                Arc::new(MemTable::try_new(schema.clone(), vec![vec![]]).unwrap()),
                &cancel,
            )
            .unwrap();
        let b = session
            .with_provider(
                "numbers".into(),
                Arc::new(MemTable::try_new(schema, vec![vec![]]).unwrap()),
                &cancel,
            )
            .unwrap();
        assert!(!Witness::capture(&a).unwrap().matches(&b));
        let mut changed = session.clone();
        Arc::make_mut(&mut changed.native).function_bindings =
            Functions::from_state(&context.state());
        assert!(!proof.matches(&changed));
    }
}
