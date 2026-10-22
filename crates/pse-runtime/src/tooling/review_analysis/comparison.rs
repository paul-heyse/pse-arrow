// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive before/after comparisons over actual producer identities, never local ordinals.
use super::*;
use StatementKind::{Established, Limitation};
fn text<'a>(event: &'a Event<'_>, name: &str) -> Option<&'a str> {
    let mut fields = event
        .record
        .values
        .iter()
        .filter(|field| field.name == name && field.availability.is_none());
    match (&fields.next()?.value, fields.next()) {
        (Value::Text(value), None) => Some(value),
        _ => None,
    }
}
fn count(event: &Event<'_>, name: &str) -> Option<u64> {
    let mut fields = event
        .record
        .values
        .iter()
        .filter(|field| field.name == name && field.availability.is_none());
    let field = fields.next()?;
    if fields.next().is_some() {
        return None;
    }
    match field.value {
        Value::Count(value) => Some(value),
        Value::Integer(value) => value.try_into().ok(),
        _ => None,
    }
}
fn admit_record(b: &mut Builder, event: &Event<'_>) -> bool {
    b.admit(&event.member, &event.pointer)
        && event
            .record
            .values
            .iter()
            .enumerate()
            .all(|(index, _)| b.admit(&event.member, &format!("{}/values/{index}", event.pointer)))
}
struct EngineCapture<'a> {
    observation: std::borrow::Cow<'a, pse_engine::session::CopiedPlanObservation>,
    reference: AnalysisSourceReference,
}
struct EngineCaptures<'a> {
    captures: Vec<EngineCapture<'a>>,
    incomplete: bool,
    loss: Vec<AnalysisSourceReference>,
}
fn engine_captures<'a>(
    b: &mut Builder,
    set: &'a FrozenEvidenceSet,
    operation: OperationId,
) -> Result<EngineCaptures<'a>, ObservationError> {
    let mut captures = Vec::new();
    let mut incomplete = false;
    let mut loss = Vec::new();
    let mut mirrored = BTreeSet::new();
    let mut payloads = BTreeSet::new();
    for (context_index, context) in set
        .contexts
        .iter()
        .enumerate()
        .filter(|(_, context)| questions::engine_context_selected(set, context, &[operation]))
    {
        for (fact_index, fact) in context
            .facts
            .iter()
            .enumerate()
            .filter(|(_, fact)| fact.codec == "engine-execution-observations.v2")
        {
            b.checkpoint()?;
            if !payloads.insert(fact.payload.as_slice()) {
                continue;
            }
            let member = format!("contexts/{context_index}");
            let pointer = format!("/facts/{fact_index}/payload");
            if !b.admit(&member, &pointer) {
                incomplete = true;
                continue;
            }
            let trace: pse_engine::session::CopiedExecutionObservations =
                serde_json::from_slice(&fact.payload)
                    .map_err(|error| invalid(error.to_string()))?;
            for (index, flat) in set.engine_observations.iter().enumerate() {
                if questions::same_engine_trace(&trace, flat) {
                    mirrored.insert(index);
                }
            }
            if trace.dropped > 0 {
                incomplete = true;
                loss.push(source(&member, &pointer, Value::Count(trace.dropped), true));
            }
            for observation in trace.observations {
                if captures.len() == 2 {
                    break;
                }
                captures.push(EngineCapture { observation: std::borrow::Cow::Owned(observation), reference: source(&member,&pointer,Value::Text("exact contextual physical-session receipt; decoded topology coordinates".into()),false) });
            }
        }
    }
    for (trace_index, trace) in set.engine_observations.iter().enumerate() {
        b.checkpoint()?;
        if mirrored.contains(&trace_index) {
            continue;
        }
        let member = format!("engine_observations/{trace_index}");
        if trace.dropped > 0
            && (trace.operation == Some(operation)
                || trace
                    .observations
                    .iter()
                    .any(|observation| observation.operation == Some(operation)))
        {
            incomplete = true;
            if b.admit(&member, "/dropped") {
                loss.push(source(
                    &member,
                    "/dropped",
                    Value::Count(trace.dropped),
                    true,
                ));
            }
        }
        for (observation_index, observation) in trace
            .observations
            .iter()
            .enumerate()
            .filter(|(_, observation)| observation.operation == Some(operation))
        {
            if captures.len() == 2 {
                break;
            }
            let pointer = format!("/observations/{observation_index}");
            if !b.admit(&member, &pointer) {
                incomplete = true;
                continue;
            }
            captures.push(EngineCapture {
                observation: std::borrow::Cow::Borrowed(observation),
                reference: source(
                    &member,
                    format!("{pointer}/physical_graph"),
                    Value::Text("actual physical topology".into()),
                    observation
                        .physical_graph
                        .as_ref()
                        .is_some_and(|graph| graph.truncated),
                ),
            });
        }
    }
    Ok(EngineCaptures {
        captures,
        incomplete,
        loss,
    })
}
pub(super) fn analyze(
    b: &mut Builder,
    set: &FrozenEvidenceSet,
    events: &[Event<'_>],
) -> Result<(), ObservationError> {
    let Some(baseline) = b.document.request.baseline else {
        return Ok(());
    };
    let current = b.document.request.operation;
    if b.selected(AnalysisQuestion::Preparation) {
        let mut groups: BTreeMap<_, [Vec<&Event<'_>>; 2]> = BTreeMap::new();
        for event in events.iter().filter(|event| {
            !event.predecessor
                && event.record.family == "compiler_query.v1"
                && questions::eligible(
                    AnalysisQuestion::Preparation,
                    questions::declared_role(set, event.record),
                )
        }) {
            b.checkpoint()?;
            if !admit_record(b, event) {
                continue;
            }
            let operation = event.record.producer.operation;
            let side = if operation == baseline {
                0
            } else if operation == current {
                1
            } else {
                continue;
            };
            let (Some(family), Some(subject), Some(_generation)) = (
                text(event, "query_family"),
                text(event, "semantic_subject"),
                count(event, "workspace_generation"),
            ) else {
                continue;
            };
            if b.admit(&event.member, &event.pointer) {
                groups
                    .entry((family.to_owned(), subject.to_owned()))
                    .or_default()[side]
                    .push(event);
            }
        }
        for ((family, subject), populations) in groups {
            let counts = |events: &[&Event<'_>]| {
                ["execute", "validate", "wait", "discard"].map(|phase| {
                    events
                        .iter()
                        .filter(|event| event.record.phase == phase)
                        .count()
                })
            };
            let before = counts(&populations[0]);
            let after = counts(&populations[1]);
            let matched = !populations[0].is_empty() && !populations[1].is_empty();
            b.add(AnalysisQuestion::Preparation, if matched { Established } else { Limitation }, "preparation.semantic_events",format!("Semantic query {family}, subject {subject}: baseline execute/validate/wait/discard {before:?}; selected {after:?}; correspondence={matched}. Generation-scoped owner mappings supply attribution across captures. Missing events are absent observations, not proof of reuse or zero root-wide work. Execution counts establish no cache-miss cause or wait duration."),populations.iter().flat_map(|events| events.iter().take(4).map(|event| event.reference(None))).collect(),vec![]);
        }
    }
    if b.selected(AnalysisQuestion::EvaluatorWork) {
        let mut groups: BTreeMap<_, [Vec<&Event<'_>>; 2]> = BTreeMap::new();
        for event in events.iter().filter(|event| {
            !event.predecessor
                && event.record.family == "evaluator_census.v1"
                && questions::eligible(
                    AnalysisQuestion::EvaluatorWork,
                    questions::declared_role(set, event.record),
                )
        }) {
            if !admit_record(b, event) {
                continue;
            }
            let operation = event.record.producer.operation;
            let side = if operation == baseline {
                0
            } else if operation == current {
                1
            } else {
                continue;
            };
            let (Some(body), Some(product)) = (text(event, "body"), text(event, "product")) else {
                continue;
            };
            if b.admit(&event.member, &event.pointer) {
                groups
                    .entry((body.to_owned(), product.to_owned()))
                    .or_default()[side]
                    .push(event);
            }
        }
        for ((body, product), populations) in groups {
            if populations.iter().any(|events| events.len() != 1) {
                b.add(AnalysisQuestion::EvaluatorWork,Limitation,"evaluator.correspondence_ambiguous",format!("Body {body}, product {product}: baseline {} and selected {} census records; exact demand correspondence is ambiguous or missing. Template ordinals and representation counts are not semantic keys.",populations[0].len(),populations[1].len()),populations.iter().flat_map(|events| events.iter().take(2).map(|event| event.reference(None))).collect(),vec![]);
                continue;
            }
            let before = populations[0][0];
            let after = populations[1][0];
            let names = [
                "order",
                "derivative_components",
                "second_pairs",
                "outputs",
                "coordinates",
                "stages",
                "arithmetic_blocks",
                "arithmetic_input_entries",
                "arithmetic_output_entries",
                "provider_stages",
                "control_stages",
                "library_operations",
                "library_instructions",
                "instruction_storage",
                "frame_entries",
                "result_entries",
                "worker_storage",
            ];
            let changes = names
                .iter()
                .map(|name| {
                    format!(
                        "{name}: {:?} -> {:?}",
                        count(before, name),
                        count(after, name)
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            b.add(AnalysisQuestion::EvaluatorWork,Established,"evaluator.body_product_change",format!("Same recorded semantic body {body} and product {product}: {changes}. Actual formal output selections {:?} -> {:?}, coordinate selections {:?} -> {:?}. These are static producer census facts; absent index selections leave coordinate correspondence unavailable. Multiple demands remain ambiguous; instruction counts establish neither branch frequency nor time or a winner.",text(before,"demand_outputs"),text(after,"demand_outputs"),text(before,"demand_coordinates"),text(after,"demand_coordinates")),vec![before.reference(None),after.reference(None)],vec![]);
        }
    }
    if b.selected(AnalysisQuestion::Preparation) {
        let before_population = engine_captures(b, set, baseline)?;
        let after_population = engine_captures(b, set, current)?;
        let before = &before_population.captures;
        let after = &after_population.captures;
        if before.len() == 1
            && after.len() == 1
            && !before_population.incomplete
            && !after_population.incomplete
        {
            let left_observation = &before[0].observation;
            let right_observation = &after[0].observation;
            if let (Some(left), Some(right)) = (
                &left_observation.physical_graph,
                &right_observation.physical_graph,
            ) {
                let comparison = left.compare(right);
                let references = vec![before[0].reference.clone(), after[0].reference.clone()];
                b.add(AnalysisQuestion::Preparation,if comparison.incomplete { Limitation } else { Established },"engine.operator_correspondence",format!("Unique operator/schema/adjacency correspondences {:?}; ambiguous baseline {:?}, selected {:?}; unmatched baseline {:?}, selected {:?}; incomplete={}. Local ordinals are matched only through unique operator context. This establishes no expression/provider equivalence, execution completion, or performance winner.",comparison.matched,comparison.ambiguous_before,comparison.ambiguous_after,comparison.unmatched_before,comparison.unmatched_after,comparison.incomplete),references.clone(),vec![]);
                let operators = |observation: &pse_engine::session::CopiedPlanObservation| -> Option<BTreeMap<String,usize>> {
                    let graph: serde_json::Value = serde_json::from_str(&observation.logical_plan).ok()?;
                    let mut operators = BTreeMap::new();
                    for node in graph.get("Nodes")?.as_array()? {
                        let operator = node.get("Native")?.get(0)?.get("Plan")?.get("Node Type")?.as_str()?;
                        *operators.entry(operator.to_owned()).or_default() += 1;
                    }
                    Some(operators)
                };
                if let (Some(left), Some(right)) =
                    (operators(left_observation), operators(right_observation))
                {
                    b.add(AnalysisQuestion::Preparation,Established,"engine.logical_operator_populations",format!("Actual producer logical graphs retain baseline operator populations {left:?}, selected {right:?}. Unique graph nodes count once; counts and native text do not establish stable source correspondence or execution work."),references,vec![]);
                }
            } else {
                b.add(AnalysisQuestion::Preparation,Limitation,"engine.topology_unavailable","One selected original capture has no structured physical topology (Off, Contract, refused capture or not physically planned). No topology or operator work is invented.",vec![before[0].reference.clone(),after[0].reference.clone()],vec![]);
            }
        } else if !before.is_empty()
            || !after.is_empty()
            || before_population.incomplete
            || after_population.incomplete
        {
            let references = before
                .iter()
                .chain(after)
                .map(|capture| capture.reference.clone())
                .chain(before_population.loss)
                .chain(after_population.loss)
                .collect();
            b.add(AnalysisQuestion::Preparation,Limitation,"engine.capture_correspondence_ambiguous",format!("Baseline at least {} and selected at least {} physical observations; incomplete baseline={}, selected={}: dropped or uninspected captures and ambiguous populations cannot establish unique query-level correspondence. No capture is paired by its ordinal.",before.len(),after.len(),before_population.incomplete,after_population.incomplete),references,vec![]);
        }
    }
    Ok(())
}
