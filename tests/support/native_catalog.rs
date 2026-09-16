// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native integrity execution for catalog integration fixtures.
#![allow(
    dead_code,
    reason = "each integration binary uses its relevant fixture constructors"
)]

#[path = "session_factory.rs"]
mod session_factory;
use pse_catalog::computation::{ProducedStage, StageProducer};
use pse_catalog::store::membership::{AdmissionContext, SemanticValidator};
use pse_catalog::{Catalog, CatalogError, session::SessionFactory};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{RecordBatch, columnar::FieldCheckedBatch};
use pse_schema::{
    Registry,
    model::{PassSpec, RelationKey},
};
pub(crate) use session_factory::factory as from_reserver;
use std::{
    any::Any,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub(crate) fn factory(catalog: &Catalog) -> Arc<SessionFactory> {
    Arc::clone(catalog.session_factory())
}

pub(crate) fn with_invariants(catalog: Catalog) -> Catalog {
    let validator = pse_rules::validator::InvariantValidator::new(Arc::clone(catalog.registry()));
    catalog.with_semantic_validator(Arc::new(validator))
}

/// Register the declared one-input/one-output native projection used by a fixture.
#[expect(
    clippy::unwrap_used,
    reason = "fixture callers name their actual registered pass"
)]
pub(crate) fn with_projection(
    catalog: Catalog,
    pass: &str,
    value_increment: Option<u64>,
) -> Catalog {
    let factory = factory(&catalog);
    let invariants = pse_rules::validator::InvariantValidator::new(Arc::clone(catalog.registry()));
    let producer = Arc::new(Projection {
        spec: catalog.registry().pass(pass).unwrap().clone(),
        factory,
        value_increment,
    });
    catalog.with_semantic_validator(Arc::new(Validator {
        invariants,
        producer,
    }))
}

#[derive(Debug)]
struct Validator {
    invariants: pse_rules::validator::InvariantValidator,
    producer: Arc<Projection>,
}
impl SemanticValidator for Validator {
    fn validate<'a>(
        &'a self,
        registry: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.invariants.validate(registry, rows, session, cancel)
    }
    fn validate_checked<'a>(
        &'a self,
        registry: &'a Registry,
        rows: &'a BTreeMap<RelationKey, FieldCheckedBatch>,
        session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.invariants
            .validate_checked(registry, rows, session, cancel)
    }
    fn validate_sidecar<'a>(
        &'a self,
        registry: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.invariants
            .validate_sidecar(registry, rows, session, cancel)
    }
    fn validate_affected<'a>(
        &'a self,
        registry: &'a Registry,
        rows: &'a BTreeMap<RelationKey, FieldCheckedBatch>,
        changed: &'a BTreeSet<RelationKey>,
        session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.invariants
            .validate_affected(registry, rows, changed, session, cancel)
    }
    fn stage_producer(&self, pass: SemanticId) -> Option<Arc<dyn StageProducer>> {
        (pass == self.producer.spec.id).then(|| {
            let producer: Arc<dyn StageProducer> = self.producer.clone();
            producer
        })
    }
}

#[derive(Debug)]
struct Projection {
    spec: PassSpec,
    factory: Arc<SessionFactory>,
    value_increment: Option<u64>,
}
impl StageProducer for Projection {
    fn spec(&self) -> &PassSpec {
        &self.spec
    }
    fn bind_session(
        &self,
        catalog: &Catalog,
        inputs: &AdmissionContext,
        _: Option<&(dyn Any + Send + Sync)>,
        cancel: &CancellationToken,
    ) -> Result<pse_catalog::session::SnapshotSession, CatalogError> {
        self.factory
            .open_session(
                inputs.parents.values().cloned().collect(),
                Arc::clone(catalog.registry()),
                cancel,
            )
            .map(|session| {
                session.with_purpose(pse_schema::model::provider::OperationPurpose::Construct)
            })
    }
    fn execute<'a>(
        &'a self,
        catalog: &'a Catalog,
        inputs: &'a AdmissionContext,
        _: Option<&'a (dyn Any + Send + Sync)>,
        _: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<ProducedStage, CatalogError>> {
        Box::pin(async move {
            use datafusion::logical_expr::{LogicalPlanBuilder, col, lit};
            use pse_catalog::session::output::{
                declare_relation_output, forget_relation_annotations,
            };
            let [input] = self.spec.inputs.as_slice() else {
                return Err(invalid("fixture input count"));
            };
            let [output] = self.spec.outputs.as_slice() else {
                return Err(invalid("fixture output count"));
            };
            let source = catalog
                .registry()
                .relation(&input.relation)
                .ok_or_else(|| invalid("source declaration"))?;
            let target = catalog
                .registry()
                .relation(&output.relation)
                .ok_or_else(|| invalid("output declaration"))?;
            let parent = inputs
                .parents
                .get(input.port)
                .ok_or_else(|| invalid("input role"))?;
            let source = parent
                .relation(source.key.namespace.as_str(), source.key.name)
                .ok_or_else(|| invalid("input relation"))?;
            let session = self.factory.candidate_checked_roles(
                BTreeMap::from([("input".to_owned(), source.checked().clone())]),
                Arc::clone(catalog.registry()),
                cancel,
            )?;
            let native = || {
                let plan =
                    forget_relation_annotations(session.scan_role("input").map_err(|error| {
                        datafusion::common::DataFusionError::External(Box::new(error))
                    })?)?;
                let expressions = target
                    .columns
                    .iter()
                    .map(|column| {
                        let expression = match (column.name(), self.value_increment) {
                            ("value", Some(increment)) => col("value") + lit(increment),
                            _ => col(column.name()),
                        };
                        expression.alias(column.name())
                    })
                    .collect::<Vec<_>>();
                let plan = LogicalPlanBuilder::from(plan)
                    .project(expressions)?
                    .build()?;
                declare_relation_output(plan, catalog.registry(), target)
            };
            let plan = native().map_err(|error| CatalogError::UserModel {
                message: error.to_string(),
            })?;
            let completed = session.prepare(plan, cancel)?.execute(cancel).await?;
            let output_batch = completed.checked_relation(catalog.registry(), target, cancel)?;
            Ok(ProducedStage {
                outputs: BTreeMap::from([(output.port.to_owned(), output_batch)]),
                findings: Vec::new(),
                derivations: Vec::new(),
                plans: Vec::new(),
            })
        })
    }
}
fn invalid(message: &str) -> CatalogError {
    CatalogError::UserModel {
        message: message.to_owned(),
    }
}
