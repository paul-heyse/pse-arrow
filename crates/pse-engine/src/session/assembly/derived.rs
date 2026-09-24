// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete selected dependencies for owner-local immutable derivations.
use crate::{EngineError, provider::binding::BindingKey, session::EngineSession};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::RelationKey;
use std::{
    collections::BTreeMap,
    sync::{Arc, Weak},
};

#[derive(Clone, Debug)]
enum Source {
    Absent,
    Plan(datafusion::logical_expr::LogicalPlan),
    Checked(FieldCheckedBatch),
    Provider(Weak<crate::provider::binding::TableBinding>),
}
/// Exact source owners, including absence; never a hash-based validity assertion.
#[derive(Clone, Debug)]
pub struct InputSelection {
    generation: pse_ids::SemanticId,
    registry: Weak<pse_schema::Registry>,
    sources: BTreeMap<BindingKey, Source>,
    policy: Arc<super::super::policy::EffectivePolicy>,
    planner: Option<Weak<dyn super::super::policy::RequirementPlanner>>,
}
impl InputSelection {
    /// Conservative bookkeeping extent, excluding source buffers already charged
    /// to their checked owners. Callers retaining a selection reserve this extent.
    pub fn retained_extent(&self) -> usize {
        let settings = self
            .policy
            .settings
            .iter()
            .map(|(key, value)| {
                key.capacity()
                    .saturating_add(value.capacity())
                    .saturating_add(128)
            })
            .fold(0usize, usize::saturating_add);
        let origins = self
            .policy
            .setting_origins
            .iter()
            .map(|(key, values)| {
                key.capacity()
                    .saturating_add(values.len().saturating_mul(128))
                    .saturating_add(128)
            })
            .fold(0usize, usize::saturating_add);
        size_of::<Self>()
            .saturating_add(self.sources.len().saturating_mul(512))
            .saturating_add(self.policy.requirements.len().saturating_mul(128))
            .saturating_add(settings)
            .saturating_add(origins)
    }
    /// Capture immutable inputs. Observed, mutable or unqualified providers return
    /// `None`; they must derive fresh values instead of populating retained reuse.
    /// # Errors
    /// Effective policy cannot be composed.
    pub fn capture(
        session: &EngineSession,
        keys: impl IntoIterator<Item = RelationKey>,
    ) -> Result<Option<Self>, EngineError> {
        Self::capture_inner(session, keys.into_iter().map(BindingKey::Relation), false)
    }
    pub(crate) fn capture_plans(
        session: &EngineSession,
        keys: impl IntoIterator<Item = RelationKey>,
    ) -> Result<Option<Self>, EngineError> {
        Self::capture_inner(session, keys.into_iter().map(BindingKey::Relation), true)
    }
    /// Capture exact qualified roles as well as declared-relation roles, including absence.
    /// # Errors
    /// Effective policy admission fails; mutable or observed sources return `None`.
    pub fn capture_roles(
        session: &EngineSession,
        keys: impl IntoIterator<Item = BindingKey>,
    ) -> Result<Option<Self>, EngineError> {
        Self::capture_inner(session, keys, false)
    }
    fn capture_inner(
        session: &EngineSession,
        keys: impl IntoIterator<Item = BindingKey>,
        plans: bool,
    ) -> Result<Option<Self>, EngineError> {
        let mut sources = BTreeMap::new();
        for key in keys {
            let binding = session.bindings().get(&key);
            let source = if let Some(binding) = binding {
                if binding.provider.is::<datafusion::datasource::MemTable>()
                    || binding.mutation.is_some()
                    || binding
                        .effects
                        .iter()
                        .any(|effect| *effect != pse_schema::model::provider::OperationEffect::Read)
                {
                    return Ok(None);
                }
                if let Some(checked) = &binding.checked {
                    Source::Checked(checked.clone())
                } else if plans && let Some(plan) = binding.provider.get_logical_plan() {
                    Source::Plan(plan.into_owned())
                } else if binding.witness.is_some() {
                    Source::Provider(Arc::downgrade(&binding))
                } else {
                    return Ok(None);
                }
            } else {
                Source::Absent
            };
            sources.insert(key, source);
        }
        Ok(Some(Self {
            generation: session.implementation_generation(),
            registry: Arc::downgrade(session.registry()),
            sources,
            policy: session.selection()?.effective.clone(),
            planner: session.requirement_planner.as_ref().map(Arc::downgrade),
        }))
    }
    /// Compare the complete selection without scanning values or rebuilding state.
    /// # Errors
    /// Effective policy cannot be composed.
    pub fn matches(&self, session: &EngineSession) -> Result<bool, EngineError> {
        if self.generation != session.implementation_generation()
            || !self
                .registry
                .upgrade()
                .is_some_and(|owner| Arc::ptr_eq(&owner, session.registry()))
            || self.policy != session.selection()?.effective
            || match (&self.planner, &session.requirement_planner) {
                (None, None) => false,
                (Some(old), Some(new)) => !old.upgrade().is_some_and(|old| Arc::ptr_eq(&old, new)),
                _ => true,
            }
        {
            return Ok(false);
        }
        for (key, source) in &self.sources {
            let binding = session.bindings().get(key);
            if binding.as_ref().is_some_and(|binding| {
                binding.mutation.is_some()
                    || binding
                        .effects
                        .iter()
                        .any(|effect| *effect != pse_schema::model::provider::OperationEffect::Read)
            }) {
                return Ok(false);
            }
            let same = match (source, binding) {
                (Source::Absent, None) => true,
                (Source::Plan(old), Some(binding)) => binding
                    .provider
                    .get_logical_plan()
                    .is_some_and(|new| crate::operation::ports::same_input(old, &new)),
                (Source::Checked(old), Some(binding)) => binding
                    .checked
                    .as_ref()
                    .is_some_and(|new| old.same_source(new)),
                (Source::Provider(old), Some(binding)) => old.upgrade().is_some_and(|old| {
                    Arc::ptr_eq(&old.provider, &binding.provider)
                        && old.reference == binding.reference
                        && old.relation == binding.relation
                        && old.witness == binding.witness
                        && old.source_dependencies == binding.source_dependencies
                }),
                _ => false,
            };
            if !same {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    use crate::session::{EngineFactory, ExecutionSettings, ThreadBudget, native_engine_profile};
    use datafusion::arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::DataType,
    };
    use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};
    #[test]
    fn selected_right_hand_absence_empty_and_changes_invalidate_but_unrelated_edits_reuse() {
        let mut builder = pse_schema::RegistryBuilder::new();
        for name in ["left", "right", "unrelated"] {
            builder.declare_relation(
                RelationDecl::new(
                    Namespace::Authored,
                    name,
                    1,
                    Authority::Authored,
                    SnapshotClass::Model,
                    "Selection unit",
                )
                .columns(vec![FieldContract::key(
                    "value",
                    FieldContract::native(DataType::Int64),
                    "Value",
                )])
                .pk(&["value"]),
            );
        }
        let registry = Arc::new(builder.build().unwrap());
        let make = |name: &str, values: Vec<i64>| {
            let spec = registry.relation(name).unwrap();
            (
                spec.key,
                FieldCheckedBatch::admit(
                    &registry,
                    spec,
                    RecordBatch::try_new(
                        Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap()),
                        vec![Arc::new(Int64Array::from(values))],
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
        };
        let factory = EngineFactory::new(
            Arc::default(),
            Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20)),
            ExecutionSettings::default(),
            ThreadBudget {
                pool_threads: 1.try_into().unwrap(),
                target_partitions: 1.try_into().unwrap(),
            },
            native_engine_profile(),
        )
        .unwrap();
        let cancel = pse_columnar::CancellationToken::new();
        let session = |rows| {
            factory
                .candidate_checked(rows, registry.clone(), &cancel)
                .unwrap()
        };
        let mut rows = BTreeMap::from([make("authored.left", vec![1])]);
        let keys = [
            registry.relation("authored.left").unwrap().key,
            registry.relation("authored.right").unwrap().key,
        ];
        let initial = session(rows.clone());
        rows.insert(
            keys[0],
            initial
                .bindings()
                .get(&BindingKey::Relation(keys[0]))
                .unwrap()
                .checked
                .clone()
                .unwrap(),
        );
        let absent = InputSelection::capture(&initial, keys).unwrap().unwrap();
        let (key, value) = make("authored.unrelated", vec![20]);
        rows.insert(key, value);
        assert!(absent.matches(&session(rows.clone())).unwrap());
        let (key, empty) = make("authored.right", vec![]);
        rows.insert(key, empty);
        let with_empty = session(rows.clone());
        assert!(!absent.matches(&with_empty).unwrap());
        let selected = InputSelection::capture(&with_empty, keys).unwrap().unwrap();
        rows.insert(
            key,
            with_empty
                .bindings()
                .get(&BindingKey::Relation(key))
                .unwrap()
                .checked
                .clone()
                .unwrap(),
        );
        assert!(selected.matches(&session(rows.clone())).unwrap());
        rows.insert(key, make("authored.right", vec![]).1);
        assert!(!selected.matches(&session(rows.clone())).unwrap());
        rows.insert(key, make("authored.right", vec![3]).1);
        assert!(!selected.matches(&session(rows)).unwrap());
    }

    use crate::provider::witness::InputWitness;
    use datafusion::common::TableReference;
    #[derive(Debug)]
    struct Witness(u8);
    impl InputWitness for Witness {
        fn reference(&self) -> TableReference {
            TableReference::full("unit", "authored", "values")
        }
        fn value(&self) -> &dyn std::any::Any {
            self
        }
        fn equivalent(&self, other: &dyn InputWitness) -> bool {
            other
                .value()
                .downcast_ref::<Self>()
                .is_some_and(|value| value.0 == self.0)
        }
        fn descriptor(&self) -> datafusion::common::Result<Vec<u8>> {
            Ok(vec![self.0])
        }
    }
    #[test]
    fn the_same_provider_cannot_hide_a_changed_source_witness() {
        use crate::provider::{binding::TableBinding, witness::SourceWitness};
        use datafusion::datasource::{MemTable, empty::EmptyTable};
        let mut builder = pse_schema::RegistryBuilder::new();
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "values",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "Witness unit",
            )
            .columns(vec![FieldContract::key(
                "value",
                FieldContract::native(DataType::Int64),
                "Value",
            )])
            .pk(&["value"]),
        );
        let registry = Arc::new(builder.build().unwrap());
        let key = registry.relation("authored.values").unwrap().key;
        let schema = Arc::new(
            pse_schema::arrow::relation_schema(&registry, registry.relation_by_key(key).unwrap())
                .unwrap(),
        );
        let provider = Arc::new(
            MemTable::try_new(
                schema.clone(),
                vec![vec![RecordBatch::new_empty(schema.clone())]],
            )
            .unwrap(),
        );
        let factory = EngineFactory::new(
            Arc::default(),
            Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20)),
            ExecutionSettings::default(),
            ThreadBudget {
                pool_threads: 1.try_into().unwrap(),
                target_partitions: 1.try_into().unwrap(),
            },
            native_engine_profile(),
        )
        .unwrap();
        let cancel = pse_columnar::CancellationToken::new();
        let mut first = factory
            .candidate(BTreeMap::new(), registry, &cancel)
            .unwrap();
        let mut binding = TableBinding::new(Witness(1).reference(), provider, Some(key), None);
        binding.witness = Some(SourceWitness::new(Arc::new(Witness(1))));
        first
            .bind_source(BindingKey::Relation(key), binding.clone())
            .unwrap();
        assert!(
            InputSelection::capture(&first, [key]).unwrap().is_none(),
            "an exact witness does not make MemTable storage immutable"
        );
        let role = BindingKey::Native(Witness(1).reference());
        first.bind_source(role.clone(), binding.clone()).unwrap();
        assert!(
            InputSelection::capture_roles(&first, [role.clone()])
                .unwrap()
                .is_none()
        );
        binding.provider = Arc::new(EmptyTable::new(schema));
        first
            .bindings
            .replace_table(&binding.reference, &binding)
            .unwrap();
        let selected = InputSelection::capture(&first, [key]).unwrap().unwrap();
        let qualified = InputSelection::capture_roles(&first, [role])
            .unwrap()
            .unwrap();
        assert!(selected.matches(&first).unwrap());
        let mut changed = first.clone();
        binding.witness = Some(SourceWitness::new(Arc::new(Witness(2))));
        changed
            .bindings
            .replace_table(&binding.reference, &binding)
            .unwrap();
        assert!(!selected.matches(&changed).unwrap());
        assert!(!qualified.matches(&changed).unwrap());
        assert!(selected.matches(&first).unwrap());
    }
}
