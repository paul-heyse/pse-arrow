// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned Arrow facts under small, isolated registry contracts.
#![allow(clippy::unwrap_used, reason = "fixed test declarations must admit")]
#![allow(
    dead_code,
    reason = "shared fixture fields are consumed by different integration test binaries"
)]
use datafusion::{arrow::array::RecordBatch, execution::runtime_env::RuntimeEnv};
use pse_catalog::session::RelationFacts;
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, SnapshotSession, ThreadBudget, native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_rules::strata::{
    LocatedRuleInput, RuleBindings, RuleInputLocation, StratumLimits, StratumOutcome,
    execute_strata,
};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, DerivationGranularity, EnumDecl, FieldContract, FieldContract as T,
        Namespace as N, RelationDecl, RelationKey, RuleDecl, RuleSpec, SnapshotClass as S,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::{NonZeroU32, NonZeroUsize},
    sync::Arc,
};

pub(crate) fn builder() -> RegistryBuilder {
    let catalog = pse_schema::registry().unwrap();
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    for name in ["TruthValue", "RuleOutcomeReason", "RuleSupportKind"] {
        let spec = catalog.enum_spec(name).unwrap();
        builder.declare_enum(EnumDecl::platform(spec.name, spec.members.clone()));
    }
    for name in [
        "inferred.rule_outcomes",
        "provenance.rule_support_edges",
        "provenance.derivations",
        "provenance.constructed_supports",
    ] {
        let spec = catalog.relation(name).unwrap();
        // The isolated fixture has no self-description tables. Project the actual
        // evidence fields while dropping only foreign-key references to that absent catalog.
        let columns = spec
            .columns
            .iter()
            .cloned()
            .map(FieldContract::without_fk)
            .collect();
        builder.declare_relation(
            RelationDecl::new(
                spec.key.namespace,
                spec.key.name,
                spec.key.version,
                spec.authority,
                spec.snapshot_class,
                spec.doc,
            )
            .columns(columns)
            .pk(&spec.primary_key)
            .granularity(DerivationGranularity::Rule),
        );
    }
    builder
}
pub(crate) fn input(
    builder: &mut RegistryBuilder,
    columns: Vec<FieldContract>,
    keys: &[&'static str],
) {
    builder.declare_relation(
        RelationDecl::new(
            N::Authored,
            "input",
            1,
            Authority::Authored,
            S::Model,
            "Isolated source facts",
        )
        .pk(keys)
        .columns(columns),
    );
}
pub(crate) fn head(
    builder: &mut RegistryBuilder,
    name: &'static str,
    assertions: &'static str,
    keys: &[&'static str],
    columns: Vec<FieldContract>,
) {
    let projection = pse_schema::model::rule::assertion_columns(&columns);
    builder.declare_relation(
        RelationDecl::new(
            N::Inferred,
            name,
            1,
            Authority::Derived,
            S::Derived,
            "Finite fixture head",
        )
        .pk(keys)
        .columns(columns)
        .granularity(DerivationGranularity::Rule),
    );
    builder.declare_relation(
        RelationDecl::new(
            N::Provenance,
            assertions,
            1,
            Authority::Derived,
            S::Derived,
            "Typed fixture assertions",
        )
        .pk(&["assertion_id"])
        .columns(projection)
        .granularity(DerivationGranularity::Rule),
    );
}
pub(crate) fn id(name: &'static str) -> FieldContract {
    FieldContract::key(
        name,
        T::native(datafusion::arrow::datatypes::DataType::UInt32),
        "Finite identity",
    )
}
pub(crate) fn declare(builder: &mut RegistryBuilder, rule: RuleDecl) {
    builder.declare_rule(rule);
}
pub(crate) struct Fixture {
    pub registry: Arc<Registry>,
    pub session: SnapshotSession,
    pub facts: Arc<RelationFacts>,
    pub rules: Vec<RuleSpec>,
    pub bindings: BTreeMap<SemanticId, RuleBindings>,
    pub outputs: BTreeSet<RelationKey>,
}
impl Fixture {
    pub(crate) fn new(builder: RegistryBuilder, rows: Vec<Vec<Cell>>, partitions: usize) -> Self {
        let registry = Arc::new(builder.build().unwrap());
        let spec = registry.relation("authored.input").unwrap();
        let batch = pse_relations::cells::batch_from_cells(&registry, spec, &rows).unwrap();
        drop(rows);
        let memory = FixedBudget::new(256 << 20);
        let factory = Arc::new(
            SessionFactory::new(
                Arc::new(RuntimeEnv::default()),
                memory.clone(),
                ExecutionSettings::default(),
                ThreadBudget {
                    pool_threads: NonZeroUsize::new(partitions).unwrap(),
                    target_partitions: NonZeroUsize::new(partitions).unwrap(),
                },
                native_engine_profile(),
            )
            .unwrap()
            .with_query_planner(Arc::new(
                pse_catalog::session::planner::UnifiedPlanner::new(vec![Arc::new(
                    pse_rules::strata::native::RuleExtensionPlanner,
                )]),
            )),
        );
        let session = factory
            .candidate(
                BTreeMap::from([(spec.key, batch)]),
                Arc::clone(&registry),
                &CancellationToken::new(),
            )
            .unwrap();
        let facts = Arc::new(RelationFacts::from_checked(
            session.checked_input(&spec.key).unwrap(),
        ));
        let rules = registry
            .rules()
            .iter()
            .filter(|rule| {
                matches!(rule.head, pse_schema::model::RuleHead::Relation(_))
                    && rule.assertion_relation.is_some()
            })
            .cloned()
            .collect::<Vec<_>>();
        let bindings = rules
            .iter()
            .map(|rule| {
                let ports = rule
                    .plan
                    .dependencies()
                    .into_iter()
                    .map(|(name, port, _)| {
                        let spec = registry.relation(name).unwrap();
                        let location = if name == "authored.input" {
                            RuleInputLocation::Facts(Arc::clone(&facts))
                        } else {
                            RuleInputLocation::Workspace
                        };
                        (
                            port.to_owned(),
                            LocatedRuleInput {
                                relation: spec.key,
                                location,
                            },
                        )
                    })
                    .collect();
                (rule.id, RuleBindings { ports })
            })
            .collect();
        let mut outputs = BTreeSet::new();
        for rule in &rules {
            outputs.insert(registry.relation(rule.head.relation()).unwrap().key);
            outputs.insert(
                registry
                    .relation(rule.assertion_relation.as_deref().unwrap())
                    .unwrap()
                    .key,
            );
        }
        for name in ["inferred.rule_outcomes", "provenance.rule_support_edges"] {
            outputs.insert(registry.relation(name).unwrap().key);
        }
        Self {
            registry,
            session,
            facts,
            rules,
            bindings,
            outputs,
        }
    }
    pub(crate) async fn run(
        &self,
        max_rounds: u32,
    ) -> Result<StratumOutcome, pse_rules::RuleError> {
        execute_strata(
            &self.rules,
            &self.bindings,
            &self.session,
            &self.registry,
            &self.outputs,
            StratumLimits {
                max_rounds: NonZeroU32::new(max_rounds).unwrap(),
            },
            &CancellationToken::new(),
        )
        .await
    }
    pub(crate) fn rows(&self, outcome: &StratumOutcome, name: &str) -> Vec<Vec<Cell>> {
        let spec = self.registry.relation(name).unwrap();
        pse_relations::cells::cells_from_batch(&self.registry, spec, &outcome.relations[&spec.key])
            .unwrap()
    }
    pub(crate) fn batch<'a>(&self, outcome: &'a StratumOutcome, name: &str) -> &'a RecordBatch {
        &outcome.relations[&self.registry.relation(name).unwrap().key]
    }
}

/// Native execution preserves the original typed diagnostic inside `CatalogError`.
pub(crate) fn rule_cause(error: &pse_rules::RuleError) -> &pse_rules::RuleError {
    match error {
        pse_rules::RuleError::Execution { source, .. } => rule_cause(source),
        pse_rules::RuleError::Catalog(pse_catalog::CatalogError::Semantic(source)) => {
            let source: &dyn std::error::Error = source.as_ref();
            source
                .downcast_ref::<pse_rules::RuleError>()
                .map_or(error, rule_cause)
        }
        _ => error,
    }
}
