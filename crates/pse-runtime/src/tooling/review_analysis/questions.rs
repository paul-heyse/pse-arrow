// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Question-specific factual interpretation; source policy owners remain authoritative.
use super::*;
use AnalysisQuestion as Q;
use StatementKind::{Established, Hypothesis, Limitation};
use pse_diagnostics::observation::{MetricSnapshot, ProducerRole};

pub(super) fn limitations(b: &mut Builder) {
    for (q, code, message, next) in [
        (
            Q::Preparation,
            "preparation.scope",
            "Invocation v2 begins after earlier preparation. Missing parent construction/memo/lookup/rebind work is unknown, not zero. Changed scientific/provider support may legitimately require new preparation.",
            NextObservation::InspectRelated,
        ),
        (
            Q::EvaluatorWork,
            "evaluator.population",
            "Requested calls, completed calls, actual evaluations, cache projections and derivative construction are different populations. Shared contributors and provider effects are not multiplied per authored alias or erased by cache hits.",
            NextObservation::InspectRelated,
        ),
        (
            Q::EvaluationCost,
            "evaluation.cost_scope",
            "Only compatible completed count/duration populations support inclusive means. Missing product/input/start/reuse context withholds a normalized winner. Inclusive elapsed cost may include implicit work or contention; CPU stacks/nested work are needed for intrinsic cost attribution.",
            NextObservation::CpuProfile,
        ),
        (
            Q::RootProgress,
            "root.progress_scope",
            "Trial checkpoints and accepted iterates retain their producer meanings. Raw-map ratios are not a contraction proof, and a native search direction is not accepted physical displacement. Missing executed physical pullback remains unknown.",
            NextObservation::InspectRelated,
        ),
        (
            Q::DynamicWork,
            "dynamics.scope",
            "Forward, sensitivity, quadrature, backward and replay populations remain separate. Requested directions/outputs do not count executed internal vectors. Qualified local-error contributions do not certify global trajectory error.",
            NextObservation::InspectRelated,
        ),
        (
            Q::DependencyReadiness,
            "dependency.scope",
            "Current-attempt prefixes do not close historical retry membership. Recorded readiness and exact links localize missing endpoints; overlap/nesting alone cannot establish a critical path or limiting dependency.",
            NextObservation::InspectRelated,
        ),
        (
            Q::FactorRecovery,
            "factor.scope",
            "Outer factor calls do not count inner retry/refinement work. Substitutions do not each incur a new factorization. Native matrix coordinates may be scaled, permuted or perturbed; absent inner hooks remain unavailable.",
            NextObservation::InspectRelated,
        ),
        (
            Q::Waiting,
            "waiting.scope",
            "Admission/queue, supervisor join, worker/native, qualification, terminal, response and later access are separate boundaries. Join wait can overlap native work; shared-flight waiters are not repeated construction. Wall minus process CPU is not wait time; scheduler/off-CPU evidence is required for a cause.",
            NextObservation::TimelineCapture,
        ),
        (
            Q::Comparability,
            "comparison.scope",
            "Missing effective options, queried defaults, original physical inputs or start/reuse context never proves equality. Existing invocation comparison refusals remain authoritative; unlike scientific products are not ranked.",
            NextObservation::InspectRelated,
        ),
        (
            Q::PhysicalAttribution,
            "physical.mapping_scope",
            "Only supplied original source/variable/row/occurrence identities and actual transform witnesses support attribution. Aliases, eliminated/fixed coordinates and Schur/KKT/cone transforms require their actual mappings. No universal dual/stationarity pullback or fresh diagnostic evaluation is implied.",
            NextObservation::InspectRelated,
        ),
    ] {
        b.add(q, Limitation, code, message, vec![], vec![next]);
    }
}
pub(super) fn metric_question(name: &str) -> Option<Q> {
    if name.starts_with("compiler.") {
        Some(Q::Preparation)
    } else if name.starts_with("math.demand.") || name.starts_with("math.census.") {
        Some(Q::EvaluatorWork)
    } else if name.starts_with("math.")
        && [
            "construction",
            "preparation",
            "rebind",
            "cache_reuse",
            "memo",
            "lookup",
        ]
        .iter()
        .any(|term| name.contains(term))
    {
        Some(Q::Preparation)
    } else if name.starts_with("diffsol.klu.")
        || name.starts_with("diffsol.faer.")
        || name.starts_with("idas.linear.")
        || name.starts_with("arclength.faer.")
        || name.starts_with("conditioning.")
        || name.starts_with("native.linear.")
        || name.starts_with("linear.")
        || name.starts_with("factor.")
    {
        Some(Q::FactorRecovery)
    } else if name.starts_with("dynamic.")
        || name.starts_with("idas.")
        || name.starts_with("diffsol.")
    {
        Some(Q::DynamicWork)
    } else if [
        "kinsol.",
        "diffsol_nl.",
        "ipopt.",
        "pounce.",
        "uno.",
        "clarabel.",
        "highs.",
        "scip.",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
    {
        Some(Q::RootProgress)
    } else {
        None
    }
}
pub(super) fn metric_ref(
    member: &str,
    pointer: &str,
    index: usize,
    metric: &MetricSnapshot,
) -> AnalysisSourceReference {
    source(
        member,
        format!(
            "{pointer}/metrics/{index}/{}",
            if matches!(metric.cumulative, Value::Unavailable(_)) {
                "latest"
            } else {
                "cumulative"
            }
        ),
        if matches!(metric.cumulative, Value::Unavailable(_)) {
            metric.latest.clone()
        } else {
            metric.cumulative.clone()
        },
        !matches!(metric.cumulative, Value::Unavailable(_))
            && (metric.cumulative_partial || metric.resets > 0),
    )
}
pub(super) fn operations(
    b: &mut Builder,
    set: &FrozenEvidenceSet,
    ids: &[OperationId],
) -> Result<(), ObservationError> {
    for id in ids {
        b.checkpoint()?;
        let Some((member, pointer, op)) = operation_member(set, *id) else {
            continue;
        };
        for (i, metric) in op.metrics.iter().enumerate() {
            b.checkpoint()?;
            let Some(q) = metric_question(&metric.name) else {
                continue;
            };
            if !b.selected(q) {
                continue;
            }
            let path = format!("{pointer}/metrics/{i}");
            if !b.admit(&member, &path) {
                continue;
            }
            let latest = matches!(metric.cumulative, Value::Unavailable(_));
            let partial = (!latest && (metric.cumulative_partial || metric.resets > 0))
                || op.coverage.curtailed
                || !op.coverage.available;
            b.add(q,Established,"owner.metric",format!("{}: {} {:?} {}, producer {:?}, generation {}, resets {}, observed_at {:?}. {} A selected earlier window does not reconstruct an earlier owner state.",metric.name,if latest {"latest"} else {"cumulative"},if latest {&metric.latest} else {&metric.cumulative},metric.unit,metric.producer,metric.generation,metric.resets,metric.observed_at,if latest {"This source value is a nonadditive gauge/text/state or an absolute checkpoint whose additive baseline is unavailable; it is not a work subtotal. The original cumulative availability remains in the supplied metric."} else {"This is the original additive publication population; generation/reset/unknown-baseline limits remain source-owned."}),vec![metric_ref(&member,&pointer,i,metric)],vec![]);
            if metric.name == "diffsol.klu.refactor.conditioning_rejected"
                && matches!(metric.cumulative, Value::Count(count) if count > 0)
            {
                b.add(Q::FactorRecovery, Hypothesis, "factor.conditioning_recovery", "KLU reports actual conditioning-rejected refactors. The source's rejected-full-fallback and factor populations show recovery work separately; no additional iteration or exclusive cost is inferred.", vec![metric_ref(&member,&pointer,i,metric)], vec![NextObservation::CpuProfile]);
            }
            if metric.name.starts_with("diffsol.")
                && metric.name.contains("number_of_linear_solver_setups_from_")
            {
                b.add(Q::DynamicWork, Established, "dynamics.setup_cause", format!("{} retains its actual Diffsol setup cause (checkpoint restoration, convergence/error-test recovery or successful-step refresh), independently of a final segment summary. It does not count a generic solver iteration.", metric.name), vec![metric_ref(&member,&pointer,i,metric)], vec![]);
            }
            if partial {
                b.add(q,Limitation,"owner.metric_partial","Counter resets, unavailable collection or source loss prevents treating this retained value as complete work.",vec![metric_ref(&member,&pointer,i,metric)],vec![NextObservation::AwaitOwnerCheckpoint]);
            }
            if q == Q::DynamicWork
                && matches!(metric.cumulative,Value::Count(value) if value>0)
                && [
                    "error_test_failures",
                    "convergence_failures",
                    "nonlinear_failures",
                ]
                .iter()
                .any(|name| metric.name.contains(name))
            {
                b.add(q,Hypothesis,"dynamics.numerical_difficulty_observed","The native owner reports actual error-test or convergence failures. These are evidence of numerical difficulty in this named segment; they do not measure global trajectory error or exclude reconstruction and replay costs.",vec![metric_ref(&member,&pointer,i,metric)],vec![NextObservation::InspectRelated]);
            }
        }
        // Actual owner-clock relationships, never a subtraction of unrelated clocks.
        for (i, phase) in op.phases.iter().enumerate() {
            if !b.selected(Q::Waiting) {
                continue;
            }
            let path = format!("{pointer}/phases/{i}");
            if !b.admit(&member, &path) {
                continue;
            }
            b.add(Q::Waiting,Established,"waiting.active_phase",format!("Owner last published active phase {} on operation {}. This is observed activity, not a scheduler-blocking cause.",phase.kind,op.id),vec![source(&member,format!("{path}/kind"),Value::Text(phase.kind.clone()),op.coverage.stale)],vec![NextObservation::TimelineCapture]);
        }
        for (i, child) in op.children.iter().enumerate() {
            if !b.selected(Q::Waiting) {
                continue;
            }
            let path = format!("{pointer}/children/{i}");
            if !b.admit(&member, &path) {
                continue;
            }
            b.add(Q::Waiting,Established,"waiting.relationship",format!("Recorded {:?} relationship from {} to {}, with available={} and independent completion {:?}. Shared work is charged only at its performing owner.",child.relationship,op.id,child.id,child.available,child.finished_at),vec![source(&member,format!("{path}/available"),Value::Boolean(child.available),!child.available)],vec![NextObservation::InspectRelated]);
        }
    }
    Ok(())
}
pub(super) fn family_question(family: &str) -> Option<Q> {
    match family {
        "compiler_query.v1" => Some(Q::Preparation),        "evaluator_demand_batch.v1" | "evaluator_demand.v1" | "evaluator_census.v1" => {
            Some(Q::EvaluatorWork)
        }
        "root_raw_map.v1"
        | "root_evaluation.v1"
        | "nonlinear_iteration.v1"
        | "nonlinear_iteration.v2"
        | "conic_iteration.v1"
        | "branch_and_bound.v1" => Some(Q::RootProgress),
        "dynamic_segment_summary.v1"
        | "dynamic_replay_effort.v1"
        | "dynamic_output.v1"
        | "dynamic_adjoint.v1"
        | "dynamic_native_return.v1"
        | "dynamic_local_error.v1" => Some(Q::DynamicWork),
        "native_factor.v1"
        | "native_factor.v2"
        | "native_refinement.v1"
        | "native_refinement.v2"
        | "native_linear_call.v1"
        | "native_linear_work.v1" => Some(Q::FactorRecovery),
        _ => None,
    }
}
pub(super) fn declared_role(
    set: &FrozenEvidenceSet,
    record: &CoherentRecord,
) -> Option<ProducerRole> {
    set.operation(record.producer.operation)
        .and_then(|op| {
            op.producers
                .iter()
                .find(|producer| producer.descriptor.id == record.producer)
                .map(|producer| producer.descriptor.role)
        })
        .or_else(|| {
            set.detail_pages
                .iter()
                .flat_map(|page| &page.producers)
                .find(|producer| producer.id == record.producer)
                .map(|producer| producer.role)
        })
        .or_else(|| {
            set.detail_pages
                .iter()
                .flat_map(|page| &page.activations)
                .find(|activation| {
                    activation.producer.id == record.producer && activation.family == record.family
                })
                .map(|activation| activation.producer.role)
        })
}
pub(super) fn eligible(q: Q, role: Option<ProducerRole>) -> bool {
    match q {
        Q::Preparation => role == Some(ProducerRole::Workflow),
        Q::EvaluatorWork => role == Some(ProducerRole::Evaluator),
        Q::RootProgress | Q::DynamicWork | Q::FactorRecovery => matches!(
            role,
            Some(
                ProducerRole::NativeCallback
                    | ProducerRole::NativeIteration
                    | ProducerRole::NativeFactor
            )
        ),
        _ => false,
    }
}
fn field_text<'a>(event: &'a Event<'_>, name: &str) -> Option<&'a str> {
    let mut values = event
        .record
        .values
        .iter()
        .filter(|field| field.name == name && field.availability.is_none());
    match (&values.next()?.value, values.next()) {
        (Value::Text(value), None) => Some(value),
        _ => None,
    }
}
fn field_count(event: &Event<'_>, name: &str) -> Option<u64> {
    let mut values = event
        .record
        .values
        .iter()
        .filter(|field| field.name == name && field.availability.is_none());
    let field = values.next()?;
    if values.next().is_some() {
        return None;
    }
    match field.value {
        Value::Count(value) => Some(value),
        Value::Integer(value) => u64::try_from(value).ok(),
        _ => None,
    }
}
fn field_admitted(b: &Builder, event: &Event<'_>, name: &str) -> bool {
    event
        .record
        .values
        .iter()
        .enumerate()
        .filter(|(_, field)| field.name == name)
        .all(|(index, _)| {
            b.inspected.contains(&format!(
                "{}#{}/values/{index}",
                event.member, event.pointer
            ))
        })
        && event.record.values.iter().any(|field| field.name == name)
}
fn event_evidence(b: &mut Builder, event: &Event<'_>) -> Vec<AnalysisSourceReference> {
    let mut result = vec![event.reference(None)];
    for (i, field) in event.record.values.iter().enumerate() {
        let pointer = format!("{}/values/{i}", event.pointer);
        if i >= 32 {
            b.omitted.insert(format!("{}#{pointer}", event.member));
            continue;
        }
        if b.admit(&event.member, &pointer) {
            let mut reference = event.reference(Some(i));
            reference.partial |= field.availability.is_some();
            result.push(reference);
        }
    }
    result
}
pub(super) fn records(
    b: &mut Builder,
    set: &FrozenEvidenceSet,
    events: &[Event<'_>],
) -> Result<(), ObservationError> {
    let mut populations: BTreeMap<(String, String), (usize, Vec<AnalysisSourceReference>)> =
        BTreeMap::new();
    for event in events {
        b.checkpoint()?;
        let Some(q) = family_question(&event.record.family) else {
            continue;
        };
        if !b.selected(q) {
            continue;
        }
        if !b.admit(&event.member, &event.pointer) {
            continue;
        }
        if event.predecessor {
            b.add(q,Established,"event.predecessor","The last original event before the selected interval is retained only as a predecessor, not charged as selected-window work.",vec![event.reference(None)],vec![]);
            continue;
        }
        if !eligible(q, declared_role(set, event.record)) {
            b.add(q,Limitation,"event.producer_unavailable","The supplied family label has no matching eligible performing-producer declaration. Original fields remain readable, but family text alone does not authorize a numerical/work interpretation.",vec![event.reference(None)],vec![NextObservation::InspectRelated]);
            continue;
        }
        let population = populations
            .entry((event.record.family.clone(), event.record.phase.clone()))
            .or_default();
        population.0 += 1;
        if population.1.len() < 16 {
            population.1.push(event.reference(None));
        }
        let evidence = event_evidence(b, event);
        let values = event
            .record
            .values
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                b.inspected.contains(&format!(
                    "{}#{}/values/{index}",
                    event.member, event.pointer
                ))
            })
            .map(|(_, field)| {
                format!(
                    "{}={:?} [{}]{}",
                    field.name,
                    field.value,
                    field.unit,
                    if field.availability.is_some() {
                        " (unavailable)"
                    } else {
                        ""
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let message = match q {
            Q::Preparation => format!("Actual compiler {} event: query family {:?}, semantic subject {:?}, workspace generation {:?}. {}. An unmapped event stays unattributed. Execution is not a proved cache-miss cause; validation is not root-wide preparation or a duration.", event.record.phase, field_text(event, "query_family"), field_text(event, "semantic_subject"), field_count(event, "workspace_generation"), values),
            Q::EvaluatorWork => format!(
                "Actual {} event, demand {:?}, requested_order {:?}: {}. Batch increments and per-row cumulative histories are separate; these populations are never added together.",
                event.record.family,
                field_admitted(b, event, "demand")
                    .then(|| field_text(event, "demand"))
                    .flatten(),
                field_admitted(b, event, "requested_order")
                    .then(|| field_count(event, "requested_order"))
                    .flatten(),
                values
            ),
            Q::RootProgress => format!(
                "Actual {} / {} checkpoint, coordinates {:?}, norm {:?}: {}. Accepted/trial semantics and physical error ratios are retained literally.",
                event.record.family,
                event.record.phase,
                event.record.coordinates,
                event.record.norm,
                values
            ),
            Q::DynamicWork => format!(
                "Actual {} / {} segment: {}. Replay acceptance remains separate from forward, sensitivity, quadrature and backward counters; no requested output is charged as an internal vector.",
                event.record.family, event.record.phase, values
            ),
            Q::FactorRecovery => format!(
                "Actual {} / {} record: {}. Parent-call/path ordinals scope recovery; factor work is not multiplied by substitution count.",
                event.record.family, event.record.phase, values
            ),
            _ => String::new(),
        };
        b.add(
            q,
            Established,
            "event.owner_tuple",
            message,
            evidence.clone(),
            vec![],
        );
        if q == Q::FactorRecovery
            && [
                "mc64_retry_attempts",
                "mc64_to_infnorm_fallbacks",
                "static_pivot_perturbations",
                "correction_batches_issued",
            ]
            .iter()
            .any(|name| {
                field_admitted(b, event, name)
                    && field_count(event, name).is_some_and(|count| count > 0)
            })
        {
            b.add(q,Hypothesis,"factor.recovery_contributor","The source reports actual retries, fallback, perturbation or correction work. This may contribute to extra work, but exclusive cost and hidden inner attempts require same-parent durations or profiling.",evidence,vec![NextObservation::CpuProfile]);
        }
        if q == Q::EvaluatorWork
            && field_admitted(b, event, "math.demand.order_upgrade_misses")
            && field_count(event, "math.demand.order_upgrade_misses").is_some_and(|n| n > 0)
        {
            b.add(q,Hypothesis,"evaluator.order_upgrade","Actual order-upgrade misses are a candidate explanation for additional executed work. This does not charge separate derivative construction or shared arithmetic more than once.",vec![event.reference(None)],vec![NextObservation::InspectRelated]);
        }
    }
    for ((family, phase), (count, references)) in populations {
        if let Some(q) = family_question(&family) {
            b.add(q,Established,"event.logical_population",format!("Inspected {count} unique coherent {family} events in phase {phase}. Empty events count once; multiple fields, current/recent copies and equal timestamps do not multiply this retained population. This is not an assertion of complete source coverage."),references,vec![]);
        }
    }
    if b.selected(Q::FactorRecovery) {
        factor_groups(b, set, events);
    }
    if b.selected(Q::EvaluationCost) {
        evaluation_cost(b, set, events);
    }
    // Batch records contain increments; indexed-row records contain cumulative histories.
    // Aggregate only the former, once per whole event, preserving performing generation/demand/order.
    type BatchKey = (EventKey, String, u64);
    type BatchSubtotal = (Option<u64>, Option<u64>, Vec<AnalysisSourceReference>);
    let mut batches: BTreeMap<BatchKey, BatchSubtotal> = BTreeMap::new();
    for event in events.iter().filter(|event| {
        b.selected(Q::EvaluatorWork)
            && !event.predecessor
            && event.record.family == "evaluator_demand_batch.v1"
            && eligible(Q::EvaluatorWork, declared_role(set, event.record))
            && b.inspected
                .contains(&format!("{}#{}", event.member, event.pointer))
    }) {
        if ![
            "demand",
            "requested_order",
            "math.demand.completed",
            "math.demand.evaluations",
        ]
        .iter()
        .all(|name| field_admitted(b, event, name))
        {
            continue;
        }
        let Some(demand) = field_text(event, "demand") else {
            continue;
        };
        let Some(order) = field_count(event, "requested_order") else {
            continue;
        };
        let r = event.record;
        let group = (
            (
                r.producer.operation.process_instance,
                r.producer.operation.sequence,
                r.producer.sequence,
                r.generation,
                0,
            ),
            demand.to_owned(),
            order,
        );
        let entry = batches
            .entry(group)
            .or_insert((Some(0), Some(0), Vec::new()));
        entry.0 = entry
            .0
            .zip(field_count(event, "math.demand.completed"))
            .and_then(|(a, b)| a.checked_add(b));
        entry.1 = entry
            .1
            .zip(field_count(event, "math.demand.evaluations"))
            .and_then(|(a, b)| a.checked_add(b));
        if entry.2.len() < 16 {
            entry.2.push(event.reference(None));
        }
    }
    for ((identity, demand, order), (completed, evaluations, references)) in batches {
        b.add(Q::EvaluatorWork,Established,"evaluator.batch_subtotal",format!("Performing operation {:?}/{}, producer {}, generation {}, demand {}, requested order {}: inspected batch increments contain completed={:?}, actual evaluations={:?}. None means a missing/overflowed subtotal. Indexed-row cumulative history and census rows were excluded; retained subtotals do not establish full-run totals.",identity.0,identity.1,identity.2,identity.3,demand,order,completed,evaluations),references,vec![]);
    }
    // Reuse the selected original numerical policy, over a copied admitted suffix only.
    if let Some(policy) = b
        .document
        .request
        .numerical_history_policy
        .clone()
        .filter(|_| b.selected(Q::RootProgress))
        && let Some(op) = set.operation(b.document.request.operation)
    {
        let mut snapshot = op.clone();
        for descriptor in set
            .detail_pages
            .iter()
            .flat_map(|page| &page.producers)
            .chain(set.detail_pages.iter().flat_map(|page| {
                page.activations
                    .iter()
                    .map(|activation| &activation.producer)
            }))
            .filter(|descriptor| descriptor.id.operation == op.id)
        {
            if !snapshot
                .producers
                .iter()
                .any(|producer| producer.descriptor.id == descriptor.id)
            {
                snapshot
                    .producers
                    .push(pse_diagnostics::observation::ProducerSnapshot {
                        descriptor: descriptor.clone(),
                        current: None,
                        recent: vec![],
                        coverage: pse_diagnostics::observation::RecordCoverage {
                            partial: true,
                            ..Default::default()
                        },
                    });
            }
        }
        let mut admitted_events = BTreeSet::new();
        for event in events
            .iter()
            .filter(|event| !event.predecessor && event.record.producer.operation == op.id)
        {
            if !b.admit(&event.member, &event.pointer) {
                continue;
            }
            let mut complete = true;
            for index in 0..event.record.values.len() {
                complete &= b.admit(&event.member, &format!("{}/values/{index}", event.pointer));
            }
            if complete {
                admitted_events.insert(key(event.record));
            }
        }
        for producer in &mut snapshot.producers {
            producer.recent = events
                .iter()
                .filter(|event| {
                    admitted_events.contains(&key(event.record))
                        && event.record.producer == producer.descriptor.id
                })
                .map(|event| event.record.clone())
                .collect();
            producer.recent.sort_by_key(|record| record.sequence);
            producer.current = producer.recent.last().cloned();
            producer.coverage.partial |= b.document.omitted_statements > 0 || !b.omitted.is_empty();
        }
        for statement in pse_diagnostics::inspection::explain_numerical_history(&snapshot, &policy)
        {
            let evidence = statement
                .evidence
                .iter()
                .filter_map(|reference| {
                    events
                        .iter()
                        .find(|event| {
                            event.record.producer.operation == reference.operation
                                && Some(event.record.producer) == reference.producer
                                && Some(event.record.generation) == reference.generation
                                && Some(event.record.sequence) == reference.record_sequence
                        })
                        .map(|event| {
                            event.reference(reference.metric.as_ref().and_then(|name| {
                                event
                                    .record
                                    .values
                                    .iter()
                                    .position(|field| &field.name == name)
                            }))
                        })
                })
                .collect();
            b.add(
                Q::RootProgress,
                statement.kind,
                &statement.code,
                statement.message,
                evidence,
                statement.next,
            );
        }
    }
    Ok(())
}
fn invocation_question(code: &str) -> Q {
    if code.contains("preparation") {
        Q::Preparation
    } else if code.contains("inclusive_interval")
        || code.contains("supervisor_endpoint")
        || code.contains("owner_interval")
        || code.contains("opaque_cost")
        || code.contains("timing_unavailable")
    {
        Q::Waiting
    } else if code.contains("seconds_per_iteration") {
        Q::Comparability
    } else if code.contains("unit_cost") || code.contains("causal_cost") {
        Q::EvaluationCost
    } else if code.contains("recorded_work") {
        Q::EvaluatorWork
    } else {
        Q::Comparability
    }
}
fn invocation_pointer(doc: &crate::invocation::InvocationEvidenceDocument, field: &str) -> String {
    // The typed comparison owner supplies values and required field names. Only optional
    // containers/tagged leaves need ancestor resolution; no invocation JSON copy is built.
    let Some(rest) = field.strip_prefix("/attempts/steps/") else {
        return if field.starts_with("/preparations/") && doc.preparations.is_none() {
            "/preparations".into()
        } else {
            field.into()
        };
    };
    let Some((index, rest)) = rest.split_once('/') else {
        return field.into();
    };
    let Some(step) = index
        .parse::<usize>()
        .ok()
        .and_then(|index| doc.attempts.steps.get(index))
    else {
        return "/attempts/steps".into();
    };
    let prefix = format!("/attempts/steps/{index}");
    if rest.starts_with("start/") && step.start.is_none() {
        return format!("{prefix}/start");
    }
    if rest.starts_with("work/") && step.work.is_none() {
        return format!("{prefix}/work");
    }
    for (name, values) in [
        ("effective_options", &step.effective_options),
        ("native_defaults", &step.native_defaults),
    ] {
        if let Some(key) = rest
            .strip_prefix(name)
            .and_then(|rest| rest.strip_prefix('/'))
        {
            let (escaped, suffix) = key.split_once('/').unwrap_or((key, ""));
            let key = escaped.replace("~1", "/").replace("~0", "~");
            let Some(value) = values.get(&key) else {
                return format!("{prefix}/{name}");
            };
            if suffix == "value" && matches!(value, pse_model::diagnostic::Observation::Missing) {
                return format!("{prefix}/{name}/{escaped}");
            }
        }
    }
    if rest.starts_with("native_observations/") {
        let Some(native) = &step.native_observations else {
            return format!("{prefix}/native_observations");
        };
        if rest == "native_observations/counters/wall_seconds/value" {
            if let pse_backend_native::solve::NativeCounters::Ipopt { wall_seconds, .. } =
                &native.counters
            {
                if wall_seconds.is_none() {
                    return format!("{prefix}/native_observations/counters/wall_seconds");
                }
            } else {
                return format!("{prefix}/native_observations/counters");
            }
        }
    }
    field.into()
}
fn copy_invocation_statements(
    b: &mut Builder,
    set: &FrozenEvidenceSet,
    document: crate::tooling::inspection_comparison::InvocationComparisonDocument,
) {
    for statement in document.statements {
        if !b.selected(invocation_question(&statement.code)) {
            continue;
        }
        let mut evidence = Vec::new();
        let mut complete = true;
        for reference in statement.evidence {
            if let Some((i, original)) = set.invocations.iter().enumerate().find(|(_, doc)| {
                doc.run_id == reference.invocation.run_id
                    && doc.attempt_id == reference.invocation.attempt_id
                    && doc.operation == reference.invocation.operation
            }) {
                let member = format!("invocations/{i}");
                if b.admit(&member, &reference.field) {
                    let pointer = invocation_pointer(original, &reference.field);
                    let missing = pointer != reference.field;
                    let value = if missing {
                        Value::Unavailable(format!(
                            "original field {} is absent below this retained parent",
                            reference.field
                        ))
                    } else {
                        reference.value
                    };
                    evidence.push(source(member, pointer, value, reference.partial || missing));
                } else {
                    complete = false;
                }
            } else {
                complete = false;
            }
        }
        if complete {
            b.add(
                invocation_question(&statement.code),
                statement.kind,
                &statement.code,
                statement.message,
                evidence,
                statement.next,
            );
        }
    }
    if b.document.request.question.is_none() {
        b.document.omitted_statements = b
            .document
            .omitted_statements
            .saturating_add(document.omitted_statements);
        // These omissions are owned by the existing bounded interpreter, not invented zero-sized sources.
        for index in 0..document.omitted_source_items {
            b.omitted.insert(format!(
                "invocation-owner-omission/{:?}/{:?}/{index}",
                document.baseline, document.candidate
            ));
        }
    }
}
pub(super) fn invocations(
    b: &mut Builder,
    set: &FrozenEvidenceSet,
) -> Result<(), ObservationError> {
    let selected = set
        .invocations
        .iter()
        .enumerate()
        .filter(|(_, doc)| doc.operation == Some(b.document.request.operation))
        .collect::<Vec<_>>();
    if selected.len() > 1 {
        return Err(invalid(
            "multiple invocation members claim the selected operation",
        ));
    }
    let Some((index, doc)) = selected.first().copied() else {
        return Ok(());
    };
    let member = format!("invocations/{index}");
    if !b.admit(&member, "") {
        return Ok(());
    }
    let window_is_full = b
        .document
        .selections
        .iter()
        .find(|selection| selection.operation == b.document.request.operation)
        .zip(set.operation(b.document.request.operation))
        .is_some_and(|(selection, snapshot)| {
            selection
                .interval_start
                .is_some_and(|start| start <= snapshot.interval_start)
                && selection
                    .interval_end
                    .is_some_and(|end| end >= snapshot.interval_end)
        });
    if !matches!(b.document.request.window, RecordedWindow::Recorded) && !window_is_full {
        b.add(Q::Comparability, Limitation, "comparison.absolute_window_unaligned",
            "Selected absolute source-clock window does not establish full invocation coverage. Full-invocation elapsed/work comparisons are withheld; separately retained original step facts remain readable.",
            vec![source(&member, "/clock_scope", Value::Text(doc.clock_scope.clone()), true)], vec![NextObservation::InspectRelated]);
    }
    let policy = crate::tooling::inspection_comparison::ComparisonPolicy {
        statements: 256,
        source_items: b.document.request.source_items.min(64),
    };
    copy_invocation_statements(
        b,
        set,
        crate::tooling::inspection_comparison::explain_invocation(doc, policy),
    );
    if let Some(baseline) =
        b.document.request.baseline.filter(|_| {
            matches!(b.document.request.window, RecordedWindow::Recorded) || window_is_full
        })
    {
        let baseline = set
            .invocations
            .iter()
            .filter(|doc| doc.operation == Some(baseline))
            .collect::<Vec<_>>();
        if baseline.len() > 1 {
            return Err(invalid(
                "multiple invocation members claim the baseline operation",
            ));
        }
        if let Some(baseline) = baseline.first() {
            copy_invocation_statements(
                b,
                set,
                crate::tooling::inspection_comparison::compare_invocations(baseline, doc, policy),
            );
        }
    }
    // Actual per-step physical conclusions are source facts, not acceptance inferred from history.
    for (i, step) in doc.attempts.steps.iter().enumerate() {
        if !b.selected(Q::RootProgress) {
            continue;
        }
        if !b.admit(&member, &format!("/attempts/steps/{i}")) {
            continue;
        }
        if let Some(value) = &step.normalized_physical_violation {
            b.add(Q::RootProgress,Established,"root.original_assessment",format!("Original step {} retained normalized physical violation {:?}; accepted={:?}. Native residual/step norms are not substituted for this original assessment.",step.step,value,step.accepted),vec![source(&member,format!("/attempts/steps/{i}/accepted"),step.accepted.map_or_else(||Value::Unavailable("original acceptance absent".into()),Value::Boolean),step.accepted.is_none())],vec![]);
        }
    }
    Ok(())
}
/// Shared exact membership selection for readiness statements and their prerequisites.
/// Request/set validation owns the finite selector bounds; missing supplied indices never match.
pub(super) fn inventory_selected(
    set: &FrozenEvidenceSet,
    request: &AnalysisRequest,
    ids: &[OperationId],
    index: usize,
) -> bool {
    let Some(inventory) = set.inventories.get(index) else {
        return false;
    };
    if !request.inventory_members.is_empty() {
        return request.inventory_members.contains(&index);
    }
    inventory.members.iter().any(|member| {
        member.operation.is_some_and(|id| ids.contains(&id))
            || set.invocations.iter().any(|invocation| {
                invocation.operation.is_some_and(|id| ids.contains(&id))
                    && member
                        .run
                        .as_ref()
                        .zip(invocation.canonical_run.as_ref())
                        .is_some_and(|(member, run)| member == run)
                    && member.attempt.as_ref().is_none_or(|attempt| {
                        invocation.canonical_attempt.as_ref() == Some(attempt)
                    })
            })
    })
}
pub(super) fn inventories(
    b: &mut Builder,
    set: &FrozenEvidenceSet,
    ids: &[OperationId],
) -> Result<(), ObservationError> {
    if !b.selected(Q::DependencyReadiness) {
        return Ok(());
    }
    for (i, inventory) in set.inventories.iter().enumerate() {
        b.checkpoint()?;
        if !inventory_selected(set, &b.document.request, ids, i) {
            continue;
        }
        let explicit = b.document.request.inventory_members.contains(&i);
        let member = format!("inventories/{i}");
        if b.admit(&member, "") {
            b.add(Q::DependencyReadiness, Established, "dependency.inventory_selection",
                format!("Exact supplied inventory {i}; explicit={explicit}, truncated={}. Its retained selection and generation fences describe membership, including unresolved gaps; a shared producer is not charged per occurrence.", inventory.truncated),
                vec![source(&member, "", Value::Text("exact inventory and retained fences".into()), inventory.truncated)], vec![]);
        }
        for (j, item) in inventory.members.iter().enumerate() {
            b.checkpoint()?;
            let pointer = format!("/members/{j}");
            if !b.admit(&member, &pointer) {
                continue;
            }
            let ready = item.terminal && item.result_manifest.is_some();
            b.add(Q::DependencyReadiness,Established,if ready {"dependency.endpoint_retained"} else {"dependency.readiness_unestablished"},format!("Exact member {} ({}) occurrence {:?}, attempt {:?}, operation {:?}: terminal={}, outcome={:?}, result_manifest={}, readable_prefix={:?}. {}",item.key,item.role,item.occurrence,item.attempt,item.operation,item.terminal,item.outcome,item.result_manifest.is_some(),item.readable_prefix,if ready {"A terminal manifest is retained; product eligibility remains original authority."} else {"This member lacks a retained terminal/manifest premise for complete result readiness."}),vec![source(&member,format!("{pointer}/key"),Value::Text(item.key.clone()),inventory.truncated),source(&member,format!("{pointer}/terminal"),Value::Boolean(item.terminal),inventory.truncated),source(&member,format!("{pointer}/result_manifest"),item.result_manifest.clone().map_or_else(||Value::Unavailable("no exact closed result manifest".into()),Value::Text),!ready)],vec![NextObservation::InspectRelated]);
            for (k, link) in item.links.iter().enumerate() {
                let pointer = format!("{pointer}/links/{k}");
                if !b.admit(&member, &pointer) {
                    continue;
                }
                let resolved = inventory
                    .members
                    .iter()
                    .any(|target| target.key == link.target);
                b.add(Q::DependencyReadiness,if resolved {Established} else {Limitation},"dependency.recorded_link",format!("{} --{}--> {}: supplied target present={resolved}. This is an actual relationship, without a wakeup/critical-path assertion.",item.key,link.role,link.target),vec![source(&member,format!("{pointer}/target"),Value::Text(link.target.clone()),!resolved)],vec![NextObservation::InspectRelated]);
            }
        }
        for (j, gap) in inventory.gaps.iter().enumerate() {
            if b.admit(&member, &format!("/gaps/{j}")) {
                b.add(
                    Q::DependencyReadiness,
                    Limitation,
                    "dependency.inventory_gap",
                    format!("{}: {}", gap.member, gap.reason),
                    vec![source(
                        &member,
                        format!("/gaps/{j}/reason"),
                        Value::Text(gap.reason.clone()),
                        true,
                    )],
                    vec![NextObservation::InspectRelated],
                );
            }
        }
    }
    Ok(())
}
pub(super) fn context(
    b: &mut Builder,
    set: &FrozenEvidenceSet,
    ids: &[OperationId],
) -> Result<(), ObservationError> {
    if !b.selected(Q::PhysicalAttribution) && !b.selected(Q::Comparability) {
        return Ok(());
    }
    for (i, context) in set.contexts.iter().enumerate() {
        b.checkpoint()?;
        if !context.operation.is_some_and(|id| ids.contains(&id)) {
            continue;
        }
        let member = format!("contexts/{i}");
        for (j, original) in context.sources.iter().enumerate() {
            if !b.selected(Q::PhysicalAttribution) {
                continue;
            }
            if !b.admit(&member, &format!("/sources/{j}")) {
                continue;
            }
            b.add(Q::PhysicalAttribution,Established,"physical.source_closure",format!("Original source identity {} with physical identity {} retains {} authored objects, {} physical sources and {} provider descriptors. This identifies an original closure; it does not establish a native coordinate pullback.",original.identity,original.physical_identity,original.objects.len(),original.physical_sources.len(),original.providers.len()),vec![source(&member,format!("/sources/{j}/identity"),Value::Text(original.identity.to_string()),false),source(&member,format!("/sources/{j}/physical_identity"),Value::Text(original.physical_identity.to_string()),false)],vec![]);
        }
        for (j, fact) in context.facts.iter().enumerate() {
            b.checkpoint()?;
            let path = format!("/facts/{j}");
            if !b.admit(&member, &path) {
                continue;
            }
            if fact.codec == "scientific-coordinates.v1" && b.selected(Q::PhysicalAttribution) {
                match serde_json::from_slice::<crate::workflow::ReviewScientificCoordinates>(&fact.payload) {
                    Ok(coordinates)=>b.add(Q::PhysicalAttribution,Established,"physical.original_coordinates",format!("{} retains original scientific step {}, model {:?}, source {}, physical {}; {} state, {} input, {} output and {} integral coordinates. Native transform/pullback is not inferred from these original coordinates.",fact.path,coordinates.step,coordinates.model_id,coordinates.source_identity,coordinates.physical_identity,coordinates.state.len(),coordinates.inputs.len(),coordinates.outputs.len(),coordinates.integrals.len()),vec![source(&member,format!("{path}/codec"),Value::Text(fact.codec.clone()),false),source(&member,format!("{path}/path"),Value::Text(fact.path.clone()),false)],vec![]),
                    Err(_)=>b.add(Q::PhysicalAttribution,Limitation,"physical.coordinates_decode","The retained original coordinate payload cannot be decoded under its declared codec; no mapping was inferred.",vec![source(&member,format!("{path}/codec"),Value::Text(fact.codec.clone()),true)],vec![NextObservation::InspectRelated]),
                }
            } else if fact.codec == "physical-input-bits.v1" && b.selected(Q::Comparability) {
                b.add(Q::Comparability,Established,"comparison.physical_inputs_retained",format!("Exact original physical-input bits are retained at {}. Their presence is not equality to another run; comparison requires the corresponding complete selected context.",fact.path),vec![source(&member,format!("{path}/codec"),Value::Text(fact.codec.clone()),false)],vec![]);
            }
        }
        for (j, unavailable) in context.unavailable.iter().enumerate() {
            if b.selected(Q::PhysicalAttribution) && b.admit(&member, &format!("/unavailable/{j}"))
            {
                b.add(
                    Q::PhysicalAttribution,
                    Limitation,
                    "physical.context_unavailable",
                    format!("{}: {}", unavailable.path, unavailable.reason),
                    vec![source(
                        &member,
                        format!("/unavailable/{j}/reason"),
                        Value::Text(unavailable.reason.clone()),
                        true,
                    )],
                    vec![NextObservation::InspectRelated],
                );
            }
        }
    }
    Ok(())
}
/// A retained run context can associate earlier physical-session facts without
/// relabeling their original engine operation or claiming a common clock.
pub(super) fn engine_context_selected(
    set: &FrozenEvidenceSet,
    context: &crate::workflow::ReviewContextDocument,
    ids: &[OperationId],
) -> bool {
    context
        .operation
        .is_some_and(|operation| ids.contains(&operation))
        || (context.operation.is_none()
            && set.invocations.iter().any(|invocation| {
                invocation
                    .operation
                    .is_some_and(|operation| ids.contains(&operation))
                    && context.run_id == Some(invocation.run_id)
                    && context.attempt_id == invocation.attempt_id
            }))
}
struct EngineSource {
    member: String,
    payload: Option<String>,
}
impl EngineSource {
    fn admit(&self, b: &mut Builder, path: &str) -> bool {
        let pointer = self
            .payload
            .as_ref()
            .map_or_else(|| path.to_owned(), |payload| format!("{payload}@{path}"));
        b.admit(&self.member, &pointer)
    }
    fn reference(
        &self,
        path: impl AsRef<str>,
        value: Value,
        partial: bool,
    ) -> AnalysisSourceReference {
        source(
            &self.member,
            self.payload.as_deref().unwrap_or(path.as_ref()),
            value,
            partial,
        )
    }
    fn coordinate(&self, path: &str) -> String {
        self.payload.as_ref().map_or_else(String::new, |_| format!("Decoded payload coordinate {path}; the source reference addresses the actual retained byte array."))
    }
}
pub(super) fn same_engine_trace(
    left: &pse_engine::session::CopiedExecutionObservations,
    right: &pse_engine::session::CopiedExecutionObservations,
) -> bool {
    left.operation == right.operation
        && left.policy == right.policy
        && left.dropped == right.dropped
        && left.observations.len() == right.observations.len()
        && left
            .observations
            .iter()
            .zip(&right.observations)
            .all(|(left, right)| {
                left.operation == right.operation
                    && left.policy == right.policy
                    && left.logical_plan == right.logical_plan
                    && left.physical_plan == right.physical_plan
                    && left.physical_graph == right.physical_graph
                    && left.rules == right.rules
                    && left.diagnostic == right.diagnostic
                    && match (&left.execution, &right.execution) {
                        (None, None) => true,
                        (Some(left), Some(right)) => {
                            left.execution == right.execution
                                && left.status == right.status
                                && left.truncated == right.truncated
                                && left.observed_after_start == right.observed_after_start
                                && left.collection_elapsed == right.collection_elapsed
                                && left.metrics.len() == right.metrics.len()
                                && left
                                    .metrics
                                    .iter()
                                    .zip(&right.metrics)
                                    .all(|(left, right)| {
                                        left.node == right.node
                                            && left.operator == right.operator
                                            && left.name == right.name
                                            && left.partition == right.partition
                                            && left.labels == right.labels
                                            && left.unit == right.unit
                                            && left.kind == right.kind
                                            && left.value == right.value
                                    })
                        }
                        _ => false,
                    }
            })
}
fn engine_trace(
    b: &mut Builder,
    trace: &pse_engine::session::CopiedExecutionObservations,
    ids: &[OperationId],
    origin: &EngineSource,
    contextual: bool,
) -> Result<(), ObservationError> {
    use pse_engine::session::{CopiedExecutionStatus as S, CopiedObservationPolicy as P};
    if (contextual
        || trace
            .operation
            .is_some_and(|operation| ids.contains(&operation)))
        && let Some(policy) = trace.policy
        && origin.admit(b, "/policy")
    {
        let reference = origin.reference("/policy", Value::Text(format!("{policy:?}")), false);
        b.add(Q::Preparation, Established, "engine.actual_session_policy", format!("The actual performing session operation {:?} retained policy {policy:?}, {} copied query observations and {} refused observations. An empty Off session remains an explicit original fact. {}", trace.operation, trace.observations.len(), trace.dropped, origin.coordinate("/policy")), vec![reference.clone()], vec![]);
        if policy == P::Off {
            b.add(Q::EvaluationCost, Limitation, "engine.off", "The actual session's capture policy was Off. Missing query/operator observations are unavailable, not zero work.", vec![reference], vec![NextObservation::InspectRelated]);
        }
    }
    for (j, observation) in trace.observations.iter().enumerate() {
        b.checkpoint()?;
        if !contextual && !observation.operation.is_some_and(|id| ids.contains(&id)) {
            continue;
        }
        let pointer = format!("/observations/{j}");
        if !origin.admit(b, &pointer) {
            continue;
        }
        let reference = origin.reference(
            format!("{pointer}/policy"),
            Value::Text(
                match observation.policy {
                    P::Off => "off",
                    P::Contract => "contract",
                    P::Diagnostic => "diagnostic",
                }
                .into(),
            ),
            observation.diagnostic.is_some(),
        );
        b.add(Q::Preparation,Established,"engine.actual_policy",format!("Actual engine policy {:?}, original observation operation {:?}; retained {} fired rules; logical text {} bytes, physical text {:?} bytes. Contract has no plan text; Diagnostic text is bounded. Export did not enable instrumentation. {}",observation.policy,observation.operation,observation.rules.len(),observation.logical_plan.len(),observation.physical_plan.as_ref().map(String::len),origin.coordinate(&format!("{pointer}/policy"))),vec![reference.clone()],vec![]);
        if observation.policy == P::Off {
            b.add(Q::EvaluationCost,Limitation,"engine.off","Engine capture was Off for this original observation; absent operators are unavailable, not zero work.",vec![reference],vec![NextObservation::InspectRelated]);
            continue;
        }
        if let Some(graph) = &observation.physical_graph {
            b.add(Q::Preparation, Established, "engine.physical_topology", format!("Actual structured physical topology: {} unique operators, {} ordered dependency edges, truncated={}. Native metric node ordinals share this capture-local traversal; cache-producer edges are explicit. This is not canonical or numerical dependency authority. {}",graph.nodes.len(),graph.edges.len(),graph.truncated,origin.coordinate(&format!("{pointer}/physical_graph"))),vec![origin.reference(format!("{pointer}/physical_graph"),Value::Text("actual physical topology".into()),graph.truncated)],vec![]);
        }
        let Some(execution) = &observation.execution else {
            b.add(Q::EvaluationCost,Limitation,"engine.metrics_unavailable","No owner-copied operator population was retained. Plan/rule capture alone does not establish executed operator cost.",vec![reference],vec![NextObservation::InspectRelated]);
            continue;
        };
        b.add(Q::Waiting,Established,"engine.lifetime",format!("Engine execution {} on original observation operation {:?}, status {:?}, observed_after_start {:?}, observer collection {:?}, metrics={}, truncated={}. These are the original physical-capture clock durations; context association does not align them with a selected absolute operation window. Running/abandoned/failed populations never establish exhausted invocation coverage or exclusive CPU. {}",execution.execution,observation.operation,execution.status,execution.observed_after_start,execution.collection_elapsed,execution.metrics.len(),execution.truncated,origin.coordinate(&format!("{pointer}/execution/execution"))),vec![origin.reference(format!("{pointer}/execution/execution"),Value::Count(execution.execution),execution.truncated || execution.status!=S::Completed)],vec![]);
        for (k, metric) in execution.metrics.iter().enumerate() {
            if !b.selected(Q::EvaluationCost) {
                continue;
            }
            let path = format!("{pointer}/execution/metrics/{k}");
            if !origin.admit(b, &path) {
                continue;
            }
            b.add(Q::EvaluationCost,Established,"engine.operator_metric",format!("Execution {} on original operation {:?}, node {}, operator {}, partition {:?}, labels {:?}: {}={:?} {:?}, {:?}. Native gauges are state, counters are work; no cross-node inclusive time or gauge summation is performed. {}",execution.execution,observation.operation,metric.node,metric.operator,metric.partition,metric.labels,metric.name,metric.value,metric.unit,metric.kind,origin.coordinate(&format!("{path}/value"))),vec![origin.reference(format!("{path}/value"),metric.value.and_then(|value|u64::try_from(value).ok()).map_or_else(||Value::Unavailable("native compound or unrepresentable metric".into()),Value::Count),execution.truncated || execution.status!=S::Completed)],vec![]);
        }
    }
    if trace.dropped > 0
        && (contextual
            || trace
                .operation
                .is_some_and(|operation| ids.contains(&operation))
            || trace
                .observations
                .iter()
                .any(|observation| observation.operation.is_some_and(|id| ids.contains(&id))))
        && origin.admit(b, "/dropped")
    {
        b.add(Q::EvaluationCost,Limitation,"engine.collector_dropped",format!("Actual engine collector refused {} observations; the retained population cannot establish all engine work.",trace.dropped),vec![origin.reference("/dropped",Value::Count(trace.dropped),true)],vec![NextObservation::InspectRelated]);
    }
    Ok(())
}
pub(super) fn engine(
    b: &mut Builder,
    set: &FrozenEvidenceSet,
    ids: &[OperationId],
) -> Result<(), ObservationError> {
    if !b.selected(Q::Preparation) && !b.selected(Q::EvaluationCost) && !b.selected(Q::Waiting) {
        return Ok(());
    }
    // Decode one admitted payload at a time. Exact contextual copies take source
    // precedence over their flattened mirrors; no session name or clock inference joins them.
    let mut mirrored = BTreeSet::new();
    for (context_index, context) in set
        .contexts
        .iter()
        .enumerate()
        .filter(|(_, context)| engine_context_selected(set, context, ids))
    {
        for (fact_index, fact) in context
            .facts
            .iter()
            .enumerate()
            .filter(|(_, fact)| fact.codec == "engine-execution-observations.v2")
        {
            b.checkpoint()?;
            // A repeated same-byte attachment is the same supplied receipt, even if
            // original physical source positions repeat in this exact run context.
            if set.contexts[..context_index]
                .iter()
                .filter(|context| engine_context_selected(set, context, ids))
                .flat_map(|context| &context.facts)
                .chain(context.facts[..fact_index].iter())
                .any(|previous| previous.codec == fact.codec && previous.payload == fact.payload)
            {
                continue;
            }
            let member = format!("contexts/{context_index}");
            let payload = format!("/facts/{fact_index}/payload");
            if !b.admit(&member, &payload) {
                continue;
            }
            let trace: pse_engine::session::CopiedExecutionObservations =
                serde_json::from_slice(&fact.payload)
                    .map_err(|error| invalid(error.to_string()))?;
            for (index, flat) in set.engine_observations.iter().enumerate() {
                b.checkpoint()?;
                if same_engine_trace(&trace, flat) {
                    mirrored.insert(index);
                }
            }
            b.add(Q::Preparation, Established, "engine.context_association", format!("Exact selected run context {:?}, operation {:?}, retains the actual physical-session receipt from original session operation {:?}. The original operation is preserved; this association supplies no absolute clock alignment or selected-window timing claim.", context.run_id, context.operation, trace.operation), vec![source(&member, &payload, Value::Text("exact original physical-session receipt".into()), false)], vec![]);
            engine_trace(
                b,
                &trace,
                ids,
                &EngineSource {
                    member,
                    payload: Some(payload),
                },
                true,
            )?;
        }
    }
    for (index, trace) in set.engine_observations.iter().enumerate() {
        b.checkpoint()?;
        if !mirrored.contains(&index) {
            engine_trace(
                b,
                trace,
                ids,
                &EngineSource {
                    member: format!("engine_observations/{index}"),
                    payload: None,
                },
                false,
            )?;
        }
    }
    Ok(())
}
/// Multipart native details are one inner observation, not one factor per fragment.
fn factor_groups(b: &mut Builder, set: &FrozenEvidenceSet, events: &[Event<'_>]) {
    let mut groups: BTreeMap<(EventKey, String, u64), Vec<&Event<'_>>> = BTreeMap::new();
    for event in events.iter().filter(|event| {
        !event.predecessor
            && matches!(
                event.record.family.as_str(),
                "native_factor.v2" | "native_refinement.v2"
            )
            && eligible(Q::FactorRecovery, declared_role(set, event.record))
    }) {
        if !b
            .inspected
            .contains(&format!("{}#{}", event.member, event.pointer))
        {
            continue;
        }
        let Some(group) = field_count(event, "detail_group_ordinal") else {
            b.add(Q::FactorRecovery,Limitation,"factor.group_identity_unavailable","Native multipart record has no exact group ordinal; no inner-observation population is reconstructed.",vec![event.reference(None)],vec![NextObservation::InspectRelated]);
            continue;
        };
        let r = event.record;
        groups
            .entry((
                (
                    r.producer.operation.process_instance,
                    r.producer.operation.sequence,
                    r.producer.sequence,
                    r.generation,
                    0,
                ),
                r.family.clone(),
                group,
            ))
            .or_default()
            .push(event);
    }
    for ((_, family, group), parts) in groups {
        let first = parts[0];
        let expected = field_count(first, "detail_parts");
        let call = field_count(first, "call_ordinal");
        let mut metadata_admitted = true;
        for part in &parts {
            for name in [
                "backend",
                "call_ordinal",
                "parent_call_ordinal",
                "parent_call_present",
                "call_path",
                "call_primitive",
                "detail_group_ordinal",
                "detail_part",
                "detail_parts",
            ] {
                if let Some(index) = part
                    .record
                    .values
                    .iter()
                    .position(|field| field.name == name)
                {
                    metadata_admitted &=
                        b.admit(&part.member, &format!("{}/values/{index}", part.pointer));
                }
            }
        }
        if !metadata_admitted {
            b.add(Q::FactorRecovery,Limitation,"factor.group_incomplete","Selected source allowance curtailed multipart group identity metadata. No complete inner observation or recovery subtotal was reconstructed.",vec![first.reference(None)],vec![NextObservation::InspectRelated]);
            continue;
        }
        let identity_names = [
            "backend",
            "call_ordinal",
            "parent_call_ordinal",
            "parent_call_present",
            "call_path",
            "call_primitive",
            "detail_parts",
        ];
        let same_identity = parts.iter().all(|part| {
            identity_names.iter().all(|name| {
                let left = first.record.values.iter().find(|field| field.name == *name);
                let right = part.record.values.iter().find(|field| field.name == *name);
                match (left, right) {
                    (Some(left), Some(right)) => {
                        left.value == right.value && left.availability == right.availability
                    }
                    (None, None) => true,
                    _ => false,
                }
            })
        });
        let unique_fields = parts.iter().all(|part| {
            let names = part
                .record
                .values
                .iter()
                .map(|field| field.name.as_str())
                .collect::<BTreeSet<_>>();
            names.len() == part.record.values.len()
        });
        let ordinals = parts
            .iter()
            .filter_map(|part| field_count(part, "detail_part"))
            .collect::<BTreeSet<_>>();
        let parent_present = first
            .record
            .values
            .iter()
            .find(|field| field.name == "parent_call_present" && field.availability.is_none())
            .and_then(|field| match field.value {
                Value::Boolean(value) => Some(value),
                _ => None,
            });
        let known_call = call.is_some()
            && field_text(first, "backend") == Some("pounce")
            && matches!(
                field_text(first, "call_path"),
                Some("monolithic" | "schur_f" | "schur_s")
            )
            && matches!(
                field_text(first, "call_primitive"),
                Some("factor" | "backsolve")
            )
            && parent_present.is_some()
            && (parent_present == Some(false)
                || field_count(first, "parent_call_ordinal").is_some());
        let complete = expected.is_some_and(|count| {
            count > 0
                && count <= 4096
                && ordinals.len() == parts.len()
                && count as usize == parts.len()
                && ordinals.iter().copied().eq(0..count)
        }) && known_call
            && same_identity
            && metadata_admitted
            && unique_fields;
        let references = parts
            .iter()
            .take(16)
            .map(|part| part.reference(None))
            .collect();
        b.add(Q::FactorRecovery,if complete {Established} else {Limitation},if complete {"factor.group_complete"} else {"factor.group_incomplete"},format!("{family} inner detail group {group}, call {:?}, path {:?}: supplied {} parts, declared {:?}, distinct part ordinals {:?}, consistent identity={same_identity}. {}",call,field_text(first,"call_path"),parts.len(),expected,ordinals,if complete {"The selected multipart inner observation is structurally complete; this does not establish all calls or exclusive factor cost."} else {"Missing, duplicate, out-of-range or inconsistent parts/fields prevent a complete inner-observation claim; original fragments remain readable."}),references,vec![NextObservation::InspectRelated]);
        if !complete {
            continue;
        }
        for name in [
            "mc64_retry_attempts",
            "mc64_to_infnorm_fallbacks",
            "static_pivot_perturbations",
            "correction_batches_issued",
            "correction_batches_completed",
            "rhs_issued",
            "rhs_completed",
            "residual_products",
        ] {
            let fields = parts
                .iter()
                .flat_map(|part| {
                    part.record
                        .values
                        .iter()
                        .enumerate()
                        .filter(move |(_, field)| field.name == name)
                        .map(move |(index, _)| (*part, index))
                })
                .collect::<Vec<_>>();
            if let [(part, index)] = fields.as_slice() {
                if !b.admit(&part.member, &format!("{}/values/{index}", part.pointer)) {
                    continue;
                }
                let reference = part.reference(Some(*index));
                b.add(Q::FactorRecovery,Established,"factor.group_work",format!("Exact {family} detail group {group}, parent-scoped call {:?}: {name}={:?} [{}]. This is inner source work, charged once independently of how many substituted right-hand sides use the factor.",call,part.record.values[*index].value,part.record.values[*index].unit),vec![reference],vec![]);
            } else if !fields.is_empty() {
                b.add(Q::FactorRecovery,Limitation,"factor.group_field_ambiguous",format!("{family} group {group} repeats field {name}; no recovery total is selected."),vec![first.reference(None)],vec![NextObservation::InspectRelated]);
            }
        }
    }
}
fn evaluation_cost(b: &mut Builder, set: &FrozenEvidenceSet, events: &[Event<'_>]) {
    let mut retained = Vec::new();
    for event in events {
        if event.predecessor
            || event.record.family != "evaluator_demand_batch.v1"
            || !eligible(Q::EvaluatorWork, declared_role(set, event.record))
        {
            continue;
        }
        if !b.admit(&event.member, &event.pointer) {
            continue;
        }
        let names = [
            "actual_evaluations_completed",
            "actual_evaluations_failed",
            "actual_evaluations_completed_nanoseconds",
            "actual_evaluations_failed_nanoseconds",
            "actual_evaluation_timing_scope",
            "demand",
            "requested_order",
        ];
        let mut evidence = Vec::new();
        let mut admitted = true;
        for name in names {
            if let Some(index) = event
                .record
                .values
                .iter()
                .position(|field| field.name == name)
            {
                if b.admit(&event.member, &format!("{}/values/{index}", event.pointer)) {
                    evidence.push(event.reference(Some(index)));
                } else {
                    admitted = false;
                }
            }
        }
        if !admitted {
            continue;
        }
        let completed = field_count(event, "actual_evaluations_completed");
        let nanoseconds = field_count(event, "actual_evaluations_completed_nanoseconds");
        let failed = field_count(event, "actual_evaluations_failed");
        let scope = field_text(event, "actual_evaluation_timing_scope");
        match (completed,nanoseconds,failed,scope) {
            (Some(completed),Some(nanoseconds),Some(failed),Some(scope)) if completed>0 => {
                retained.push((event,completed,nanoseconds));
                b.add(Q::EvaluationCost,Established,"evaluation.actual_inclusive_mean",format!("Demand {:?}, requested order {:?}, performing producer {:?}, generation {}: {} completed actual local evaluator invocations consumed {} inclusive nanoseconds ({:.6} seconds per completed actual invocation); {} failed actual invocations are excluded from both numerator and denominator. Scope: {}. Cache/projection request completions are excluded. This is a retained-batch mean, not exclusive CPU or a normalized cross-context winner.",field_text(event,"demand"),field_count(event,"requested_order"),event.record.producer,event.record.generation,completed,nanoseconds,nanoseconds as f64/1e9/completed as f64,failed,scope),evidence,vec![NextObservation::CpuProfile]);
            },
            (Some(0),Some(0),Some(_),Some(_))=>b.add(Q::EvaluationCost,Established,"evaluation.no_completed_actual_calls","This owner-timed batch contains zero completed actual evaluator invocations. Cache hits and failed invocations do not create a completed actual-call mean.",evidence,vec![]),
            _=>b.add(Q::EvaluationCost,Limitation,"evaluation.actual_timing_unavailable","This source batch lacks a valid matched selected completed-actual-evaluator count/duration population or timing scope. Global demand completion, request phase time and solver iteration time are not substituted for actual evaluation cost.",if evidence.is_empty(){vec![event.reference(None)]}else{evidence},vec![NextObservation::InspectRelated]),
        }
    }
    for pair in retained.windows(2) {
        let (left, lc, ln) = pair[0];
        let (right, rc, rn) = pair[1];
        if left.record.producer != right.record.producer
            || left.record.generation != right.record.generation
            || field_text(left, "demand") != field_text(right, "demand")
            || field_count(left, "requested_order") != field_count(right, "requested_order")
        {
            continue;
        }
        b.add(Q::EvaluationCost,Established,"evaluation.retained_population_contrast",format!("Two retained same-owner/generation/demand/order batches report completed actual calls {lc} then {rc}, inclusive nanoseconds {ln} then {rn}, and descriptive seconds per completed actual invocation {:.6} then {:.6}. These are literal batch populations; original product/input/start/reuse compatibility is not established by equal demand/order, so no normalized winner or intrinsic evaluator regression is inferred.",ln as f64/1e9/lc as f64,rn as f64/1e9/rc as f64),vec![left.reference(None),right.reference(None)],vec![NextObservation::InspectRelated,NextObservation::CpuProfile]);
    }
}
