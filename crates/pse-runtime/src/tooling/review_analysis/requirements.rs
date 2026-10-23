// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared question prerequisites over exact supplied subjects; no live reconstruction.
use super::{AnalysisQuestion, AnalysisRequest, AnalysisSourceReference};
use crate::observation::ObservationError;
use crate::tooling::frozen_inspection::FrozenEvidenceSet;
use pse_diagnostics::observation::{OperationId, Value};
use serde::{Deserialize, Serialize};

/// One scientific or diagnostic premise required by a selected question.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequiredFact {
    /// Original authored model and its source interpretation.
    Model,
    /// Actual submitted physical inputs, including constructor inputs.
    Input,
    /// Materialized solver/runtime settings rather than a current default.
    EffectiveSettings,
    /// Original retained coordinate witness.
    PhysicalCoordinates,
    /// Exact original invocation facts for comparison.
    Invocation,
    /// An actual supplied observation for the selected question.
    OperationEvidence,
}
/// Missing premises remain separate from statement confidence and publication status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequirementStatus {
    /// At least one original supplied source supports this premise.
    Available,
    /// The premise is absent or its interpretation is unavailable.
    Missing,
}
/// A bounded prerequisite for one exact question and subject.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExplanatoryRequirement {
    /// Question whose explanation depends on this fact.
    pub question: AnalysisQuestion,
    /// Exact original operation, never a current-run alias.
    pub subject: OperationId,
    /// Required semantic premise.
    pub fact: RequiredFact,
    /// Actual supplied-fact availability.
    pub status: RequirementStatus,
    /// At most eight real pointers into supplied bytes, not decoded payload shortcuts.
    pub sources: Vec<AnalysisSourceReference>,
    /// Explicit bounded reason when the premise is absent.
    pub missing_reason: Option<String>,
}
// Decode only the factual portions of retained constructors. These records
// contain no executable bindings, factory implementations or replay methods.
mod dynamic_facts {
    use crate::workflow::{ReplayCaseInputs, ReplaySolverSettings, SimulationProfile};
    use pse_ids::SemanticId;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Deserialize)]
    pub(super) struct Inputs {
        #[serde(rename = "case")]
        _case: ReplayCaseInputs,
    }
    #[derive(Deserialize)]
    pub(super) struct Settings {
        #[serde(rename = "profile")]
        _profile: SimulationProfile,
    }
    #[derive(Deserialize)]
    struct AnalysisInputs {
        #[serde(rename = "analysis")]
        _analysis: Inputs,
    }
    #[derive(Deserialize)]
    struct SolverSettings {
        #[serde(rename = "settings")]
        _settings: ReplaySolverSettings,
    }
    #[derive(Deserialize)]
    struct AnalysisSettings {
        #[serde(rename = "analysis")]
        _analysis: SolverSettings,
    }
    #[derive(Deserialize)]
    pub(super) struct HorizonInputs {
        #[serde(rename = "plant")]
        _plant: Inputs,
        #[serde(rename = "inputs")]
        _inputs: Vec<(SemanticId, u64)>,
        #[serde(rename = "estimator")]
        _estimator: Option<AnalysisInputs>,
        #[serde(rename = "controller")]
        _controller: Option<AnalysisInputs>,
    }
    #[derive(Deserialize)]
    pub(super) struct HorizonSettings {
        #[serde(rename = "plant")]
        _plant: Settings,
        #[serde(rename = "estimator")]
        _estimator: Option<AnalysisSettings>,
        #[serde(rename = "controller")]
        _controller: Option<AnalysisSettings>,
    }
    #[derive(Deserialize)]
    struct Bound {
        #[serde(rename = "id")]
        _id: SemanticId,
        #[serde(rename = "lower")]
        _lower: Option<u64>,
        #[serde(rename = "upper")]
        _upper: Option<u64>,
    }
    #[derive(Deserialize)]
    pub(super) struct ShootingInputs {
        #[serde(rename = "simulation")]
        _simulation: Inputs,
        #[serde(rename = "nodes")]
        _nodes: Vec<u64>,
        #[serde(rename = "controls")]
        _controls: Vec<Bound>,
        #[serde(rename = "path")]
        _path: Vec<Bound>,
        #[serde(rename = "terminal")]
        _terminal: BTreeMap<SemanticId, u64>,
        #[serde(rename = "integral")]
        _integral: BTreeMap<SemanticId, u64>,
        #[serde(rename = "initial")]
        _initial: Option<Vec<u64>>,
    }
    #[derive(Deserialize)]
    pub(super) struct ShootingSettings {
        #[serde(rename = "simulation")]
        _simulation: Settings,
        #[serde(rename = "solver")]
        _solver: ReplaySolverSettings,
    }
}
fn supports_payload(fact: RequiredFact, retained: &crate::workflow::ReviewContextFact) -> bool {
    if retained.payload.is_empty() {
        return false;
    }
    let payload = &retained.payload;
    match (fact, retained.codec.as_str()) {
        (RequiredFact::Input, "replay-simulation-constructor.v2") => {
            serde_json::from_slice::<dynamic_facts::Inputs>(payload).is_ok()
        }
        (RequiredFact::Input, "replay-horizon-constructor.v2") => {
            serde_json::from_slice::<dynamic_facts::HorizonInputs>(payload).is_ok()
        }
        (RequiredFact::Input, "replay-shooting-constructor.v2") => {
            serde_json::from_slice::<dynamic_facts::ShootingInputs>(payload).is_ok()
        }
        (RequiredFact::EffectiveSettings, "replay-simulation-constructor.v2") => {
            serde_json::from_slice::<dynamic_facts::Settings>(payload).is_ok()
        }
        (RequiredFact::EffectiveSettings, "replay-horizon-constructor.v2") => {
            serde_json::from_slice::<dynamic_facts::HorizonSettings>(payload).is_ok()
        }
        (RequiredFact::EffectiveSettings, "replay-shooting-constructor.v2") => {
            serde_json::from_slice::<dynamic_facts::ShootingSettings>(payload).is_ok()
        }
        (
            RequiredFact::Input,
            "physical-input-bits.v1"
            | "replay-case-inputs.v2"
            | "replay-case-constructor.v2"
            | "replay-fit-constructor.v3",
        )
        | (
            RequiredFact::PhysicalCoordinates,
            "scientific-coordinates.v1" | "scientific-context.v1",
        )
        | (RequiredFact::Invocation, "invocation-evidence.v2") => true,
        _ => false,
    }
}
/// The same original-context prerequisite policy used for selected scientific products.
pub(crate) fn context_has(
    context: &crate::workflow::ReviewContextDocument,
    fact: RequiredFact,
) -> bool {
    match fact {
        RequiredFact::Model => !context.sources.is_empty(),
        RequiredFact::EffectiveSettings => {
            !context.solver_settings.is_empty()
                || context
                    .facts
                    .iter()
                    .any(|retained| supports_payload(fact, retained))
        }
        _ => context
            .facts
            .iter()
            .any(|retained| supports_payload(fact, retained)),
    }
}
fn reference(pointer: String) -> AnalysisSourceReference {
    AnalysisSourceReference {
        member: "frozen".into(),
        pointer,
        event: None,
        value: Value::Text("original retained prerequisite".into()),
        partial: false,
    }
}
/// Use the same original populations as the engine consumer. A contextual
/// association preserves the original session/observation IDs and clock scope.
fn engine_sources(
    trace: &pse_engine::session::CopiedExecutionObservations,
    question: AnalysisQuestion,
    subject: OperationId,
    base: &str,
    contextual: bool,
    add: &mut impl FnMut(AnalysisSourceReference),
) {
    use pse_engine::session::{CopiedExecutionStatus, CopiedObservationPolicy};
    let mut emit = |coordinate: &str, value: Value, partial| {
        let mut source = reference(if contextual {
            base.to_owned()
        } else {
            format!("{base}{coordinate}")
        });
        source.partial = partial;
        source.value = if contextual {
            Value::Text(format!(
                "Original engine receipt, session operation {:?}; decoded payload coordinate {coordinate}: {value:?}. The reference addresses retained bytes; context association does not align original clocks.",
                trace.operation
            ))
        } else {
            value
        };
        add(source);
    };
    if question == AnalysisQuestion::Preparation
        && (contextual || trace.operation == Some(subject))
        && let Some(policy) = trace.policy
    {
        emit("/policy", Value::Text(format!("{policy:?}")), false);
    }
    for (j, observation) in trace
        .observations
        .iter()
        .enumerate()
        .filter(|(_, observation)| contextual || observation.operation == Some(subject))
    {
        let pointer = format!("/observations/{j}");
        if question == AnalysisQuestion::Preparation {
            emit(
                &format!("{pointer}/policy"),
                Value::Text(format!("{:?}", observation.policy)),
                observation.diagnostic.is_some(),
            );
        }
        if observation.policy == CopiedObservationPolicy::Off {
            continue;
        }
        if let Some(execution) = &observation.execution {
            let partial =
                execution.truncated || execution.status != CopiedExecutionStatus::Completed;
            if question == AnalysisQuestion::Waiting {
                emit(
                    &format!("{pointer}/execution/execution"),
                    Value::Count(execution.execution),
                    partial,
                );
            }
            if question == AnalysisQuestion::EvaluationCost {
                for (k, metric) in execution
                    .metrics
                    .iter()
                    .enumerate()
                    .filter(|(_, metric)| metric.value.is_some())
                {
                    let value = metric
                        .value
                        .and_then(|value| u64::try_from(value).ok())
                        .map_or_else(
                            || {
                                Value::Unavailable(
                                    "native metric cannot be represented as UInt64".into(),
                                )
                            },
                            Value::Count,
                        );
                    emit(
                        &format!("{pointer}/execution/metrics/{k}/value"),
                        value,
                        partial,
                    );
                }
            }
        }
    }
}
// Diagnostic parents and query/admission operations do not independently
// establish scientific runs. Explicit endpoints always remain accountable,
// including when no invocation or context survived for them.
fn scientific_subject(
    set: &FrozenEvidenceSet,
    request: &AnalysisRequest,
    subject: OperationId,
) -> bool {
    subject == request.operation
        || request.baseline == Some(subject)
        || set
            .invocations
            .iter()
            .any(|invocation| invocation.operation == Some(subject))
        || set
            .contexts
            .iter()
            .any(|context| context.operation == Some(subject) && context.run_id.is_some())
}
/// Assess selected-question prerequisites once for analysis and offline publication.
/// At most 384 requirements, eight sources each and 1024 characters per missing reason.
pub fn assess(
    set: &FrozenEvidenceSet,
    request: &AnalysisRequest,
    selected: &[OperationId],
) -> Result<Vec<ExplanatoryRequirement>, ObservationError> {
    let mut result = Vec::new();
    let mut subjects = selected.to_vec();
    if !subjects.contains(&request.operation) {
        subjects.push(request.operation);
    }
    if let Some(baseline) = request.baseline
        && !subjects.contains(&baseline)
    {
        subjects.push(baseline);
    }
    subjects.sort_by_key(|id| (id.process_instance, id.sequence));
    subjects.dedup();
    for question in AnalysisQuestion::ALL
        .into_iter()
        .filter(|q| request.question.is_none_or(|chosen| chosen == *q))
    {
        let facts: &[RequiredFact] = match question {
            AnalysisQuestion::PhysicalAttribution => &[
                RequiredFact::Model,
                RequiredFact::Input,
                RequiredFact::EffectiveSettings,
                RequiredFact::PhysicalCoordinates,
            ],
            AnalysisQuestion::Comparability => &[
                RequiredFact::Invocation,
                RequiredFact::Model,
                RequiredFact::Input,
                RequiredFact::EffectiveSettings,
            ],
            _ => &[RequiredFact::OperationEvidence],
        };
        for subject in &subjects {
            if matches!(
                question,
                AnalysisQuestion::PhysicalAttribution | AnalysisQuestion::Comparability
            ) && !scientific_subject(set, request, *subject)
            {
                continue;
            }
            for fact in facts {
                if result.len() == 384 {
                    return Err(ObservationError::Format(
                        "question prerequisites exceed finite 384-item allowance".into(),
                    ));
                }
                let mut sources = Vec::new();
                let mut add = |source| {
                    if sources.len() < 8 {
                        sources.push(source);
                    }
                };
                if *fact == RequiredFact::OperationEvidence {
                    let relevant = |family: &str| {
                        super::questions::family_question(family) == Some(question)
                            || (question == AnalysisQuestion::EvaluationCost
                                && super::questions::family_question(family)
                                    == Some(AnalysisQuestion::EvaluatorWork))
                    };
                    let mut operation = |op: &pse_diagnostics::observation::OperationSnapshot,
                                         pointer: String| {
                        if question == AnalysisQuestion::Waiting {
                            add(reference(pointer.clone()));
                        }
                        for (j, metric) in op.metrics.iter().enumerate() {
                            if super::questions::metric_question(&metric.name) == Some(question) {
                                add(super::questions::metric_ref(
                                    pointer.trim_start_matches('/'),
                                    "",
                                    j,
                                    metric,
                                ));
                            }
                        }
                        for (p, producer) in op.producers.iter().enumerate() {
                            for (r, record) in producer.recent.iter().enumerate() {
                                if relevant(&record.family) {
                                    add(reference(format!("{pointer}/producers/{p}/recent/{r}")));
                                }
                            }
                            if let Some(record) = &producer.current
                                && relevant(&record.family)
                            {
                                add(reference(format!("{pointer}/producers/{p}/current")));
                            }
                        }
                    };
                    for (i, op) in set
                        .operations
                        .iter()
                        .enumerate()
                        .filter(|(_, op)| op.id == *subject)
                    {
                        operation(op, format!("/operations/{i}"));
                    }
                    for (i, checkpoint) in set.checkpoints.iter().enumerate() {
                        for (j, op) in checkpoint.operations.iter().enumerate() {
                            if op.id == *subject {
                                operation(op, format!("/checkpoints/{i}/operations/{j}"));
                            }
                        }
                    }
                    for (i, record) in set.records.iter().enumerate().filter(|(_, record)| {
                        record.producer.operation == *subject && relevant(&record.family)
                    }) {
                        let _ = record;
                        add(reference(format!("/records/{i}")));
                    }
                    for (i, page) in set.detail_pages.iter().enumerate() {
                        for (j, record) in page.records.iter().enumerate() {
                            if record.producer.operation == *subject && relevant(&record.family) {
                                add(reference(format!("/detail_pages/{i}/records/{j}")));
                            }
                        }
                    }
                }
                if *fact == RequiredFact::OperationEvidence
                    && question == AnalysisQuestion::DependencyReadiness
                {
                    for i in 0..set.inventories.len() {
                        if super::questions::inventory_selected(set, request, &[*subject], i) {
                            let mut source = reference(format!("/inventories/{i}"));
                            source.partial = set.inventories[i].truncated;
                            add(source);
                        }
                    }
                }
                if *fact == RequiredFact::OperationEvidence
                    && matches!(
                        question,
                        AnalysisQuestion::Preparation
                            | AnalysisQuestion::EvaluationCost
                            | AnalysisQuestion::Waiting
                    )
                {
                    let mut mirrored = std::collections::BTreeSet::new();
                    for (i, context) in set.contexts.iter().enumerate().filter(|(_, context)| {
                        super::questions::engine_context_selected(set, context, &[*subject])
                    }) {
                        for (j, retained) in
                            context.facts.iter().enumerate().filter(|(_, retained)| {
                                retained.codec == "engine-execution-observations.v2"
                            })
                        {
                            // Preserve one receipt when multiple source positions
                            // retain exactly the same original payload bytes.
                            if set.contexts[..i]
                                .iter()
                                .filter(|context| {
                                    super::questions::engine_context_selected(
                                        set,
                                        context,
                                        &[*subject],
                                    )
                                })
                                .flat_map(|context| &context.facts)
                                .chain(context.facts[..j].iter())
                                .any(|previous| {
                                    previous.codec == retained.codec
                                        && previous.payload == retained.payload
                                })
                            {
                                continue;
                            }
                            let trace: pse_engine::session::CopiedExecutionObservations =
                                serde_json::from_slice(&retained.payload)
                                    .map_err(|error| ObservationError::Format(error.to_string()))?;
                            for (index, flat) in set.engine_observations.iter().enumerate() {
                                if super::questions::same_engine_trace(&trace, flat) {
                                    mirrored.insert(index);
                                }
                            }
                            engine_sources(
                                &trace,
                                question,
                                *subject,
                                &format!("/contexts/{i}/facts/{j}/payload"),
                                true,
                                &mut add,
                            );
                        }
                    }
                    for (i, trace) in set
                        .engine_observations
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !mirrored.contains(i))
                    {
                        engine_sources(
                            trace,
                            question,
                            *subject,
                            &format!("/engine_observations/{i}"),
                            false,
                            &mut add,
                        );
                    }
                }
                if *fact == RequiredFact::Invocation {
                    for (i, invocation) in set.invocations.iter().enumerate() {
                        if invocation.operation == Some(*subject) {
                            add(reference(format!("/invocations/{i}")));
                        }
                    }
                }
                for (i, context) in set.contexts.iter().enumerate().filter(|(_, c)| {
                    c.operation == Some(*subject)
                        || (c.operation.is_none()
                            && set.invocations.iter().any(|invocation| {
                                invocation.operation == Some(*subject)
                                    && c.run_id == Some(invocation.run_id)
                                    && c.attempt_id == invocation.attempt_id
                            }))
                }) {
                    match fact {
                        RequiredFact::Model if !context.sources.is_empty() => {
                            add(reference(format!("/contexts/{i}/sources")))
                        }
                        RequiredFact::EffectiveSettings if !context.solver_settings.is_empty() => {
                            add(reference(format!("/contexts/{i}/solver_settings")))
                        }
                        _ => {}
                    }
                    for (j, retained) in context.facts.iter().enumerate() {
                        let supports = supports_payload(*fact, retained);
                        if supports && !retained.payload.is_empty() {
                            add(reference(format!("/contexts/{i}/facts/{j}/payload")));
                        }
                    }
                }
                let missing = sources.is_empty();
                result.push(ExplanatoryRequirement { question, subject: *subject, fact: *fact,
                    status: if missing { RequirementStatus::Missing } else { RequirementStatus::Available }, sources,
                    missing_reason: missing.then(|| format!("{question:?} requires original {fact:?} for exact operation {subject}; no supporting supplied fact is available").chars().take(1024).collect()) });
            }
        }
    }
    Ok(result)
}

/// Validate the bounded wire shape independently of supplied-source resolution.
pub fn validate(requirements: &[ExplanatoryRequirement]) -> Result<(), ObservationError> {
    if requirements.len() > 384 {
        return Err(ObservationError::Format(
            "too many explanatory requirements".into(),
        ));
    }
    for requirement in requirements {
        if requirement.sources.len() > 8
            || requirement
                .missing_reason
                .as_ref()
                .is_some_and(|reason| reason.chars().count() > 1024)
            || (requirement.status == RequirementStatus::Available
                && (requirement.sources.is_empty() || requirement.missing_reason.is_some()))
            || (requirement.status == RequirementStatus::Missing
                && (!requirement.sources.is_empty() || requirement.missing_reason.is_none()))
        {
            return Err(ObservationError::Format(
                "inconsistent bounded explanatory requirement".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{ReviewContextDocument, ReviewContextFact, ReviewContextRequest};
    use pse_diagnostics::observation::{ObservationHub, Policy};
    use pse_model::document::Version;
    use std::{sync::Arc, time::Duration};

    fn id() -> OperationId {
        OperationId {
            process_instance: [11, 12],
            sequence: 13,
        }
    }
    fn requested(question: AnalysisQuestion) -> AnalysisRequest {
        let mut request = AnalysisRequest::new(id());
        request.question = Some(question);
        request
    }
    fn premises(
        set: &FrozenEvidenceSet,
        question: AnalysisQuestion,
    ) -> Vec<ExplanatoryRequirement> {
        assess(set, &requested(question), &[id()]).unwrap()
    }
    fn resolve_sources(set: &FrozenEvidenceSet, required: &[ExplanatoryRequirement]) {
        let value = serde_json::to_value(set).unwrap();
        validate(required).unwrap();
        for requirement in required {
            assert!(requirement.sources.len() <= 8);
            for source in &requirement.sources {
                assert_eq!(source.member, "frozen");
                assert!(
                    value.pointer(&source.pointer).is_some(),
                    "{}",
                    source.pointer
                );
            }
        }
    }
    fn context(codec: &str, body: &impl Serialize) -> ReviewContextDocument {
        let selection = ReviewContextRequest {
            include_sources: false,
            ..Default::default()
        };
        let mut context = ReviewContextDocument::pre_handle_failure(
            selection,
            Some(id()),
            "retained factual selection".into(),
            vec![],
        )
        .unwrap();
        context.facts.push(ReviewContextFact {
            path: "/constructor".into(),
            codec: codec.into(),
            payload: serde_json::to_vec(body).unwrap(),
        });
        context.validate().unwrap();
        context
    }
    fn requirement(
        required: &[ExplanatoryRequirement],
        fact: RequiredFact,
    ) -> &ExplanatoryRequirement {
        required
            .iter()
            .find(|requirement| requirement.fact == fact)
            .unwrap()
    }
    fn invocation() -> crate::invocation::InvocationEvidenceDocument {
        use crate::invocation::{InvocationEndpoints, InvocationOutcome};
        let run_id = pse_model::generated::identities::RunId::from_bytes([14; 16]);
        crate::invocation::InvocationEvidenceDocument {
            version: Default::default(),
            run_id,
            attempt_id: None,
            operation: Some(id()),
            family: "modeling".into(),
            clock_scope: "original owner".into(),
            timing_unavailable: None,
            canonical_run: Some("canonical:run/exact".into()),
            canonical_attempt: Some("canonical:attempt/exact".into()),
            outcome: InvocationOutcome::Completed,
            endpoints: InvocationEndpoints::default(),
            intervals: vec![],
            omitted_intervals: 0,
            products: vec![],
            omitted_product_steps: 0,
            preparations: None,
            attempts: Arc::new(crate::workflow::AttemptSummary::empty_for_test(run_id)),
        }
    }
    fn complete_scientific_context() -> ReviewContextDocument {
        use crate::workflow::{ReviewScientificCoordinates, ReviewSourceDocument};
        use pse_ids::ContentHash;
        let run = invocation().run_id;
        let source = ContentHash::from_bytes([31; 32]);
        let physical = ContentHash::from_bytes([32; 32]);
        let mut retained = context(
            "physical-input-bits.v1",
            &std::collections::BTreeMap::<String, u64>::new(),
        );
        retained.selection.include_sources = true;
        retained.run_id = Some(run);
        retained.failure = None;
        retained.unavailable.clear();
        retained.sources.push(ReviewSourceDocument {
            research: None,
            context: None,
            identity: source,
            interpretation: "original source".into(),
            build: ContentHash::from_bytes([33; 32]),
            physical_identity: physical,
            physical_sources: vec![],
            providers: vec![],
            objects: vec![],
        });
        retained.source_positions.push(0);
        retained
            .solver_settings
            .push(crate::workflow::ReplaySolverSettings::from_profile(
                &crate::workflow::tests::profile(),
            ));
        let coordinates = ReviewScientificCoordinates {
            run_id: run,
            step: 0,
            model_id: None,
            fit_id: None,
            case_id: None,
            instance_id: None,
            source_identity: source,
            physical_identity: physical,
            preparation_identity: None,
            profile_identity: None,
            configuration: pse_model::Bytes::new(vec![]),
            binding: None,
            state: vec![],
            inputs: vec![],
            outputs: vec![],
            integrals: vec![],
            time: None,
            mode_names: vec![],
            mode_identities: vec![],
        };
        retained.facts.push(ReviewContextFact {
            path: "/steps/0/context".into(),
            codec: "scientific-coordinates.v1".into(),
            payload: serde_json::to_vec(&coordinates).unwrap(),
        });
        retained.validate().unwrap();
        retained
    }
    #[test]
    fn scientific_prerequisites_skip_recording_and_query_parents_without_scientific_runs() {
        let recording = OperationId {
            process_instance: id().process_instance,
            sequence: 40,
        };
        let query = OperationId {
            process_instance: id().process_instance,
            sequence: 41,
        };
        // Even an actual pre-handle engine receipt does not establish a second
        // scientific run for a diagnostic-only parent.
        let mut pre_handle = context(
            "engine-execution-observations.v2",
            &performed_trace(Some(query)),
        );
        pre_handle.operation = Some(query);
        assert!(pre_handle.run_id.is_none());
        let set = FrozenEvidenceSet {
            invocations: vec![invocation()],
            contexts: vec![complete_scientific_context(), pre_handle],
            ..Default::default()
        };
        let selected = [id(), recording, query];
        for question in [
            AnalysisQuestion::PhysicalAttribution,
            AnalysisQuestion::Comparability,
        ] {
            let required = assess(&set, &requested(question), &selected).unwrap();
            assert_eq!(required.len(), 4);
            assert!(
                required.iter().all(|premise| premise.subject == id()
                    && premise.status == RequirementStatus::Available)
            );
            resolve_sources(&set, &required);
        }
        // Diagnostic questions still assess the actual selected operation scopes.
        let required = assess(&set, &requested(AnalysisQuestion::Preparation), &selected).unwrap();
        assert!(
            selected
                .iter()
                .all(|subject| required.iter().any(|premise| premise.subject == *subject))
        );
        assert_eq!(
            required
                .iter()
                .find(|premise| premise.subject == query)
                .unwrap()
                .status,
            RequirementStatus::Available
        );
        assert_eq!(
            required
                .iter()
                .find(|premise| premise.subject == recording)
                .unwrap()
                .status,
            RequirementStatus::Missing
        );
        // Explicit endpoints remain mandatory even without scientific evidence.
        for question in [
            AnalysisQuestion::PhysicalAttribution,
            AnalysisQuestion::Comparability,
        ] {
            let mut request = requested(question);
            request.baseline = Some(recording);
            let required = assess(&set, &request, &selected).unwrap();
            assert_eq!(
                required
                    .iter()
                    .filter(|premise| premise.subject == recording)
                    .count(),
                4
            );
            assert!(
                required
                    .iter()
                    .filter(|premise| premise.subject == recording)
                    .all(|premise| premise.status == RequirementStatus::Missing)
            );
            let empty = assess(
                &FrozenEvidenceSet::default(),
                &requested(question),
                &selected,
            )
            .unwrap();
            assert_eq!(empty.len(), 4);
            assert!(
                empty.iter().all(|premise| premise.subject == id()
                    && premise.status == RequirementStatus::Missing)
            );
        }
    }
    #[test]
    fn scientific_prerequisites_keep_actual_child_invocations_and_context_runs_independent() {
        let child = OperationId {
            process_instance: id().process_instance,
            sequence: 42,
        };
        let contextual = OperationId {
            process_instance: id().process_instance,
            sequence: 43,
        };
        let mut child_invocation = invocation();
        child_invocation.operation = Some(child);
        child_invocation.run_id = pse_model::generated::identities::RunId::from_bytes([44; 16]);
        child_invocation.attempts = Arc::new(crate::workflow::AttemptSummary::empty_for_test(
            child_invocation.run_id,
        ));
        child_invocation.canonical_run = Some("canonical:run/child".into());
        child_invocation.canonical_attempt = Some("canonical:attempt/child".into());
        let mut child_context = context(
            "physical-input-bits.v1",
            &std::collections::BTreeMap::<String, u64>::new(),
        );
        child_context.operation = Some(contextual);
        child_context.run_id = Some(pse_model::generated::identities::RunId::from_bytes(
            [45; 16],
        ));
        child_context.failure = None;
        child_context.unavailable.clear();
        child_context.validate().unwrap();
        let set = FrozenEvidenceSet {
            invocations: vec![invocation(), child_invocation],
            contexts: vec![complete_scientific_context(), child_context],
            ..Default::default()
        };
        for question in [
            AnalysisQuestion::PhysicalAttribution,
            AnalysisQuestion::Comparability,
        ] {
            let required = assess(&set, &requested(question), &[id(), child, contextual]).unwrap();
            assert_eq!(required.len(), 12);
            assert!(
                required
                    .iter()
                    .filter(|premise| premise.subject == id())
                    .all(|premise| premise.status == RequirementStatus::Available)
            );
            let child_required: Vec<_> = required
                .iter()
                .filter(|premise| premise.subject == child)
                .collect();
            assert_eq!(child_required.len(), 4);
            assert!(child_required.iter().all(|premise| premise.status
                == if premise.fact == RequiredFact::Invocation {
                    RequirementStatus::Available
                } else {
                    RequirementStatus::Missing
                }));
            let contextual_required: Vec<_> = required
                .iter()
                .filter(|premise| premise.subject == contextual)
                .collect();
            assert_eq!(contextual_required.len(), 4);
            assert!(contextual_required.iter().all(|premise| premise.status
                == if premise.fact == RequiredFact::Input {
                    RequirementStatus::Available
                } else {
                    RequirementStatus::Missing
                }));
            resolve_sources(&set, &required);
        }
    }
    #[test]
    fn prerequisites_reuse_exact_inventory_associations_and_explicit_gap_selection() {
        use crate::workflow::{
            ReviewAttemptSelection, ReviewInventoryGap, ReviewInventoryMember,
            ReviewInventorySelection,
        };
        let mut set = FrozenEvidenceSet {
            invocations: vec![invocation()],
            ..Default::default()
        };
        set.inventories.push(ReviewInventorySelection {
            version: Version,
            scope: "canonical retained page".into(),
            attempts: ReviewAttemptSelection::Retained,
            members: vec![ReviewInventoryMember {
                key: "canonical:attempt/exact".into(),
                role: "canonical_attempt".into(),
                run: Some("canonical:run/exact".into()),
                attempt: Some("canonical:attempt/exact".into()),
                operation: None,
                generation: Some(4),
                occurrence: Some(2),
                terminal: false,
                outcome: None,
                result_manifest: None,
                readable_prefix: Some(7),
                links: vec![],
            }],
            gaps: vec![],
            truncated: false,
        });
        let required = premises(&set, AnalysisQuestion::DependencyReadiness);
        assert_eq!(required[0].status, RequirementStatus::Available);
        assert_eq!(required[0].sources[0].pointer, "/inventories/0");
        resolve_sources(&set, &required);
        set.inventories[0].members[0].attempt = Some("canonical:attempt/other".into());
        assert_eq!(
            premises(&set, AnalysisQuestion::DependencyReadiness)[0].status,
            RequirementStatus::Missing
        );
        set.inventories.push(ReviewInventorySelection {
            version: Version,
            scope: "explicit gap-only selection".into(),
            attempts: ReviewAttemptSelection::Current,
            members: vec![],
            gaps: vec![ReviewInventoryGap {
                member: "unknown exact member".into(),
                reason: "retained generation unavailable".into(),
            }],
            truncated: true,
        });
        let mut request = requested(AnalysisQuestion::DependencyReadiness);
        request.inventory_members = vec![1];
        let required = assess(&set, &request, &[id()]).unwrap();
        assert_eq!(required[0].status, RequirementStatus::Available);
        assert_eq!(required[0].sources[0].pointer, "/inventories/1");
        assert!(required[0].sources[0].partial);
        resolve_sources(&set, &required);
    }
    #[test]
    fn prerequisites_cite_latest_native_gauge_and_original_additive_counter() {
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("actual native scalar owners");
        operation
            .context()
            .gauge("idas.newton.shift", "inverse-time", 0.25);
        operation.context().count(
            "diffsol.work.number_of_linear_solver_setups_from_checkpoint",
            "count",
            3,
        );
        let snapshot = operation.reader().snapshot(Duration::from_secs(60));
        let mut request = AnalysisRequest::new(operation.context().id());
        request.question = Some(AnalysisQuestion::DynamicWork);
        let set = FrozenEvidenceSet {
            operations: vec![snapshot],
            ..Default::default()
        };
        let required = assess(&set, &request, &[request.operation]).unwrap();
        assert!(required[0].sources.iter().any(
            |source| source.pointer.ends_with("/latest") && source.value == Value::number(0.25)
        ));
        assert!(required[0].sources.iter().any(
            |source| source.pointer.ends_with("/cumulative") && source.value == Value::Count(3)
        ));
        resolve_sources(&set, &required);
    }
    #[test]
    fn prerequisites_accept_actual_engine_populations_preserve_off_and_unknown_cost() {
        use pse_engine::session::*;
        let mut set = FrozenEvidenceSet {
            engine_observations: vec![CopiedExecutionObservations {
                policy: Some(CopiedObservationPolicy::Off),
                operation: Some(id()),
                observations: vec![],
                dropped: 0,
            }],
            ..Default::default()
        };
        assert_eq!(
            premises(&set, AnalysisQuestion::Preparation)[0].status,
            RequirementStatus::Available
        );
        assert_eq!(
            premises(&set, AnalysisQuestion::EvaluationCost)[0].status,
            RequirementStatus::Missing
        );
        assert_eq!(
            premises(&set, AnalysisQuestion::Waiting)[0].status,
            RequirementStatus::Missing
        );
        set.engine_observations[0].policy = Some(CopiedObservationPolicy::Contract);
        set.engine_observations[0]
            .observations
            .push(CopiedPlanObservation {
                operation: Some(id()),
                policy: CopiedObservationPolicy::Contract,
                logical_plan: String::new(),
                physical_plan: Some(String::new()),
                physical_graph: None,
                rules: vec![],
                diagnostic: None,
                execution: Some(CopiedExecutionSnapshot {
                    execution: 42,
                    status: CopiedExecutionStatus::Abandoned,
                    truncated: true,
                    observed_after_start: Duration::from_secs(2),
                    collection_elapsed: Duration::from_micros(4),
                    metrics: (0..12)
                        .map(|node| ObservedMetric {
                            node,
                            operator: "actual operator".into(),
                            name: "peak_memory_usage".into(),
                            partition: Some(0),
                            labels: vec![],
                            unit: MetricUnit::Bytes,
                            kind: MetricKind::Gauge,
                            value: Some(128),
                        })
                        .collect(),
                }),
            });
        for question in [
            AnalysisQuestion::Preparation,
            AnalysisQuestion::EvaluationCost,
            AnalysisQuestion::Waiting,
        ] {
            let required = premises(&set, question);
            assert_eq!(required[0].status, RequirementStatus::Available);
            if question != AnalysisQuestion::Preparation {
                assert!(required[0].sources.iter().all(|source| source.partial));
            }
            resolve_sources(&set, &required);
        }
        set.engine_observations[0].observations[0]
            .execution
            .as_mut()
            .unwrap()
            .metrics
            .clear();
        assert_eq!(
            premises(&set, AnalysisQuestion::EvaluationCost)[0].status,
            RequirementStatus::Missing
        );
        set.engine_observations[0].operation = None;
        set.engine_observations[0].observations[0].operation = None;
        assert_eq!(
            premises(&set, AnalysisQuestion::Preparation)[0].status,
            RequirementStatus::Missing
        );
    }
    fn performed_trace(
        original: Option<OperationId>,
    ) -> pse_engine::session::CopiedExecutionObservations {
        use pse_engine::session::*;
        CopiedExecutionObservations {
            policy: Some(CopiedObservationPolicy::Contract),
            operation: original,
            dropped: 1,
            observations: vec![CopiedPlanObservation {
                operation: original,
                policy: CopiedObservationPolicy::Contract,
                logical_plan: String::new(),
                physical_plan: Some(String::new()),
                physical_graph: None,
                rules: vec![],
                diagnostic: None,
                execution: Some(CopiedExecutionSnapshot {
                    execution: 17,
                    status: CopiedExecutionStatus::Abandoned,
                    truncated: true,
                    observed_after_start: Duration::from_secs(2),
                    collection_elapsed: Duration::from_micros(3),
                    metrics: vec![ObservedMetric {
                        node: 0,
                        operator: "actual retained scan".into(),
                        name: "elapsed_compute".into(),
                        partition: Some(0),
                        labels: vec![],
                        unit: MetricUnit::Nanoseconds,
                        kind: MetricKind::Counter,
                        value: Some(123),
                    }],
                }),
            }],
        }
    }
    #[test]
    fn prerequisites_follow_selected_context_engine_receipts_without_rewriting_original_operations()
    {
        let foreign = OperationId {
            process_instance: [21, 22],
            sequence: 23,
        };
        for original in [Some(foreign), None] {
            let trace = performed_trace(original);
            let mut unrelated = context("engine-execution-observations.v2", &trace);
            unrelated.operation = Some(foreign);
            let mut set = FrozenEvidenceSet {
                contexts: vec![unrelated],
                engine_observations: vec![trace.clone()],
                ..Default::default()
            };
            for question in [
                AnalysisQuestion::Preparation,
                AnalysisQuestion::EvaluationCost,
                AnalysisQuestion::Waiting,
            ] {
                assert_eq!(
                    premises(&set, question)[0].status,
                    RequirementStatus::Missing,
                    "an unrelated context cannot associate its earlier session with the requested operation"
                );
            }
            let selected = context("engine-execution-observations.v2", &trace);
            set.contexts.push(selected);
            for question in [
                AnalysisQuestion::Preparation,
                AnalysisQuestion::EvaluationCost,
                AnalysisQuestion::Waiting,
            ] {
                let required = premises(&set, question);
                assert_eq!(required[0].status, RequirementStatus::Available);
                assert!(
                    required[0]
                        .sources
                        .iter()
                        .all(|source| source.pointer == "/contexts/1/facts/0/payload")
                );
                assert!(required[0].sources.iter().all(|source| matches!(&source.value, Value::Text(label) if label.contains("decoded payload coordinate") && label.contains("does not align original clocks"))));
                resolve_sources(&set, &required);
            }
            let retained: pse_engine::session::CopiedExecutionObservations =
                serde_json::from_slice(&set.contexts[1].facts[0].payload).unwrap();
            assert_eq!(retained.operation, original);
            assert_eq!(retained.observations[0].operation, original);
            // The same exact supplied invocation identities also support a
            // context whose own operation was not retained.
            let invocation = invocation();
            set.contexts[1].operation = None;
            set.contexts[1].run_id = Some(invocation.run_id);
            set.contexts[1].attempt_id = invocation.attempt_id;
            set.invocations.push(invocation);
            assert_eq!(
                premises(&set, AnalysisQuestion::Preparation)[0].status,
                RequirementStatus::Available
            );
            set.contexts[1].attempt_id = Some(
                pse_model::generated::identities::AttemptId::from_bytes([99; 16]),
            );
            assert_eq!(
                premises(&set, AnalysisQuestion::Preparation)[0].status,
                RequirementStatus::Missing
            );
        }
    }
    #[test]
    fn prerequisites_prefer_context_payload_over_exact_flat_mirrors_and_reject_malformed_receipts()
    {
        let trace = performed_trace(Some(id()));
        let mut selected = context("engine-execution-observations.v2", &trace);
        let mut duplicate = selected.facts[0].clone();
        duplicate.path = "/physical_admission/1/engine".into();
        selected.facts.push(duplicate);
        let mut set = FrozenEvidenceSet {
            contexts: vec![selected],
            engine_observations: vec![trace],
            ..Default::default()
        };
        let required = premises(&set, AnalysisQuestion::Preparation);
        assert_eq!(
            required[0].sources.len(),
            2,
            "one policy and one observation, without repeated source positions or flat mirrors"
        );
        assert!(
            required[0]
                .sources
                .iter()
                .all(|source| source.pointer == "/contexts/0/facts/0/payload")
        );
        let required = premises(&set, AnalysisQuestion::EvaluationCost);
        assert_eq!(
            required[0].sources.len(),
            1,
            "the original metric contributes once"
        );
        assert!(
            matches!(&required[0].sources[0].value, Value::Text(label) if label.contains("/observations/0/execution/metrics/0/value") && label.contains("Count(123)"))
        );
        resolve_sources(&set, &required);
        set.contexts[0].facts.truncate(1);
        set.contexts[0].facts[0].payload = br#"{"observations":"invalid owner shape"}"#.to_vec();
        assert!(assess(&set, &requested(AnalysisQuestion::Preparation), &[id()]).is_err());
    }
    #[test]
    fn prerequisites_decode_dynamic_input_and_settings_facts_without_replay_admission() {
        let profile = serde_json::to_value(crate::workflow::SimulationProfile::default()).unwrap();
        let case = serde_json::json!({"values": {}, "members": {}, "variables": {}});
        let simulation = serde_json::json!({"source": 0, "case": case, "profile": profile, "bindings": "opaque retained descriptor"});
        let solver = serde_json::to_value(crate::workflow::ReplaySolverSettings::from_profile(
            &crate::workflow::tests::profile(),
        ))
        .unwrap();
        let horizon = serde_json::json!({"plant": simulation, "inputs": [], "estimator": null, "controller": null});
        let shooting = serde_json::json!({"simulation": simulation, "solver": solver, "nodes": [], "controls": [], "path": [],
            "terminal": {}, "integral": {}, "initial": null});
        for (codec, body, dynamic_path) in [
            ("replay-simulation-constructor.v2", simulation, "/profile"),
            ("replay-horizon-constructor.v2", horizon, "/plant/profile"),
            (
                "replay-shooting-constructor.v2",
                shooting,
                "/simulation/profile",
            ),
        ] {
            let context = context(codec, &body);
            assert!(context_has(&context, RequiredFact::Input));
            assert!(context_has(&context, RequiredFact::EffectiveSettings));
            let mut set = FrozenEvidenceSet {
                contexts: vec![context],
                ..Default::default()
            };
            let required = premises(&set, AnalysisQuestion::PhysicalAttribution);
            assert_eq!(
                requirement(&required, RequiredFact::Input).status,
                RequirementStatus::Available
            );
            assert_eq!(
                requirement(&required, RequiredFact::EffectiveSettings).status,
                RequirementStatus::Available
            );
            assert_eq!(
                requirement(&required, RequiredFact::Model).status,
                RequirementStatus::Missing
            );
            resolve_sources(&set, &required);
            let mut missing_settings = body.clone();
            *missing_settings.pointer_mut(dynamic_path).unwrap() = serde_json::Value::Null;
            set.contexts[0].facts[0].payload = serde_json::to_vec(&missing_settings).unwrap();
            assert!(context_has(&set.contexts[0], RequiredFact::Input));
            assert!(!context_has(
                &set.contexts[0],
                RequiredFact::EffectiveSettings
            ));
            let input_path = dynamic_path.strip_suffix("profile").unwrap().to_owned() + "case";
            let mut missing_input = body;
            *missing_input.pointer_mut(&input_path).unwrap() = serde_json::Value::Null;
            set.contexts[0].facts[0].payload = serde_json::to_vec(&missing_input).unwrap();
            assert!(!context_has(&set.contexts[0], RequiredFact::Input));
            assert!(context_has(
                &set.contexts[0],
                RequiredFact::EffectiveSettings
            ));
        }
    }
}
