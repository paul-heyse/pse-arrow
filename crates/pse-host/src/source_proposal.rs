// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable working-source replacement proposals, without filesystem application.
//!
//! Paths identify explicitly supplied operands, not canonical filesystem objects. Each source
//! retains its exact original text; a collection is not an atomic working-tree snapshot.

use crate::source_read::source_digest;
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    ffi::OsString,
    ops::Range,
    path::{Path, PathBuf},
};

/// Exact native working path and digest of the original UTF-8 bytes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, JsonSchema)]
pub struct SourceIdentity {
    path: OsString,
    source_sha256: String,
}
impl SourceIdentity {
    /// Original absolute operand, retaining non-UTF8 platform bytes.
    pub fn path(&self) -> &Path {
        Path::new(&self.path)
    }
    /// SHA256 of every original source byte.
    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }
}

/// One immutable original, supplied by the acquisition owner rather than reread here.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct WorkingSource {
    identity: SourceIdentity,
    text: String,
}
impl WorkingSource {
    /// Bind an explicit absolute path to exact text, without filesystem access or normalization.
    pub fn new(path: PathBuf, text: String) -> Result<Self, ProposalError> {
        if !path.is_absolute() {
            return Err(ProposalError::RelativePath(path));
        }
        Ok(Self {
            identity: SourceIdentity {
                path: path.into_os_string(),
                source_sha256: source_digest(&text),
            },
            text,
        })
    }
    /// Address of these original bytes.
    pub fn identity(&self) -> &SourceIdentity {
        &self.identity
    }
    /// Exact original UTF-8 text, including comments and malformed syntax.
    pub fn text(&self) -> &str {
        &self.text
    }
}

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

/// Checked replacement-only proposal; it grants no write or Git-publication authority.
///
/// Create/delete/rename operations are outside this slice and must be refused by producers,
/// never silently translated into text edits. All candidates and the selected subset differ.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ChangeProposal {
    producer: ProducerContext,
    sources: Vec<WorkingSource>,
    candidates: Vec<EditCandidate>,
    selected: Vec<usize>,
    conflicts: Vec<CandidateConflict>,
    limitations: Vec<String>,
    unresolved_placeholders: Vec<UnresolvedPlaceholder>,
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
        sources: Vec<WorkingSource>,
        candidates: Vec<EditCandidate>,
        selected: Vec<usize>,
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
                .insert(source.identity.path(), (index, Vec::new()))
                .is_some()
            {
                return Err(ProposalError::DuplicateSource(
                    source.identity.path().into(),
                ));
            }
        }
        for (index, candidate) in candidates.iter().enumerate() {
            let (source_index, indices) = by_path
                .get_mut(candidate.source.path())
                .ok_or_else(|| ProposalError::MissingSource(candidate.source.path().into()))?;
            let source = &sources[*source_index];
            if source.identity != candidate.source {
                return Err(ProposalError::StaleSource(candidate.source.path().into()));
            }
            if source.text.get(candidate.range.clone()).is_none() {
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
        for (path, (_, indices)) in &mut by_path {
            if indices.is_empty() {
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
            candidates,
            selected,
            conflicts,
            limitations,
            unresolved_placeholders,
        })
    }
    /// Producer authority and selected configuration, distinct from source validity.
    pub fn producer(&self) -> &ProducerContext {
        &self.producer
    }
    /// Only the immutable original inputs actually addressed by candidates.
    pub fn sources(&self) -> &[WorkingSource] {
        &self.sources
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
    pub fn validate_sources(&self, current: &[WorkingSource]) -> Result<(), ProposalError> {
        for original in &self.sources {
            let mut same_path = current
                .iter()
                .filter(|source| source.identity.path == original.identity.path);
            let source = same_path
                .next()
                .ok_or_else(|| ProposalError::MissingSource(original.identity.path().into()))?;
            if same_path.next().is_some() {
                return Err(ProposalError::DuplicateSource(
                    original.identity.path().into(),
                ));
            }
            if source.identity != original.identity {
                return Err(ProposalError::StaleSource(original.identity.path().into()));
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
            .find(|source| source.identity == candidate.source)?;
        Some(EditInspection {
            candidate: candidate_index,
            source: &source.identity,
            range: candidate.range.clone(),
            original: source.text.get(candidate.range.clone())?,
            replacement: &candidate.replacement,
        })
    }
}

/// Typed construction or stale-input refusal, before an output envelope is created.
#[derive(Debug, thiserror::Error)]
pub enum ProposalError {
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
