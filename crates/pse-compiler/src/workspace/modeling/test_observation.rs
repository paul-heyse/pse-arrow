// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private observations for compiler tests. Missing or incomplete evidence is never zero work.
use super::*;
use pse_diagnostics::observation::{
    EnterGuard, ObservationContext, ObservationHub, OperationGuard, Policy, Value,
};
use salsa::plumbing::{Ingredient, ZalsaDatabase};
use std::{
    sync::{Mutex, OnceLock},
    time::Duration,
};

/// Selected query families; semantic-body admission uses its ordinary counters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Query {
    Selected,
    Specialized,
    Projection,
    InstancePrelude,
    FunctionFragment,
    FieldFragment,
    DocumentTable,
    Parsed,
    Resolved,
    Typed,
    EffectiveMembers,
    Occurrence,
    CheckedExpressions,
    ExpressionReferences,
}
#[derive(Debug, Default)]
struct Trace {
    truncated: bool,
    unregistered: bool,
    total: usize,
    executions: BTreeMap<Query, usize>,
}
#[derive(Clone, Debug, Default)]
pub(in crate::workspace) struct QueryObservations(Arc<Mutex<Trace>>);
impl QueryObservations {
    pub(in crate::workspace) fn observer(&self) -> Arc<GenerationObserver> {
        Arc::new(GenerationObserver {
            trace: self.0.clone(),
            queries: OnceLock::new(),
        })
    }
    fn complete(&self) -> Option<std::sync::MutexGuard<'_, Trace>> {
        let trace = self.0.lock().unwrap();
        (trace.total != 0 && !trace.truncated && !trace.unregistered).then_some(trace)
    }
    pub(super) fn count(&self, query: Query) -> usize {
        self.complete()
            .expect("query observation missing, unregistered or truncated")
            .executions
            .get(&query)
            .copied()
            .unwrap_or(0)
    }
    pub(super) fn total(&self) -> usize {
        self.complete()
            .expect("query observation missing, unregistered or truncated")
            .total
    }
}

/// Salsa allocates ingredient identities per database. Register from that generation's
/// real tracked functions, before any tracked function is executed.
pub(in crate::workspace) struct GenerationObserver {
    trace: Arc<Mutex<Trace>>,
    queries: OnceLock<BTreeMap<salsa::IngredientIndex, Query>>,
}
impl GenerationObserver {
    pub(in crate::workspace) fn register(&self, db: &CompilerDatabase) {
        let zalsa = db.zalsa();
        let mut queries = BTreeMap::new();
        let mut register = |index, query| {
            assert!(
                queries.insert(index, query).is_none(),
                "ambiguous query ingredient"
            );
        };
        for (index, query) in [
            (
                selected::fn_ingredient_(db, zalsa).ingredient_index(),
                Query::Selected,
            ),
            (
                specialized::fn_ingredient_(db, zalsa).ingredient_index(),
                Query::Specialized,
            ),
            (
                instance_prelude::fn_ingredient_(db, zalsa).ingredient_index(),
                Query::InstancePrelude,
            ),
            (
                function_fragment::fn_ingredient_(db, zalsa).ingredient_index(),
                Query::FunctionFragment,
            ),
            (
                field_fragment::fn_ingredient_(db, zalsa).ingredient_index(),
                Query::FieldFragment,
            ),
            (
                document_table::fn_ingredient_(db, zalsa).ingredient_index(),
                Query::DocumentTable,
            ),
            (executable::projection_ingredient(db), Query::Projection),
        ]
        .into_iter()
        .chain(granular::observation_ingredients(db))
        {
            register(index, query);
        }
        self.queries.set(queries).expect("one mapping per database");
    }

    pub(in crate::workspace) fn observe(&self, event: &salsa::Event) {
        if let salsa::EventKind::WillExecute { database_key } = event.kind {
            let mut trace = self.trace.lock().unwrap();
            let Some(queries) = self.queries.get() else {
                trace.unregistered = true;
                return;
            };
            if trace.total == 1_000_000 {
                trace.truncated = true;
                return;
            }
            trace.total += 1;
            // Unselected ingredients count toward the full trace, never toward a selected family.
            if let Some(query) = queries.get(&database_key.ingredient_index()) {
                *trace.executions.entry(*query).or_default() += 1;
            }
        }
    }
}

pub(super) fn observed_workspace(
    context: CompilerContext,
    limits: WorkspaceLimits,
) -> (CompilerWorkspace, QueryObservations) {
    let observations = QueryObservations::default();
    let workspace =
        CompilerWorkspace::with_shared_events(context, limits, None, Some(observations.clone()))
            .unwrap();
    (workspace, observations)
}

pub(super) fn metric_count(context: &ObservationContext, name: &str) -> u64 {
    let snapshot = context.reader().snapshot(Duration::from_secs(60));
    let coverage = &snapshot.coverage;
    assert!(
        coverage.enabled
            && coverage.available
            && !coverage.curtailed
            && !coverage.interrupted
            && coverage.dropped_updates == 0
            && coverage.refused_items == 0
            && !coverage.counter_overflow,
        "incomplete aggregate observation for {name}: {coverage:?}"
    );
    let metric = snapshot
        .metrics
        .iter()
        .find(|metric| metric.name == name)
        .unwrap_or_else(|| panic!("missing aggregate observation for {name}"));
    match metric.cumulative {
        Value::Count(value) => value,
        ref other => panic!("unexpected counter for {name}: {other:?}"),
    }
}

pub(super) struct BodyObservations {
    _hub: Arc<ObservationHub>,
    _operation: OperationGuard,
    context: ObservationContext,
}
impl BodyObservations {
    pub(super) fn new() -> Self {
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("semantic body admission control");
        let context = operation.context();
        // An explicit producer baseline permits zero before first admission. A missing
        // metric or lost update still refuses observation, and cold controls prove that
        // actual admissions advance these same counters.
        context.count("compiler.semantic_body_admissions", "count", 0);
        context.count("compiler.semantic_body_admissions_completed", "count", 0);
        Self {
            _hub: hub,
            _operation: operation,
            context,
        }
    }
    pub(super) fn enter(&self) -> EnterGuard {
        self.context.enter()
    }
    pub(super) fn admissions(&self) -> usize {
        usize::try_from(metric_count(
            &self.context,
            "compiler.semantic_body_admissions",
        ))
        .unwrap()
    }
    pub(super) fn completed(&self) -> usize {
        usize::try_from(metric_count(
            &self.context,
            "compiler.semantic_body_admissions_completed",
        ))
        .unwrap()
    }
}

#[test]
fn query_observation_requires_complete_evidence_and_positive_cold_invalidation_controls() {
    let absent = QueryObservations::default();
    assert!(absent.complete().is_none());
    let (mut workspace, observed) = observed_workspace(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    );
    let rows = |value| {
        pse_authoring::language::parse(
            &format!("package observed {{ def Root {{ var x:Scalar; eq e:x=={value}; }} }}"),
            SemanticId::from_bytes([95; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap()
    };
    let cold = workspace
        .publish_modeling(rows(1), PhysicalScope::default())
        .unwrap();
    let count = observed.count(Query::Parsed);
    let resolutions = observed.count(Query::Resolved);
    assert!(count > 0);
    assert!(resolutions > 0);
    let warm = workspace
        .publish_modeling(rows(1), PhysicalScope::default())
        .unwrap();
    assert!(Arc::ptr_eq(&cold, &warm));
    assert_eq!(observed.count(Query::Parsed), count);
    let edited = workspace
        .publish_modeling(rows(2), PhysicalScope::default())
        .unwrap();
    assert_eq!(observed.count(Query::Parsed), count + 1);
    assert_eq!(observed.count(Query::Resolved), resolutions);
    let unregistered = absent.observer();
    let mut fresh = CompilerWorkspace::with_events(
        workspace.inputs.clone(),
        workspace.limits,
        Some(Box::new(move |event| unregistered.observe(&event))),
    )
    .unwrap();
    let fresh_edited = fresh
        .publish_modeling(rows(2), PhysicalScope::default())
        .unwrap();
    assert_eq!(edited.checked(), fresh_edited.checked());
    assert!(absent.0.lock().unwrap().unregistered);
    assert!(absent.complete().is_none());
    observed.0.lock().unwrap().truncated = true;
    assert!(observed.complete().is_none());
}
