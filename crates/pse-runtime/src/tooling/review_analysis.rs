// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded, passive cross-owner questions over explicitly supplied immutable facts.
//! No classifier here changes scientific acceptance or activates collection.
use crate::{observation::ObservationError};
use crate::tooling::{frozen_inspection::{FrozenEvidenceSet, RecordedWindow}};
use pse_diagnostics::{
    NextObservation, StatementKind,
    observation::{CoherentRecord, OperationId, ProducerId, Value},
};
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};
mod physical;
mod preparation;
mod questions;
pub(crate) mod render;
pub mod requirements;
pub use render::{render_markdown, render_markdown_with_sources};
#[cfg(test)]
mod tests;

/// Named review questions. They are independent of optional profiler availability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisQuestion {
    /// Stable construction versus value binding.
    Preparation,
    /// Demand versus actual work and derivative order.
    EvaluatorWork,
    /// Matched complete inclusive call populations.
    EvaluationCost,
    /// Accepted/trial/raw-map and physical error progress.
    RootProgress,
    /// Forward/sensitivity/quadrature/backward/replay work.
    DynamicWork,
    /// Exact study membership and readiness.
    DependencyReadiness,
    /// Parent factor recovery versus substitutions.
    FactorRecovery,
    /// Actual interval populations, without inferred scheduler blocking.
    Waiting,
    /// Existing invocation comparison policy and refusals.
    Comparability,
    /// Actual source coordinates and retained mapping witnesses.
    PhysicalAttribution,
}
impl AnalysisQuestion {
    /// Stable question order, also used by local renderers.
    pub const ALL: [Self; 10] = [
        Self::Preparation,
        Self::EvaluatorWork,
        Self::EvaluationCost,
        Self::RootProgress,
        Self::DynamicWork,
        Self::DependencyReadiness,
        Self::FactorRecovery,
        Self::Waiting,
        Self::Comparability,
        Self::PhysicalAttribution,
    ];
    /// Human-readable question; no numerical authority lives in this wording.
    pub fn title(self) -> &'static str {
        match self {
            Self::Preparation => "Did an edit rebuild stable preparation or bind values?",
            Self::EvaluatorWork => "Was there more evaluator work or broader derivative demand?",
            Self::EvaluationCost => "Did each actual evaluation become more costly?",
            Self::RootProgress => {
                "Is root/recycle progress dominated by trials, rejection or physical error?"
            }
            Self::DynamicWork => "Is dynamic work numerical difficulty or reconstruction?",
            Self::DependencyReadiness => "Which fitting/study dependency limits readiness?",
            Self::FactorRecovery => "Did factor recovery cause extra work?",
            Self::Waiting => "What does end-to-end waiting mean?",
            Self::Comparability => "Are completed runs comparable?",
            Self::PhysicalAttribution => "Can a finding be followed to physical model terms?",
        }
    }
}
fn default_sources() -> usize {
    1024
}
fn default_statements() -> usize {
    128
}
/// Explicit finite review policy; thresholds are forwarded to the existing numerical owner.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRequest {
    /// Current request contract; unsupported versions refuse at decode.
    pub version: Version<2>,
    /// Exact selected original operation.
    pub operation: OperationId,
    /// Optional exact baseline, never selected by name or a current pointer.
    pub baseline: Option<OperationId>,
    /// Exact supplied inventory indices; empty uses exact operation/run/attempt associations.
    #[serde(default)]
    pub inventory_members: Vec<usize>,
    /// One bounded question; absent selects all ten.
    #[serde(default)]
    pub question: Option<AnalysisQuestion>,
    /// Original source-clock selection.
    #[serde(default)]
    pub window: RecordedWindow,
    /// Unique source records/scalars inspected, capped at 4096.
    #[serde(default = "default_sources")]
    pub source_items: usize,
    /// Returned statements, capped at 256; all ten questions remain represented.
    #[serde(default = "default_statements")]
    pub statements: usize,
    /// Explicit existing numerical interpretation policy, never a convergence policy.
    pub numerical_history_policy: Option<pse_diagnostics::inspection::NumericalHistoryPolicy>,
}
impl AnalysisRequest {
    /// Ordinary finite review with no inferred baseline or numerical threshold.
    pub fn new(operation: OperationId) -> Self {
        Self {
            version: Version,
            operation,
            baseline: None,
            inventory_members: vec![],
            question: None,
            window: RecordedWindow::Recorded,
            source_items: default_sources(),
            statements: default_statements(),
            numerical_history_policy: None,
        }
    }
}
/// Whole event identity; time, family and iteration alone cannot identify a source event.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisEventIdentity {
    /// Includes operation and exact performing producer instance.
    pub producer: ProducerId,
    /// Source reset/attempt generation.
    pub generation: u64,
    /// Source-issued event sequence before fanout.
    pub sequence: u64,
    /// Independent sequence within the declared source family.
    pub family_sequence: Option<u64>,
    /// Exact source field ordinal, absent for a whole-event fact.
    pub field_ordinal: Option<usize>,
}
/// Reference into the original supplied set, independent of its filesystem location.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisSourceReference {
    /// Explicit pack member identity; `frozen` names the supplied immutable set.
    pub member: String,
    /// RFC 6901 pointer relative to this member; empty addresses the whole member.
    pub pointer: String,
    /// Exact coherent-event identity when applicable.
    pub event: Option<AnalysisEventIdentity>,
    /// Source-native value category; UInt64 never passes through floating point.
    pub value: Value,
    /// Original coverage or availability limits this fact.
    pub partial: bool,
}
/// A source-linked fact, qualified hypothesis or explicit limitation.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisStatement {
    /// Existing shared interpretation strength.
    pub kind: StatementKind,
    /// Stable semantic code.
    pub code: String,
    /// Bounded explanation generated from the typed source facts.
    pub message: String,
    /// Actual members supporting this explanation.
    pub evidence: Vec<AnalysisSourceReference>,
    /// Optional discriminating observations; analysis never performs them.
    pub next: Vec<NextObservation>,
}
/// Independent answer, present even when all its required sources are unavailable.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuestionAnalysis {
    /// The selected bounded question.
    pub question: AnalysisQuestion,
    /// Facts and limits, with no silent empty-success substitution.
    pub statements: Vec<AnalysisStatement>,
}
/// Relation of an actual source-clock selection to the requested operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisSelectionScope {
    /// The requested process clock supplies the interval.
    Direct,
    /// A related or baseline operation retains its own independent interval.
    Independent,
    /// No operation snapshot supplies a coverage interval; supplied records remain readable.
    Unknown,
}
/// Actual selection, retaining clock and interval uncertainty without a reader-time anchor.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisSelection {
    /// Exact selected original operation.
    pub operation: OperationId,
    /// Original process-clock identity; equal numeric timestamps do not align other clocks.
    pub process_clock: [u64; 2],
    /// Actual lower bound, absent for unknown coverage.
    #[schemars(with = "Option<pse_model::document::ClosedDuration>")]
    pub interval_start: Option<Duration>,
    /// Actual upper bound, absent for unknown coverage.
    #[schemars(with = "Option<pse_model::document::ClosedDuration>")]
    pub interval_end: Option<Duration>,
    /// Direct, independently selected, or unknown source coverage.
    pub scope: AnalysisSelectionScope,
    /// Whether absolute requested-clock window claims can be made for this selection.
    pub aligned_to_request: bool,
}
/// Admitted result. Its allocation owner lives through borrowing and serialization.
#[derive(Debug)]
pub struct Analysis {
    document: AnalysisDocument,
    lease: Arc<pse_columnar::AllocationLease>,
}
impl Analysis {
    /// Borrow the serializable data-only result without dropping its admission owner.
    pub fn document(&self) -> &AnalysisDocument {
        &self.document
    }
    /// Capacity retained by this result's native allocation owner.
    pub fn retained_bytes(&self) -> usize {
        self.lease.size()
    }
}
impl std::ops::Deref for Analysis {
    type Target = AnalysisDocument;
    fn deref(&self) -> &Self::Target {
        &self.document
    }
}
impl Serialize for Analysis {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.document.serialize(serializer)
    }
}
/// Shared typed product for live-copy, frozen, CLI and offline pack consumers.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisDocument {
    /// Current result contract.
    pub version: Version<2>,
    /// Effective bounded policy actually used.
    pub request: AnalysisRequest,
    /// Requested independent questions in stable order; all ten when no selector is supplied.
    pub questions: Vec<QuestionAnalysis>,
    /// Actual source-clock interval used for every selected operation.
    pub selections: Vec<AnalysisSelection>,
    /// Question-dependent explanatory premises over the supplied set.
    pub requirements: Vec<requirements::ExplanatoryRequirement>,
    /// Unique original source items inspected; shared facts are counted once.
    pub inspected_source_items: usize,
    /// Unique eligible source items not inspected due to the selected allowance.
    pub omitted_source_items: usize,
    /// Derived statements omitted by the response allowance.
    pub omitted_statements: usize,
}
impl AnalysisDocument {
    /// Validate this bounded product's shape. Source resolution remains the explicit pack owner's job.
    /// # Errors
    /// Missing/duplicate/unrequested questions, unsupported policy or impossible response accounting.
    pub fn validate(&self) -> Result<(), ObservationError> {
        requirements::validate(&self.requirements)?;
        if self
            .requirements
            .iter()
            .enumerate()
            .any(|(index, requirement)| {
                self.request
                    .question
                    .is_some_and(|question| question != requirement.question)
                    || !self
                        .selections
                        .iter()
                        .any(|selection| selection.operation == requirement.subject)
                    || self.requirements[..index].iter().any(|prior| {
                        prior.subject == requirement.subject
                            && prior.question == requirement.question
                            && prior.fact == requirement.fact
                    })
            })
        {
            return Err(invalid(
                "analysis requirements contradict selected questions or subjects",
            ));
        }
        let expected = AnalysisQuestion::ALL
            .into_iter()
            .filter(|question| {
                self.request
                    .question
                    .is_none_or(|selected| selected == *question)
            })
            .collect::<Vec<_>>();
        if self.request.source_items == 0
            || self.request.source_items > 4096
            || self.request.statements < expected.len()
            || self.request.statements > 256
            || self.inspected_source_items > self.request.source_items
        {
            return Err(invalid(
                "analysis product has invalid effective bounds or source accounting",
            ));
        }
        if let RecordedWindow::Interval { start, end } = &self.request.window
            && start > end
        {
            return Err(invalid("analysis product interval start exceeds end"));
        }
        if self
            .request
            .numerical_history_policy
            .as_ref()
            .is_some_and(|policy| policy.version != 1)
        {
            return Err(invalid(
                "unsupported analysis numerical-history policy version",
            ));
        }
        if self
            .questions
            .iter()
            .map(|question| question.question)
            .collect::<Vec<_>>()
            != expected
        {
            return Err(invalid(
                "analysis product must contain each requested question exactly once in stable order",
            ));
        }
        if self.request.inventory_members.len() > 32
            || self
                .request
                .inventory_members
                .iter()
                .enumerate()
                .any(|(index, member)| self.request.inventory_members[..index].contains(member))
            || self.selections.is_empty()
            || self.selections.len() > 64
            || self.requirements.len() > 384
            || self
                .selections
                .iter()
                .enumerate()
                .any(|(index, selection)| {
                    selection.process_clock != selection.operation.process_instance
                        || self.selections[..index]
                            .iter()
                            .any(|prior| prior.operation == selection.operation)
                        || selection.aligned_to_request
                            != (selection.process_clock == self.request.operation.process_instance)
                        || matches!(selection.scope, AnalysisSelectionScope::Direct)
                            && !selection.aligned_to_request
                        || matches!(selection.scope, AnalysisSelectionScope::Independent)
                            && selection.aligned_to_request
                        || matches!(selection.scope, AnalysisSelectionScope::Unknown)
                            && (selection.interval_start.is_some()
                                || selection.interval_end.is_some())
                        || selection.interval_start.is_some() != selection.interval_end.is_some()
                        || selection
                            .interval_start
                            .zip(selection.interval_end)
                            .is_some_and(|(start, end)| start > end)
                })
            || !self
                .selections
                .iter()
                .any(|selection| selection.operation == self.request.operation)
            || self.request.baseline.is_some_and(|baseline| {
                !self
                    .selections
                    .iter()
                    .any(|selection| selection.operation == baseline)
            })
        {
            return Err(invalid(
                "analysis product has invalid exact source-clock or inventory selection",
            ));
        }
        let mut statements = 0usize;
        for question in &self.questions {
            if question.statements.is_empty() {
                return Err(invalid(
                    "analysis requested question has no answer or explicit limitation",
                ));
            }
            statements = statements
                .checked_add(question.statements.len())
                .ok_or_else(|| invalid("analysis statement count overflow"))?;
            for statement in &question.statements {
                for reference in &statement.evidence {
                    if reference.member.is_empty()
                        || (!reference.pointer.is_empty() && !reference.pointer.starts_with('/'))
                    {
                        return Err(invalid(
                            "analysis reference requires explicit member and JSON pointer",
                        ));
                    }
                    let mut bytes = reference.pointer.bytes();
                    while let Some(byte) = bytes.next() {
                        if byte == b'~' && !matches!(bytes.next(), Some(b'0' | b'1')) {
                            return Err(invalid(
                                "analysis reference has invalid JSON pointer escape",
                            ));
                        }
                    }
                }
            }
        }
        if statements > self.request.statements {
            return Err(invalid(
                "analysis returned more statements than its effective allowance",
            ));
        }
        Ok(())
    }
}
struct Builder {
    document: AnalysisDocument,
    inspected: BTreeSet<String>,
    omitted: BTreeSet<String>,
    statements: usize,
    cancel: pse_columnar::CancellationToken,
}
fn invalid(reason: impl Into<String>) -> ObservationError {
    ObservationError::Format(reason.into())
}
fn bounded_text(text: &str) -> String {
    text.chars().take(1024).collect()
}
fn bounded_value(value: Value) -> Value {
    match value {
        Value::Text(text) if text.len() > 2048 => Value::Unavailable(
            "source text exceeds finite analysis view; original member retains it".into(),
        ),
        Value::Unavailable(text) => Value::Unavailable(bounded_text(&text)),
        value => value,
    }
}
fn source(
    member: impl Into<String>,
    pointer: impl Into<String>,
    value: Value,
    partial: bool,
) -> AnalysisSourceReference {
    AnalysisSourceReference {
        member: "frozen".into(),
        pointer: format!("/{}{}", member.into(), pointer.into()),
        event: None,
        value: bounded_value(value),
        partial,
    }
}
impl Builder {
    fn checkpoint(&self) -> Result<(), ObservationError> {
        self.cancel.checkpoint().map_err(|e| invalid(e.to_string()))
    }
    fn selected(&self, question: AnalysisQuestion) -> bool {
        !self.cancel.is_cancelled()
            && self
                .document
                .request
                .question
                .is_none_or(|selected| selected == question)
    }
    fn admit(&mut self, member: &str, pointer: &str) -> bool {
        if self.cancel.is_cancelled() {
            return false;
        }
        let key = format!("{member}#{pointer}");
        if self.inspected.contains(&key) {
            return true;
        }
        if self.inspected.len() >= self.document.request.source_items {
            self.omitted.insert(key);
            return false;
        }
        self.omitted.remove(&key);
        self.inspected.insert(key);
        true
    }
    fn add(
        &mut self,
        q: AnalysisQuestion,
        kind: StatementKind,
        code: &str,
        message: impl Into<String>,
        evidence: Vec<AnalysisSourceReference>,
        next: Vec<NextObservation>,
    ) {
        if !self.selected(q) {
            return;
        }
        if self.statements >= self.document.request.statements {
            self.document.omitted_statements += 1;
            return;
        }
        if let Some(question) = self
            .document
            .questions
            .iter_mut()
            .find(|entry| entry.question == q)
        {
            question.statements.push(AnalysisStatement {
                kind,
                code: code.into(),
                message: bounded_text(&message.into()),
                evidence,
                next,
            });
            self.statements += 1;
        }
    }
}
type EventKey = ([u64; 2], u64, u64, u64, u64);
fn key(record: &CoherentRecord) -> EventKey {
    (
        record.producer.operation.process_instance,
        record.producer.operation.sequence,
        record.producer.sequence,
        record.generation,
        record.sequence,
    )
}
struct Event<'a> {
    record: &'a CoherentRecord,
    member: String,
    pointer: String,
    partial: bool,
    predecessor: bool,
}
impl Event<'_> {
    fn reference(&self, ordinal: Option<usize>) -> AnalysisSourceReference {
        AnalysisSourceReference {
            member: "frozen".into(),
            pointer: format!(
                "/{}{}",
                self.member,
                ordinal.map_or_else(
                    || format!("{}/family", self.pointer),
                    |i| format!("{}/values/{i}/value", self.pointer)
                )
            ),
            event: Some(AnalysisEventIdentity {
                producer: self.record.producer,
                generation: self.record.generation,
                sequence: self.record.sequence,
                family_sequence: self.record.family_sequence,
                field_ordinal: ordinal,
            }),
            value: bounded_value(ordinal.and_then(|i| self.record.values.get(i)).map_or_else(
                || Value::Text(self.record.family.clone()),
                |field| field.value.clone(),
            )),
            partial: self.partial,
        }
    }
}
fn operation_member(
    set: &FrozenEvidenceSet,
    id: OperationId,
) -> Option<(
    String,
    String,
    &pse_diagnostics::observation::OperationSnapshot,
)> {
    for (i, op) in set.operations.iter().enumerate() {
        if op.id == id {
            return Some((format!("operations/{i}"), String::new(), op));
        }
    }
    for (i, image) in set.checkpoints.iter().enumerate() {
        for (j, op) in image.operations.iter().enumerate() {
            if op.id == id {
                return Some((format!("checkpoints/{i}"), format!("/operations/{j}"), op));
            }
        }
    }
    None
}
fn events<'a>(
    set: &'a FrozenEvidenceSet,
    ids: &[OperationId],
    selections: &[AnalysisSelection],
    b: &mut Builder,
) -> Result<Vec<Event<'a>>, ObservationError> {
    let mut records: BTreeMap<EventKey, Event<'a>> = BTreeMap::new();
    let mut insert = |record: &'a CoherentRecord,
                      member: String,
                      pointer: String,
                      partial: bool|
     -> Result<(), ObservationError> {
        b.checkpoint()?;
        if !ids.contains(&record.producer.operation) {
            return Ok(());
        }
        // Select before growing the event index. Independent clocks never see parent numbers.
        if selections
            .iter()
            .find(|selection| selection.operation == record.producer.operation)
            .and_then(|selection| selection.interval_end)
            .is_some_and(|end| record.observed_at > end)
        {
            return Ok(());
        }
        let identity = key(record);
        if let Some(previous) = records.get(&identity) {
            if !preparation::same_record(previous.record, record) {
                return Err(invalid("conflicting full coherent event identity"));
            }
            // Identical current/recent/detail copies represent one logical event, never two work items.
            return Ok(());
        }
        records.insert(
            identity,
            Event {
                record,
                member,
                pointer,
                partial,
                predecessor: false,
            },
        );
        Ok(())
    };
    // Explicit original members take reference precedence over compact hub copies.
    for (i, record) in set.records.iter().enumerate() {
        insert(record, format!("records/{i}"), String::new(), true)?;
    }
    for (i, page) in set.detail_pages.iter().enumerate() {
        for (j, record) in page.records.iter().enumerate() {
            insert(
                record,
                format!("detail_pages/{i}"),
                format!("/records/{j}"),
                page.partial || page.next_offset.is_some(),
            )?;
        }
    }
    for id in ids {
        if let Some((member, pointer, op)) = operation_member(set, *id) {
            for (p, producer) in op.producers.iter().enumerate() {
                for (i, record) in producer.recent.iter().enumerate() {
                    insert(
                        record,
                        member.clone(),
                        format!("{pointer}/producers/{p}/recent/{i}"),
                        producer.coverage.partial,
                    )?;
                }
                if let Some(record) = &producer.current {
                    insert(
                        record,
                        member.clone(),
                        format!("{pointer}/producers/{p}/current"),
                        producer.coverage.partial,
                    )?;
                }
            }
        }
    }
    let mut selected = Vec::new();
    let mut predecessors: BTreeMap<([u64; 2], u64, u64, u64, String), Event<'a>> = BTreeMap::new();
    for (_, mut event) in records {
        b.checkpoint()?;
        let selection = selections
            .iter()
            .find(|selection| selection.operation == event.record.producer.operation);
        let interval = selection.map_or((Duration::ZERO, Duration::MAX), |selection| {
            (
                selection.interval_start.unwrap_or(Duration::ZERO),
                selection.interval_end.unwrap_or(Duration::MAX),
            )
        });
        if event.record.observed_at > interval.1 {
            continue;
        }
        if event.record.observed_at < interval.0 {
            event.predecessor = true;
            let r = event.record;
            let stream = (
                r.producer.operation.process_instance,
                r.producer.operation.sequence,
                r.producer.sequence,
                r.generation,
                r.family.clone(),
            );
            if predecessors
                .get(&stream)
                .is_none_or(|prior| prior.record.sequence < r.sequence)
            {
                predecessors.insert(stream, event);
            }
        } else if b.admit(&event.member, &event.pointer) {
            selected.push(event);
        }
    }
    for event in predecessors.into_values() {
        b.checkpoint()?;
        if b.admit(&event.member, &event.pointer) {
            selected.push(event);
        }
    }
    selected.sort_by_key(|event| key(event.record));
    Ok(selected)
}
/// Analyze only supplied immutable facts, with explicit source and response limits.
/// # Errors
/// Unsupported source codecs, duplicate/conflicting source identities, invalid intervals or policy.
pub fn analyze(
    set: &FrozenEvidenceSet,
    request: &AnalysisRequest,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<Analysis, ObservationError> {
    let lease = preparation::admit(set, request, pool, cancel)?;
    set.validate()?;
    cancel.checkpoint().map_err(|e| invalid(e.to_string()))?;
    let mut declarations = BTreeMap::new();
    for descriptor in set
        .operations
        .iter()
        .chain(set.checkpoints.iter().flat_map(|image| &image.operations))
        .flat_map(|op| op.producers.iter().map(|producer| &producer.descriptor))
        .chain(set.detail_pages.iter().flat_map(|page| &page.producers))
        .chain(set.detail_pages.iter().flat_map(|page| {
            page.activations
                .iter()
                .map(|activation| &activation.producer)
        }))
    {
        cancel
            .checkpoint()
            .map_err(|error| invalid(error.to_string()))?;
        let id = descriptor.id;
        let key = (
            id.operation.process_instance,
            id.operation.sequence,
            id.sequence,
        );
        if let Some((role, name)) =
            declarations.insert(key, (descriptor.role, descriptor.name.as_str()))
            && (role != descriptor.role || name != descriptor.name)
        {
            return Err(invalid("conflicting performing producer declarations"));
        }
    }
    if request.source_items == 0
        || request.statements < if request.question.is_some() { 1 } else { 10 }
    {
        return Err(invalid(
            "analysis needs a positive source allowance and one statement slot per requested question",
        ));
    }
    if let RecordedWindow::Interval { start, end } = &request.window
        && start > end
    {
        return Err(invalid("analysis interval start exceeds end"));
    }
    if request
        .numerical_history_policy
        .as_ref()
        .is_some_and(|policy| policy.version != 1)
    {
        return Err(invalid("unsupported numerical-history policy version"));
    }
    let mut request = request.clone();
    request.source_items = request.source_items.min(4096);
    request.statements = request.statements.min(256);
    let mut ids = vec![request.operation];
    if let Some(id) = request.baseline
        && !ids.contains(&id)
    {
        ids.push(id);
    }
    // Extend only through actual supplied relationship owners; no discovery or guessed ancestry.
    if let Some(op) = set.operation(request.operation) {
        for child in &op.children {
            cancel
                .checkpoint()
                .map_err(|error| invalid(error.to_string()))?;
            if ids.len() >= 64 && !ids.contains(&child.id) {
                return Err(invalid(
                    "analysis operation selection exceeds 64 exact subjects",
                ));
            }
            if !ids.contains(&child.id) {
                ids.push(child.id);
            }
        }
    }
    for op in set
        .operations
        .iter()
        .chain(set.checkpoints.iter().flat_map(|image| &image.operations))
    {
        if op
            .children
            .iter()
            .any(|child| child.id == request.operation)
            && !ids.contains(&op.id)
        {
            ids.push(op.id);
        }
    }
    if request.inventory_members.len() > 32
        || request
            .inventory_members
            .iter()
            .enumerate()
            .any(|(i, member)| {
                *member >= set.inventories.len() || request.inventory_members[..i].contains(member)
            })
    {
        return Err(invalid(
            "analysis inventory selection needs at most 32 unique valid supplied indices",
        ));
    }
    if ids.len() > 64 {
        return Err(invalid(
            "analysis operation selection exceeds 64 exact subjects",
        ));
    }
    let selections = preparation::selections(set, &request, &ids);
    let required = requirements::assess(set, &request, &ids)?;
    let requested_questions = AnalysisQuestion::ALL
        .into_iter()
        .filter(|question| {
            request
                .question
                .is_none_or(|selected| selected == *question)
        })
        .collect::<Vec<_>>();
    let mut b = Builder {
        document: AnalysisDocument {
            version: Version,
            request,
            selections,
            requirements: required,
            questions: requested_questions
                .into_iter()
                .map(|question| QuestionAnalysis {
                    question,
                    statements: vec![],
                })
                .collect(),
            inspected_source_items: 0,
            omitted_source_items: 0,
            omitted_statements: 0,
        },
        inspected: BTreeSet::new(),
        omitted: BTreeSet::new(),
        statements: 0,
        cancel: cancel.clone(),
    };
    let selections = b.document.selections.clone();
    let events = events(set, &ids, &selections, &mut b)?;
    questions::limitations(&mut b);
    for selection in &selections {
        if selection.scope == AnalysisSelectionScope::Unknown {
            b.add(AnalysisQuestion::Waiting, StatementKind::Limitation, "selection.coverage_unknown",
                format!("Operation {} has no supplied operation snapshot. Its supplied coherent records are retained with unknown interval coverage; no reader clock or inferred clock mapping selected them.", selection.operation), vec![], vec![NextObservation::InspectRelated]);
        } else if !selection.aligned_to_request
            && !matches!(b.document.request.window, RecordedWindow::Recorded)
        {
            b.add(AnalysisQuestion::Waiting, StatementKind::Limitation, "selection.independent_clock",
                format!("Operation {} uses its own retained interval {:?}..{:?} on process clock {:?}. The requested operation's numeric absolute window was not applied; no cross-clock ordering or absolute overlap is established. Full-invocation comparisons retain their independent original-clock meaning.", selection.operation, selection.interval_start, selection.interval_end, selection.process_clock), vec![], vec![NextObservation::TimelineCapture]);
        }
    }
    questions::operations(&mut b, set, &ids)?;
    questions::records(&mut b, set, &events)?;
    questions::invocations(&mut b, set)?;
    b.checkpoint()?;
    questions::inventories(&mut b, set, &ids)?;
    questions::context(&mut b, set, &ids)?;
    questions::engine(&mut b, set, &ids)?;
    physical::analyze(&mut b, set, &ids, &events)?;
    b.checkpoint()?;
    for question in &mut b.document.questions {
        if !question
            .statements
            .iter()
            .any(|statement| statement.kind == StatementKind::Established)
            && let Some(scope) = question.statements.first_mut()
        {
            scope.message.push_str(" No source-linked facts were returned for this question within the supplied inputs and selected bounds; its required sources remain unavailable to this answer.");
        }
    }
    b.document.inspected_source_items = b.inspected.len();
    b.document.omitted_source_items = b.omitted.len();
    b.document.validate()?;
    Ok(Analysis {
        document: b.document,
        lease,
    })
}
