// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Semantic source demand, using native projection analysis within producer regions.
use super::{EngineSession, engine, traversal};
use crate::EngineError;
use datafusion::{
    common::{
        Result,
        tree_node::{Transformed, TreeNodeRecursion},
    },
    datasource::{MemTable, provider_as_source},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, TableScan, TableSource},
    optimizer::OptimizerRule,
};
use pse_columnar::CancellationToken;
use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    sync::Arc,
};

/// Observable input facts in addition to projected field values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DependencyAspect {
    /// Existence of rows, including a zero-row observation.
    Presence,
    /// Duplicate counts.
    Multiplicity,
    /// Meaningful input order.
    Ordering,
    /// Required checks or effects.
    Requirement,
    /// Negative/absent input evidence.
    Absence,
}
/// Semantic evidence is richer than the optimizer's output-column contract.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticDemand {
    /// Input field ordinals; an empty set can still demand rows.
    pub columns: BTreeSet<usize>,
    /// Source identity fields, independent of projected values.
    pub keys: BTreeSet<usize>,
    /// Independent semantic observations required from this input.
    pub aspects: BTreeSet<DependencyAspect>,
}
impl SemanticDemand {
    /// Conservative demand for a finite or otherwise opaque algorithm input.
    pub fn whole(plan: &LogicalPlan) -> Self {
        Self {
            keys: BTreeSet::new(),
            columns: (0..plan.schema().fields().len()).collect(),
            aspects: [
                DependencyAspect::Presence,
                DependencyAspect::Multiplicity,
                DependencyAspect::Ordering,
                DependencyAspect::Requirement,
                DependencyAspect::Absence,
            ]
            .into_iter()
            .collect(),
        }
    }
}

/// One actual native source and the observations that must survive reuse.
#[derive(Clone, Debug)]
pub struct DependencySource {
    /// Native projected scan, including predicate fields and exact provider owner.
    pub scan: TableScan,
    /// Required observations; ordering cannot be established by bag equality.
    pub aspects: BTreeSet<DependencyAspect>,
}

struct Producer {
    input: LogicalPlan,
    source: Arc<dyn TableSource>,
    demanded: BTreeSet<usize>,
    aspects: BTreeSet<DependencyAspect>,
    visited: bool,
}
impl EngineSession {
    /// Derive actual source projections before value optimization can erase reads.
    /// Hidden producers are analyzed once per accumulated demand; their optimizer
    /// boundary and executable plan remain unchanged. No source is executed.
    /// # Errors
    /// Native projection analysis, cancellation, or resource refusal.
    #[expect(
        clippy::too_many_lines,
        reason = "one bounded demand fixed point keeps producer discovery, native projection and source capture in the same admission scope"
    )]
    pub fn dependency_sources(
        &self,
        plan: &LogicalPlan,
        cancel: &CancellationToken,
    ) -> Result<Vec<DependencySource>, EngineError> {
        let work =
            pse_columnar::MemoryConsumer::new("session:dependency-regions").register(&self.pool);
        let mut producers = Vec::<Producer>::new();
        let mut identities = HashMap::new();
        let mut pending = VecDeque::from([(
            plan.clone(),
            BTreeSet::from([
                DependencyAspect::Presence,
                DependencyAspect::Multiplicity,
                DependencyAspect::Absence,
            ]),
        )]);
        let mut sources = Vec::new();
        let mut operations = std::collections::HashSet::new();
        let state = self.bound_state()?;
        while let Some((plan, mut aspects)) = pending.pop_front() {
            cancel.checkpoint()?;
            // Bound the actual retained graph before native recursive rewriting.
            traversal::visit(
                &plan,
                traversal::Purpose::Rewrite,
                &self.pool,
                cancel,
                |_, _| Ok(TreeNodeRecursion::Continue),
            )
            .map_err(engine)?;
            let staged = plan
                .transform_down_with_subqueries(|node| {
                    if let LogicalPlan::Extension(extension) = &node
                        && let Some(operation) = extension
                            .node
                            .as_any()
                            .downcast_ref::<crate::operation::Operation>()
                        && super::cache::logical::closed(&node, &self.pool, cancel)?
                    {
                        let demands = operation.dependencies(&SemanticDemand::whole(&node))?;
                        if operations.insert((super::admission::identity(&node), aspects.clone())) {
                            for (input, mut demand) in node.inputs().into_iter().zip(demands) {
                                work.try_grow(512 + input.schema().fields().len() * 128)
                                    .map_err(|error| {
                                        pse_columnar::external(EngineError::from(error))
                                    })?;
                                let columns = demand
                                    .columns
                                    .union(&demand.keys)
                                    .map(|index| {
                                        Expr::Column(input.schema().columns()[*index].clone())
                                    })
                                    .collect::<Vec<_>>();
                                // Even zero-column presence/absence demand retains the
                                // scan, so exact comparisons still observe multiplicity.
                                demand.aspects.extend(aspects.iter().copied());
                                pending.push_back((
                                    LogicalPlanBuilder::from(input.clone())
                                        .project(columns)?
                                        .build()?,
                                    demand.aspects,
                                ));
                            }
                        }
                        return Ok(Transformed::new(
                            LogicalPlan::EmptyRelation(datafusion::logical_expr::EmptyRelation {
                                produce_one_row: false,
                                schema: node.schema().clone(),
                            }),
                            true,
                            TreeNodeRecursion::Jump,
                        ));
                    }
                    let Some(input) = super::cache::evidence_input(&node) else {
                        if let LogicalPlan::Extension(extension) = &node {
                            if extension
                                .node
                                .as_any()
                                .is::<super::contract::ExecutionContract>()
                            {
                                aspects.insert(DependencyAspect::Requirement);
                            } else {
                                // Unknown or open extension semantics may depend on
                                // physical order. Retain exact selection evidence.
                                aspects.extend(SemanticDemand::whole(&node).aspects);
                            }
                        }
                        return Ok(Transformed::no(node));
                    };
                    if !super::cache::logical::closed(input, &self.pool, cancel)? {
                        return Ok(Transformed::yes(input.clone()));
                    }
                    let key = super::admission::identity(&node);
                    let index = if let Some(index) = identities.get(&key) {
                        *index
                    } else {
                        work.try_grow(1024)
                            .map_err(|error| pse_columnar::external(EngineError::from(error)))?;
                        let index = producers.len();
                        let source = provider_as_source(Arc::new(MemTable::try_new(
                            Arc::new(datafusion::arrow::datatypes::Schema::new(
                                input
                                    .schema()
                                    .fields()
                                    .iter()
                                    .enumerate()
                                    .map(|(index, field)| {
                                        field.as_ref().clone().with_name(format!("c{index}"))
                                    })
                                    .collect::<Vec<_>>(),
                            )),
                            vec![vec![]],
                        )?));
                        producers.push(Producer {
                            input: input.clone(),
                            source,
                            demanded: BTreeSet::new(),
                            aspects: BTreeSet::new(),
                            visited: false,
                        });
                        identities.insert(key, index);
                        index
                    };
                    let name = format!("evidence_{index}");
                    let scan = LogicalPlanBuilder::scan(
                        name.clone(),
                        producers[index].source.clone(),
                        None,
                    )?
                    .build()?;
                    let aliases = input
                        .schema()
                        .iter()
                        .enumerate()
                        .map(|(ordinal, (qualifier, field))| {
                            Expr::Column(datafusion::common::Column::new(
                                Some(name.clone()),
                                format!("c{ordinal}"),
                            ))
                            .alias_qualified(qualifier.cloned(), field.name())
                        })
                        .collect::<Vec<_>>();
                    let projection = LogicalPlanBuilder::from(scan).project(aliases)?.build()?;
                    Ok(Transformed::new(projection, true, TreeNodeRecursion::Jump))
                })
                .map_err(engine)?
                .data;
            let projected = datafusion::optimizer::optimize_projections::OptimizeProjections::new()
                .rewrite(staged, &state)
                .map_err(engine)?
                .data;
            traversal::visit(
                &projected,
                traversal::Purpose::Evidence,
                &self.pool,
                cancel,
                |node, scope| {
                    if let LogicalPlan::TableScan(scan) = node {
                        if let Some(producer) = producers
                            .iter_mut()
                            .find(|producer| Arc::ptr_eq(&producer.source, &scan.source))
                        {
                            let columns = scan.projection.clone().unwrap_or_else(|| {
                                (0..producer.input.schema().fields().len()).collect()
                            });
                            let before = producer.demanded.len();
                            let previous_aspects = producer.aspects.len();
                            producer.demanded.extend(columns);
                            producer.aspects.extend(aspects.iter().copied());
                            if !producer.visited
                                || producer.demanded.len() != before
                                || producer.aspects.len() != previous_aspects
                            {
                                producer.visited = true;
                                let expressions = producer
                                    .demanded
                                    .iter()
                                    .map(|index| {
                                        Expr::Column(
                                            producer.input.schema().columns()[*index].clone(),
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                pending.push_back((
                                    LogicalPlanBuilder::from(producer.input.clone())
                                        .project(expressions)?
                                        .build()?,
                                    producer.aspects.clone(),
                                ));
                            }
                        } else {
                            let provider =
                                datafusion::datasource::source_as_provider(&scan.source)?;
                            if provider
                                .downcast_ref::<datafusion_catalog::cte_worktable::CteWorkTable>()
                                .is_some()
                            {
                                super::admission::admit_scan(
                                    scan,
                                    &self.registry,
                                    &[],
                                    &scope.worktables,
                                )?;
                            } else {
                                work.try_grow(512 + scan.projected_schema.fields().len() * 128)
                                    .map_err(|error| {
                                        pse_columnar::external(EngineError::from(error))
                                    })?;
                                sources.push(DependencySource {
                                    scan: scan.clone(),
                                    aspects: aspects.clone(),
                                });
                            }
                        }
                    }
                    Ok(TreeNodeRecursion::Continue)
                },
            )
            .map_err(engine)?;
        }
        Ok(sources)
    }
}

#[cfg(test)]
mod scoped_reuse_unit {
    use super::*;
    use datafusion::{
        arrow::datatypes::{DataType, Field, Schema},
        execution::{context::SessionContext, session_state::SessionStateBuilder},
        logical_expr::col,
    };
    #[derive(Debug)]
    struct Selection {
        schema: Arc<Schema>,
        family: crate::operation::Family,
        invalid: bool,
    }
    #[async_trait::async_trait]
    impl crate::operation::Definition for Selection {
        fn name(&self) -> &'static str {
            "dependency-selection"
        }
        fn schema(&self) -> Arc<Schema> {
            self.schema.clone()
        }
        fn family(&self) -> crate::operation::Family {
            self.family
        }
        fn dependencies(&self, _: &[LogicalPlan], _: &SemanticDemand) -> Vec<SemanticDemand> {
            if self.invalid {
                return vec![];
            }
            vec![SemanticDemand {
                columns: [0].into_iter().collect(),
                keys: BTreeSet::new(),
                aspects: [
                    DependencyAspect::Absence,
                    DependencyAspect::Multiplicity,
                    DependencyAspect::Ordering,
                ]
                .into_iter()
                .collect(),
            }]
        }
        async fn prepare(
            self: Arc<Self>,
            _: &[Expr],
            _: &[LogicalPlan],
            _: &[Arc<dyn datafusion::physical_plan::ExecutionPlan>],
            _: &datafusion::execution::session_state::SessionState,
        ) -> Result<Arc<dyn crate::operation::Body>> {
            Err(datafusion::common::DataFusionError::Internal(
                "dependency inspection must not prepare or execute".into(),
            ))
        }
    }
    fn check_operation_families(
        session: &EngineSession,
        scan: &LogicalPlan,
        cancel: &CancellationToken,
    ) {
        for family in [
            crate::operation::Family::Finite,
            crate::operation::Family::Command,
            crate::operation::Family::Observation,
            crate::operation::Family::Foreign,
        ] {
            let plan = crate::operation::Operation::plan(
                Arc::new(Selection {
                    schema: scan.schema().inner().clone(),
                    family,
                    invalid: false,
                }),
                vec![scan.clone()],
            )
            .unwrap();
            let plan = session.cache_plan(plan, cancel).unwrap();
            let reads = session.dependency_sources(&plan, cancel).unwrap();
            assert_eq!(reads.len(), 1);
            assert_eq!(reads[0].scan.projection, Some(vec![0]));
            assert!(reads[0].aspects.contains(&DependencyAspect::Ordering));
        }
        let invalid = crate::operation::Operation::plan(
            Arc::new(Selection {
                schema: scan.schema().inner().clone(),
                family: crate::operation::Family::Finite,
                invalid: true,
            }),
            vec![scan.clone()],
        )
        .unwrap();
        assert!(session.dependency_sources(&invalid, cancel).is_err());
    }
    #[test]
    fn hidden_producer_preserves_narrow_values_and_required_fields() {
        let context = SessionContext::new();
        let budget: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(8 << 20));
        let registry = Arc::new(pse_schema::RegistryBuilder::new().build().unwrap());
        let cancel = CancellationToken::new();
        let factory = crate::session::EngineFactory::from_builder(
            context.runtime_env(),
            budget.clone(),
            "scope-unit",
            SessionStateBuilder::from(context.state()),
        );
        let schema = Arc::new(Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Int64, false),
        ]));
        let table = Arc::new(MemTable::try_new(schema, vec![vec![]]).unwrap());
        let session = factory
            .candidate_checked(std::collections::BTreeMap::new(), registry, &cancel)
            .unwrap()
            .with_provider("source".into(), table.clone(), &cancel)
            .unwrap();
        let scan = LogicalPlanBuilder::scan("source", provider_as_source(table), None)
            .unwrap()
            .build()
            .unwrap();
        check_operation_families(&session, &scan, &cancel);
        let cached = session.cache_plan(scan.clone(), &cancel).unwrap();
        let plan = LogicalPlanBuilder::from(cached)
            .project([col("source.a")])
            .unwrap()
            .build()
            .unwrap();
        let reads = session.dependency_sources(&plan, &cancel).unwrap();
        assert_eq!(reads.len(), 1);
        assert_eq!(reads[0].scan.projection, Some(vec![0]));
        let joined = LogicalPlanBuilder::from(scan.clone())
            .alias("left")
            .unwrap()
            .cross_join(
                LogicalPlanBuilder::from(scan.clone())
                    .alias("right")
                    .unwrap()
                    .build()
                    .unwrap(),
            )
            .unwrap()
            .build()
            .unwrap();
        let cached = session.cache_plan(joined, &cancel).unwrap();
        let duplicate = LogicalPlanBuilder::from(cached)
            .project([col("left.a"), col("right.b")])
            .unwrap()
            .build()
            .unwrap();
        let reads = session.dependency_sources(&duplicate, &cancel).unwrap();
        assert_eq!(reads.len(), 2);
        assert!(
            reads
                .iter()
                .any(|source| source.scan.projection == Some(vec![0]))
        );
        assert!(
            reads
                .iter()
                .any(|source| source.scan.projection == Some(vec![1]))
        );
        // A requirement remains an independent semantic read even if its fields
        // are absent from the requested output tuple.
        let required = super::super::contract::ExecutionContract::plan(
            plan,
            Some(scan),
            [pse_schema::model::provider::OperationEffect::Read]
                .into_iter()
                .collect(),
        );
        let reads = session.dependency_sources(&required, &cancel).unwrap();
        assert!(reads.iter().any(|scan| {
            scan.scan
                .projection
                .as_ref()
                .is_none_or(|columns| columns.contains(&1))
        }));
        cancel.cancel();
        assert!(session.dependency_sources(&required, &cancel).is_err());
    }
}
