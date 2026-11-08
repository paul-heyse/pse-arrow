// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable source-bound change proposals, without filesystem application.
//!
//! Paths identify explicitly supplied operands, not canonical filesystem objects. Each source
//! retains its exact original text; a collection is not an atomic working-tree snapshot.

pub use crate::source_read::{SourceIdentity, SourceText};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    ops::Range,
    path::{Path, PathBuf},
};
mod grammar;
#[path = "../../../scripts/source-change-proposal.rs"]
pub(crate) mod handoff;
pub use grammar::{GrammarEdit, preview_grammar};

/// Producer provenance; syntax mode has no semantic identity or resolution field.
///
/// These are producer attestations, not independent proof of backend correctness, compilation
/// or applicability. Configuration identifies the producer's selected context, not a default.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(tag = "authority", rename_all = "snake_case")]
pub enum ProducerContext {
    /// Source-only matching or construction, without name resolution.
    Syntax {
        /// Actual backend and version.
        backend: String,
        /// Explicit syntax edition, grammar/strictness or other selected policy.
        configuration: String,
    },
    /// An actual semantic producer retains its synchronization context.
    Semantic {
        /// Actual semantic backend and version.
        backend: String,
        /// Selected workspace/package/target/features/configuration identity.
        configuration: String,
        /// Producer's synchronized analysis/source generation.
        synchronized_generation: String,
    },
}

/// Producer input: one candidate replacement in original UTF-8 byte coordinates.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct EditCandidate {
    source: SourceIdentity,
    range: Range<usize>,
    replacement: String,
}
impl EditCandidate {
    /// Supply a candidate; the proposal constructor checks source binding and range validity.
    pub fn new(source: SourceIdentity, range: Range<usize>, replacement: String) -> Self {
        Self {
            source,
            range,
            replacement,
        }
    }
    /// Original addressed input.
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }
    /// Half-open byte range in the original source; an empty range is an insertion.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    /// Exact proposed replacement, without formatting or placeholder expansion.
    pub fn replacement(&self) -> &str {
        &self.replacement
    }
}

/// Overlapping candidates remain visible even when only one is selected.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, JsonSchema)]
pub struct CandidateConflict {
    /// First index in the complete candidate collection.
    pub first: usize,
    /// Second index in the complete candidate collection.
    pub second: usize,
}

/// An unresolved producer snippet or editing obligation, retained verbatim.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct UnresolvedPlaceholder {
    /// Index in all candidates, not the selected subset.
    pub candidate: usize,
    /// Producer explanation; no placeholder is silently filled by this owner.
    pub description: String,
}

/// Explicit regular-file effects, retained independently from replacement candidates.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum FileOperation {
    /// Proposed new regular file. Absence is an application-time precondition.
    Create {
        /// Absolute destination operand; absence remains an application-time precondition.
        path: OsString,
        /// Exact proposed UTF-8 contents.
        text: String,
    },
    /// Delete the exact addressed working original.
    Delete {
        /// Exact addressed original working file.
        source: SourceIdentity,
    },
    /// Move the exact working original; selected edits still address its original bytes.
    Rename {
        /// Exact addressed original working file.
        source: SourceIdentity,
        /// Absolute destination operand.
        destination: OsString,
    },
}

/// Checked proposal; it grants no write, compilation or Git-publication authority.
#[derive(Debug)]
pub struct ChangeProposal {
    producer: ProducerContext,
    sources: Vec<SourceText>,
    file_operations: Vec<FileOperation>,
    candidates: Vec<EditCandidate>,
    selected: Vec<usize>,
    conflicts: Vec<CandidateConflict>,
    limitations: Vec<String>,
    unresolved_placeholders: Vec<UnresolvedPlaceholder>,
    handoff: Option<handoff::ProposalHandoff>,
}

/// Borrowed output: exact addressed identities and selected before/after regions.
/// Full originals remain inside the checked proposal for validation, without making
/// an unrelated large file the default response to a one-region preview.
#[derive(Serialize, JsonSchema)]
#[schemars(rename = "ChangeProposal")]
struct ProposalView<'a> {
    #[schemars(extend("const" = PROPOSAL_VERSION))]
    version: u32,
    producer: &'a ProducerContext,
    sources: Vec<&'a SourceIdentity>,
    file_operations: &'a [FileOperation],
    candidates: &'a [EditCandidate],
    selected: &'a [usize],
    selected_edits: Vec<EditInspection<'a>>,
    conflicts: &'a [CandidateConflict],
    limitations: &'a [String],
    unresolved_placeholders: &'a [UnresolvedPlaceholder],
    #[serde(skip_serializing_if = "Option::is_none")]
    handoff: Option<&'a handoff::ProposalHandoff>,
}
const PROPOSAL_VERSION: u32 = 2;
impl Serialize for ChangeProposal {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ProposalView {
            version: PROPOSAL_VERSION,
            producer: &self.producer,
            sources: self.sources.iter().map(SourceText::identity).collect(),
            file_operations: &self.file_operations,
            candidates: &self.candidates,
            selected: &self.selected,
            selected_edits: (0..self.selected.len())
                .filter_map(|index| self.inspect(index))
                .collect(),
            conflicts: &self.conflicts,
            limitations: &self.limitations,
            unresolved_placeholders: &self.unresolved_placeholders,
            handoff: self.handoff.as_ref(),
        }
        .serialize(serializer)
    }
}
impl JsonSchema for ChangeProposal {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        ProposalView::schema_name()
    }
    fn schema_id() -> std::borrow::Cow<'static, str> {
        concat!(module_path!(), "::ChangeProposal").into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        ProposalView::json_schema(generator)
    }
}

/// Borrowed selected replacement inspection, retaining exact before/after text.
#[derive(Debug, Serialize, JsonSchema)]
pub struct EditInspection<'a> {
    /// Index in the complete candidate set.
    pub candidate: usize,
    /// Address of the original source.
    pub source: &'a SourceIdentity,
    /// Half-open UTF-8 original coordinates.
    pub range: Range<usize>,
    /// Exact replaced bytes, without whole-file syntax regeneration.
    pub original: &'a str,
    /// Exact replacement text, including unresolved snippets.
    pub replacement: &'a str,
}

fn overlaps(a: &Range<usize>, b: &Range<usize>) -> bool {
    if a.is_empty() && b.is_empty() {
        return a.start == b.start;
    }
    if a.is_empty() {
        return b.start <= a.start && a.start < b.end;
    }
    if b.is_empty() {
        return a.start <= b.start && b.start < a.end;
    }
    a.start < b.end && b.start < a.end
}

impl ChangeProposal {
    /// Check all addressed originals, candidate coordinates and the selected nonoverlapping set.
    ///
    /// Sources must be exactly the candidate-addressed inputs: unrelated sources are refused.
    /// Adjacent replacements are valid. Two insertions at one position, or an insertion at
    /// another edit's start/interior, conflict because no ordering has been selected.
    pub fn new(
        producer: ProducerContext,
        sources: Vec<SourceText>,
        candidates: Vec<EditCandidate>,
        selected: Vec<usize>,
        limitations: Vec<String>,
        unresolved_placeholders: Vec<UnresolvedPlaceholder>,
    ) -> Result<Self, ProposalError> {
        Self::with_file_operations(
            producer,
            sources,
            candidates,
            selected,
            vec![],
            limitations,
            unresolved_placeholders,
        )
    }
    /// Validate regular-file effects together with source-bound replacements once.
    /// No target absence, filesystem permission, application or index mutation is asserted.
    pub fn with_file_operations(
        producer: ProducerContext,
        sources: Vec<SourceText>,
        candidates: Vec<EditCandidate>,
        selected: Vec<usize>,
        file_operations: Vec<FileOperation>,
        limitations: Vec<String>,
        unresolved_placeholders: Vec<UnresolvedPlaceholder>,
    ) -> Result<Self, ProposalError> {
        let (backend, configuration, generation) = match &producer {
            ProducerContext::Syntax {
                backend,
                configuration,
            } => (backend, configuration, None),
            ProducerContext::Semantic {
                backend,
                configuration,
                synchronized_generation,
            } => (backend, configuration, Some(synchronized_generation)),
        };
        if backend.trim().is_empty()
            || configuration.trim().is_empty()
            || generation.is_some_and(|value| value.trim().is_empty())
        {
            return Err(ProposalError::MissingProducerContext);
        }
        let mut by_path = BTreeMap::new();
        for (index, source) in sources.iter().enumerate() {
            if by_path
                .insert(source.identity().path(), (index, Vec::new()))
                .is_some()
            {
                return Err(ProposalError::DuplicateSource(
                    source.identity().path().into(),
                ));
            }
        }
        for (index, candidate) in candidates.iter().enumerate() {
            let (source_index, indices) = by_path
                .get_mut(candidate.source.path())
                .ok_or_else(|| ProposalError::MissingSource(candidate.source.path().into()))?;
            let source = &sources[*source_index];
            if source.identity() != &candidate.source {
                return Err(ProposalError::StaleSource(candidate.source.path().into()));
            }
            if source.text().get(candidate.range.clone()).is_none() {
                return Err(ProposalError::InvalidRange(index));
            }
            indices.push(index);
        }
        let mut selected_set = std::collections::BTreeSet::new();
        for &index in &selected {
            if index >= candidates.len() {
                return Err(ProposalError::UnknownCandidate(index));
            }
            if !selected_set.insert(index) {
                return Err(ProposalError::DuplicateSelection(index));
            }
        }
        for placeholder in &unresolved_placeholders {
            if placeholder.candidate >= candidates.len() {
                return Err(ProposalError::UnknownCandidate(placeholder.candidate));
            }
        }
        let mut conflicts = Vec::new();
        let mut operated = std::collections::BTreeSet::new();
        let mut destinations = std::collections::BTreeSet::new();
        for operation in &file_operations {
            let (original, destination) = match operation {
                FileOperation::Create { path, .. } => (None, Some(Path::new(path))),
                FileOperation::Delete { source } => (Some(source), None),
                FileOperation::Rename {
                    source,
                    destination,
                } => (Some(source), Some(Path::new(destination))),
            };
            if let Some(source) = original {
                if !matches!(source.origin(), crate::source_read::SourceOrigin::Working) {
                    return Err(ProposalError::UnsupportedFileOperation);
                }
                let (index, edits) = by_path
                    .get(source.path())
                    .ok_or_else(|| ProposalError::MissingSource(source.path().into()))?;
                if sources[*index].identity() != source {
                    return Err(ProposalError::StaleSource(source.path().into()));
                }
                if !operated.insert(source.path())
                    || (matches!(operation, FileOperation::Delete { .. }) && !edits.is_empty())
                {
                    return Err(ProposalError::ConflictingFileOperation(
                        source.path().into(),
                    ));
                }
            }
            if let Some(path) = destination {
                if !path.is_absolute() {
                    return Err(ProposalError::RelativePath(path.into()));
                }
                if !destinations.insert(path) || by_path.contains_key(path) {
                    return Err(ProposalError::ConflictingFileOperation(path.into()));
                }
            }
        }
        for (path, (_, indices)) in &mut by_path {
            if indices.is_empty() && !operated.contains(path) {
                return Err(ProposalError::UnusedSource((*path).into()));
            }
            indices.sort_unstable_by_key(|&index| {
                (candidates[index].range.start, candidates[index].range.end)
            });
            for (offset, &first) in indices.iter().enumerate() {
                let range = &candidates[first].range;
                for &second in &indices[offset + 1..] {
                    let other = &candidates[second].range;
                    if other.start > range.end {
                        break;
                    }
                    if overlaps(range, other) {
                        if selected_set.contains(&first) && selected_set.contains(&second) {
                            return Err(ProposalError::SelectedOverlap { first, second });
                        }
                        conflicts.push(CandidateConflict { first, second });
                    }
                }
            }
        }
        Ok(Self {
            producer,
            sources,
            file_operations,
            candidates,
            selected,
            conflicts,
            limitations,
            unresolved_placeholders,
            handoff: None,
        })
    }

    /// Attach a digest-only projection of the selected original edits to a checkout receipt.
    /// The caller observes the checkout identity and no edit is applied here.
    pub(crate) fn bind_handoff(
        &mut self,
        root: &Path,
        checkout_id: String,
        checkout_revision: String,
    ) -> bool {
        self.handoff = self.reviewed_handoff(root, checkout_id, checkout_revision);
        self.handoff.is_some()
    }

    fn reviewed_handoff(
        &self,
        root: &Path,
        checkout_id: String,
        checkout_revision: String,
    ) -> Option<handoff::ProposalHandoff> {
        use crate::source_read::SourceOrigin;
        if !matches!(self.producer, ProducerContext::Semantic { .. })
            || !self.file_operations.is_empty()
        {
            return None;
        }
        let mut groups: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for &index in &self.selected {
            if self
                .unresolved_placeholders
                .iter()
                .any(|placeholder| placeholder.candidate == index)
            {
                return None;
            }
            let edit = self.candidates.get(index)?;
            groups.entry(edit.source().path()).or_default().push(edit);
            if groups.len() > handoff::MAX_FILES {
                return None;
            }
        }
        let mut files = Vec::with_capacity(groups.len());
        for (path, mut edits) in groups {
            let relative = path.strip_prefix(root).ok()?;
            let source = self
                .sources
                .iter()
                .find(|source| source.identity().path() == path)?;
            if !matches!(source.identity().origin(), SourceOrigin::Working) {
                return None;
            }
            edits.sort_unstable_by_key(|edit| (edit.range.start, edit.range.end));
            let text = source.text();
            let mut digest = Sha256::new();
            let mut cursor = 0;
            for edit in edits {
                digest.update(text.get(cursor..edit.range.start)?.as_bytes());
                digest.update(edit.replacement().as_bytes());
                cursor = edit.range.end;
            }
            digest.update(text.get(cursor..)?.as_bytes());
            files.push(handoff::ProposalFile {
                path: relative.to_owned(),
                before_sha256: source.identity().source_sha256().into(),
                after_sha256: digest
                    .finalize()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
            });
        }
        let handoff = handoff::ProposalHandoff {
            version: handoff::VERSION,
            checkout_id,
            checkout_revision,
            files,
        };
        handoff.validate().ok()?;
        (serde_json::to_vec(&handoff).ok()?.len() <= handoff::MAX_BYTES).then_some(handoff)
    }
    /// Producer authority and selected configuration, distinct from source validity.
    pub fn producer(&self) -> &ProducerContext {
        &self.producer
    }
    /// Only the immutable original inputs actually addressed by candidates.
    pub fn sources(&self) -> &[SourceText] {
        &self.sources
    }
    /// Explicit file effects, independent of matches and selected replacements.
    pub fn file_operations(&self) -> &[FileOperation] {
        &self.file_operations
    }
    /// Complete match/edit candidate collection, including overlapping alternatives.
    pub fn candidates(&self) -> &[EditCandidate] {
        &self.candidates
    }
    /// Explicit selected indices, without equating matches to applied writes.
    pub fn selected(&self) -> &[usize] {
        &self.selected
    }
    /// Conflicts among all candidates, including unselected alternatives.
    pub fn conflicts(&self) -> &[CandidateConflict] {
        &self.conflicts
    }
    /// Original producer omissions and unsupported scope.
    pub fn limitations(&self) -> &[String] {
        &self.limitations
    }
    /// Original unresolved editing intent.
    pub fn unresolved_placeholders(&self) -> &[UnresolvedPlaceholder] {
        &self.unresolved_placeholders
    }

    /// Compare only addressed inputs with freshly acquired bytes; unrelated sources are ignored.
    /// This is a stale-input check, never concurrency control or application permission.
    pub fn validate_sources(&self, current: &[SourceText]) -> Result<(), ProposalError> {
        for original in &self.sources {
            let mut same_path = current
                .iter()
                .filter(|source| source.identity().same_operand(original.identity()));
            let source = same_path
                .next()
                .ok_or_else(|| ProposalError::MissingSource(original.identity().path().into()))?;
            if same_path.next().is_some() {
                return Err(ProposalError::DuplicateSource(
                    original.identity().path().into(),
                ));
            }
            if source.identity() != original.identity() {
                return Err(ProposalError::StaleSource(
                    original.identity().path().into(),
                ));
            }
        }
        Ok(())
    }

    /// Inspect a selected replacement by selected-set position, borrowing retained originals.
    pub fn inspect(&self, selected_position: usize) -> Option<EditInspection<'_>> {
        let &candidate_index = self.selected.get(selected_position)?;
        let candidate = &self.candidates[candidate_index];
        let source = self
            .sources
            .iter()
            .find(|source| source.identity() == &candidate.source)?;
        Some(EditInspection {
            candidate: candidate_index,
            source: source.identity(),
            range: candidate.range.clone(),
            original: source.text().get(candidate.range.clone())?,
            replacement: &candidate.replacement,
        })
    }
}

/// Typed construction or stale-input refusal, before an output envelope is created.
#[derive(Debug, thiserror::Error)]
pub enum ProposalError {
    /// Grammar-only construction refused an unsupported node or malformed fragment.
    #[error("grammar preview: {0}")]
    Grammar(String),
    /// Only regular working files have supported create/delete/rename proposal semantics.
    #[error("file operations require addressed working files")]
    UnsupportedFileOperation,
    /// File effects target the same original/destination, or deleting an edited source.
    #[error("conflicting file operation at {0:?}")]
    ConflictingFileOperation(PathBuf),
    /// A working operand must bind an explicit absolute path.
    #[error("working source path must be absolute: {0:?}")]
    RelativePath(PathBuf),
    /// Same operand supplied twice, even if its text agrees.
    #[error("duplicate working source: {0:?}")]
    DuplicateSource(PathBuf),
    /// Candidate has no original evidence or current evidence is missing.
    #[error("missing addressed working source: {0:?}")]
    MissingSource(PathBuf),
    /// Supplied source is not addressed by any candidate.
    #[error("working source is not addressed by a candidate: {0:?}")]
    UnusedSource(PathBuf),
    /// Exact original bytes have changed.
    #[error("working source changed; reacquire the proposal: {0:?}")]
    StaleSource(PathBuf),
    /// Byte range is reversed, outside the original, or splits a UTF-8 character.
    #[error("candidate {0} has an invalid original UTF-8 byte range")]
    InvalidRange(usize),
    /// An index must address the complete candidate collection.
    #[error("unknown edit candidate {0}")]
    UnknownCandidate(usize),
    /// Selected set cannot contain the same edit twice.
    #[error("edit candidate {0} was selected more than once")]
    DuplicateSelection(usize),
    /// Selected replacements have no defined independent application.
    #[error("selected edit candidates {first} and {second} overlap")]
    SelectedOverlap {
        /// First candidate index.
        first: usize,
        /// Second candidate index.
        second: usize,
    },
    /// Authority without its actual backend/configuration/generation is incomplete.
    #[error(
        "proposal producer needs its backend, configuration and semantic generation when applicable"
    )]
    MissingProducerContext,
}
pse_diagnostics::impl_diagnostic! {
    ProposalError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

#[cfg(test)]
mod tests;
