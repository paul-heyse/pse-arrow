// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bound obligations share completion for exact immutable inputs. The context
//! indexes weak definitions, so a retained plan cannot cycle through its assembly.
use super::{EngineSession, assembly::InputSelection};
use crate::{
    EngineError,
    operation::{Body, Definition, Operation, completion::Completion},
};
use datafusion::{
    arrow::{array::RecordBatch, datatypes::SchemaRef},
    common::{DataFusionError, Result},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, LogicalPlan},
    physical_plan::{
        ExecutionPlan, SendableRecordBatchStream, execute_stream, stream::RecordBatchStreamAdapter,
    },
};
use futures_util::{FutureExt, TryStreamExt};
use pse_columnar::CancellationToken;
use pse_ids::SemanticId;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, Weak},
};

struct ObligationDependencies(BTreeMap<SemanticId, BTreeSet<pse_schema::model::RelationKey>>);

#[derive(Debug, Default)]
struct Obligations {
    bound: Mutex<Vec<Weak<Bound>>>,
    valid: Mutex<Vec<Arc<Receipt>>>,
}
#[derive(Debug)]
struct Receipt {
    declarations: BTreeSet<SemanticId>,
    implementation: Arc<ObligationImplementation>,
    selected: InputSelection,
    schema: datafusion::common::DFSchemaRef,
    _allocation: pse_columnar::MemoryReservation,
}
/// Actual checker implementation owner; identity is never a caller-supplied digest.
#[derive(Debug, Default)]
pub struct ObligationImplementation {
    _private: (),
}
#[derive(Clone, Debug)]
struct Bound {
    declarations: BTreeSet<SemanticId>,
    implementation: Arc<ObligationImplementation>,
    selected: Option<InputSelection>,
    schema: SchemaRef,
    plan: LogicalPlan,
    completion: Arc<Completion<()>>,
    receipt: Option<Arc<Receipt>>,
    cache: Arc<Obligations>,
    physical: Vec<Arc<dyn ExecutionPlan>>,
}
impl EngineSession {
    /// Bind a zero-violation obligation to its complete selected dependency set.
    /// Repeated consumers share only the actual bound checker and successful outcome.
    /// # Errors
    /// Invalid policy, cancelled preparation, or native operation construction.
    pub fn bind_obligation(
        &self,
        plan: LogicalPlan,
        declarations: &BTreeSet<SemanticId>,
        implementation: &Arc<ObligationImplementation>,
        cancel: &CancellationToken,
    ) -> std::result::Result<LogicalPlan, EngineError> {
        self.bind_obligations(&[(plan, declarations.clone())], implementation, cancel)?
            .pop()
            .ok_or_else(|| {
                super::engine(DataFusionError::Internal("bound obligation absent".into()))
            })
    }

    /// Bind sibling obligations with shared intrinsic inspection and separate
    /// dependency selections. A sibling contract cannot qualify another root.
    /// # Errors
    /// Invalid policy, cancellation, resource refusal or native construction.
    pub fn bind_obligations(
        &self,
        inputs: &[(LogicalPlan, BTreeSet<SemanticId>)],
        implementation: &Arc<ObligationImplementation>,
        cancel: &CancellationToken,
    ) -> std::result::Result<Vec<LogicalPlan>, EngineError> {
        let facts = self
            .plan_freshness_many(inputs.iter().map(|(plan, _)| plan), cancel)
            .map_err(super::engine)?;
        inputs
            .iter()
            .zip(facts)
            .map(|((plan, declarations), facts)| {
                self.bind_obligation_fresh(
                    plan.clone(),
                    declarations,
                    implementation,
                    cancel,
                    facts.required,
                )
            })
            .collect()
    }

    fn bind_obligation_fresh(
        &self,
        plan: LogicalPlan,
        declarations: &BTreeSet<SemanticId>,
        implementation: &Arc<ObligationImplementation>,
        cancel: &CancellationToken,
        fresh: bool,
    ) -> std::result::Result<LogicalPlan, EngineError> {
        cancel.checkpoint()?;
        if let Some(plan) = self.reuse_obligation(declarations, implementation, cancel)? {
            return Ok(plan);
        }
        let dependencies = self
            .registry()
            .derived_implementation(|| {
                let mut inputs =
                    BTreeMap::<SemanticId, BTreeSet<pse_schema::model::RelationKey>>::new();
                for spec in self.registry().relations() {
                    for name in spec.checks.keys() {
                        if let Some(id) = spec.row_check_id(name) {
                            inputs.entry(id).or_default().insert(spec.key);
                        }
                    }
                }
                for invariant in self.registry().invariants() {
                    let keys = invariant
                        .inputs
                        .iter()
                        .chain([&invariant.relation])
                        .filter_map(|name| self.registry().relation(name).map(|spec| spec.key))
                        .collect();
                    inputs.insert(invariant.id, keys);
                }
                Ok(ObligationDependencies(inputs))
            })
            .map_err(pse_columnar::external)
            .map_err(super::engine)?;
        let mut keys = BTreeSet::new();
        let mut resolved = BTreeSet::new();
        for declaration in declarations {
            if let Some(inputs) = dependencies.0.get(declaration) {
                keys.extend(inputs);
                resolved.insert(*declaration);
            }
        }
        let fresh = fresh || declarations.is_empty() || resolved != *declarations;
        let selection = if fresh {
            None
        } else {
            InputSelection::capture_plans(self, keys.iter().copied())?
        };
        let cache = self.retained_extension::<Obligations>();
        // Durable successful receipts retain only checked/witnessed sources, never
        // a logical plan that could retain its own SessionContext.
        let receipt = if fresh {
            None
        } else {
            InputSelection::capture(self, keys)?
                .map(|selected| {
                    let allocation = pse_columnar::MemoryConsumer::new("engine:obligation-receipt")
                        .register(self.pool());
                    allocation
                        .try_grow(
                            8192_usize
                                .saturating_add(declarations.len().saturating_mul(256))
                                .saturating_add(selected.retained_extent()),
                        )
                        .map_err(super::engine)?;
                    Ok::<_, EngineError>(Arc::new(Receipt {
                        declarations: declarations.clone(),
                        implementation: implementation.clone(),
                        selected,
                        schema: plan.schema().clone(),
                        _allocation: allocation,
                    }))
                })
                .transpose()?
        };
        let bound = Arc::new(Bound {
            declarations: declarations.clone(),
            implementation: implementation.clone(),
            selected: selection,
            schema: plan.schema().inner().clone(),
            plan: plan.clone(),
            completion: Arc::default(),
            receipt,
            cache: cache.clone(),
            physical: vec![],
        });
        if bound.selected.is_some() {
            let mut entries = cache.bound.lock().map_err(|_| {
                super::engine(DataFusionError::Internal(
                    "obligation index lock poisoned".into(),
                ))
            })?;
            entries.retain(|entry| entry.strong_count() != 0);
            // Live plans own their definitions; this bounded weak index owns none.
            if entries.len() == 64 {
                entries.remove(0);
            }
            entries.push(Arc::downgrade(&bound));
        }
        Operation::plan(bound, vec![plan]).map_err(super::engine)
    }
    /// Find an exact live obligation before recompiling its native template.
    /// # Errors
    /// Policy composition, cancellation or completion-lock failure.
    pub fn reuse_obligation(
        &self,
        declarations: &BTreeSet<SemanticId>,
        implementation: &Arc<ObligationImplementation>,
        cancel: &CancellationToken,
    ) -> std::result::Result<Option<LogicalPlan>, EngineError> {
        cancel.checkpoint()?;
        let cache = self.retained_extension::<Obligations>();
        for receipt in cache
            .valid
            .lock()
            .map_err(|_| {
                super::engine(DataFusionError::Internal(
                    "obligation receipt lock poisoned".into(),
                ))
            })?
            .iter()
        {
            if receipt.declarations == *declarations
                && Arc::ptr_eq(&receipt.implementation, implementation)
                && receipt.selected.matches(self)?
            {
                return Ok(Some(LogicalPlan::EmptyRelation(
                    datafusion::logical_expr::EmptyRelation {
                        produce_one_row: false,
                        schema: receipt.schema.clone(),
                    },
                )));
            }
        }
        let entries = cache.bound.lock().map_err(|_| {
            super::engine(DataFusionError::Internal(
                "obligation index lock poisoned".into(),
            ))
        })?;
        for entry in entries.iter().filter_map(Weak::upgrade) {
            if entry.declarations != *declarations
                || !Arc::ptr_eq(&entry.implementation, implementation)
            {
                continue;
            }
            let Some(selection) = &entry.selected else {
                continue;
            };
            if !selection.matches(self)? {
                continue;
            }
            match entry.completion.completed().map_err(super::engine)? {
                Some(Ok(_)) => {
                    return Ok(Some(LogicalPlan::EmptyRelation(
                        datafusion::logical_expr::EmptyRelation {
                            produce_one_row: false,
                            schema: entry.plan.schema().clone(),
                        },
                    )));
                }
                Some(Err(_)) => {}
                None => {
                    return Operation::plan(entry.clone(), vec![entry.plan.clone()])
                        .map(Some)
                        .map_err(super::engine);
                }
            }
        }
        Ok(None)
    }
}
#[async_trait::async_trait]
impl Definition for Bound {
    fn name(&self) -> &'static str {
        "RequiredInvariant"
    }
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
    async fn prepare(
        self: Arc<Self>,
        _: &[Expr],
        logical: &[LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        _: &SessionState,
    ) -> Result<Arc<dyn Body>> {
        if logical.len() != 1 {
            return Err(DataFusionError::Plan(
                "obligation child arity changed".into(),
            ));
        }
        if crate::operation::ports::same_input(&logical[0], &self.plan) {
            Ok(Arc::new(Self {
                physical: inputs.to_vec(),
                ..self.as_ref().clone()
            }))
        } else {
            Ok(Arc::new(Self {
                plan: logical[0].clone(),
                physical: inputs.to_vec(),
                completion: Arc::default(),
                receipt: None,
                selected: None,
                ..self.as_ref().clone()
            }))
        }
    }
}
impl Body for Bound {
    fn rebind(&self, inputs: &[Arc<dyn ExecutionPlan>]) -> Result<Option<Arc<dyn Body>>> {
        if inputs.len() == self.physical.len()
            && inputs
                .iter()
                .zip(&self.physical)
                .all(|(a, b)| crate::operation::ports::same_physical_input(a, b))
        {
            return Ok(None);
        }
        Ok(Some(Arc::new(Self {
            physical: inputs.to_vec(),
            completion: Arc::default(),
            receipt: None,
            selected: None,
            ..self.clone()
        })))
    }
    fn execute(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let [input]: [_; 1] = inputs
            .try_into()
            .map_err(|_| DataFusionError::Plan("obligation child arity changed".into()))?;
        let completion = self.completion.clone();
        let schema = self.schema.clone();
        let receipt = self.receipt.clone();
        let cache = self.cache.clone();
        let stream = futures_util::stream::once(async move {
            completion
                .get(|| {
                    async move {
                        let mut stream = execute_stream(input, context)?;
                        let mut violated = false;
                        while let Some(batch) = stream.try_next().await? {
                            violated |= batch.num_rows() != 0;
                        }
                        if violated {
                            return Err(crate::operation::FailureKind::Invalid.error(
                                DataFusionError::Execution(
                                    "scoped invariant requirements produced violations".into(),
                                ),
                            ));
                        }
                        if let Some(receipt) = receipt {
                            let mut valid = cache.valid.lock().map_err(|_| {
                                DataFusionError::Internal("obligation receipt lock poisoned".into())
                            })?;
                            if valid.len() == 64 {
                                valid.remove(0);
                            }
                            valid.push(receipt);
                        }
                        Ok(())
                    }
                    .boxed()
                })
                .await?;
            Ok(RecordBatch::new_empty(schema))
        });
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.schema.clone(),
            stream,
        )))
    }
}

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    use crate::session::{EngineFactory, ExecutionSettings, ThreadBudget, native_engine_profile};
    use datafusion::arrow::datatypes::{DataType, Field, Schema};
    use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};
    use std::{
        collections::BTreeMap,
        sync::atomic::{AtomicUsize, Ordering},
    };

    #[derive(Debug)]
    struct Check(Arc<AtomicUsize>, SchemaRef);
    #[async_trait::async_trait]
    impl Definition for Check {
        fn name(&self) -> &'static str {
            "UnitInvariant"
        }
        fn schema(&self) -> SchemaRef {
            self.1.clone()
        }
        async fn prepare(
            self: Arc<Self>,
            _: &[Expr],
            _: &[LogicalPlan],
            _: &[Arc<dyn ExecutionPlan>],
            _: &SessionState,
        ) -> Result<Arc<dyn Body>> {
            Ok(self)
        }
    }
    impl Body for Check {
        fn execute(
            &self,
            _: Vec<Arc<dyn ExecutionPlan>>,
            _: Arc<TaskContext>,
        ) -> Result<SendableRecordBatchStream> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(Box::pin(RecordBatchStreamAdapter::new(
                self.1.clone(),
                futures_util::stream::iter([Ok(RecordBatch::new_empty(self.1.clone()))]),
            )))
        }
    }
    #[tokio::test]
    #[expect(
        clippy::too_many_lines,
        reason = "one isolated lifecycle fixture exercises retained success and invalidation together"
    )]
    async fn settled_obligation_survives_plan_drop_but_absence_empty_and_checker_changes_invalidate()
     {
        let mut builder = pse_schema::RegistryBuilder::new();
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "inputs",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "obligation input",
            )
            .pk(&["value"])
            .columns(vec![FieldContract::from_field(Field::new(
                "value",
                DataType::Int64,
                false,
            ))])
            .checks(BTreeMap::from([("positive".into(), "value > 0".into())])),
        );
        let registry = Arc::new(builder.build().unwrap());
        let spec = registry.relation("authored.inputs").unwrap();
        let declarations = BTreeSet::from([spec.row_check_id("positive").unwrap()]);
        let cancel = CancellationToken::new();
        let factory = EngineFactory::new(
            Arc::default(),
            Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20)),
            ExecutionSettings::default(),
            ThreadBudget {
                pool_threads: 1.try_into().unwrap(),
                target_partitions: 1.try_into().unwrap(),
            },
            native_engine_profile(),
        )
        .unwrap();
        let session = factory
            .candidate(BTreeMap::new(), registry.clone(), &cancel)
            .unwrap();
        let implementation = Arc::default();
        let calls = Arc::new(AtomicUsize::new(0));
        let plan = Operation::plan(
            Arc::new(Check(
                calls.clone(),
                Arc::new(Schema::new(vec![Field::new(
                    "violation",
                    DataType::Int64,
                    false,
                )])),
            )),
            vec![],
        )
        .unwrap();
        let first = session
            .bind_obligation(plan, &declarations, &implementation, &cancel)
            .unwrap();
        let second = session
            .reuse_obligation(&declarations, &implementation, &cancel)
            .unwrap()
            .unwrap();
        session
            .prepare_rule_plan(first, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        session
            .prepare_rule_plan(second, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(matches!(
            session
                .reuse_obligation(&declarations, &implementation, &cancel)
                .unwrap(),
            Some(LogicalPlan::EmptyRelation(_))
        ));
        assert!(
            session
                .reuse_obligation(&declarations, &Arc::default(), &cancel)
                .unwrap()
                .is_none()
        );
        let empty = RecordBatch::new_empty(Arc::new(
            pse_schema::arrow::relation_schema(&registry, spec).unwrap(),
        ));
        let empty_session = factory
            .candidate(
                BTreeMap::from([(spec.key, empty)]),
                registry.clone(),
                &cancel,
            )
            .unwrap();
        assert!(
            empty_session
                .reuse_obligation(&declarations, &implementation, &cancel)
                .unwrap()
                .is_none()
        );
        assert!(
            session
                .reuse_obligation(&BTreeSet::from([SemanticId::NIL]), &implementation, &cancel)
                .unwrap()
                .is_none()
        );
    }
}
