// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original findings and explicit native correspondences, using validated retained maps.
use super::*;
use AnalysisQuestion::PhysicalAttribution as Q;
use StatementKind::{Established, Limitation};

fn mapped(
    b: &mut Builder,
    context: &crate::workflow::ReviewContextDocument,
    member: &str,
    scope: &str,
    members: &[pse_ids::SemanticId],
    origin: &AnalysisSourceReference,
) -> Result<(), ObservationError> {
    b.checkpoint()?;
    let path = format!("{scope}/source_map");
    let Some((fact, _)) = context
        .facts
        .iter()
        .enumerate()
        .find(|(_, fact)| fact.codec == "modeling-source-map.v1" && fact.path == path)
    else {
        b.add(Q, Limitation, "physical.mapping_unavailable", format!("No retained source map establishes original member attribution in exact scope {scope}. Eliminated coordinates and transformed native dual/stationarity mappings remain unknown."), vec![origin.clone()], vec![NextObservation::InspectRelated]);
        return Ok(());
    };
    let payload = format!("/facts/{fact}/payload");
    if !b.admit(member, &payload) {
        return Ok(());
    }
    let projections = context
        .source_map_attribution(scope, members)
        .map_err(|error| invalid(error.to_string()))?;
    if projections.is_empty() {
        b.add(Q, Limitation, "physical.mapping_unavailable", format!("The exact supplied map {scope} has no supported original correspondence for the selected members. No elimination or native pullback is inferred."), vec![origin.clone(), source(member, &payload, Value::Text("original map bytes".into()), true)], vec![NextObservation::InspectRelated]);
    }
    for projection in projections {
        b.checkpoint()?;
        let reference = source(
            member,
            format!("/facts/{}/payload", projection.fact),
            Value::Text(format!(
                "original {} {}",
                projection.role, projection.member
            )),
            false,
        );
        b.add(Q, Established, "physical.original_member", format!("Original {} {} in exact scope {scope}, nominal {:?}, declaration {:?}, instance {:?}, display path {:?}; decoded map locations {:?}, authored field spans {:?}. The real source reference is the retained payload byte array; decoded locations are descriptive coordinates inside that payload. Static eligibility does not establish executed branch work. {}", projection.role, projection.member, projection.nominal, projection.declaration, projection.instance, projection.display_path, projection.payload_paths, projection.fields, projection.native_pullback_unavailable), vec![origin.clone(), reference.clone()], vec![]);
        for occurrence in projection.occurrences.iter().take(16) {
            b.checkpoint()?;
            let terms = occurrence
                .terms
                .iter()
                .take(16)
                .map(|term| format!("{} -> {} ({})", term.local, term.member, term.payload_path))
                .collect::<Vec<_>>()
                .join(", ");
            b.add(Q, Established, "physical.original_occurrence", format!("Member {} follows original occurrence {} at ordinal {}, formal {:?}, output {:?}, affine/contribution scale {}, offset {:?}, decoded location {}. Original check-token mappings: [{}]. {} additional terms are retained in the original payload. Repeated aliases/addends preserve their distinct ordinals; this supplies original structure, without an executed native dual/stationarity pullback.", projection.member, occurrence.instance, occurrence.ordinal, occurrence.formal, occurrence.output, occurrence.scale, occurrence.offset, occurrence.payload_path, terms, occurrence.terms.len().saturating_sub(16)), vec![origin.clone(), reference.clone()], vec![]);
        }
        for term in projection.source_terms.iter().take(16) {
            b.checkpoint()?;
            b.add(Q, Established, "physical.original_source_term", format!("Original internal source member {} follows occurrence {} at ordinal {}: local check token {}, decoded map location {}. This supplies source attribution without inventing a row, free coordinate, affine coefficient or executed branch.", projection.member,term.instance,term.ordinal,term.local,term.payload_path),vec![origin.clone(),reference.clone()],vec![]);
        }
        if projection.occurrences.len() > 16 {
            b.add(Q, Limitation, "physical.occurrence_prefix", format!("{} further original occurrences remain in the supplied map beyond the finite interpreted prefix.", projection.occurrences.len() - 16), vec![reference], vec![NextObservation::InspectRelated]);
        }
    }
    Ok(())
}

pub(super) fn analyze(
    b: &mut Builder,
    set: &FrozenEvidenceSet,
    ids: &[OperationId],
    events: &[Event<'_>],
) -> Result<(), ObservationError> {
    if !b.selected(Q) {
        return Ok(());
    }
    for (invocation_index, invocation) in set
        .invocations
        .iter()
        .enumerate()
        .filter(|(_, invocation)| invocation.operation.is_some_and(|id| ids.contains(&id)))
    {
        b.checkpoint()?;
        let invocation_member = format!("invocations/{invocation_index}");
        for (step_index, step) in invocation.attempts.steps.iter().enumerate() {
            for (finding_index, finding) in step.findings.iter().enumerate() {
                b.checkpoint()?;
                let pointer =
                    format!("/attempts/steps/{step_index}/findings/{finding_index}/source");
                if !b.admit(&invocation_member, &pointer) {
                    continue;
                }
                let origin = source(
                    &invocation_member,
                    &pointer,
                    Value::Text(finding.source.to_string()),
                    false,
                );
                b.add(Q, Established, "physical.original_finding", format!("Original step {} finding on member {}: physical violation {:?}, tolerance {:?}. These are original producer-owned quantities, independent of native residual/step norms.", step.step, finding.source, finding.violation, finding.tolerance), vec![origin.clone()], vec![]);
                let contexts = set.contexts.iter().enumerate().filter(|(_, context)| {
                    context.run_id == Some(invocation.run_id)
                        && context.operation == invocation.operation
                });
                let mut supplied = false;
                for (index, context) in contexts {
                    supplied = true;
                    mapped(
                        b,
                        context,
                        &format!("contexts/{index}"),
                        &format!("/steps/{}", step.step),
                        &[finding.source],
                        &origin,
                    )?;
                }
                if !supplied {
                    b.add(Q, Limitation, "physical.mapping_unavailable", "No exact supplied run/operation context maps this original finding to source terms.", vec![origin], vec![NextObservation::InspectRelated]);
                }
            }
        }
    }
    for (context_index, context) in set
        .contexts
        .iter()
        .enumerate()
        .filter(|(_, context)| context.operation.is_some_and(|id| ids.contains(&id)))
    {
        b.checkpoint()?;
        let member = format!("contexts/{context_index}");
        for (fact_index, fact) in context.facts.iter().enumerate() {
            b.checkpoint()?;
            let pointer = format!("/facts/{fact_index}/payload");
            if fact.codec == "native-attempt-record.v1" {
                if !b.admit(&member, &pointer) {
                    continue;
                }
                let native: pse_backend_native::solve::NativeAttemptRecordDocument =
                    serde_json::from_slice(&fact.payload)
                        .map_err(|error| invalid(error.to_string()))?;
                for (role, coordinates) in [("variable", &native.variables), ("row", &native.rows)]
                {
                    for (index, coordinate) in coordinates.iter().enumerate() {
                        b.checkpoint()?;
                        // Count decoded scalar inspections while retaining a real raw-payload pointer.
                        if !b.admit(&member, &format!("{pointer}@/{role}/{index}")) {
                            continue;
                        }
                        let origin = source(
                            &member,
                            &pointer,
                            Value::Text(format!(
                                "native original {role} {coordinate} at ordinal {index}"
                            )),
                            false,
                        );
                        mapped(b, context, &member, &fact.path, &[*coordinate], &origin)?;
                    }
                }
            } else if fact.codec == "scientific-coordinates.v1" {
                if !b.admit(&member, &pointer) {
                    continue;
                }
                let coordinates: crate::workflow::ReviewScientificCoordinates =
                    serde_json::from_slice(&fact.payload)
                        .map_err(|error| invalid(error.to_string()))?;
                for event in events.iter().filter(|event| {
                    !event.predecessor
                        && event.record.family == "dynamic_local_error.v1"
                        && Some(event.record.producer.operation) == context.operation
                        && questions::eligible(
                            AnalysisQuestion::DynamicWork,
                            questions::declared_role(set, event.record),
                        )
                }) {
                    b.checkpoint()?;
                    let Some((ordinal, field)) =
                        event.record.values.iter().enumerate().find(|(_, field)| {
                            field.name == "state_identity" && field.availability.is_none()
                        })
                    else {
                        continue;
                    };
                    let Value::Text(identity) = &field.value else {
                        continue;
                    };
                    let Some(index) = event
                        .record
                        .values
                        .iter()
                        .find(|field| field.name == "state_index" && field.availability.is_none())
                        .and_then(|field| match field.value {
                            Value::Count(index) => usize::try_from(index).ok(),
                            Value::Integer(index) => usize::try_from(index).ok(),
                            _ => None,
                        })
                    else {
                        continue;
                    };
                    let Some(coordinate) = coordinates
                        .state
                        .get(index)
                        .filter(|coordinate| coordinate.id.to_string() == *identity)
                    else {
                        continue;
                    };
                    if !b.admit(
                        &event.member,
                        &format!("{}/values/{ordinal}", event.pointer),
                    ) {
                        continue;
                    }
                    b.add(Q, Established, "physical.dynamic_state", format!("Actual local-error contributor state_index {index} and exact state identity {identity} match original scientific step {} coordinate quantity {}, unit {}, scale {}, offset {} in decoded /state/{index}. This follows the retained original coordinate and source event; local-error contribution is not global trajectory error or a universal native pullback.", coordinates.step, coordinate.quantity_id, coordinate.unit_id, coordinate.scale, coordinate.offset), vec![event.reference(Some(ordinal)), source(&member, &pointer, Value::Text(format!("original state {identity}")), false)], vec![]);
                }
            }
        }
    }
    Ok(())
}
