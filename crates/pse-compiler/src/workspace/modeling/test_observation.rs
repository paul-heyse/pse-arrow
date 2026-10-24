// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private observations for compiler tests. Missing or incomplete evidence is never zero work.
use super::*;
use pse_diagnostics::observation::{
    EnterGuard, ObservationContext, ObservationHub, OperationGuard, Policy, Value,
};
use std::{sync::Mutex, time::Duration};

/// Query spellings belong only here; semantic-body admission uses its ordinary counters.
#[derive(Clone, Copy, Debug)]
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
impl Query {
    fn name(self) -> &'static str {
        match self {
            Self::Selected => "selected(",
            Self::Specialized => "specialized(",
            Self::Projection => "projection(",
            Self::InstancePrelude => "instance_prelude(",
            Self::FunctionFragment => "function_fragment(",
            Self::FieldFragment => "field_fragment(",
            Self::DocumentTable => "document_table(",
            Self::Parsed => "parsed(",
            Self::Resolved => "resolved(",
            Self::Typed => "typed(",
            Self::EffectiveMembers => "effective_members(",
            Self::Occurrence => "occurrence(",
            Self::CheckedExpressions => "checked_expressions(",
            Self::ExpressionReferences => "expression_references(",
        }
    }
}

#[derive(Debug, Default)]
struct Trace {
    observed: bool,
    truncated: bool,
    executions: Vec<String>,
}
#[derive(Clone, Debug, Default)]
pub(super) struct QueryObservations(Arc<Mutex<Trace>>);
impl QueryObservations {
    pub(super) fn callback(&self) -> Box<dyn Fn(salsa::Event) + Send + Sync> {
        let trace = self.0.clone();
        Box::new(move |event| {
            let mut trace = trace.lock().unwrap();
            trace.observed = true;
            if matches!(event.kind, salsa::EventKind::WillExecute { .. }) {
                // An exceeded bound invalidates this observation instead of silently
                // turning its retained prefix into a smaller execution count.
                if trace.executions.len() == 1_000_000 {
                    trace.truncated = true;
                } else {
                    trace.executions.push(format!("{:?}", event.kind));
                }
            }
        })
    }
    fn complete(&self) -> Option<std::sync::MutexGuard<'_, Trace>> {
        let trace = self.0.lock().unwrap();
        (trace.observed && !trace.truncated).then_some(trace)
    }
    pub(super) fn count(&self, query: Query) -> usize {
        self.complete()
            .expect("query observation missing or truncated")
            .executions
            .iter()
            .filter(|event| event.contains(query.name()))
            .count()
    }
    pub(super) fn total(&self) -> usize {
        self.complete()
            .expect("query observation missing or truncated")
            .executions
            .len()
    }
}

pub(super) fn observed_workspace(
    context: CompilerContext,
    limits: WorkspaceLimits,
) -> (CompilerWorkspace, QueryObservations) {
    let observations = QueryObservations::default();
    let workspace =
        CompilerWorkspace::with_events(context, limits, Some(observations.callback())).unwrap();
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
    assert!(count > 0);
    let warm = workspace
        .publish_modeling(rows(1), PhysicalScope::default())
        .unwrap();
    assert!(Arc::ptr_eq(&cold, &warm));
    assert_eq!(observed.count(Query::Parsed), count);
    workspace
        .publish_modeling(rows(2), PhysicalScope::default())
        .unwrap();
    assert!(observed.count(Query::Parsed) > count);
    observed.0.lock().unwrap().truncated = true;
    assert!(observed.complete().is_none());
}
