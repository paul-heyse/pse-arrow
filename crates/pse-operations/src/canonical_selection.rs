// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Protected selected reads and semantic premises for durable compilation products.

use crate::{
    canonical::{CanonicalError, CanonicalStore, PROTECTED_BEGIN, ProtectedSelection},
    generated::surreal as wire,
};
pub use pse_model::generated::enums::CanonicalProducerClass;
use pse_model::generated::runtime::{
    canonical_memberships::Row as Membership, canonical_products::Row as Product,
    canonical_versions::Row as ObjectVersion,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use surrealdb::types::Object;

/// Continuation bound to the original issuing authority and immutable source.
/// It conveys no payload access or receiving compatibility.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProductHeaderCursor {
    issuing_store: String,
    problem: String,
    revision: String,
    after: String,
}
/// Compact original publication provenance. Scientific payloads are never read.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProductHeader {
    /// Original association and request lookup identity.
    pub key: String,
    /// Exact original canonical source, not receiving compatibility.
    pub revision: String,
    /// Original producer namespace; no runtime qualification is implied.
    pub producer: String,
    /// Source-owned producer capability class; receipt text confers no class.
    pub producer_class: CanonicalProducerClass,
    /// Native scientific interpretation recorded at publication.
    pub interpretation: String,
    /// Original content and dependency descriptor reference.
    pub descriptor: String,
    /// Original sealed payload identity; no bytes are hydrated.
    pub content: String,
    /// Original sealed receipt identity; no bytes are hydrated.
    pub receipt: String,
}
/// One bounded observational page of currently rooted original publications.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProductHeaderPage {
    /// Present original publications. Presence never establishes receiving eligibility.
    pub rows: Vec<ProductHeader>,
    /// Continue under this same exact authority/source selection.
    pub next: Option<ProductHeaderCursor>,
}

/// Reserved writer role for compiler-issued descriptions eligible for scientific replay.
pub const SCIENTIFIC_PRODUCER_PREFIX: &str = "pse.qualified-math-producer.v1:";

/// Reserved independently observed exact deployment-local replay admission.
pub const LOCAL_RUNTIME_PRODUCER_PREFIX: &str = "pse.local-runtime.v2:";
// Emit a scoped equality only when selected. Keeping an optional OR in SQL
// hides the equality from the native composite (problem, scope, key) access path.
fn membership_scope_predicate(scoped: bool, parameter: &str) -> String {
    if scoped {
        format!(" AND scope = ${parameter}")
    } else {
        String::new()
    }
}

// The writer increments this same head guard when publishing membership changes.
// Register it before observing the head so either commit order conflicts/retries
// under protected_query's original clock, or reads the exact historical interval.
const MEMBERSHIP_HEAD_BEGIN: &str = r#"
SELECT * FROM type::record('canonical_guards', 'head:' + $problem) FOR UPDATE;
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $selected_head = SELECT * FROM ONLY type::record('canonical_problems', $problem);"#;

fn membership_page_query(scoped: bool, predicates: &str, limit: usize) -> String {
    let historical = format!(
        "SELECT * FROM canonical_memberships WHERE {predicates} AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) ORDER BY key LIMIT {limit} TIMEOUT $pse_rpc_timeout"
    );
    if scoped {
        // Keep the equality literal in the emitted query: an OR/optional predicate
        // hides the (problem, scope, to_sequence, key) equality prefix from planning.
        format!(
            "IF $selected_head.head = $revision AND $selected_head.sequence = $sequence {{ SELECT * FROM canonical_memberships WHERE {predicates} AND to_sequence = NONE ORDER BY key LIMIT {limit} TIMEOUT $pse_rpc_timeout }} ELSE {{ {historical} }}"
        )
    } else {
        historical
    }
}

fn membership_page_sql(scoped: bool) -> String {
    let scope = membership_scope_predicate(scoped, "scope");
    let head = if scoped { MEMBERSHIP_HEAD_BEGIN } else { "" };
    let page = membership_page_query(
        scoped,
        &format!("problem = $problem{scope} AND key > $after"),
        64,
    );
    format!(
        "{PROTECTED_BEGIN}{head}\nLET $rpc_result = {page};\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"
    )
}

#[cfg(all(test, feature = "canonical-tests"))]
pub(crate) fn test_membership_page_sql() -> String {
    membership_page_sql(true)
}

// Materialize the target suppliers once per protected query, rather than once
// per candidate membership. IN preserves existence semantics for repeated edges;
// the current membership, kind, scope and page predicates still select authority.
fn membership_reference_predicate(targeted: bool, suffix: &str) -> (String, String) {
    if targeted {
        (
            format!(
                "\nLET $suppliers{suffix} = SELECT VALUE source_version FROM canonical_edges WHERE target_scope = $target_scope{suffix} AND target_name = $target_name{suffix} TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);"
            ),
            format!(" AND version IN $suppliers{suffix}"),
        )
    } else {
        (String::new(), String::new())
    }
}

/// Maximum exact inventories sharing one protected 64-membership response.
pub const SELECTED_INVENTORY_BATCH: usize = 8;

/// Private-cursor complete membership read, tied to one selected-read pin. A
/// cancelled/dropped cursor leaves its read ineligible for publication/reuse.
#[derive(Debug)]
pub struct SelectedMemberships {
    key: String,
    selection: ProtectedSelection,
    scope: Option<String>,
    source_kind: Option<String>,
    target: Option<(String, String)>,
    after: String,
    pages: Vec<Vec<(String, String)>>,
    member_count: usize,
    bytes: usize,
}

/// A semantic premise actually inspected while resolving an immutable selection.
/// Conflict generations are deliberately separate from these immutable meanings.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Premise {
    Name {
        scope: String,
        name: String,
        version: Option<String>,
    },
    Scope {
        scope: String,
        members: Vec<(String, String)>,
    },
    Logical {
        logical: String,
        version: Option<String>,
    },
    KindScope {
        scope: Option<String>,
        source_kind: String,
        members: Vec<(String, String)>,
    },
    References {
        scope: String,
        name: String,
        source_kind: String,
        members: Vec<(String, String)>,
    },
    Interpretation {
        role: String,
        identity: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Dependencies {
    interpretation: String,
    premises: Vec<Premise>,
}

/// Immutable complete selection premises. This carries no protection or write authority;
/// every reuse rechecks its meaning under a newly acquired protected revision.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SelectedDependencies {
    dependencies: Dependencies,
    retained_bytes: usize,
}
impl SelectedDependencies {
    /// Conservative owned premise extent; callers reserve this before snapshotting.
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
}

/// Maximum retained admissions inspected together; history remains unbounded.
pub const SELECTED_RECHECK_WINDOW: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RecheckKey {
    Logical(String),
    Name(String, String),
}
impl Premise {
    fn recheck_key(&self, names: bool) -> Option<(RecheckKey, &Option<String>)> {
        match self {
            Self::Logical { logical, version } if !names => {
                Some((RecheckKey::Logical(logical.clone()), version))
            }
            Self::Name {
                scope,
                name,
                version,
            } if names => Some((RecheckKey::Name(scope.clone(), name.clone()), version)),
            _ => None,
        }
    }
}

/// Transient observations for at most eight candidates under one exact protected
/// immutable epoch. These observations never become consumer premises themselves.
/// The owner must reserve `scratch_bytes` before constructing this value.
#[derive(Debug)]
pub struct SelectedRecheckWindow {
    protection: String,
    revision: crate::canonical::Revision,
    candidates: Vec<std::sync::Arc<SelectedDependencies>>,
    actual: BTreeMap<RecheckKey, Option<String>>,
    inventories: BTreeMap<Premise, bool>,
    qualified: Option<usize>,
    #[cfg(test)]
    counts: RecheckCounts,
}
#[cfg(test)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct RecheckCounts {
    guards: usize,
    logicals: usize,
    names: usize,
    inventory_pages: usize,
}
impl SelectedRecheckWindow {
    /// Conservative transient extent, including adverse present versions of keys
    /// whose old premises recorded absence. No actual version exceeds 4096 bytes.
    pub fn scratch_bytes(
        read: &SelectedRead,
        candidates: &[std::sync::Arc<SelectedDependencies>],
    ) -> Result<usize, CanonicalError> {
        if candidates.is_empty() || candidates.len() > SELECTED_RECHECK_WINDOW {
            return Err(CanonicalError::PayloadLimit);
        }
        let keys = candidates.iter().try_fold(0usize, |count, candidate| {
            count
                .checked_add(
                    candidate
                        .dependencies
                        .premises
                        .iter()
                        .filter(|p| matches!(p, Premise::Name { .. } | Premise::Logical { .. }))
                        .count(),
                )
                .ok_or(CanonicalError::PayloadLimit)
        })?;
        let inventory = candidates.iter().any(|candidate| {
            candidate.dependencies.premises.iter().any(|p| {
                matches!(
                    p,
                    Premise::Scope { .. } | Premise::KindScope { .. } | Premise::References { .. }
                )
            })
        });
        // A previously empty namespace can return a complete adverse 64-row
        // page. Logical/name replies also include unrelated membership identity
        // fields, each bounded by the same 4096-byte identity contract. Reserve
        // one response/decode workspace, rather than multiplying it by history.
        let response_rows = keys.min(256).max(if inventory { 64 } else { 0 });
        let response = response_rows
            .checked_mul(
                crate::canonical_staging::SELECTED_OBJECT_HEADER_SCRATCH
                    / crate::canonical_staging::SELECTED_OBJECT_BATCH,
            )
            .ok_or(CanonicalError::PayloadLimit)?;
        let epoch = pse_model::HeapUsage::heap_bytes(read.selection.revision())
            .checked_mul(4)
            .and_then(|n| n.checked_add(read.selection.key().len() + 8192))
            .and_then(|n| n.checked_add(response))
            .ok_or(CanonicalError::PayloadLimit)?;
        candidates.iter().try_fold(epoch, |bytes, candidate| {
            let keys = candidate
                .dependencies
                .premises
                .iter()
                .filter(|p| matches!(p, Premise::Name { .. } | Premise::Logical { .. }))
                .count();
            bytes
                .checked_add(
                    candidate
                        .retained_bytes()
                        .checked_mul(4)
                        .ok_or(CanonicalError::PayloadLimit)?,
                )
                .and_then(|n| n.checked_add(keys.checked_mul(4096 * 8 + 1024)?))
                .ok_or(CanonicalError::PayloadLimit)
        })
    }
    /// Construct only after the issued scratch extent has been admitted.
    pub fn new(
        read: &SelectedRead,
        candidates: &[std::sync::Arc<SelectedDependencies>],
    ) -> Result<Self, CanonicalError> {
        Self::scratch_bytes(read, candidates)?;
        Ok(Self {
            protection: read.selection.key().into(),
            revision: read.selection.revision().clone(),
            candidates: candidates.to_vec(),
            actual: BTreeMap::new(),
            inventories: BTreeMap::new(),
            qualified: None,
            #[cfg(test)]
            counts: RecheckCounts::default(),
        })
    }
    /// Observation sharing requires both the exact pin/actual row and the same
    /// ordered candidate population, including complete namespace expectations.
    pub fn matches(
        &self,
        read: &SelectedRead,
        candidates: &[std::sync::Arc<SelectedDependencies>],
    ) -> bool {
        self.protection == read.selection.key()
            && &self.revision == read.selection.revision()
            && self.candidates == candidates
    }
    fn check_epoch(&self, read: &SelectedRead) -> Result<(), CanonicalError> {
        read.complete()?;
        if self.protection != read.selection.key() || &self.revision != read.selection.revision() {
            return Err(CanonicalError::Configuration(
                "candidate observations belong to another protected epoch".into(),
            ));
        }
        Ok(())
    }
    /// Merge only the complete candidate qualified by the most recent guarded
    /// search. Callers reserve its retained extent before this allocation.
    pub fn merge_qualified(
        &self,
        read: &mut SelectedRead,
        index: usize,
    ) -> Result<(), CanonicalError> {
        self.check_epoch(read)?;
        if self.qualified != Some(index) {
            return Err(CanonicalError::Configuration(
                "candidate was not completely qualified".into(),
            ));
        }
        let mut delta = SelectedRead::new(read.selection.clone());
        for premise in &self.candidates[index].dependencies.premises {
            match premise.clone() {
                Premise::Name {
                    scope,
                    name,
                    version,
                } => {
                    delta.names.insert((scope, name), version);
                }
                Premise::Logical { logical, version } => {
                    delta.logicals.insert(logical, version);
                }
                Premise::Scope { scope, members } => {
                    delta.scopes.insert(scope, members);
                }
                Premise::KindScope {
                    scope,
                    source_kind,
                    members,
                } => {
                    delta.kinds.insert((scope, source_kind), members);
                }
                Premise::References {
                    scope,
                    name,
                    source_kind,
                    members,
                } => {
                    delta.references.insert((scope, name, source_kind), members);
                }
                Premise::Interpretation { role, identity } => {
                    delta.interpretations.insert(role, identity);
                }
            }
        }
        read.merge_delta(delta)
    }
}

/// Operations-issued immutable premise receipt for one exact actual canonical row.
/// It retains no pin or publication authority; adoption checks a new live pin.
#[derive(Clone, Debug)]
pub struct SelectionDependencyReceipt {
    revision: crate::canonical::Revision,
    dependencies: std::sync::Arc<SelectedDependencies>,
    retained_bytes: usize,
}
impl SelectionDependencyReceipt {
    /// The owned premise snapshot, shared with the consuming admission.
    pub fn dependencies(&self) -> &std::sync::Arc<SelectedDependencies> {
        &self.dependencies
    }
    /// Complete owned receipt extent, including its snapshot and actual revision.
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
}

// Borrowing projection serializes the exact dependency frame without cloning its
// owned premise inventories merely to establish the bounded encoded extent.
struct BorrowedDependencies<'a>(&'a SelectedRead);
struct BorrowedPremises<'a>(&'a SelectedRead);
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum BorrowedPremise<'a> {
    Name {
        scope: &'a str,
        name: &'a str,
        version: &'a Option<String>,
    },
    Scope {
        scope: &'a str,
        members: &'a [(String, String)],
    },
    Interpretation {
        role: &'a str,
        identity: &'a str,
    },
    Logical {
        logical: &'a str,
        version: &'a Option<String>,
    },
    KindScope {
        scope: &'a Option<String>,
        source_kind: &'a str,
        members: &'a [(String, String)],
    },
    References {
        scope: &'a str,
        name: &'a str,
        source_kind: &'a str,
        members: &'a [(String, String)],
    },
}
impl Serialize for BorrowedDependencies<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut frame = serializer.serialize_struct("Dependencies", 2)?;
        frame.serialize_field("interpretation", wire::INTERPRETATION)?;
        frame.serialize_field("premises", &BorrowedPremises(self.0))?;
        frame.end()
    }
}
impl Serialize for BorrowedPremises<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let read = self.0;
        let mut seq = serializer.serialize_seq(None)?;
        for ((scope, name), version) in &read.names {
            seq.serialize_element(&BorrowedPremise::Name {
                scope,
                name,
                version,
            })?;
        }
        for (scope, members) in &read.scopes {
            seq.serialize_element(&BorrowedPremise::Scope { scope, members })?;
        }
        for (role, identity) in &read.interpretations {
            seq.serialize_element(&BorrowedPremise::Interpretation { role, identity })?;
        }
        for (logical, version) in &read.logicals {
            seq.serialize_element(&BorrowedPremise::Logical { logical, version })?;
        }
        for ((scope, source_kind), members) in &read.kinds {
            seq.serialize_element(&BorrowedPremise::KindScope {
                scope,
                source_kind,
                members,
            })?;
        }
        for ((scope, name, source_kind), members) in &read.references {
            seq.serialize_element(&BorrowedPremise::References {
                scope,
                name,
                source_kind,
                members,
            })?;
        }
        seq.end()
    }
}
struct DependencyCounter(usize);
impl std::io::Write for DependencyCounter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|n| *n <= 32 * 1024 * 1024)
            .ok_or_else(|| std::io::Error::other("dependency frame exceeds finite limit"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Logical and immutable version identities selected for each scoped source kind.
type KindSelections = BTreeMap<(Option<String>, String), Vec<(String, String)>>;

// A clone preserves its exact snapshot token; the first mutation on either branch
// gets a globally distinct token. Exhaustion refuses capture rather than wrapping.
fn dependency_epoch() -> Option<u64> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.try_update(
        std::sync::atomic::Ordering::Relaxed,
        std::sync::atomic::Ordering::Relaxed,
        |value| value.checked_add(1),
    )
    .ok()
}

/// A compiler's protected read session. Recorded premises include absent names and
/// explicitly enumerated scopes, rather than only positive object references.
#[derive(Clone, Debug)]
pub struct SelectedRead {
    selection: ProtectedSelection,
    dependency_epoch: Option<u64>,
    names: BTreeMap<(String, String), Option<String>>,
    scopes: BTreeMap<String, Vec<(String, String)>>,
    interpretations: BTreeMap<String, String>,
    logicals: BTreeMap<String, Option<String>>,
    kinds: KindSelections,
    references: BTreeMap<(String, String, String), Vec<(String, String)>>,
    pending_inventories: std::collections::BTreeSet<String>,
}
impl SelectedRead {
    /// Start from the store-issued live protection acquired before scientific work.
    pub fn new(selection: ProtectedSelection) -> Self {
        Self {
            selection,
            dependency_epoch: dependency_epoch(),
            names: BTreeMap::new(),
            scopes: BTreeMap::new(),
            interpretations: BTreeMap::new(),
            logicals: BTreeMap::new(),
            kinds: BTreeMap::new(),
            references: BTreeMap::new(),
            pending_inventories: Default::default(),
        }
    }
    /// Exact canonical revision; it is not a source-content hash.
    pub fn selection(&self) -> &ProtectedSelection {
        &self.selection
    }
    /// Finish this reader without releasing protection still owned by another follower.
    pub async fn finish(self) -> Result<(), CanonicalError> {
        self.selection.finish().await
    }
    /// Record a consumed compiler/provider/configuration interpretation.
    pub fn interpretation(&mut self, role: String, identity: String) -> Result<(), CanonicalError> {
        if let Some(previous) = self.interpretations.get(&role)
            && previous != &identity
        {
            return Err(CanonicalError::Configuration(
                "one role consumed incompatible interpretations".into(),
            ));
        }
        self.invalidate_material();
        self.interpretations.insert(role, identity);
        Ok(())
    }
    fn merge_delta(&mut self, delta: Self) -> Result<(), CanonicalError> {
        self.complete()?;
        delta.complete()?;
        fn compatible<K: Ord, V: PartialEq>(
            original: &BTreeMap<K, V>,
            delta: &BTreeMap<K, V>,
        ) -> bool {
            delta
                .iter()
                .all(|(key, value)| original.get(key).is_none_or(|previous| previous == value))
        }
        if !compatible(&self.names, &delta.names)
            || !compatible(&self.scopes, &delta.scopes)
            || !compatible(&self.interpretations, &delta.interpretations)
            || !compatible(&self.logicals, &delta.logicals)
            || !compatible(&self.kinds, &delta.kinds)
            || !compatible(&self.references, &delta.references)
        {
            return Err(CanonicalError::Configuration(
                "qualified premises contradict already consumed meaning".into(),
            ));
        }
        fn insert_delta<K: Ord, V>(original: &mut BTreeMap<K, V>, delta: BTreeMap<K, V>) {
            // Entry insertion preserves existing nodes/values; bulk append may
            // rebuild the entire baseline tree even for one candidate premise.
            for (key, value) in delta {
                original.entry(key).or_insert(value);
            }
        }
        self.invalidate_material();
        insert_delta(&mut self.names, delta.names);
        insert_delta(&mut self.scopes, delta.scopes);
        insert_delta(&mut self.interpretations, delta.interpretations);
        insert_delta(&mut self.logicals, delta.logicals);
        insert_delta(&mut self.kinds, delta.kinds);
        insert_delta(&mut self.references, delta.references);
        Ok(())
    }
    fn invalidate_material(&mut self) {
        self.dependency_epoch = dependency_epoch();
    }
    /// Exact encoded dependency extent, counted before cloning or allocating JSON.
    pub fn dependency_bytes(&self) -> Result<usize, CanonicalError> {
        self.complete()?;
        let mut counter = DependencyCounter(0);
        serde_json::to_writer(&mut counter, &BorrowedDependencies(self))
            .map_err(|_| CanonicalError::PayloadLimit)?;
        Ok(counter.0)
    }
    /// Allocation-free bound for dependency cloning, escaping, JSON and blob copies.
    /// The caller additionally reserves its portable payload's assembly copies.
    pub fn publication_scratch_bytes(&self) -> Result<usize, CanonicalError> {
        self.complete()?;
        let mut bytes = 4096usize;
        let mut add = |length: usize| -> Result<(), CanonicalError> {
            bytes = bytes
                .checked_add(length.checked_mul(32).ok_or(CanonicalError::PayloadLimit)?)
                .and_then(|n| n.checked_add(1024))
                .ok_or(CanonicalError::PayloadLimit)?;
            Ok(())
        };
        for ((scope, name), version) in &self.names {
            add(scope.len() + name.len() + version.as_ref().map_or(0, String::len))?;
        }
        for (scope, members) in &self.scopes {
            add(scope.len())?;
            for (name, version) in members {
                add(name.len() + version.len())?;
            }
        }
        for (role, identity) in &self.interpretations {
            add(role.len() + identity.len())?;
        }
        for (logical, version) in &self.logicals {
            add(logical.len() + version.as_ref().map_or(0, String::len))?;
        }
        for ((scope, kind), members) in &self.kinds {
            add(scope.as_ref().map_or(0, String::len) + kind.len())?;
            for (logical, version) in members {
                add(logical.len() + version.len())?;
            }
        }
        for ((scope, name, kind), members) in &self.references {
            add(scope.len() + name.len() + kind.len())?;
            for (logical, version) in members {
                add(logical.len() + version.len())?;
            }
        }
        Ok(bytes)
    }
    fn complete(&self) -> Result<(), CanonicalError> {
        if self.pending_inventories.is_empty() {
            Ok(())
        } else {
            Err(CanonicalError::Configuration(
                "selected namespace inventory is incomplete".into(),
            ))
        }
    }
    /// Snapshot completed premises after reserving `publication_scratch_bytes()`.
    /// The snapshot never extends the lifetime of this read's protection.
    pub fn snapshot_dependencies(&self) -> Result<SelectedDependencies, CanonicalError> {
        self.complete()?;
        Ok(SelectedDependencies {
            retained_bytes: self.publication_scratch_bytes()?,
            dependencies: self.dependencies(),
        })
    }
    /// Reserve this extent before snapshotting the actual-row premise receipt.
    pub fn snapshot_receipt_scratch_bytes(&self) -> Result<usize, CanonicalError> {
        let revision = self.selection.revision();
        let mut bytes = self.publication_scratch_bytes()?;
        for extent in [
            size_of::<SelectionDependencyReceipt>() + 128,
            revision.key.len(),
            revision.problem.len(),
            revision.parent.as_ref().map_or(0, String::len),
            revision.operation.len(),
            revision.request.len(),
            revision.interpretation.len(),
        ] {
            bytes = bytes
                .checked_add(extent)
                .ok_or(CanonicalError::PayloadLimit)?;
        }
        Ok(bytes)
    }
    /// Capture only completed qualified premises and their store-issued actual row.
    /// The caller reserves `snapshot_receipt_scratch_bytes` before this allocation.
    pub fn snapshot_receipt(&self) -> Result<SelectionDependencyReceipt, CanonicalError> {
        let retained_bytes = self.snapshot_receipt_scratch_bytes()?;
        Ok(SelectionDependencyReceipt {
            revision: self.selection.revision().clone(),
            dependencies: std::sync::Arc::new(self.snapshot_dependencies()?),
            retained_bytes,
        })
    }
    fn dependencies(&self) -> Dependencies {
        let premises =
            self.names
                .iter()
                .map(|((scope, name), version)| Premise::Name {
                    scope: scope.clone(),
                    name: name.clone(),
                    version: version.clone(),
                })
                .chain(self.scopes.iter().map(|(scope, members)| Premise::Scope {
                    scope: scope.clone(),
                    members: members.clone(),
                }))
                .chain(self.interpretations.iter().map(|(role, identity)| {
                    Premise::Interpretation {
                        role: role.clone(),
                        identity: identity.clone(),
                    }
                }))
                .collect();
        let mut premises: Vec<Premise> = premises;
        premises.extend(
            self.logicals
                .iter()
                .map(|(logical, version)| Premise::Logical {
                    logical: logical.clone(),
                    version: version.clone(),
                }),
        );
        premises.extend(self.kinds.iter().map(|((scope, source_kind), members)| {
            Premise::KindScope {
                scope: scope.clone(),
                source_kind: source_kind.clone(),
                members: members.clone(),
            }
        }));
        premises.extend(
            self.references
                .iter()
                .map(
                    |((scope, name, source_kind), members)| Premise::References {
                        scope: scope.clone(),
                        name: name.clone(),
                        source_kind: source_kind.clone(),
                        members: members.clone(),
                    },
                ),
        );
        Dependencies {
            interpretation: wire::INTERPRETATION.into(),
            premises,
        }
    }
}

/// One complete selected premise snapshot and its exact existing wire encoding.
/// The caller reserves `allocation_bytes` before capture; the retained opaque owner
/// keeps that actual allocation charged without introducing a columnar dependency.
/// This receipt carries no store protection or permission to publish a new root.
#[derive(Debug)]
pub struct SelectionMaterial {
    epoch: u64,
    protection: String,
    problem: String,
    revision: String,
    dependencies: SelectedDependencies,
    encoded: Vec<u8>,
    receipt: crate::canonical_staging::ReceiptBlob,
    retained_bytes: usize,
    _owner: std::sync::Arc<dyn std::fmt::Debug + Send + Sync>,
}
impl SelectionMaterial {
    /// Bound owned premises, encoding and receipt metadata before allocation.
    pub fn allocation_bytes(read: &SelectedRead) -> Result<usize, CanonicalError> {
        read.publication_scratch_bytes()?
            .checked_add(read.dependency_bytes()?)
            .and_then(|n| n.checked_add(read.selection.key().len()))
            .and_then(|n| n.checked_add(read.selection.revision().problem.len()))
            .and_then(|n| n.checked_add(read.selection.revision().key.len()))
            .and_then(|n| n.checked_add(size_of::<Self>() + 256))
            .ok_or(CanonicalError::PayloadLimit)
    }
    /// Capture after every acquired predicate has merged into this exact read.
    pub fn capture(
        read: &SelectedRead,
        owner: std::sync::Arc<dyn std::fmt::Debug + Send + Sync>,
    ) -> Result<std::sync::Arc<Self>, CanonicalError> {
        let epoch = read.dependency_epoch.ok_or(CanonicalError::PayloadLimit)?;
        let retained_bytes = Self::allocation_bytes(read)?;
        let mut encoded = Vec::with_capacity(read.dependency_bytes()?);
        serde_json::to_writer(&mut encoded, &BorrowedDependencies(read))
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        let receipt = crate::canonical_staging::ReceiptBlob::capture(&encoded)?;
        Ok(std::sync::Arc::new(Self {
            epoch,
            protection: read.selection.key().to_owned(),
            problem: read.selection.revision().problem.clone(),
            revision: read.selection.revision().key.clone(),
            dependencies: read.snapshot_dependencies()?,
            encoded,
            receipt,
            retained_bytes,
            _owner: owner,
        }))
    }
    /// Shared allocation extent, counted once by an inventory retaining this Arc.
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
    /// Exact complete premises, without protection or permission.
    pub fn selected_dependencies(&self) -> &SelectedDependencies {
        &self.dependencies
    }
    fn matches_selection(&self, read: &SelectedRead) -> Result<bool, CanonicalError> {
        read.complete()?;
        Ok(read.dependency_epoch == Some(self.epoch)
            && read.selection.key() == self.protection
            && read.selection.revision().key == self.revision
            && read.selection.revision().problem == self.problem)
    }
    /// Compare two complete snapshots once before rebinding a body inventory.
    pub fn rebinding_from(
        self: &std::sync::Arc<Self>,
        previous: &std::sync::Arc<Self>,
        read: &SelectedRead,
    ) -> Result<Option<SelectionRebinding>, CanonicalError> {
        if !self.matches_selection(read)? {
            return Err(CanonicalError::Configuration(
                "selection material is stale".into(),
            ));
        }
        Ok(
            (self.problem == previous.problem && self.encoded == previous.encoded).then(|| {
                SelectionRebinding {
                    previous: previous.clone(),
                    current: self.clone(),
                }
            }),
        )
    }
}

/// Once-checked equality between two complete premise snapshots. Individual body
/// bindings check only their source Arc and the current read's mutation identity.
#[derive(Debug)]
pub struct SelectionRebinding {
    previous: std::sync::Arc<SelectionMaterial>,
    current: std::sync::Arc<SelectionMaterial>,
}

/// Owned exact encoding, independent of write permission.
/// Clones share immutable material. Every consumer must recheck `selected_dependencies`
/// under its current protected read; this value is never replay or publication authority.
#[derive(Clone, Debug)]
pub struct ProductDescription {
    material: std::sync::Arc<DescriptionMaterial>,
    selection: std::sync::Arc<SelectionMaterial>,
}
#[derive(Debug)]
struct DescriptionMaterial {
    proposed_key: String,
    product: Product,
    blob: crate::canonical_staging::ProductBlob,
    payload: std::sync::Arc<Vec<u8>>,
    _payload_owner: std::sync::Arc<dyn std::fmt::Debug + Send + Sync>,
    retained_bytes: usize,
}
impl ProductDescription {
    /// Encode a complete selected body once without accessing or admitting to a store.
    pub fn prepare(read: &SelectedRead, product: Product) -> Result<Self, CanonicalError> {
        let selection = SelectionMaterial::capture(read, std::sync::Arc::new(()))?;
        Self::prepare_with_selection(read, product, selection)
    }
    /// Encode one body using the once-captured complete selected material.
    pub fn prepare_with_selection(
        read: &SelectedRead,
        mut product: Product,
        selection: std::sync::Arc<SelectionMaterial>,
    ) -> Result<Self, CanonicalError> {
        let payload = std::sync::Arc::new(std::mem::take(&mut product.payload).into_vec());
        Self::prepare_with_shared_payload(
            read,
            product,
            selection,
            payload,
            std::sync::Arc::new(()),
        )
    }
    /// Borrow an immutable, already charged scientific encoding. The retained owner
    /// is the actual allocation owner, not a caller's numeric accounting assertion.
    pub fn prepare_with_shared_payload(
        read: &SelectedRead,
        mut product: Product,
        selection: std::sync::Arc<SelectionMaterial>,
        payload: std::sync::Arc<Vec<u8>>,
        payload_owner: std::sync::Arc<dyn std::fmt::Debug + Send + Sync>,
    ) -> Result<Self, CanonicalError> {
        if !product.payload.is_empty() {
            return Err(CanonicalError::Configuration(
                "shared body header must not duplicate payload".into(),
            ));
        }
        if !selection.matches_selection(read)? {
            return Err(CanonicalError::Configuration(
                "selection material is stale".into(),
            ));
        }
        if [
            &product.key,
            &product.problem,
            &product.revision,
            &product.producer,
            &product.interpretation,
        ]
        .iter()
        .any(|value| value.len() > crate::canonical_staging::IDENTITY_BYTES)
        {
            return Err(CanonicalError::PayloadLimit);
        }
        if product.problem != read.selection().revision().problem
            || product.revision != read.selection().revision().key
            || product.interpretation != wire::INTERPRETATION
        {
            return Err(CanonicalError::Configuration(
                "product selection/interpretation mismatch".into(),
            ));
        }
        if payload.len() > 65 * 1024 * 1024 + 4096
            || product.request.len() > crate::canonical::PAYLOAD_BYTES
        {
            return Err(CanonicalError::PayloadLimit);
        }
        let blob = crate::canonical_staging::ProductBlob::with_receipt(
            payload.as_slice(),
            &product.producer,
            selection.receipt.clone(),
        )?;
        product.payload = serde_json::to_vec(&blob)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?
            .into();
        product.dependencies = Vec::new().into();
        if product.request.len().saturating_add(product.payload.len())
            > crate::canonical::PAYLOAD_BYTES
        {
            return Err(CanonicalError::PayloadLimit);
        }
        let retained_bytes = payload
            .capacity()
            .checked_add(product.request.len())
            .and_then(|n| n.checked_add(product.payload.len()))
            .and_then(|n| n.checked_add(8 * crate::canonical_staging::IDENTITY_BYTES))
            .ok_or(CanonicalError::PayloadLimit)?;
        let proposed_key = std::mem::take(&mut product.key);
        product.revision = String::new();
        Ok(Self {
            material: std::sync::Arc::new(DescriptionMaterial {
                proposed_key,
                product,
                blob,
                payload,
                _payload_owner: payload_owner,
                retained_bytes,
            }),
            selection,
        })
    }
    /// Exact declared class of this original producer binding.
    pub fn producer_class(&self) -> CanonicalProducerClass { self.material.product.producer_class }
    /// Immutable complete premises, without protection or permission.
    pub fn selected_dependencies(&self) -> &SelectedDependencies {
        self.selection.selected_dependencies()
    }
    /// Conservative owned encoded material and complete premise allocation extent.
    pub fn retained_bytes(&self) -> usize {
        self.material.retained_bytes + self.selection.retained_bytes()
    }
    /// Per-body extent; the shared selection material is counted by its inventory.
    pub fn exclusive_retained_bytes(&self) -> usize {
        self.material.retained_bytes
    }
    /// Additional owned descriptor/header bytes when the body owner is retained separately.
    pub fn descriptor_allocation_bytes(&self) -> usize {
        self.material
            .retained_bytes
            .saturating_sub(self.material.payload.capacity())
    }
    /// Once-captured shared selected material retained through uncertain settlement.
    pub fn selection_material(&self) -> &std::sync::Arc<SelectionMaterial> {
        &self.selection
    }
    /// Bind unchanged immutable body/blob bytes to an already checked equivalent
    /// current receipt. Existing association ACKs still use the original descriptor.
    pub fn rebind_selection(
        &self,
        read: &SelectedRead,
        binding: &SelectionRebinding,
    ) -> Result<Self, CanonicalError> {
        if !std::sync::Arc::ptr_eq(&self.selection, &binding.previous)
            || !binding.current.matches_selection(read)?
        {
            return Err(CanonicalError::Configuration(
                "selection rebinding is stale".into(),
            ));
        }
        Ok(Self {
            material: self.material.clone(),
            selection: binding.current.clone(),
        })
    }
    /// Exact stored descriptor; acknowledgment does not hydrate its referenced blocks.
    pub fn descriptor(&self) -> &[u8] {
        self.material.product.payload.as_slice()
    }
    /// Check the exact complete consumer premises without cloning or reencoding them.
    pub fn matches_read(&self, read: &SelectedRead) -> Result<bool, CanonicalError> {
        read.complete()?;
        if self.selection.matches_selection(read)? {
            return Ok(true);
        }
        let dependencies = &self.selection.dependencies.dependencies;
        let mut premises = dependencies.premises.iter();
        Ok(self.material.product.problem == read.selection().revision().problem
            && dependencies.interpretation == wire::INTERPRETATION
            && read.names.iter().all(|((s, n), v)| matches!(premises.next(),
                Some(Premise::Name { scope, name, version }) if s == scope && n == name && v == version))
            && read.scopes.iter().all(|(s, m)| matches!(premises.next(),
                Some(Premise::Scope { scope, members }) if s == scope && m == members))
            && read.interpretations.iter().all(|(r, i)| matches!(premises.next(),
                Some(Premise::Interpretation { role, identity }) if r == role && i == identity))
            && read.logicals.iter().all(|(l, v)| matches!(premises.next(),
                Some(Premise::Logical { logical, version }) if l == logical && v == version))
            && read.kinds.iter().all(|((s, k), m)| matches!(premises.next(),
                Some(Premise::KindScope { scope, source_kind, members }) if s == scope && k == source_kind && m == members))
            && read.references.iter().all(|((s, n, k), m)| matches!(premises.next(),
                Some(Premise::References { scope, name, source_kind, members }) if s == scope && n == name && k == source_kind && m == members))
            && premises.next().is_none())
    }
    fn publication(&self, read: &SelectedRead) -> Result<Product, CanonicalError> {
        if !self.selection.matches_selection(read)? {
            return Err(CanonicalError::Configuration(
                "new product root requires current selection material".into(),
            ));
        }
        let material = &self.material;
        let payload = material.payload.as_slice();
        let dependencies = self.selection.encoded.as_slice();
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalProductV1);
        hash.str(&material.proposed_key)
            .str(read.selection.key())
            .str(&material.product.problem)
            .str(&material.product.interpretation)
            .str(&material.product.producer)
            .str(material.product.producer_class.as_str())
            .part(material.product.request.as_slice())
            .part(payload)
            .part(dependencies);
        let mut product = material.product.clone();
        product.key = hash.finish_hash().to_hex();
        product.revision = read.selection().revision().key.clone();
        Ok(product)
    }
}

/// Small discovery receipt. Its private header permits budgeting, never replay authority.
#[derive(Debug)]
pub struct ProductCandidate {
    product: Product,
    descriptor: crate::canonical_staging::ProductBlob,
    retained_bytes: usize,
}
impl ProductCandidate {
    /// Exact declared class; discovery alone grants no scientific admission.
    pub fn producer_class(&self) -> CanonicalProducerClass { self.product.producer_class }

    /// Cursor for the next discovery after an incompatible candidate.
    pub fn key(&self) -> &str {
        &self.product.key
    }
    /// Conservative assembly, copy and dependency parsing allocation bound.
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
}

/// A persisted description whose complete recorded premises match this protected
/// immutable selection and the reader's explicitly supplied interpretations.
#[derive(Debug)]
pub struct ReusableProduct {
    product: Product,
}
impl ReusableProduct {
    /// Compatible immutable product identity.
    pub fn key(&self) -> &str {
        &self.product.key
    }
    /// Qualified portable payload; the scientific owner still decodes its own format.
    pub fn payload(&self) -> &[u8] {
        self.product.payload.as_slice()
    }
    /// Exact producer class retained through canonical qualification.
    pub fn producer_class(&self) -> CanonicalProducerClass { self.product.producer_class }
    /// Exact producer interpretation used for eligibility.
    pub fn producer(&self) -> &str {
        &self.product.producer
    }
}

impl CanonicalStore {
    /// Adopt qualified immutable premises only for the same exact actual revision.
    /// One thin guarded RPC verifies the new pin and actual stored row; changed
    /// revisions still require full dependency qualification. No new product is admitted.
    pub async fn adopt_dependencies_from_same_revision(
        &self,
        read: &mut SelectedRead,
        receipt: &SelectionDependencyReceipt,
    ) -> Result<bool, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity.scope(async {
        read.complete()?;
        if read.selection.revision() != &receipt.revision {
            return Ok(false);
        }
        let mut response = self.protected_query(&read.selection.revision().problem, crate::canonical::QueryEffect::Read, "canonical_selection::adopt_dependencies_from_same_revision",
            || Ok(self.db.query(format!(
                "{PROTECTED_BEGIN}\nLET $rpc_result = SELECT * FROM ONLY type::record('canonical_revisions', $revision) TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"
            ))
                .bind(("problem", read.selection.revision().problem.clone()))
                .bind(("revision", read.selection.revision().key.clone()))
                .bind(("sequence", crate::canonical_codec::encode_uint(read.selection.revision().sequence)?))
                .bind(("protection", read.selection.key().to_owned()))),
        ).await?;
        let actual: Option<Object> = response.take(response.num_statements().saturating_sub(2))?;
        let Some(actual) = actual.map(wire::decode_canonical_revisions).transpose()? else {
            return Ok(false);
        };
        if actual != receipt.revision {
            return Ok(false);
        }
        let mut delta = SelectedRead::new(read.selection.clone());
        for premise in &receipt.dependencies.dependencies.premises {
            match premise.clone() {
                Premise::Name {
                    scope,
                    name,
                    version,
                } => {
                    delta.names.insert((scope, name), version);
                }
                Premise::Scope { scope, members } => {
                    delta.scopes.insert(scope, members);
                }
                Premise::Logical { logical, version } => {
                    delta.logicals.insert(logical, version);
                }
                Premise::KindScope {
                    scope,
                    source_kind,
                    members,
                } => {
                    delta.kinds.insert((scope, source_kind), members);
                }
                Premise::References {
                    scope,
                    name,
                    source_kind,
                    members,
                } => {
                    delta.references.insert((scope, name, source_kind), members);
                }
                Premise::Interpretation { role, identity } => {
                    delta.interpretations.insert(role, identity);
                }
            }
        }
        read.merge_delta(delta)?;
        Ok(true)
            }).await
    }
    /// Resolve exact authored identities, including inspected absent identities.
    pub async fn resolve_logicals(
        &self,
        read: &mut SelectedRead,
        logicals: &[String],
    ) -> Result<Vec<Membership>, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity.scope(async {
        if logicals.len() > 256 {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut response = self.protected_query(&read.selection.revision().problem, crate::canonical::QueryEffect::Read, "canonical_selection::resolve_logicals", || Ok(self.db.query(format!("{PROTECTED_BEGIN}\nLET $rpc_result = SELECT * FROM canonical_memberships WHERE problem = $problem AND logical IN $logicals AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"))
            .bind(("problem", read.selection.revision().problem.clone())).bind(("revision", read.selection.revision().key.clone())).bind(("protection", read.selection.key().to_owned())).bind(("logicals", logicals.to_vec())).bind(("sequence", crate::canonical_codec::encode_uint(read.selection.revision().sequence)?)))).await?;
        let rows: Vec<Object> = response.take(response.num_statements().saturating_sub(2))?;
        let members = rows
            .into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<Vec<_>, _>>()?;
        read.invalidate_material();
        for logical in logicals {
            let mut matching = members.iter().filter(|member| &member.logical == logical);
            let version = matching.next().map(|member| member.version.clone());
            if matching.next().is_some() {
                return Err(CanonicalError::Configuration(
                    "immutable logical identity is not unique".into(),
                ));
            }
            read.logicals.insert(logical.clone(), version);
        }
        Ok(members)
            }).await
    }
    /// Enumerate a requested declaration kind without hydrating unrelated source objects.
    pub async fn resolve_kind_scope(
        &self,
        read: &mut SelectedRead,
        scope: Option<&str>,
        source_kind: &str,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                let mut cursor = self.kind_pages(read, scope, source_kind)?;
                let mut members = Vec::new();
                loop {
                    let page = self.next_membership_page(read, &mut cursor).await?;
                    if page.is_empty() {
                        break;
                    }
                    members.extend(page);
                }
                Ok(members)
            })
            .await
    }
    /// Follow authored inverse references at this revision; scientific resolution stays
    /// with the compiler, while the complete filtered membership is a reuse premise.
    pub async fn resolve_references(
        &self,
        read: &mut SelectedRead,
        target_scope: &str,
        target_name: &str,
        source_kind: &str,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                let mut cursor =
                    self.reference_pages(read, target_scope, target_name, source_kind)?;
                let mut members = Vec::new();
                loop {
                    let page = self.next_membership_page(read, &mut cursor).await?;
                    if page.is_empty() {
                        break;
                    }
                    members.extend(page);
                }
                Ok(members)
            })
            .await
    }

    /// Begin an exact full namespace inventory; only its protected terminal page
    /// establishes completeness. The cursor's next key cannot be supplied by callers.
    pub fn scope_pages(
        &self,
        read: &mut SelectedRead,
        scope: &str,
    ) -> Result<SelectedMemberships, CanonicalError> {
        self.membership_cursor(read, Some(scope), None, None)
    }
    /// Begin a complete filtered inventory, including an empty scoped kind.
    pub fn kind_pages(
        &self,
        read: &mut SelectedRead,
        scope: Option<&str>,
        source_kind: &str,
    ) -> Result<SelectedMemberships, CanonicalError> {
        self.membership_cursor(read, scope, Some(source_kind), None)
    }
    /// Begin a complete inverse scientific supplier inventory.
    pub fn reference_pages(
        &self,
        read: &mut SelectedRead,
        scope: &str,
        name: &str,
        source_kind: &str,
    ) -> Result<SelectedMemberships, CanonicalError> {
        self.membership_cursor(read, None, Some(source_kind), Some((scope, name)))
    }
    fn membership_cursor(
        &self,
        read: &mut SelectedRead,
        scope: Option<&str>,
        source_kind: Option<&str>,
        target: Option<(&str, &str)>,
    ) -> Result<SelectedMemberships, CanonicalError> {
        if scope
            .into_iter()
            .chain(source_kind)
            .chain(target.into_iter().flat_map(|(scope, name)| [scope, name]))
            .any(|value| value.len() > crate::canonical_staging::IDENTITY_BYTES)
            || read.pending_inventories.len() >= 64
        {
            return Err(CanonicalError::PayloadLimit);
        }
        let key = uuid::Uuid::new_v4().simple().to_string();
        read.invalidate_material();
        read.pending_inventories.insert(key.clone());
        Ok(SelectedMemberships {
            key,
            selection: read.selection.clone(),
            scope: scope.map(str::to_owned),
            source_kind: source_kind.map(str::to_owned),
            target: target.map(|(scope, name)| (scope.into(), name.into())),
            after: String::new(),
            pages: Vec::new(),
            member_count: 0,
            bytes: 0,
        })
    }
    /// One protected 64-row page. Partial/abandoned cursors refuse dependency
    /// publication and qualification; the final page moves its exact premise once.
    pub async fn next_membership_page(
        &self,
        read: &mut SelectedRead,
        cursor: &mut SelectedMemberships,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                let mut pages = self
                    .next_membership_pages(read, std::slice::from_mut(cursor))
                    .await?;
                Ok(pages.remove(0))
            })
            .await
    }
    /// Group exact inventory cursors in one protected transaction. The combined
    /// response contains at most 64 memberships, regardless of cursor count.
    pub async fn next_membership_pages(
        &self,
        read: &mut SelectedRead,
        cursors: &mut [SelectedMemberships],
    ) -> Result<Vec<Vec<Membership>>, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity.scope(async {
        if cursors.is_empty() || cursors.len() > SELECTED_INVENTORY_BATCH {
            return Err(CanonicalError::PayloadLimit);
        }
        for cursor in cursors.iter() {
            if cursor.selection.key() != read.selection.key()
                || cursor.selection.revision() != read.selection.revision()
                || !read.pending_inventories.contains(&cursor.key)
            {
                return Err(CanonicalError::Configuration(
                    "selected inventory cursor differs or is complete".into(),
                ));
            }
        }
        let limit = 64 / cursors.len();
        let mut sql = PROTECTED_BEGIN.to_owned();
        if cursors.iter().any(|cursor| cursor.scope.is_some()) {
            sql.push_str(MEMBERSHIP_HEAD_BEGIN);
        }
        for (index, cursor) in cursors.iter().enumerate() {
            let scoped =
                membership_scope_predicate(cursor.scope.is_some(), &format!("scope{index}"));
            let (suppliers, referenced) =
                membership_reference_predicate(cursor.target.is_some(), &index.to_string());
            sql.push_str(&suppliers);
            let page = membership_page_query(
                cursor.scope.is_some(),
                &format!(
                    "problem = $problem{scoped} AND key > $after{index} AND ($kind{index} = NONE OR (out.kind = $kind{index} AND out.closed = true)){referenced}"
                ),
                limit,
            );
            sql.push_str(&format!("\nLET $page{index} = {page};"));
        }
        let pages = (0..cursors.len())
            .map(|index| format!("$page{index}"))
            .collect::<Vec<_>>()
            .join(",");
        sql.push_str(&format!("\nLET $rpc_result = [{pages}];\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"));
        let mut response = self
            .protected_query(
                &read.selection.revision().problem,
                crate::canonical::QueryEffect::Read,
                "canonical_selection::next_membership_pages",
                || {
                    let mut query = self
                        .db
                        .query(sql.clone())
                        .bind(("problem", read.selection.revision().problem.clone()))
                        .bind(("revision", read.selection.revision().key.clone()))
                        .bind(("protection", read.selection.key().to_owned()))
                        .bind((
                            "sequence",
                            crate::canonical_codec::encode_uint(
                                read.selection.revision().sequence,
                            )?,
                        ));
                    for (index, cursor) in cursors.iter().enumerate() {
                        query = query
                            .bind((format!("scope{index}"), cursor.scope.clone()))
                            .bind((format!("after{index}"), cursor.after.clone()))
                            .bind((format!("kind{index}"), cursor.source_kind.clone()))
                            .bind((
                                format!("target_scope{index}"),
                                cursor.target.as_ref().map(|(scope, _)| scope.clone()),
                            ))
                            .bind((
                                format!("target_name{index}"),
                                cursor.target.as_ref().map(|(_, name)| name.clone()),
                            ));
                    }
                    Ok(query)
                },
            )
            .await?;
        // Decode every bounded page before moving any cursor's premise.
        let raw_pages: Vec<Vec<Object>> =
            response.take(response.num_statements().saturating_sub(2))?;
        if raw_pages.len() != cursors.len() {
            return Err(CanonicalError::IncompleteResponse);
        }
        let pages = raw_pages
            .into_iter()
            .map(|rows| {
                rows.into_iter()
                    .map(wire::decode_canonical_memberships)
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (cursor, page) in cursors.iter_mut().zip(&pages) {
            self.accept_membership_page(read, cursor, page)?;
        }
        Ok(pages)
            }).await
    }
    fn accept_membership_page(
        &self,
        read: &mut SelectedRead,
        cursor: &mut SelectedMemberships,
        page: &[Membership],
    ) -> Result<(), CanonicalError> {
        if let Some(last) = page.last() {
            for member in page {
                let name = if cursor.source_kind.is_some() {
                    &member.logical
                } else {
                    &member.name
                };
                cursor.bytes = cursor
                    .bytes
                    .checked_add(name.len())
                    .and_then(|bytes| bytes.checked_add(member.version.len() + 64))
                    .filter(|bytes| *bytes <= 32 * 1024 * 1024)
                    .ok_or(CanonicalError::PayloadLimit)?;
            }
            let mut identities = Vec::new();
            identities.try_reserve_exact(page.len()).map_err(|error| {
                CanonicalError::Configuration(format!("selected inventory allocation: {error}"))
            })?;
            identities.extend(page.iter().map(|member| {
                (
                    if cursor.source_kind.is_some() {
                        member.logical.clone()
                    } else {
                        member.name.clone()
                    },
                    member.version.clone(),
                )
            }));
            cursor.pages.try_reserve(1).map_err(|error| {
                CanonicalError::Configuration(format!("selected inventory pages: {error}"))
            })?;
            cursor.member_count += identities.len();
            cursor.pages.push(identities);
            cursor.after = last.key.clone();
        } else {
            let mut members = Vec::new();
            members
                .try_reserve_exact(cursor.member_count)
                .map_err(|error| {
                    CanonicalError::Configuration(format!("selected complete inventory: {error}"))
                })?;
            for page in std::mem::take(&mut cursor.pages) {
                members.extend(page);
            }
            members.sort();
            read.invalidate_material();
            if let Some((scope, name)) = &cursor.target {
                let kind = cursor.source_kind.as_ref().ok_or_else(|| {
                    CanonicalError::Configuration("supplier cursor requires kind".into())
                })?;
                let key = (scope.clone(), name.clone(), kind.clone());
                if read
                    .references
                    .get(&key)
                    .is_some_and(|previous| previous != &members)
                {
                    return Err(CanonicalError::Configuration(
                        "selected supplier inventory differs".into(),
                    ));
                }
                read.references.insert(key, members);
            } else if let Some(kind) = &cursor.source_kind {
                let key = (cursor.scope.clone(), kind.clone());
                if read
                    .kinds
                    .get(&key)
                    .is_some_and(|previous| previous != &members)
                {
                    return Err(CanonicalError::Configuration(
                        "selected kind inventory differs".into(),
                    ));
                }
                read.kinds.insert(key, members);
            } else {
                let scope = cursor.scope.as_ref().ok_or_else(|| {
                    CanonicalError::Configuration("namespace cursor requires scope".into())
                })?;
                if read
                    .scopes
                    .get(scope)
                    .is_some_and(|previous| previous != &members)
                {
                    return Err(CanonicalError::Configuration(
                        "selected namespace inventory differs".into(),
                    ));
                }
                read.scopes.insert(scope.clone(), members);
            }
            read.pending_inventories.remove(&cursor.key);
        }
        Ok(())
    }
    async fn selection_membership_page(
        &self,
        selection: &ProtectedSelection,
        scope: Option<&str>,
        source_kind: Option<&str>,
        target: Option<(&str, &str)>,
        after: &str,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let scoped = membership_scope_predicate(scope.is_some(), "scope");
        let (suppliers, referenced) = membership_reference_predicate(target.is_some(), "");
        let head = if scope.is_some() {
            MEMBERSHIP_HEAD_BEGIN
        } else {
            ""
        };
        let page = membership_page_query(
            scope.is_some(),
            &format!(
                "problem = $problem{scoped} AND key > $after AND ($source_kind = NONE OR (out.kind = $source_kind AND out.closed = true)){referenced}"
            ),
            64,
        );
        let mut response = self.protected_query(&selection.revision().problem, crate::canonical::QueryEffect::Read, "canonical_selection::selection_membership_page", || Ok(self.db.query(format!("{PROTECTED_BEGIN}{head}{suppliers}\nLET $rpc_result = {page};\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"))
            .bind(("problem", selection.revision().problem.clone())).bind(("revision", selection.revision().key.clone())).bind(("protection", selection.key().to_owned())).bind(("scope", scope.map(str::to_owned))).bind(("source_kind", source_kind.map(str::to_owned))).bind(("after", after.to_owned())).bind(("target_scope", target.map(|(scope,_)| scope.to_owned()))).bind(("target_name", target.map(|(_,name)| name.to_owned()))).bind(("sequence", crate::canonical_codec::encode_uint(selection.revision().sequence)?)))).await?;
        let rows: Vec<Object> = response.take(response.num_statements().saturating_sub(2))?;
        rows.into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    // Compare against the already bounded persisted meaning. Never collect a
    // grown present scope: an extra or different member refuses on its first page.
    async fn same_membership(
        &self,
        selection: &ProtectedSelection,
        scope: Option<&str>,
        source_kind: Option<&str>,
        target: Option<(&str, &str)>,
        expected: &[(String, String)],
        logical: bool,
    ) -> Result<bool, CanonicalError> {
        self.same_membership_counted(
            selection,
            scope,
            source_kind,
            target,
            expected,
            logical,
            &mut 0,
        )
        .await
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one exact inventory comparison and its local response count"
    )]
    async fn same_membership_counted(
        &self,
        selection: &ProtectedSelection,
        scope: Option<&str>,
        source_kind: Option<&str>,
        target: Option<(&str, &str)>,
        expected: &[(String, String)],
        logical: bool,
        _pages: &mut usize,
    ) -> Result<bool, CanonicalError> {
        if expected.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Ok(false);
        }
        let mut seen = vec![false; expected.len()];
        let mut count = 0usize;
        let mut after = String::new();
        loop {
            #[cfg(test)]
            {
                *_pages += 1;
            }
            let page = self
                .selection_membership_page(selection, scope, source_kind, target, &after)
                .await?;
            let Some(last) = page.last() else {
                return Ok(count == expected.len());
            };
            after = last.key.clone();
            for member in page {
                let pair = (
                    if logical { member.logical } else { member.name },
                    member.version,
                );
                let Ok(index) = expected.binary_search(&pair) else {
                    return Ok(false);
                };
                if seen[index] {
                    return Ok(false);
                }
                seen[index] = true;
                count += 1;
                if count > expected.len() {
                    return Ok(false);
                }
            }
        }
    }
    /// Resolve names and retain every inspected presence/absence premise.
    pub async fn resolve_names(
        &self,
        read: &mut SelectedRead,
        scope: &str,
        names: &[String],
    ) -> Result<Vec<Membership>, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                let members = self.select_names(&read.selection, scope, names).await?;
                read.invalidate_material();
                for name in names {
                    let matches = members
                        .iter()
                        .filter(|member| &member.name == name)
                        .collect::<Vec<_>>();
                    if matches.len() > 1 {
                        return Err(CanonicalError::Configuration(
                            "immutable name is not unique".into(),
                        ));
                    }
                    read.names.insert(
                        (scope.into(), name.clone()),
                        matches.first().map(|member| member.version.clone()),
                    );
                }
                Ok(members)
            })
            .await
    }
    /// Resolve an exact sparse set of lexical pairs, preserving absent pairs
    /// without admitting the cross product of independently grouped scopes/names.
    pub async fn resolve_name_pairs(
        &self,
        read: &mut SelectedRead,
        pairs: &[(String, String)],
    ) -> Result<Vec<Membership>, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity.scope(async {
        if pairs.is_empty() {
            return Ok(Vec::new());
        }
        if pairs.len() > 64
            || pairs.iter().any(|(scope, name)| {
                scope.len() > crate::canonical_staging::IDENTITY_BYTES
                    || name.len() > crate::canonical_staging::IDENTITY_BYTES
            })
        {
            return Err(CanonicalError::PayloadLimit);
        }
        let predicate = (0..pairs.len())
            .map(|index| format!("(scope = $scope{index} AND name = $name{index})"))
            .collect::<Vec<_>>()
            .join(" OR ");
        let sql = format!(
            "{PROTECTED_BEGIN}\nLET $rpc_result = SELECT * FROM canonical_memberships WHERE problem = $problem AND ({predicate}) AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"
        );
        let mut response = self
            .protected_query(
                &read.selection.revision().problem,
                crate::canonical::QueryEffect::Read,
                "canonical_selection::resolve_name_pairs",
                || {
                    let mut query = self
                        .db
                        .query(sql.clone())
                        .bind(("problem", read.selection.revision().problem.clone()))
                        .bind(("revision", read.selection.revision().key.clone()))
                        .bind(("protection", read.selection.key().to_owned()))
                        .bind((
                            "sequence",
                            crate::canonical_codec::encode_uint(
                                read.selection.revision().sequence,
                            )?,
                        ));
                    for (index, (scope, name)) in pairs.iter().enumerate() {
                        query = query
                            .bind((format!("scope{index}"), scope.clone()))
                            .bind((format!("name{index}"), name.clone()));
                    }
                    Ok(query)
                },
            )
            .await?;
        let rows: Vec<Object> = response.take(response.num_statements().saturating_sub(2))?;
        let members = rows
            .into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<Vec<_>, _>>()?;
        let mut present = BTreeMap::new();
        for member in &members {
            if present
                .insert(
                    (member.scope.clone(), member.name.clone()),
                    member.version.clone(),
                )
                .is_some()
            {
                return Err(CanonicalError::Configuration(
                    "immutable name is not unique".into(),
                ));
            }
        }
        read.invalidate_material();
        for pair in pairs {
            read.names.insert(pair.clone(), present.get(pair).cloned());
        }
        Ok(members)
            }).await
    }
    /// One bounded inventory page; no cursor exposes unprotected subsequent reads.
    pub async fn membership_page(
        &self,
        selection: &ProtectedSelection,
        scope: Option<&str>,
        after: &str,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let activity = self.selection_activity(selection)?;
        activity
            .scope(async {
                let mut response = self
                    .protected_query(
                        &selection.revision().problem,
                        crate::canonical::QueryEffect::Read,
                        "canonical_selection::membership_page",
                        || {
                            Ok(self
                                .db
                                .query(membership_page_sql(scope.is_some()))
                                .bind(("problem", selection.revision().problem.clone()))
                                .bind(("revision", selection.revision().key.clone()))
                                .bind(("protection", selection.key().to_owned()))
                                .bind(("scope", scope.map(str::to_owned)))
                                .bind(("after", after.to_owned()))
                                .bind((
                                    "sequence",
                                    crate::canonical_codec::encode_uint(
                                        selection.revision().sequence,
                                    )?,
                                )))
                        },
                    )
                    .await?;
                let rows: Vec<Object> =
                    response.take(response.num_statements().saturating_sub(2))?;
                rows.into_iter()
                    .map(wire::decode_canonical_memberships)
                    .collect::<Result<_, _>>()
                    .map_err(Into::into)
            })
            .await
    }
    /// Enumerate an explicitly required scope, preserving its complete membership premise.
    pub async fn resolve_scope(
        &self,
        read: &mut SelectedRead,
        scope: &str,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                let mut cursor = self.scope_pages(read, scope)?;
                let mut members = Vec::new();
                loop {
                    let page = self.next_membership_page(read, &mut cursor).await?;
                    if page.is_empty() {
                        break;
                    }
                    members.extend(page);
                }
                Ok(members)
            })
            .await
    }
    /// Fetch an exact immutable object within a live selection. Independent pages
    /// and payloads remain below the transport's individual-message bound.
    pub async fn selected_object(
        &self,
        selection: &ProtectedSelection,
        version: &str,
    ) -> Result<Option<ObjectVersion>, CanonicalError> {
        let activity = self.selection_activity(selection)?;
        activity
            .scope(async { self.assemble_selected_object(selection, version).await })
            .await
    }
    /// Encode the selected description without a store effect or publication permission.
    pub fn prepare_description(
        &self,
        read: &SelectedRead,
        product: Product,
    ) -> Result<ProductDescription, CanonicalError> {
        ProductDescription::prepare(read, product)
    }
    /// Resolve only an already committed exact rooted descriptor. No pin is required,
    /// no row is admitted, and referenced blob corruption is checked by receiving qualification.
    pub async fn acknowledge_description(
        &self,
        description: &ProductDescription,
    ) -> Result<Option<String>, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                self.acknowledge_description_until(
                    description,
                    crate::canonical::transport::original_deadline(
                        crate::canonical::REQUEST_TIMEOUT,
                    ),
                )
                .await
            })
            .await
    }
    /// Exact settlement on the caller's original operation clock. Dropping the future
    /// stops retries; it cannot undo a transaction already committed by the server.
    pub async fn acknowledge_description_until(
        &self,
        description: &ProductDescription,
        deadline: tokio::time::Instant,
    ) -> Result<Option<String>, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                if tokio::time::Instant::now() >= deadline {
                    return Err(CanonicalError::Timeout);
                }
                crate::canonical::transport::within_clock(
                    deadline,
                    self.product_acknowledged(&description.material.product),
                )
                .await
            })
            .await
    }
    /// Settle or publish reusable material against the exact current consumer read.
    pub async fn publish_description(
        &self,
        read: &SelectedRead,
        description: &ProductDescription,
    ) -> Result<String, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                self.publish_description_until(
                    read,
                    description,
                    crate::canonical::transport::original_deadline(
                        crate::canonical::REQUEST_TIMEOUT,
                    ),
                )
                .await
            })
            .await
    }
    /// Generic publication on the caller's original clock, including staging and waiters.
    pub async fn publish_description_until(
        &self,
        read: &SelectedRead,
        description: &ProductDescription,
        deadline: tokio::time::Instant,
    ) -> Result<String, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                if description.material.product.producer_class != CanonicalProducerClass::General {
                    return Err(CanonicalError::Configuration(
                        "generic product publication cannot impersonate scientific admission"
                            .into(),
                    ));
                }
                self.publish_description_inner(read, description, deadline)
                    .await
            })
            .await
    }
    /// Publish compiler-issued immutable material through the controlled writer.
    ///
    /// # Safety
    /// Payload, request and exact dependencies must be issued by the owning successfully
    /// admitted compiler, with its construction receipts and qualified current producer.
    /// Caller-created receipts or claimed hashes cannot establish scientific admission.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 explicit scientific writer trust assertion; no unsafe memory operation"
    )]
    pub async unsafe fn publish_scientific_description(
        &self,
        read: &SelectedRead,
        description: &ProductDescription,
    ) -> Result<String, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                self.publish_scientific_description_inner(
                    read,
                    description,
                    crate::canonical::transport::original_deadline(
                        crate::canonical::REQUEST_TIMEOUT,
                    ),
                )
                .await
            })
            .await
    }
    /// Controlled publication with the original enclosing operation deadline.
    ///
    /// # Safety
    /// The same compiler admission and producer requirements as `publish_scientific_description`.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 explicit scientific writer trust assertion; no unsafe memory operation"
    )]
    pub async unsafe fn publish_scientific_description_until(
        &self,
        read: &SelectedRead,
        description: &ProductDescription,
        deadline: tokio::time::Instant,
    ) -> Result<String, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                self.publish_scientific_description_inner(read, description, deadline)
                    .await
            })
            .await
    }
    async fn publish_scientific_description_inner(
        &self,
        read: &SelectedRead,
        description: &ProductDescription,
        deadline: tokio::time::Instant,
    ) -> Result<String, CanonicalError> {
        if !description.material.product.producer_class.scientific() {
            return Err(CanonicalError::Configuration(
                "scientific publication requires its actual admitted producer class".into(),
            ));
        }
        self.publish_description_inner(read, description, deadline)
            .await
    }
    /// Publish the kernel's description together with actual selected read premises.
    pub async fn publish_product(
        &self,
        read: &SelectedRead,
        product: Product,
    ) -> Result<String, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                let deadline = crate::canonical::transport::original_deadline(
                    crate::canonical::REQUEST_TIMEOUT,
                );
                if product.producer_class != CanonicalProducerClass::General {
                    return Err(CanonicalError::Configuration(
                        "generic product publication cannot impersonate scientific admission"
                            .into(),
                    ));
                }
                let description = self.prepare_description(read, product)?;
                self.publish_description_inner(read, &description, deadline)
                    .await
            })
            .await
    }
    /// Publish a description issued by the scientific admission owner.
    ///
    /// # Safety
    /// Payload and request must come from the owning compiler's successfully admitted
    /// immutable body with exact construction receipts and qualified current producer.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 explicit scientific writer trust assertion; no unsafe memory operation"
    )]
    pub async unsafe fn publish_scientific_product(
        &self,
        read: &SelectedRead,
        product: Product,
    ) -> Result<String, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                let deadline = crate::canonical::transport::original_deadline(
                    crate::canonical::REQUEST_TIMEOUT,
                );
                if !product.producer_class.scientific() {
                    return Err(CanonicalError::Configuration(
                        "scientific publication requires its actual admitted producer class"
                            .into(),
                    ));
                }
                let description = self.prepare_description(read, product)?;
                self.publish_description_inner(read, &description, deadline)
                    .await
            })
            .await
    }
    async fn publish_description_inner(
        &self,
        read: &SelectedRead,
        description: &ProductDescription,
        deadline: tokio::time::Instant,
    ) -> Result<String, CanonicalError> {
        if tokio::time::Instant::now() >= deadline {
            return Err(CanonicalError::Timeout);
        }
        crate::canonical::transport::within_clock(deadline, async {
            if !description.matches_read(read)? {
                return Err(CanonicalError::Configuration(
                    "product description dependencies differ from current read".into(),
                ));
            }
            // The thin exact lookup precedes selection-bound hashing, payload copies,
            // stage acquisition and permission checks for any new admission.
            if let Some(key) = self
                .product_acknowledged(&description.material.product)
                .await?
            {
                return Ok(key);
            }
            self.ensure_writes()?;
            if tokio::time::Instant::now() >= deadline {
                return Err(CanonicalError::Timeout);
            }
            let product = description.publication(read)?;
            if tokio::time::Instant::now() >= deadline {
                return Err(CanonicalError::Timeout);
            }
            let stage = self
                .stage_product_parts(
                    &product.problem,
                    &product.key,
                    &description.material.blob,
                    &description.material.payload,
                    &description.selection.encoded,
                )
                .await?;
            if let Some(stage) = stage {
                return self.admit_product(&read.selection, &product, &stage).await;
            }
            // A concurrent writer owns this stage. Poll immutable acknowledgment only;
            // never acquire its generation or renew the original operation clock.
            loop {
                if let Some(key) = self.product_acknowledged(&product).await? {
                    return Ok(key);
                }
                self.ensure_writes()?;
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
    }
    /// Corrupt only a disposable native fixture's first immutable product block.
    #[cfg(any(test, feature = "test-support"))]
    pub async fn corrupt_product_block(&self, key: &str) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                if !self.database().starts_with("canonical_test_") {
                    return Err(CanonicalError::Configuration(
                        "corruption requires isolated canonical fixture".into(),
                    ));
                }
                let row = crate::canonical::request(
                    self.db.select(("canonical_product_associations", key)),
                )
                .await?
                .ok_or_else(|| CanonicalError::Configuration("fixture product missing".into()))?;
                let product = crate::canonical::decode_product_association(row)?;
                let descriptor =
                    crate::canonical_staging::ProductBlob::decode(product.payload.as_slice())?;
                crate::canonical::bounded_query(
            self.db
                .query(
                    "UPDATE type::record('canonical_payload_blocks', $block) SET payload = $bytes;",
                )
                .bind(("block", format!("{}:0", descriptor.version)))
                .bind(("bytes", surrealdb::types::Bytes::from(vec![0]))),
        )
        .await?;
                Ok(())
            })
            .await
    }
    /// Corrupt an isolated fixture's product descriptor without touching protected roots.
    #[cfg(any(test, feature = "test-support"))]
    pub async fn corrupt_product_descriptor(&self, key: &str) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        if !self.database().starts_with("canonical_test_") {
            return Err(CanonicalError::Configuration(
                "corruption requires isolated canonical fixture".into(),
            ));
        }
        crate::canonical::bounded_query(
            self.db
                .query("UPDATE type::record('canonical_product_associations', $key) SET payload = $payload;")
                .bind(("key", key.to_owned()))
                .bind((
                    "payload",
                    surrealdb::types::Bytes::from(b"corrupt descriptor".to_vec()),
                )),
        )
        .await?;
        Ok(())
            }).await
    }
    /// Enumerate only compact rooted publication headers for this exact selected source.
    /// The protected transaction checks authority, root ownership and pin expiry; it
    /// neither decodes descriptors nor establishes receiving-host compatibility.
    pub async fn product_headers(
        &self,
        selection: &ProtectedSelection,
        cursor: Option<&ProductHeaderCursor>,
    ) -> Result<ProductHeaderPage, CanonicalError> {
        let activity = self.selection_activity(selection)?;
        activity.scope(async {
            let issuer = self.issuing_store()?;
            let revision = selection.revision();
            let after = if let Some(cursor) = cursor {
                if cursor.issuing_store != issuer || cursor.problem != revision.problem
                    || cursor.revision != revision.key
                    || cursor.after.len() > crate::canonical_staging::IDENTITY_BYTES {
                    return Err(CanonicalError::Configuration("product header cursor differs from selected authority/source".into()));
                }
                cursor.after.clone()
            } else { String::new() };
            let mut response = self.protected_query(&revision.problem, crate::canonical::QueryEffect::Read,
                "canonical_selection::product_headers", || Ok(self.db.query(format!(
                    "{PROTECTED_BEGIN}\nLET $rpc_result = SELECT key, revision, producer, producer_class, interpretation, descriptor, content, receipt FROM canonical_product_associations WHERE problem = $problem AND revision = $revision AND key > $after AND issuing_store = $issuing_store AND interpretation = $interpretation AND type::record('canonical_roots', key).owner_kind = 'product' AND type::record('canonical_roots', key).owner = key AND type::record('canonical_roots', key).problem = problem AND type::record('canonical_roots', key).revision = revision ORDER BY key LIMIT 33 TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"))
                    .bind(("problem", revision.problem.clone())).bind(("revision", revision.key.clone()))
                    .bind(("sequence", crate::canonical_codec::encode_uint(revision.sequence)?))
                    .bind(("protection", selection.key().to_owned())).bind(("issuing_store", issuer.to_owned()))
                    .bind(("after", after.clone())).bind(("interpretation", wire::INTERPRETATION.to_owned()))))
                .await?;
            let mut rows: Vec<Object> = response.take(response.num_statements().saturating_sub(2))?;
            let more = rows.len() > 32;
            rows.truncate(32);
            let text = |row: &Object, field: &str| -> Result<String, CanonicalError> {
                match row.get(field) {
                    Some(surrealdb::types::Value::String(value)) if value.len() <= crate::canonical_staging::IDENTITY_BYTES => Ok(value.clone()),
                    _ => Err(CanonicalError::Configuration(format!("invalid bounded product header {field}"))),
                }
            };
            let rows = rows.iter().map(|row| Ok(ProductHeader {
                key: text(row, "key")?, revision: text(row, "revision")?, producer: text(row, "producer")?,
                producer_class: text(row,"producer_class")?.parse::<CanonicalProducerClass>().map_err(|error|CanonicalError::Configuration(error.to_string()))?,
                interpretation: text(row, "interpretation")?, descriptor: text(row, "descriptor")?,
                content: text(row, "content")?, receipt: text(row, "receipt")?,
            })).collect::<Result<Vec<_>, CanonicalError>>()?;
            let next = if more { rows.last().map(|row| ProductHeaderCursor {
                issuing_store: issuer.to_owned(), problem: revision.problem.clone(), revision: revision.key.clone(), after: row.key.clone(),
            }) } else { None };
            Ok(ProductHeaderPage { rows, next })
        }).await
    }

    /// Probe for one protected product key without decoding its bounded blob descriptor.
    /// This lets a receiving owner decide whether a producer-context check is needed
    /// before it spends that cost; a present key still requires full candidate qualification.
    pub async fn product_candidate_presence(
        &self,
        selection: &ProtectedSelection,
        request: &[u8],
        producer: &str,
        producer_class: CanonicalProducerClass,
        after: &str,
    ) -> Result<bool, CanonicalError> {
        let activity = self.selection_activity(selection)?;
        activity.scope(async {
        if request.len() > crate::canonical::PAYLOAD_BYTES
            || producer.len() > crate::canonical_staging::IDENTITY_BYTES
            || after.len() > crate::canonical_staging::IDENTITY_BYTES
        {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut response = self.protected_query(&selection.revision().problem, crate::canonical::QueryEffect::Read, "canonical_selection::product_candidate_presence",
            || {
                Ok(self
                    .db
                    .query(format!("{PROTECTED_BEGIN}\nLET $rpc_result = SELECT VALUE true FROM canonical_product_associations WHERE issuing_store = $issuing_store AND problem = $problem AND producer = $producer AND producer_class = $producer_class AND request = $request AND key > $after AND interpretation = $interpretation AND type::record('canonical_roots', key).owner_kind = 'product' AND type::record('canonical_roots', key).owner = key AND type::record('canonical_roots', key).problem = problem AND type::record('canonical_roots', key).revision = revision ORDER BY key LIMIT 1 TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"))
                    .bind(("problem", selection.revision().problem.clone()))
                    .bind(("revision", selection.revision().key.clone()))
                    .bind((
                        "sequence",
                        crate::canonical_codec::encode_uint(selection.revision().sequence)?,
                    ))
                    .bind(("protection", selection.key().to_owned()))
                    .bind(("issuing_store", self.issuing_store()?.to_owned())).bind(("producer", producer.to_owned())).bind(("producer_class", producer_class.as_str().to_owned()))
                    .bind(("request", surrealdb::types::Bytes::from(request.to_vec())))
                    .bind(("after", after.to_owned()))
                    .bind(("interpretation", wire::INTERPRETATION.to_owned())))
            },
        )
        .await?;
        let rows: Vec<bool> = response.take(response.num_statements().saturating_sub(2))?;
        Ok(rows.into_iter().next().unwrap_or(false))
            }).await
    }
    /// Discover one eligible request/producer header without hydrating its scientific bytes.
    pub async fn product_candidate(
        &self,
        selection: &ProtectedSelection,
        request: &[u8],
        producer: &str,
        producer_class: CanonicalProducerClass,
        after: &str,
    ) -> Result<Option<ProductCandidate>, CanonicalError> {
        let activity = self.selection_activity(selection)?;
        activity.scope(async {
        if request.len() > crate::canonical::PAYLOAD_BYTES
            || producer.len() > crate::canonical_staging::IDENTITY_BYTES
            || after.len() > crate::canonical_staging::IDENTITY_BYTES
        {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut response = self.protected_query(&selection.revision().problem, crate::canonical::QueryEffect::Read, "canonical_selection::product_candidate", || Ok(self.db.query(format!("{PROTECTED_BEGIN}\nLET $rpc_result = SELECT * FROM canonical_product_associations WHERE issuing_store = $issuing_store AND problem = $problem AND producer = $producer AND producer_class = $producer_class AND request = $request AND key > $after AND interpretation = $interpretation AND type::record('canonical_roots', key).owner_kind = 'product' AND type::record('canonical_roots', key).owner = key AND type::record('canonical_roots', key).problem = problem AND type::record('canonical_roots', key).revision = revision ORDER BY key LIMIT 1 TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"))
            .bind(("problem", selection.revision().problem.clone())).bind(("revision", selection.revision().key.clone())).bind(("sequence", crate::canonical_codec::encode_uint(selection.revision().sequence)?))
            .bind(("protection", selection.key().to_owned())).bind(("issuing_store", self.issuing_store()?.to_owned())).bind(("producer", producer.to_owned())).bind(("producer_class", producer_class.as_str().to_owned())).bind(("request", surrealdb::types::Bytes::from(request.to_vec()))).bind(("after", after.to_owned())).bind(("interpretation", wire::INTERPRETATION.to_owned())))).await?;
        let rows: Vec<Object> = response.take(response.num_statements().saturating_sub(2))?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let product = crate::canonical::decode_product_association(row)?;
        if !product.dependencies.is_empty() {
            return Err(CanonicalError::Configuration(
                "unknown inline product format".into(),
            ));
        }
        let descriptor = crate::canonical_staging::ProductBlob::decode(product.payload.as_slice())?;
        let retained_bytes = descriptor.retained_bytes()?;
        // Validate the exact closed manifest before any whole-blob allocation.
        self.validate_product_candidate(selection, &product, &descriptor)
            .await?;
        Ok(Some(ProductCandidate {
            product,
            descriptor,
            retained_bytes,
        }))
            }).await
    }
    /// Hydrate only after the caller reserved the issued bound; check every premise
    /// against the protected revision without contaminating rejected read sessions.
    pub async fn qualify_product(
        &self,
        read: &mut SelectedRead,
        candidate: &ProductCandidate,
    ) -> Result<Option<ReusableProduct>, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                read.complete()?;
                if candidate.product.problem != read.selection().revision().problem {
                    return Err(CanonicalError::Configuration(
                        "product candidate belongs to a different problem".into(),
                    ));
                }
                let Some(blob) = self
                    .assemble_product_blob(
                        read.selection(),
                        &candidate.product,
                        &candidate.descriptor,
                    )
                    .await?
                else {
                    return Ok(None);
                };
                let (payload, dependencies_bytes) =
                    candidate.descriptor.split(blob.payload.as_slice())?;
                let dependencies: Dependencies = serde_json::from_slice(dependencies_bytes)
                    .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
                if dependencies.interpretation != wire::INTERPRETATION {
                    return Ok(None);
                }
                if !self.recheck_dependencies(read, &dependencies).await? {
                    return Ok(None);
                }
                // Rejected candidates retain only the assembled immutable blob and parsed
                // premises. Copy scientific bytes only after complete eligibility succeeds.
                let mut product = candidate.product.clone();
                product.payload = payload.to_vec().into();
                product.dependencies = dependencies_bytes.to_vec().into();
                Ok(Some(ReusableProduct { product }))
            })
            .await
    }
    /// Resolve common keys in bounded rounds, rejecting candidates after each
    /// response. Cached observations are private to this exact protected epoch;
    /// even a fully cached search performs a new live-pin guard before granting reuse.
    pub async fn qualify_selection_window(
        &self,
        read: &SelectedRead,
        window: &mut SelectedRecheckWindow,
    ) -> Result<Option<usize>, CanonicalError> {
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                window.check_epoch(read)?;
                window.qualified = None;
                let mut live = [false; SELECTED_RECHECK_WINDOW];
                for (index, candidate) in window.candidates.iter().enumerate() {
                    live[index] = candidate.dependencies.interpretation == wire::INTERPRETATION
                        && !candidate.dependencies.premises.iter().any(|premise| {
                            matches!(premise, Premise::Interpretation { role, identity }
                        if read.interpretations.get(role) != Some(identity))
                        });
                }
                if !live.iter().any(|live| *live) {
                    return Ok(None);
                }
                // Empty logical resolution deliberately executes PROTECTED_BEGIN, even
                // when every observed key is cached or the candidate contains no keys.
                let mut guard = SelectedRead::new(read.selection.clone());
                #[cfg(test)]
                {
                    window.counts.guards += 1;
                }
                self.resolve_logicals(&mut guard, &[]).await?;
                drop(guard);
                for names in [false, true] {
                    let bound = if names { 64 } else { 256 };
                    loop {
                        // Compare all observations from previous rounds before acquiring
                        // more source. A rejected candidate contributes no later keys.
                        for (index, candidate) in window.candidates.iter().enumerate() {
                            if live[index]
                                && candidate.dependencies.premises.iter().any(|premise| {
                                    premise.recheck_key(names).is_some_and(|(key, expected)| {
                                        window
                                            .actual
                                            .get(&key)
                                            .is_some_and(|actual| actual != expected)
                                    })
                                })
                            {
                                live[index] = false;
                            }
                        }
                        let mut keys = std::collections::BTreeSet::new();
                        for (index, candidate) in window.candidates.iter().enumerate() {
                            if !live[index] {
                                continue;
                            }
                            for premise in &candidate.dependencies.premises {
                                if let Some((key, _)) = premise.recheck_key(names)
                                    && !window.actual.contains_key(&key)
                                {
                                    keys.insert(key);
                                    if keys.len() == bound {
                                        break;
                                    }
                                }
                            }
                            if keys.len() == bound {
                                break;
                            }
                        }
                        if keys.is_empty() {
                            break;
                        }
                        let mut observations = SelectedRead::new(read.selection.clone());
                        if names {
                            let pairs = keys
                                .into_iter()
                                .filter_map(|key| match key {
                                    RecheckKey::Name(scope, name) => Some((scope, name)),
                                    _ => None,
                                })
                                .collect::<Vec<_>>();
                            #[cfg(test)]
                            {
                                window.counts.names += 1;
                            }
                            self.resolve_name_pairs(&mut observations, &pairs).await?;
                            for ((scope, name), actual) in observations.names {
                                window.actual.insert(RecheckKey::Name(scope, name), actual);
                            }
                        } else {
                            let logicals = keys
                                .into_iter()
                                .filter_map(|key| match key {
                                    RecheckKey::Logical(logical) => Some(logical),
                                    _ => None,
                                })
                                .collect::<Vec<_>>();
                            #[cfg(test)]
                            {
                                window.counts.logicals += 1;
                            }
                            self.resolve_logicals(&mut observations, &logicals).await?;
                            for (logical, actual) in observations.logicals {
                                window.actual.insert(RecheckKey::Logical(logical), actual);
                            }
                        }
                    }
                }
                for (index, candidate) in window.candidates.iter().enumerate() {
                    if !live[index] {
                        continue;
                    }
                    let mut eligible = true;
                    for premise in &candidate.dependencies.premises {
                        if !matches!(
                            premise,
                            Premise::Scope { .. }
                                | Premise::KindScope { .. }
                                | Premise::References { .. }
                        ) {
                            continue;
                        }
                        let same = if let Some(same) = window.inventories.get(premise) {
                            *same
                        } else {
                            let mut pages = 0;
                            let same = match premise {
                                Premise::Scope { scope, members } => {
                                    self.same_membership_counted(
                                        read.selection(),
                                        Some(scope),
                                        None,
                                        None,
                                        members,
                                        false,
                                        &mut pages,
                                    )
                                    .await?
                                }
                                Premise::KindScope {
                                    scope,
                                    source_kind,
                                    members,
                                } => {
                                    self.same_membership_counted(
                                        read.selection(),
                                        scope.as_deref(),
                                        Some(source_kind),
                                        None,
                                        members,
                                        true,
                                        &mut pages,
                                    )
                                    .await?
                                }
                                Premise::References {
                                    scope,
                                    name,
                                    source_kind,
                                    members,
                                } => {
                                    self.same_membership_counted(
                                        read.selection(),
                                        None,
                                        Some(source_kind),
                                        Some((scope, name)),
                                        members,
                                        true,
                                        &mut pages,
                                    )
                                    .await?
                                }
                                _ => {
                                    return Err(CanonicalError::Configuration(
                                        "candidate inventory premise".into(),
                                    ));
                                }
                            };
                            #[cfg(test)]
                            {
                                window.counts.inventory_pages += pages;
                            }
                            window.inventories.insert(premise.clone(), same);
                            same
                        };
                        if !same {
                            eligible = false;
                            break;
                        }
                    }
                    if eligible {
                        window.qualified = Some(index);
                        return Ok(Some(index));
                    }
                }
                Ok(None)
            })
            .await
    }
    /// Recheck immutable in-memory premises through the same eligibility mechanism
    /// as durable scientific products. Rejected candidates do not contaminate the read.
    pub async fn recheck_selection_dependencies(
        &self,
        read: &mut SelectedRead,
        dependencies: &SelectedDependencies,
    ) -> Result<bool, CanonicalError> {
        self.recheck_dependencies(read, &dependencies.dependencies)
            .await
    }
    async fn recheck_dependencies(
        &self,
        read: &mut SelectedRead,
        dependencies: &Dependencies,
    ) -> Result<bool, CanonicalError> {
        read.complete()?;
        if dependencies.interpretation != wire::INTERPRETATION {
            return Ok(false);
        }
        // Supplied interpretations are already in memory. A mismatch grants no
        // reuse authority and requires neither source hydration nor membership RPCs.
        if dependencies.premises.iter().any(|premise| {
            matches!(premise, Premise::Interpretation { role, identity }
                if read.interpretations.get(role) != Some(identity))
        }) {
            return Ok(false);
        }
        let activity = self.selection_activity(read.selection())?;
        activity
            .scope(async {
                let mut eligible = true;
                let mut candidate_read = SelectedRead::new(read.selection.clone());
                // Reject each bounded exact positive/absent window before acquiring later
                // windows or inventories. Accepted candidates still check every premise.
                let mut logicals =
                    dependencies
                        .premises
                        .iter()
                        .filter_map(|premise| match premise {
                            Premise::Logical { logical, version } => Some((logical, version)),
                            _ => None,
                        });
                loop {
                    let window = logicals.by_ref().take(256).collect::<Vec<_>>();
                    if window.is_empty() {
                        break;
                    }
                    let selected = window
                        .iter()
                        .map(|(logical, _)| (*logical).clone())
                        .collect::<Vec<_>>();
                    self.resolve_logicals(&mut candidate_read, &selected)
                        .await?;
                    if window.iter().any(|(logical, expected)| {
                        candidate_read.logicals.get(*logical) != Some(*expected)
                    }) {
                        return Ok(false);
                    }
                }
                let mut names = dependencies
                    .premises
                    .iter()
                    .filter_map(|premise| match premise {
                        Premise::Name {
                            scope,
                            name,
                            version,
                        } => Some(((scope.clone(), name.clone()), version)),
                        _ => None,
                    });
                loop {
                    let window = names.by_ref().take(64).collect::<Vec<_>>();
                    if window.is_empty() {
                        break;
                    }
                    let pairs = window
                        .iter()
                        .map(|(pair, _)| pair.clone())
                        .collect::<Vec<_>>();
                    self.resolve_name_pairs(&mut candidate_read, &pairs).await?;
                    if window
                        .iter()
                        .any(|(pair, expected)| candidate_read.names.get(pair) != Some(*expected))
                    {
                        return Ok(false);
                    }
                }
                for premise in dependencies.premises.iter().cloned() {
                    match premise {
                        Premise::Name {
                            scope,
                            name,
                            version,
                        } => {
                            eligible &= candidate_read.names.get(&(scope, name)) == Some(&version);
                        }
                        Premise::Scope { scope, members } => {
                            eligible &= self
                                .same_membership(
                                    read.selection(),
                                    Some(&scope),
                                    None,
                                    None,
                                    &members,
                                    false,
                                )
                                .await?;
                            if eligible {
                                candidate_read.scopes.insert(scope, members);
                            }
                        }
                        Premise::Interpretation { role, identity } => {
                            eligible &= read.interpretations.get(&role) == Some(&identity)
                        }
                        Premise::Logical { logical, version } => {
                            eligible &= candidate_read.logicals.get(&logical) == Some(&version);
                        }
                        Premise::KindScope {
                            scope,
                            source_kind,
                            members,
                        } => {
                            eligible &= self
                                .same_membership(
                                    read.selection(),
                                    scope.as_deref(),
                                    Some(&source_kind),
                                    None,
                                    &members,
                                    true,
                                )
                                .await?;
                            if eligible {
                                candidate_read.kinds.insert((scope, source_kind), members);
                            }
                        }
                        Premise::References {
                            scope,
                            name,
                            source_kind,
                            members,
                        } => {
                            eligible &= self
                                .same_membership(
                                    read.selection(),
                                    None,
                                    Some(&source_kind),
                                    Some((&scope, &name)),
                                    &members,
                                    true,
                                )
                                .await?;
                            if eligible {
                                candidate_read
                                    .references
                                    .insert((scope, name, source_kind), members);
                            }
                        }
                    }
                    if !eligible {
                        break;
                    }
                }
                if eligible {
                    read.merge_delta(candidate_read)?;
                }
                Ok(eligible)
            })
            .await
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
impl CanonicalStore {
    async fn test_reuse_product(
        &self,
        read: &mut SelectedRead,
        request: &[u8],
        producer: &str,
        after: &str,
    ) -> Result<Option<ReusableProduct>, CanonicalError> {
        let mut cursor = after.to_owned();
        while let Some(candidate) = self
            .product_candidate(read.selection(), request, producer, CanonicalProducerClass::General, &cursor)
            .await?
        {
            cursor = candidate.key().to_owned();
            if let Some(product) = self.qualify_product(read, &candidate).await? {
                return Ok(Some(product));
            }
        }
        Ok(None)
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_server_unit {
    #![allow(
        clippy::expect_used,
        clippy::unwrap_used,
        reason = "isolated selection fixtures and assertions fail the test on unexpected results"
    )]
    use super::*;
    use crate::canonical::{CanonicalOptions, ObjectEdit};
    use std::{path::Path, time::Duration};
    use surrealdb::types::Value;

    fn edit(
        name: &str,
        version: &str,
        source_kind: &str,
        references: Vec<(String, String)>,
    ) -> ObjectEdit {
        ObjectEdit {
            logical: name.into(),
            scope: "p".into(),
            name: name.into(),
            version: Some(ObjectVersion {
                key: version.into(),
                logical: name.into(),
                kind: source_kind.into(),
                payload: name.as_bytes().to_vec().into(),
                interpretation: wire::INTERPRETATION.into(),
            }),
            references,
        }
    }
    async fn fixture() -> CanonicalStore {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        crate::testing::canonical_fixture_with_options(&options, true).unwrap()
    }
    async fn read(store: &CanonicalStore, revision: crate::canonical::Revision) -> SelectedRead {
        SelectedRead::new(
            store
                .protect(revision, Duration::from_secs(60))
                .await
                .unwrap(),
        )
    }
    fn product(read: &SelectedRead, payload: &[u8]) -> Product {
        Product {
            key: "selected-description".into(),
            problem: read.selection.revision().problem.clone(),
            revision: read.selection.revision().key.clone(),
            request: b"exact-request".to_vec().into(),
            payload: payload.to_vec().into(),
            dependencies: Vec::new().into(),
            producer: "qualified-fixture".into(),
            producer_class: CanonicalProducerClass::General,
            interpretation: wire::INTERPRETATION.into(),
        }
    }

    fn candidate(premises: Vec<Premise>) -> std::sync::Arc<SelectedDependencies> {
        std::sync::Arc::new(SelectedDependencies {
            // Test fixtures reserve more than their small manually declared frames.
            retained_bytes: 64 * 1024,
            dependencies: Dependencies {
                interpretation: wire::INTERPRETATION.into(),
                premises,
            },
        })
    }
    fn point_candidate(version: &str) -> std::sync::Arc<SelectedDependencies> {
        candidate(vec![
            Premise::Logical {
                logical: "Root".into(),
                version: Some(version.into()),
            },
            Premise::Logical {
                logical: "shared".into(),
                version: Some("shared1".into()),
            },
            Premise::Logical {
                logical: "absent".into(),
                version: None,
            },
            Premise::Name {
                scope: "p".into(),
                name: "shared".into(),
                version: Some("shared1".into()),
            },
            Premise::Name {
                scope: "p".into(),
                name: "absent".into(),
                version: None,
            },
        ])
    }

    #[tokio::test]
    async fn generic_product_receipt_text_never_confers_scientific_class() {
        let store=fixture().await;
        let revision=store.edit("p",None,"producer-class-source",&[]).await.unwrap();
        let selected=read(&store,revision).await;
        for producer in ["pse.qualified-math-producer.v1:claimed","pse.local-runtime.v2:claimed","pse.local-runtime.v1:historical"] {
            let mut offered=product(&selected,b"generic native product");
            offered.producer=producer.into();
            assert_eq!(offered.producer_class,CanonicalProducerClass::General);
            store.publish_product(&selected,offered).await.unwrap();
            assert!(store.product_candidate(selected.selection(),b"exact-request",producer,CanonicalProducerClass::General,"").await.unwrap().is_some());
            for class in [CanonicalProducerClass::LibraryMathematics,CanonicalProducerClass::LocalScientificReplay] {
                assert!(!store.product_candidate_presence(selected.selection(),b"exact-request",producer,class,"").await.unwrap());
                assert!(store.product_candidate(selected.selection(),b"exact-request",producer,class,"").await.unwrap().is_none());
            }
        }
        store.release(selected.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_split_products_share_independent_parts_and_keyed_ack_never_revives_roots() {
        let store = fixture().await;
        let revision = store.edit("p", None, "split-products", &[]).await.unwrap();
        let selected = read(&store, revision.clone()).await;
        let material = SelectionMaterial::capture(&selected, std::sync::Arc::new(())).unwrap();
        let first = ProductDescription::prepare_with_selection(
            &selected,
            product(&selected, b"body-one"),
            material.clone(),
        )
        .unwrap();
        let second = ProductDescription::prepare_with_selection(
            &selected,
            product(&selected, b"body-two"),
            material,
        )
        .unwrap();
        let first_key = store.publish_description(&selected, &first).await.unwrap();
        let second_key = store.publish_description(&selected, &second).await.unwrap();
        let mut result=crate::canonical::bounded_query(store.db.query("SELECT * FROM canonical_body_contents; SELECT * FROM canonical_selection_receipts; SELECT * FROM canonical_product_associations;")).await.unwrap();
        assert_eq!(result.take::<Vec<Object>>(0).unwrap().len(), 2);
        assert_eq!(result.take::<Vec<Object>>(1).unwrap().len(), 1);
        assert_eq!(result.take::<Vec<Object>>(2).unwrap().len(), 2);
        assert_eq!(
            store.acknowledge_description(&first).await.unwrap(),
            Some(first_key.clone())
        );
        store
            .drop_retained_root(
                &revision,
                &crate::canonical_retention::RetentionOwner::Product(first_key.clone()),
            )
            .await
            .unwrap();
        assert_eq!(store.acknowledge_description(&first).await.unwrap(), None);
        assert_eq!(
            store.acknowledge_description(&second).await.unwrap(),
            Some(second_key)
        );
        // A new pin may create a new association, with the same immutable body
        // and receipt, while the original explicitly released root stays absent.
        let fresh = read(&store, revision).await;
        let current = SelectionMaterial::capture(&fresh, std::sync::Arc::new(())).unwrap();
        let binding = current
            .rebinding_from(first.selection_material(), &fresh)
            .unwrap()
            .unwrap();
        let rebound = first.rebind_selection(&fresh, &binding).unwrap();
        let new_key = store.publish_description(&fresh, &rebound).await.unwrap();
        assert_ne!(new_key, first_key);
        assert_eq!(
            store.acknowledge_description(&first).await.unwrap(),
            Some(new_key)
        );
        let old_root: Option<Object> =
            crate::canonical::request(store.db.select(("canonical_roots", first_key.as_str())))
                .await
                .unwrap();
        assert!(old_root.is_none());
        let mut result = crate::canonical::bounded_query(store.db.query(
            "SELECT * FROM canonical_body_contents; SELECT * FROM canonical_selection_receipts;",
        ))
        .await
        .unwrap();
        assert_eq!(result.take::<Vec<Object>>(0).unwrap().len(), 2);
        assert_eq!(result.take::<Vec<Object>>(1).unwrap().len(), 1);
        store.release(selected.selection()).await.unwrap();
        store.release(fresh.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn shared_selection_material_captures_merged_absence_and_refuses_stale_new_roots() {
        let store = fixture().await;
        let revision = store.edit("p", None, "shared-material", &[]).await.unwrap();
        let mut selected = read(&store, revision.clone()).await;
        let old = SelectionMaterial::capture(&selected, std::sync::Arc::new(())).unwrap();
        let acquired = candidate(vec![Premise::Logical {
            logical: "acquired-absence".into(),
            version: None,
        }]);
        let mut window =
            SelectedRecheckWindow::new(&selected, std::slice::from_ref(&acquired)).unwrap();
        assert_eq!(
            store
                .qualify_selection_window(&selected, &mut window)
                .await
                .unwrap(),
            Some(0)
        );
        window.merge_qualified(&mut selected, 0).unwrap();
        assert!(
            ProductDescription::prepare_with_selection(&selected, product(&selected, b"body"), old)
                .is_err()
        );
        let material = SelectionMaterial::capture(&selected, std::sync::Arc::new(())).unwrap();
        assert_eq!(
            material.selected_dependencies().dependencies,
            acquired.dependencies
        );
        assert_eq!(
            material.encoded,
            serde_json::to_vec(&selected.dependencies()).unwrap(),
            "the shared snapshot preserves the established dependency encoding"
        );
        let description = ProductDescription::prepare_with_selection(
            &selected,
            product(&selected, b"body"),
            material.clone(),
        )
        .unwrap();
        let offered = description.material.product.clone();
        let mut current = read(&store, revision).await;
        assert!(
            store
                .recheck_selection_dependencies(&mut current, material.selected_dependencies())
                .await
                .unwrap()
        );
        assert!(description.matches_read(&current).unwrap());
        assert!(
            store
                .publish_description(&current, &description)
                .await
                .is_err(),
            "a same-premises prior pin cannot mint a new root without current capture"
        );
        assert!(
            store
                .product_acknowledged(&offered)
                .await
                .unwrap()
                .is_none()
        );
        let current_material =
            SelectionMaterial::capture(&current, std::sync::Arc::new(())).unwrap();
        let binding = current_material
            .rebinding_from(&material, &current)
            .unwrap()
            .unwrap();
        let rebound = description.rebind_selection(&current, &binding).unwrap();
        assert_eq!(
            description.descriptor().as_ptr(),
            rebound.descriptor().as_ptr()
        );
        let key = store.publish_description(&current, &rebound).await.unwrap();
        assert_eq!(
            store.acknowledge_description(&description).await.unwrap(),
            Some(key)
        );
        let mut pages = store.scope_pages(&mut current, "empty").unwrap();
        assert!(SelectionMaterial::capture(&current, std::sync::Arc::new(())).is_err());
        assert!(
            ProductDescription::prepare_with_selection(
                &current,
                product(&current, b"body"),
                current_material.clone()
            )
            .is_err()
        );
        while !store
            .next_membership_page(&mut current, &mut pages)
            .await
            .unwrap()
            .is_empty()
        {}
        assert!(
            ProductDescription::prepare_with_selection(
                &current,
                product(&current, b"body"),
                current_material
            )
            .is_err()
        );
        // A completed cursor still owns a protection follower until dropped.
        drop(pages);
        selected.finish().await.unwrap();
        current.finish().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn selected_candidate_windows_share_positive_absence_rounds_and_reject_without_contamination()
     {
        let store = fixture().await;
        let revision = store
            .edit(
                "p",
                None,
                "window-points",
                &[
                    edit("Root", "root1", "definition", Vec::new()),
                    edit("shared", "shared1", "definition", Vec::new()),
                ],
            )
            .await
            .unwrap();
        let mut current = read(&store, revision.clone()).await;
        let before = current.snapshot_dependencies().unwrap();
        // E=1 and E=8, early/late/no match: each common positive and absent
        // logical key is resolved once, irrespective of candidate count.
        for (count, accepted) in [(1, Some(0)), (8, Some(0)), (8, Some(7)), (8, None)] {
            let candidates = (0..count)
                .map(|index| {
                    point_candidate(if accepted == Some(index) {
                        "root1"
                    } else {
                        "wrong"
                    })
                })
                .collect::<Vec<_>>();
            let mut window = SelectedRecheckWindow::new(&current, &candidates).unwrap();
            assert!(
                SelectedRecheckWindow::scratch_bytes(&current, &candidates).unwrap()
                    >= 4096 * 8 * 3
            );
            assert_eq!(
                store
                    .qualify_selection_window(&current, &mut window)
                    .await
                    .unwrap(),
                accepted
            );
            assert_eq!(window.counts.logicals, 1);
            assert_eq!(window.counts.names, usize::from(accepted.is_some()));
            assert_eq!(window.counts.inventory_pages, 0);
            assert_eq!(current.snapshot_dependencies().unwrap(), before);
            assert_eq!(
                store
                    .qualify_selection_window(&current, &mut window)
                    .await
                    .unwrap(),
                accepted
            );
            assert_eq!(
                window.counts.guards, 2,
                "cached searches still guard the live pin"
            );
            assert_eq!(window.counts.logicals, 1);
            assert_eq!(window.counts.names, usize::from(accepted.is_some()));
            if let Some(index) = accepted {
                window.merge_qualified(&mut current, index).unwrap();
                // Candidate premise order is immaterial to qualification; the
                // selected snapshot emits these name/logical keys in map order.
                let mut expected = candidates[index].dependencies.clone();
                expected.premises.sort();
                assert_eq!(
                    current.snapshot_dependencies().unwrap().dependencies,
                    expected
                );
                current.finish().await.unwrap();
                current = read(&store, revision.clone()).await;
            } else {
                assert!(window.merge_qualified(&mut current, 0).is_err());
            }
        }
        // An observed presence with the maximum admitted version length must fit
        // scratch issued from a previously absent premise, and reject that premise.
        let large = "v".repeat(4096);
        let changed = store
            .edit(
                "p",
                Some("window-points"),
                "window-adverse",
                &[edit("absent", &large, "definition", Vec::new())],
            )
            .await
            .unwrap();
        let adverse = read(&store, changed).await;
        let candidates = [candidate(vec![Premise::Logical {
            logical: "absent".into(),
            version: None,
        }])];
        let mut window = SelectedRecheckWindow::new(&adverse, &candidates).unwrap();
        assert_eq!(
            store
                .qualify_selection_window(&adverse, &mut window)
                .await
                .unwrap(),
            None
        );
        assert!(adverse.logicals.is_empty());
        adverse.finish().await.unwrap();
        current.finish().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn selected_candidate_windows_memo_exact_inventory_expectations_and_merge_only_winner() {
        let store = fixture().await;
        let revision = store
            .edit(
                "p",
                None,
                "window-inventories",
                &[edit("Root", "root1", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let mut current = read(&store, revision).await;
        let rejected = candidate(vec![
            Premise::Logical {
                logical: "rejected-only-absence".into(),
                version: None,
            },
            Premise::Scope {
                scope: "p".into(),
                members: Vec::new(),
            },
        ]);
        let winner = candidate(vec![
            Premise::Scope {
                scope: "p".into(),
                members: vec![("Root".into(), "root1".into())],
            },
            Premise::KindScope {
                scope: Some("p".into()),
                source_kind: "missing-kind".into(),
                members: Vec::new(),
            },
            Premise::References {
                scope: "p".into(),
                name: "Root".into(),
                source_kind: "definition".into(),
                members: Vec::new(),
            },
        ]);
        let mut candidates = vec![rejected; 7];
        candidates.push(winner.clone());
        let mut window = SelectedRecheckWindow::new(&current, &candidates).unwrap();
        assert_eq!(
            store
                .qualify_selection_window(&current, &mut window)
                .await
                .unwrap(),
            Some(7)
        );
        assert_eq!(window.counts.logicals, 1);
        // One rejecting scope page, two complete accepting scope pages, and one
        // terminal page each for absent kind and absent inverse references.
        assert_eq!(window.counts.inventory_pages, 5);
        assert!(
            current
                .snapshot_dependencies()
                .unwrap()
                .dependencies
                .premises
                .is_empty()
        );
        assert!(window.merge_qualified(&mut current, 0).is_err());
        window.merge_qualified(&mut current, 7).unwrap();
        assert_eq!(
            current.snapshot_dependencies().unwrap().dependencies,
            winner.dependencies
        );
        assert!(!current.logicals.contains_key("rejected-only-absence"));
        assert_eq!(
            store
                .qualify_selection_window(&current, &mut window)
                .await
                .unwrap(),
            Some(7)
        );
        assert_eq!(window.counts.inventory_pages, 5);
        current.finish().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn selected_candidate_windows_bound_each_round_and_stop_after_early_mismatch() {
        let store = fixture().await;
        let revision = store
            .edit(
                "p",
                None,
                "window-rounds",
                &[edit("key000", "present", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let current = read(&store, revision).await;
        let logicals = (0..300)
            .map(|index| Premise::Logical {
                logical: format!("key{index:03}"),
                version: if index == 0 {
                    Some("present".into())
                } else {
                    None
                },
            })
            .collect::<Vec<_>>();
        let names = (0..65)
            .map(|index| Premise::Name {
                scope: "p".into(),
                name: format!("missing{index:03}"),
                version: None,
            })
            .collect::<Vec<_>>();
        let all = candidate(logicals.iter().chain(&names).cloned().collect());
        let candidates = vec![all; 8];
        let mut window = SelectedRecheckWindow::new(&current, &candidates).unwrap();
        assert_eq!(
            store
                .qualify_selection_window(&current, &mut window)
                .await
                .unwrap(),
            Some(0)
        );
        assert_eq!(
            window.counts.logicals, 2,
            "300 shared keys require two bounded rounds"
        );
        assert_eq!(
            window.counts.names, 2,
            "65 shared sparse pairs require two bounded rounds"
        );
        let mut rejected = logicals;
        rejected[0] = Premise::Logical {
            logical: "key000".into(),
            version: None,
        };
        rejected.extend(names);
        rejected.push(Premise::Scope {
            scope: "p".into(),
            members: Vec::new(),
        });
        let candidates = vec![candidate(rejected); 8];
        let mut window = SelectedRecheckWindow::new(&current, &candidates).unwrap();
        assert_eq!(
            store
                .qualify_selection_window(&current, &mut window)
                .await
                .unwrap(),
            None
        );
        assert_eq!(window.counts.logicals, 1);
        assert_eq!(window.counts.names, 0);
        assert_eq!(window.counts.inventory_pages, 0);
        assert!(
            current
                .snapshot_dependencies()
                .unwrap()
                .dependencies
                .premises
                .is_empty()
        );
        current.finish().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn selected_candidate_window_observations_require_exact_live_epoch() {
        let store = fixture().await;
        let revision = store.edit("p", None, "window-epoch", &[]).await.unwrap();
        let current = read(&store, revision.clone()).await;
        let candidates = [candidate(vec![Premise::Logical {
            logical: "absent".into(),
            version: None,
        }])];
        let mut window = SelectedRecheckWindow::new(&current, &candidates).unwrap();
        assert_eq!(
            store
                .qualify_selection_window(&current, &mut window)
                .await
                .unwrap(),
            Some(0)
        );
        let other = read(&store, revision.clone()).await;
        assert!(!window.matches(&other, &candidates));
        assert!(
            store
                .qualify_selection_window(&other, &mut window)
                .await
                .is_err()
        );
        store.release(current.selection()).await.unwrap();
        assert!(
            store
                .qualify_selection_window(&current, &mut window)
                .await
                .is_err()
        );
        let mut expired = SelectedRecheckWindow::new(&other, &candidates).unwrap();
        assert_eq!(
            store
                .qualify_selection_window(&other, &mut expired)
                .await
                .unwrap(),
            Some(0)
        );
        crate::canonical::bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_protections', $pin) SET expires_at = 0;")
                .bind(("pin", other.selection().key().to_owned())),
        )
        .await
        .unwrap();
        assert!(
            store
                .qualify_selection_window(&other, &mut expired)
                .await
                .is_err()
        );
        let empty_candidates = [candidate(Vec::new())];
        let mut empty = SelectedRecheckWindow::new(&other, &empty_candidates).unwrap();
        assert!(
            store
                .qualify_selection_window(&other, &mut empty)
                .await
                .is_err()
        );
        let incompatible = [candidate(vec![Premise::Interpretation {
            role: "physical".into(),
            identity: "foreign".into(),
        }])];
        let mut mismatch = SelectedRecheckWindow::new(&other, &incompatible).unwrap();
        assert_eq!(
            store
                .qualify_selection_window(&other, &mut mismatch)
                .await
                .unwrap(),
            None
        );
        assert_eq!(mismatch.counts, RecheckCounts::default());
        store.release(other.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn selected_candidate_windows_refuse_new_scope_kind_and_inverse_reference_members() {
        let store = fixture().await;
        let first = store
            .edit(
                "p",
                None,
                "window-namespaces-a",
                &[edit("Root", "root1", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let source = read(&store, first.clone()).await;
        let absent = [
            candidate(vec![Premise::Scope {
                scope: "empty".into(),
                members: Vec::new(),
            }]),
            candidate(vec![Premise::KindScope {
                scope: Some("p".into()),
                source_kind: "new-kind".into(),
                members: Vec::new(),
            }]),
            candidate(vec![Premise::References {
                scope: "p".into(),
                name: "Root".into(),
                source_kind: "new-kind".into(),
                members: Vec::new(),
            }]),
        ];
        for dependency in &absent {
            let mut window =
                SelectedRecheckWindow::new(&source, std::slice::from_ref(dependency)).unwrap();
            assert_eq!(
                store
                    .qualify_selection_window(&source, &mut window)
                    .await
                    .unwrap(),
                Some(0)
            );
        }
        let mut extra = edit("Extra", "extra1", "definition", Vec::new());
        extra.scope = "empty".into();
        let changed = store
            .edit(
                "p",
                Some("window-namespaces-a"),
                "window-namespaces-b",
                &[
                    extra,
                    edit(
                        "Supplier",
                        "supplier1",
                        "new-kind",
                        vec![("p".into(), "Root".into())],
                    ),
                ],
            )
            .await
            .unwrap();
        let current = read(&store, changed).await;
        for dependency in absent {
            let mut window = SelectedRecheckWindow::new(&current, &[dependency]).unwrap();
            assert_eq!(
                store
                    .qualify_selection_window(&current, &mut window)
                    .await
                    .unwrap(),
                None
            );
            assert!(
                current
                    .snapshot_dependencies()
                    .unwrap()
                    .dependencies
                    .premises
                    .is_empty()
            );
        }
        source.finish().await.unwrap();
        current.finish().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn same_actual_revision_receipt_requires_live_pin_and_preserves_changed_inventory_refusal()
     {
        let store = fixture().await;
        let revision = store
            .edit(
                "p",
                None,
                "receipt-first",
                &[edit("Root", "root1", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let mut first = read(&store, revision.clone()).await;
        store.resolve_scope(&mut first, "p").await.unwrap();
        store
            .resolve_logicals(&mut first, &["missing".into()])
            .await
            .unwrap();
        first
            .interpretation("physical".into(), "exact-provider".into())
            .unwrap();
        let scratch = first.snapshot_receipt_scratch_bytes().unwrap();
        let receipt = first.snapshot_receipt().unwrap();
        assert_eq!(receipt.retained_bytes(), scratch);
        store.release(first.selection()).await.unwrap();
        let mut second = read(&store, revision.clone()).await;
        assert!(
            store
                .adopt_dependencies_from_same_revision(&mut second, &receipt)
                .await
                .unwrap()
        );
        assert_eq!(
            second.snapshot_dependencies().unwrap(),
            **receipt.dependencies()
        );
        let accepted = second.snapshot_dependencies().unwrap();
        store.release(second.selection()).await.unwrap();
        assert!(
            store
                .adopt_dependencies_from_same_revision(&mut second, &receipt)
                .await
                .is_err(),
            "a cloned released selection cannot adopt qualified premises"
        );
        assert_eq!(second.snapshot_dependencies().unwrap(), accepted);
        let mut expired = read(&store, revision.clone()).await;
        crate::canonical::bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_protections', $pin) SET expires_at = 0;")
                .bind(("pin", expired.selection().key().to_owned())),
        )
        .await
        .unwrap();
        assert!(
            store
                .adopt_dependencies_from_same_revision(&mut expired, &receipt)
                .await
                .is_err(),
            "an expired new pin cannot inherit an old qualification"
        );
        assert!(
            expired
                .snapshot_dependencies()
                .unwrap()
                .dependencies
                .premises
                .is_empty()
        );
        store.release(expired.selection()).await.unwrap();
        let changed = store
            .edit(
                "p",
                Some("receipt-first"),
                "receipt-second",
                &[edit("Other", "other1", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let mut current = read(&store, changed).await;
        let before = current.snapshot_dependencies().unwrap();
        assert!(
            !store
                .adopt_dependencies_from_same_revision(&mut current, &receipt)
                .await
                .unwrap()
        );
        assert_eq!(current.snapshot_dependencies().unwrap(), before);
        assert!(
            !store
                .recheck_selection_dependencies(&mut current, receipt.dependencies())
                .await
                .unwrap(),
            "the changed complete scope must still reject through full qualification"
        );
        assert_eq!(current.snapshot_dependencies().unwrap(), before);
        store.release(current.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn interpretation_mismatch_rejects_before_source_acquisition_without_reuse_authority() {
        let store = fixture().await;
        let revision = store.edit("p", None, "cheap-rejection", &[]).await.unwrap();
        let mut original = read(&store, revision.clone()).await;
        store
            .resolve_logicals(&mut original, &["absent".into()])
            .await
            .unwrap();
        original
            .interpretation("physical".into(), "original".into())
            .unwrap();
        let dependencies = original.snapshot_dependencies().unwrap();
        let mut incompatible = read(&store, revision.clone()).await;
        incompatible
            .interpretation("physical".into(), "changed".into())
            .unwrap();
        store.release(incompatible.selection()).await.unwrap();
        // The expired selection cannot acquire source. An already established
        // interpretation mismatch still refuses immediately and grants no premises.
        assert!(
            !store
                .recheck_selection_dependencies(&mut incompatible, &dependencies)
                .await
                .unwrap()
        );
        assert!(incompatible.logicals.is_empty());
        let mut compatible = read(&store, revision).await;
        compatible
            .interpretation("physical".into(), "original".into())
            .unwrap();
        store.release(compatible.selection()).await.unwrap();
        assert!(
            store
                .recheck_selection_dependencies(&mut compatible, &dependencies)
                .await
                .is_err(),
            "a matching interpretation still needs actual live source authority"
        );
        store.release(original.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn sixteen_same_problem_protected_readers_share_short_turns_with_staging_and_exact_ack() {
        let store = fixture().await;
        let revision = store
            .edit(
                "p",
                None,
                "paced-first",
                &[edit("Root", "root1", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let mut seed = read(&store, revision.clone()).await;
        store
            .resolve_logicals(&mut seed, &["Root".into(), "missing".into()])
            .await
            .unwrap();
        let description = store
            .prepare_description(&seed, product(&seed, b"complete body"))
            .unwrap();
        let key = store
            .publish_description(&seed, &description)
            .await
            .unwrap();
        store.release(seed.selection()).await.unwrap();
        let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(17));
        let mut jobs = tokio::task::JoinSet::new();
        let writer = store.clone();
        let writer_barrier = barrier.clone();
        jobs.spawn(async move {
            writer_barrier.wait().await;
            writer
                .edit(
                    "p",
                    Some("paced-first"),
                    "paced-second",
                    &[edit("Other", "other1", "definition", Vec::new())],
                )
                .await
                .unwrap();
        });
        for _ in 0..16 {
            let store = store.clone();
            let revision = revision.clone();
            let barrier = barrier.clone();
            let description = description.clone();
            let key = key.clone();
            jobs.spawn(async move {
                barrier.wait().await;
                for _ in 0..2 {
                    let mut selected = read(&store, revision.clone()).await;
                    store
                        .resolve_logicals(&mut selected, &["Root".into(), "missing".into()])
                        .await
                        .unwrap();
                    assert!(description.matches_read(&selected).unwrap());
                    assert_eq!(
                        store
                            .assemble_selected_object(selected.selection(), "root1")
                            .await
                            .unwrap()
                            .unwrap()
                            .payload
                            .as_slice(),
                        b"Root"
                    );
                    let receipt = selected.snapshot_receipt().unwrap();
                    let mut adopted = read(&store, revision.clone()).await;
                    assert!(
                        store
                            .adopt_dependencies_from_same_revision(&mut adopted, &receipt)
                            .await
                            .unwrap()
                    );
                    assert_eq!(
                        adopted.snapshot_dependencies().unwrap(),
                        **receipt.dependencies()
                    );
                    let mut qualified = read(&store, revision.clone()).await;
                    assert!(
                        store
                            .recheck_selection_dependencies(&mut qualified, receipt.dependencies())
                            .await
                            .unwrap()
                    );
                    assert_eq!(
                        store.acknowledge_description(&description).await.unwrap(),
                        Some(key.clone())
                    );
                    assert_eq!(
                        store
                            .test_reuse_product(
                                &mut qualified,
                                b"exact-request",
                                "qualified-fixture",
                                ""
                            )
                            .await
                            .unwrap()
                            .unwrap()
                            .payload(),
                        b"complete body"
                    );
                    store.release(selected.selection()).await.unwrap();
                    assert!(
                        store
                            .resolve_logicals(&mut selected, &["Root".into()])
                            .await
                            .is_err()
                    );
                    store.release(adopted.selection()).await.unwrap();
                    store.release(qualified.selection()).await.unwrap();
                }
            });
        }
        while let Some(result) = jobs.join_next().await {
            result.unwrap();
        }
        assert_eq!(
            store.acknowledge_description(&description).await.unwrap(),
            Some(key)
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    async fn current_head(store: &CanonicalStore) -> String {
        let row: Option<Object> =
            crate::canonical::request(store.db.select(("canonical_problems", "p")))
                .await
                .unwrap();
        wire::decode_canonical_problems(row.unwrap()).unwrap().head
    }
    fn stored_product(mut product: Product) -> Product {
        let descriptor = crate::canonical_staging::ProductBlob::from_parts(
            product.payload.as_slice(),
            product.dependencies.as_slice(),
            &product.producer,
        )
        .unwrap();
        product.payload = serde_json::to_vec(&descriptor).unwrap().into();
        product.dependencies = Vec::new().into();
        product
    }

    #[tokio::test]
    async fn scoped_head_pages_preserve_selected_revision_in_both_head_change_orders() {
        for read_commits_first in [false, true] {
            let store = fixture().await;
            let edits = (0..70)
                .map(|index| {
                    edit(
                        &format!("member-{index:03}"),
                        &format!("v-{index}"),
                        "definition",
                        vec![],
                    )
                })
                .collect::<Vec<_>>();
            let revision = store.edit("p", None, "head-before", &edits).await.unwrap();
            let mut selected = read(&store, revision.clone()).await;
            let mut cursor = store.scope_pages(&mut selected, "p").unwrap();
            let mut observed = Vec::new();
            if read_commits_first {
                observed.extend(
                    store
                        .next_membership_page(&mut selected, &mut cursor)
                        .await
                        .unwrap(),
                );
                assert_eq!(observed.len(), 64);
            }
            // Replace every member: historical pages must never pick a single
            // new-head payload, including when a cursor already consumed a page.
            let replacements = (0..70)
                .map(|index| {
                    edit(
                        &format!("member-{index:03}"),
                        &format!("new-v-{index}"),
                        "definition",
                        vec![],
                    )
                })
                .collect::<Vec<_>>();
            let current = store
                .edit("p", Some(&revision.key), "head-after", &replacements)
                .await
                .unwrap();
            loop {
                let page = store
                    .next_membership_page(&mut selected, &mut cursor)
                    .await
                    .unwrap();
                if page.is_empty() {
                    break;
                }
                observed.extend(page);
            }
            assert_eq!(observed.len(), 70);
            assert!(
                observed
                    .iter()
                    .all(|member| member.version.starts_with("v-"))
            );
            assert!(selected.dependency_bytes().is_ok());
            let historical = store
                .selection_membership_page(selected.selection(), Some("p"), None, None, "")
                .await
                .unwrap();
            assert_eq!(historical.len(), 64);
            assert!(
                historical
                    .iter()
                    .all(|member| member.version.starts_with("v-"))
            );
            let fresh = read(&store, current).await;
            let scoped = store
                .membership_page(fresh.selection(), Some("p"), "")
                .await
                .unwrap();
            let interval_oracle = store
                .membership_page(fresh.selection(), None, "")
                .await
                .unwrap();
            assert_eq!(
                scoped
                    .iter()
                    .map(|row| (&row.key, &row.version))
                    .collect::<Vec<_>>(),
                interval_oracle
                    .iter()
                    .map(|row| (&row.key, &row.version))
                    .collect::<Vec<_>>()
            );
            assert!(
                scoped
                    .iter()
                    .all(|member| member.version.starts_with("new-v-"))
            );
            store.release(selected.selection()).await.unwrap();
            assert!(
                store
                    .membership_page(selected.selection(), Some("p"), "")
                    .await
                    .is_err()
            );
            store.release(fresh.selection()).await.unwrap();
            store.remove_isolated_fixture().await.unwrap();
        }
    }

    #[tokio::test]
    async fn paged_selected_inventory_closes_only_after_protected_terminal_page() {
        let store = fixture().await;
        let edits = (0..130)
            .map(|index| {
                edit(
                    &format!("member-{index:03}"),
                    &format!("v-{index}"),
                    "definition",
                    if index == 0 {
                        vec![("p".into(), "target".into())]
                    } else {
                        vec![]
                    },
                )
            })
            .collect::<Vec<_>>();
        let revision = store.edit("p", None, "source", &edits).await.unwrap();
        let mut selected = read(&store, revision.clone()).await;
        let mut cursor = store.scope_pages(&mut selected, "p").unwrap();
        assert_eq!(
            store
                .next_membership_page(&mut selected, &mut cursor)
                .await
                .unwrap()
                .len(),
            64
        );
        assert!(selected.scopes.is_empty());
        assert!(selected.dependency_bytes().is_err());
        assert!(selected.publication_scratch_bytes().is_err());
        assert!(
            store
                .publish_product(&selected, product(&selected, b"partial"))
                .await
                .is_err()
        );
        let mut other_pin = read(&store, revision.clone()).await;
        assert!(
            store
                .next_membership_page(&mut other_pin, &mut cursor)
                .await
                .is_err(),
            "cursor cannot be substituted into another protected read"
        );
        assert_eq!(
            store
                .next_membership_page(&mut selected, &mut cursor)
                .await
                .unwrap()
                .len(),
            64
        );
        assert_eq!(
            store
                .next_membership_page(&mut selected, &mut cursor)
                .await
                .unwrap()
                .len(),
            2
        );
        assert!(
            selected.dependency_bytes().is_err(),
            "a short page is not yet the protected terminal page"
        );
        assert!(
            store
                .next_membership_page(&mut selected, &mut cursor)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(selected.scopes["p"].len(), 130);
        assert!(selected.dependency_bytes().is_ok());
        assert!(
            store
                .next_membership_page(&mut selected, &mut cursor)
                .await
                .is_err()
        );
        let key = store
            .publish_product(&selected, product(&selected, b"complete"))
            .await
            .unwrap();
        assert_eq!(
            store
                .test_reuse_product(&mut other_pin, b"exact-request", "qualified-fixture", "")
                .await
                .unwrap()
                .unwrap()
                .key(),
            key
        );
        let mut kinds = store
            .kind_pages(&mut selected, Some("p"), "absent-kind")
            .unwrap();
        assert!(
            store
                .next_membership_page(&mut selected, &mut kinds)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            selected
                .kinds
                .get(&(Some("p".into()), "absent-kind".into())),
            Some(&vec![])
        );
        let mut suppliers = store
            .reference_pages(&mut selected, "p", "target", "definition")
            .unwrap();
        assert_eq!(
            store
                .next_membership_page(&mut selected, &mut suppliers)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(selected.dependency_bytes().is_err());
        assert!(
            store
                .next_membership_page(&mut selected, &mut suppliers)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            selected.references[&("p".into(), "target".into(), "definition".into())],
            [("member-000".into(), "v-0".into())]
        );
        let mut grouped = read(&store, revision.clone()).await;
        let mut cursors = vec![
            store.scope_pages(&mut grouped, "p").unwrap(),
            store
                .kind_pages(&mut grouped, Some("p"), "absent-kind")
                .unwrap(),
        ];
        let pages = store
            .next_membership_pages(&mut grouped, &mut cursors)
            .await
            .unwrap();
        assert_eq!(
            pages.iter().map(Vec::len).sum::<usize>(),
            32,
            "group responses share the fixed 64-row budget"
        );
        assert!(pages[1].is_empty());
        cursors.remove(1);
        let mut count = pages[0].len();
        loop {
            let pages = store
                .next_membership_pages(&mut grouped, &mut cursors)
                .await
                .unwrap();
            if pages[0].is_empty() {
                break;
            }
            count += pages[0].len();
        }
        assert_eq!(count, 130);
        assert!(grouped.dependency_bytes().is_ok());
        let mut abandoned = read(&store, revision.clone()).await;
        let mut dropped = store.scope_pages(&mut abandoned, "p").unwrap();
        assert_eq!(
            store
                .next_membership_page(&mut abandoned, &mut dropped)
                .await
                .unwrap()
                .len(),
            64
        );
        drop(dropped);
        assert!(
            abandoned.dependency_bytes().is_err(),
            "cancelled/dropped inventory needs a fresh read"
        );
        let mut expired = read(&store, revision.clone()).await;
        let mut empty = store.scope_pages(&mut expired, "missing-scope").unwrap();
        crate::canonical::bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_protections', $pin) SET expires_at = 0;")
                .bind(("pin", expired.selection().key().to_owned())),
        )
        .await
        .unwrap();
        assert!(
            store
                .next_membership_page(&mut expired, &mut empty)
                .await
                .is_err()
        );
        assert!(expired.scopes.is_empty());
        assert!(expired.dependency_bytes().is_err());
        for read in [&selected, &other_pin, &grouped, &abandoned, &expired] {
            store.release(read.selection()).await.unwrap();
        }
    }
    #[tokio::test]
    async fn grouped_lexical_pairs_preserve_sparse_selection_and_absence() {
        let store = fixture().await;
        let edits = [("left", "a"), ("left", "b"), ("right", "a"), ("right", "b")]
            .into_iter()
            .map(|(scope, name)| {
                let identity = format!("{scope}-{name}");
                let mut row = edit(&identity, &format!("v-{identity}"), "definition", vec![]);
                row.scope = scope.into();
                row.name = name.into();
                row
            })
            .collect::<Vec<_>>();
        let revision = store.edit("p", None, "pair-source", &edits).await.unwrap();
        let mut selected = read(&store, revision).await;
        let pairs = [
            ("left".into(), "a".into()),
            ("right".into(), "b".into()),
            ("right".into(), "absent".into()),
        ];
        let members = store
            .resolve_name_pairs(&mut selected, &pairs)
            .await
            .unwrap();
        let names = members
            .iter()
            .map(|row| row.logical.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names, ["left-a", "right-b"].into_iter().collect());
        assert_eq!(selected.names[&pairs[0]], Some("v-left-a".into()));
        assert_eq!(selected.names[&pairs[1]], Some("v-right-b".into()));
        assert_eq!(selected.names[&pairs[2]], None);
        assert_eq!(
            selected.names.len(),
            3,
            "cross pairs were never read premises"
        );
        store.release(selected.selection()).await.unwrap();
    }

    #[tokio::test]
    async fn grouped_inventory_preserves_exact_supplier_pairs_and_terminal_premises() {
        let store = fixture().await;
        let edits = [
            edit(
                "left-a",
                "v-left-a",
                "dataset",
                vec![("left".into(), "a".into())],
            ),
            edit(
                "right-b",
                "v-right-b",
                "dataset",
                vec![("right".into(), "b".into())],
            ),
            edit(
                "left-b-cross",
                "v-cross-left",
                "dataset",
                vec![("left".into(), "b".into())],
            ),
            edit(
                "right-a-cross",
                "v-cross-right",
                "dataset",
                vec![("right".into(), "a".into())],
            ),
        ];
        let revision = store
            .edit("p", None, "supplier-source", &edits)
            .await
            .unwrap();
        let mut selected = read(&store, revision.clone()).await;
        let mut cursors = vec![
            store
                .reference_pages(&mut selected, "left", "a", "dataset")
                .unwrap(),
            store
                .reference_pages(&mut selected, "right", "b", "dataset")
                .unwrap(),
            store
                .reference_pages(&mut selected, "missing", "a", "dataset")
                .unwrap(),
        ];
        let pages = store
            .next_membership_pages(&mut selected, &mut cursors)
            .await
            .unwrap();
        assert_eq!(
            pages[0]
                .iter()
                .map(|row| row.logical.as_str())
                .collect::<Vec<_>>(),
            ["left-a"]
        );
        assert_eq!(
            pages[1]
                .iter()
                .map(|row| row.logical.as_str())
                .collect::<Vec<_>>(),
            ["right-b"]
        );
        assert!(pages[2].is_empty());
        assert_eq!(
            selected.references[&("missing".into(), "a".into(), "dataset".into())],
            []
        );
        assert!(
            selected.dependency_bytes().is_err(),
            "nonempty inventories remain provisional until terminal page"
        );
        cursors.pop();
        assert!(
            store
                .next_membership_pages(&mut selected, &mut cursors)
                .await
                .unwrap()
                .iter()
                .all(Vec::is_empty)
        );
        assert_eq!(
            selected.references[&("left".into(), "a".into(), "dataset".into())],
            [("left-a".into(), "v-left-a".into())]
        );
        assert_eq!(
            selected.references[&("right".into(), "b".into(), "dataset".into())],
            [("right-b".into(), "v-right-b".into())]
        );
        assert!(selected.dependency_bytes().is_ok());
        let mut other = read(&store, revision).await;
        assert!(
            store
                .next_membership_pages(&mut other, &mut cursors)
                .await
                .is_err(),
            "grouped cursors cannot change protection"
        );
        for read in [&selected, &other] {
            store.release(read.selection()).await.unwrap();
        }
    }

    #[tokio::test]
    async fn inverse_supplier_set_preserves_duplicate_edges_pages_and_revision_premises() {
        let store = fixture().await;
        let mut edits = (0..130)
            .map(|index| {
                edit(
                    &format!("supplier-{index:03}"),
                    &format!("supplier-v-{index}"),
                    "dataset",
                    vec![("target".into(), "x".into()), ("target".into(), "x".into())],
                )
            })
            .collect::<Vec<_>>();
        edits.push(edit(
            "wrong-kind",
            "wrong-v",
            "definition",
            vec![("target".into(), "x".into())],
        ));
        let original = store
            .edit("p", None, "inverse-original", &edits)
            .await
            .unwrap();
        store
            .edit(
                "foreign",
                None,
                "inverse-foreign",
                &[edit(
                    "foreign",
                    "foreign-v",
                    "dataset",
                    vec![("target".into(), "x".into())],
                )],
            )
            .await
            .unwrap();
        let mut selected = read(&store, original.clone()).await;
        let mut cursor = store
            .reference_pages(&mut selected, "target", "x", "dataset")
            .unwrap();
        let mut found = std::collections::BTreeSet::new();
        for expected in [64, 64, 2, 0] {
            let page = store
                .next_membership_page(&mut selected, &mut cursor)
                .await
                .unwrap();
            assert_eq!(page.len(), expected);
            for member in page {
                assert!(found.insert((member.logical, member.version)));
            }
        }
        let expected = (0..130)
            .map(|index| {
                (
                    format!("supplier-{index:03}"),
                    format!("supplier-v-{index}"),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            found, expected,
            "duplicate edges and foreign suppliers cannot duplicate or widen membership"
        );
        let dependencies = selected.snapshot_dependencies().unwrap();
        let mut same = read(&store, original.clone()).await;
        assert!(
            store
                .recheck_selection_dependencies(&mut same, &dependencies)
                .await
                .unwrap(),
            "requalification uses the same complete paged supplier meaning"
        );
        let mut grouped = read(&store, original).await;
        let mut cursors = [
            store
                .reference_pages(&mut grouped, "target", "absent", "dataset")
                .unwrap(),
            store
                .reference_pages(&mut grouped, "target", "x", "definition")
                .unwrap(),
        ];
        let pages = store
            .next_membership_pages(&mut grouped, &mut cursors)
            .await
            .unwrap();
        assert!(pages[0].is_empty());
        assert_eq!(
            pages[1]
                .iter()
                .map(|member| member.logical.as_str())
                .collect::<Vec<_>>(),
            ["wrong-kind"]
        );
        assert!(
            store
                .next_membership_page(&mut grouped, &mut cursors[1])
                .await
                .unwrap()
                .is_empty()
        );
        assert!(grouped.dependency_bytes().is_ok());
        let mut removed = edit("supplier-000", "unused", "dataset", vec![]);
        removed.version = None;
        let current = store
            .edit("p", Some("inverse-original"), "inverse-removed", &[removed])
            .await
            .unwrap();
        let mut changed = read(&store, current).await;
        assert!(
            !store
                .recheck_selection_dependencies(&mut changed, &dependencies)
                .await
                .unwrap()
        );
        assert_eq!(
            selected.references[&("target".into(), "x".into(), "dataset".into())].len(),
            130,
            "later removal does not change the protected historical inventory"
        );
        for read in [&selected, &same, &grouped, &changed] {
            store.release(read.selection()).await.unwrap();
        }
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn scoped_inventory_preserves_exact_and_absent_premises_under_skew() {
        let store = fixture().await;
        let mut edits = (0..256)
            .map(|index| {
                edit(
                    &format!("common-{index:03}"),
                    &format!("common-v-{index}"),
                    "definition",
                    vec![],
                )
            })
            .collect::<Vec<_>>();
        let mut rare = edit(
            "rare",
            "rare-v",
            "dataset",
            vec![("target".into(), "x".into())],
        );
        rare.scope = "rare".into();
        edits.push(rare);
        let revision = store.edit("p", None, "scoped-skew", &edits).await.unwrap();
        let mut selected = read(&store, revision).await;
        let scope = store
            .membership_page(selected.selection(), Some("rare"), "")
            .await
            .unwrap();
        assert_eq!(
            scope
                .iter()
                .map(|row| row.version.as_str())
                .collect::<Vec<_>>(),
            ["rare-v"]
        );
        assert!(
            store
                .membership_page(selected.selection(), Some("absent"), "")
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            store
                .membership_page(selected.selection(), None, "")
                .await
                .unwrap()
                .len(),
            64
        );
        let mut cursors = [
            store.scope_pages(&mut selected, "rare").unwrap(),
            store
                .kind_pages(&mut selected, Some("rare"), "definition")
                .unwrap(),
            store
                .kind_pages(&mut selected, Some("rare"), "dataset")
                .unwrap(),
            store
                .reference_pages(&mut selected, "target", "x", "dataset")
                .unwrap(),
        ];
        let pages = store
            .next_membership_pages(&mut selected, &mut cursors)
            .await
            .unwrap();
        assert_eq!(pages.iter().map(Vec::len).collect::<Vec<_>>(), [1, 0, 1, 1]);
        for index in [0, 2, 3] {
            assert!(
                store
                    .next_membership_page(&mut selected, &mut cursors[index])
                    .await
                    .unwrap()
                    .is_empty()
            );
        }
        assert_eq!(selected.scopes["rare"], [("rare".into(), "rare-v".into())]);
        assert!(selected.kinds[&(Some("rare".into()), "definition".into())].is_empty());
        assert_eq!(
            selected.references[&("target".into(), "x".into(), "dataset".into())],
            [("rare".into(), "rare-v".into())]
        );
        assert!(selected.dependency_bytes().is_ok());
        store.release(selected.selection()).await.unwrap();
    }

    #[tokio::test]
    async fn bounded_qualification_rejects_grown_wide_scope_and_merges_only_matching_delta() {
        let store = fixture().await;
        let initial = store.edit("p", None, "empty", &[]).await.unwrap();
        let mut old = read(&store, initial.clone()).await;
        assert!(store.resolve_scope(&mut old, "p").await.unwrap().is_empty());
        let key = store
            .publish_product(&old, product(&old, b"small recipe"))
            .await
            .unwrap();
        let candidate = store
            .product_candidate(old.selection(), b"exact-request", "qualified-fixture", CanonicalProducerClass::General, "")
            .await
            .unwrap()
            .unwrap();
        assert!(
            candidate.retained_bytes() >= 64 * 1024 * 1024,
            "old empty meaning still precharges one full native page"
        );
        // A successful delta keeps already recorded independent meaning.
        let mut same = read(&store, initial).await;
        same.interpretation("existing".into(), "kept".into())
            .unwrap();
        assert_eq!(
            store
                .qualify_product(&mut same, &candidate)
                .await
                .unwrap()
                .unwrap()
                .key(),
            key
        );
        assert_eq!(
            same.interpretations.get("existing").map(String::as_str),
            Some("kept")
        );
        assert_eq!(same.scopes.get("p"), Some(&vec![]));
        let mut contradiction = old.clone();
        contradiction
            .scopes
            .insert("p".into(), vec![("claimed".into(), "version".into())]);
        assert!(
            store
                .qualify_product(&mut contradiction, &candidate)
                .await
                .is_err()
        );
        assert_eq!(
            contradiction.scopes["p"],
            [("claimed".into(), "version".into())]
        );
        let wide = (0..256)
            .map(|index| {
                edit(
                    &format!("wide-{index:04}-{}", "w".repeat(4086)),
                    &format!("v-{index}"),
                    "definition",
                    vec![],
                )
            })
            .collect::<Vec<_>>();
        let grown = store
            .edit("p", Some("empty"), "grown", &wide)
            .await
            .unwrap();
        let mut current = read(&store, grown).await;
        current
            .interpretation("existing".into(), "kept".into())
            .unwrap();
        let issued = store
            .product_candidate(
                current.selection(),
                b"exact-request",
                "qualified-fixture", CanonicalProducerClass::General,
                "",
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            issued.retained_bytes(),
            candidate.retained_bytes(),
            "present growth does not mint an unbudgeted extent"
        );
        assert!(
            store
                .qualify_product(&mut current, &issued)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            current.scopes.is_empty(),
            "rejected first-page growth never installs a whole-scope read"
        );
        assert!(current.names.is_empty());
        assert_eq!(
            current.interpretations.get("existing").map(String::as_str),
            Some("kept")
        );
    }

    #[tokio::test]
    async fn identical_selected_product_publications_settle_one_native_acknowledgment() {
        let store = fixture().await;
        let revision = store
            .edit(
                "p",
                None,
                "source",
                &[edit("Root", "root", "definition", vec![])],
            )
            .await
            .unwrap();
        let mut selected = read(&store, revision.clone()).await;
        store.resolve_scope(&mut selected, "p").await.unwrap();
        // Multiple transport blocks keep both discoveries in staging while their
        // exact immutable payload and selected-read operation identity coincide.
        let payload = vec![37_u8; 2 * 1024 * 1024];
        let description = store
            .prepare_description(&selected, product(&selected, &payload))
            .unwrap();
        let start = tokio::sync::Barrier::new(2);
        let first = async {
            start.wait().await;
            store.publish_description(&selected, &description).await
        };
        let second = async {
            start.wait().await;
            store.publish_description(&selected, &description).await
        };
        let (first, second) = tokio::join!(first, second);
        // Read the real native generation/state and server clock before asserting
        // either outcome: a losing live writer must not be mistaken for expiry.
        let mut response = crate::canonical::bounded_query(store.db.query("SELECT * FROM canonical_stages WHERE problem='p' AND key!='source' ORDER BY key LIMIT 2; RETURN time::micros();")).await.unwrap();
        let stages: Vec<Object> = response.take(0).unwrap();
        let now = crate::canonical_codec::decode_int(response.take::<Value>(1).unwrap()).unwrap();
        assert_eq!(
            stages.len(),
            1,
            "same selected read and product must identify one stage"
        );
        let stage = wire::decode_canonical_stages(stages.into_iter().next().unwrap()).unwrap();
        assert_eq!(
            stage.generation, 1,
            "a concurrent exact publisher cannot reacquire the live writer generation"
        );
        assert!(
            stage.expires_at > now,
            "publication stage expired: generation={}, expires_at={}, now={}, activated={}, closed={}, abandoned={}",
            stage.generation,
            stage.expires_at,
            now,
            stage.activated,
            stage.closed,
            stage.abandoned
        );
        assert!(
            first.is_ok() && second.is_ok(),
            "identical live publication must settle, first={first:?}, second={second:?}; generation={}, expires_at={}, now={}, activated={}, closed={}, abandoned={}",
            stage.generation,
            stage.expires_at,
            now,
            stage.activated,
            stage.closed,
            stage.abandoned
        );
        let key = first.unwrap();
        assert_eq!(second.unwrap(), key);
        assert_eq!(stage.key, key);
        assert!(stage.activated && stage.closed && !stage.abandoned);
        let candidate = store
            .product_candidate(
                selected.selection(),
                b"exact-request",
                "qualified-fixture", CanonicalProducerClass::General,
                "",
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(candidate.key(), key);
        assert_eq!(
            store
                .product_acknowledged(&candidate.product)
                .await
                .unwrap(),
            Some(key.clone())
        );
        let mut qualified = selected.clone();
        assert_eq!(
            store
                .qualify_product(&mut qualified, &candidate)
                .await
                .unwrap()
                .unwrap()
                .payload(),
            payload
        );
        let mut roots = crate::canonical::bounded_query(store.db.query("SELECT * FROM canonical_roots WHERE owner_kind='product' AND owner=$owner LIMIT 2;").bind(("owner", key.clone()))).await.unwrap();
        assert_eq!(roots.take::<Vec<Object>>(0).unwrap().len(), 1);
        assert_eq!(current_head(&store).await, revision.key);
        assert_eq!(
            store
                .publish_description(&selected, &description)
                .await
                .unwrap(),
            key,
            "a completed repeat settles without reopening the stage"
        );
        let row: Option<Object> =
            crate::canonical::request(store.db.select(("canonical_stages", stage.key)))
                .await
                .unwrap();
        let repeated = wire::decode_canonical_stages(row.unwrap()).unwrap();
        assert_eq!(repeated.generation, stage.generation);
        assert_eq!(repeated.expires_at, stage.expires_at);
        store.release(selected.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn reusable_description_settles_exact_root_after_revision_change_and_pin_expiry() {
        let store = fixture().await;
        let first_revision = store
            .edit(
                "p",
                None,
                "description-first",
                &[edit("Root", "root1", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let mut first = read(&store, first_revision.clone()).await;
        store
            .resolve_logicals(&mut first, &["Root".into()])
            .await
            .unwrap();
        let offered = product(&first, b"retained recipe");
        let description = store.prepare_description(&first, offered.clone()).unwrap();
        let dependencies = serde_json::to_vec(&first.dependencies()).unwrap();
        let mut original_hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalProductV1);
        original_hash
            .str(&offered.key)
            .str(first.selection().key())
            .str(&offered.problem)
            .str(&offered.interpretation)
            .str(&offered.producer)
            .part(offered.request.as_slice())
            .part(offered.payload.as_slice())
            .part(&dependencies);
        let key = store
            .publish_description(&first, &description)
            .await
            .unwrap();
        assert_eq!(
            key,
            original_hash.finish_hash().to_hex(),
            "new admission keeps the exact original framing"
        );
        let second_revision = store
            .edit(
                "p",
                Some("description-first"),
                "description-second",
                &[edit("Other", "other1", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let mut second = read(&store, second_revision).await;
        assert!(
            store
                .recheck_selection_dependencies(&mut second, description.selected_dependencies())
                .await
                .unwrap()
        );
        assert!(description.matches_read(&second).unwrap());
        assert_eq!(
            store
                .publish_description(&second, &description)
                .await
                .unwrap(),
            key
        );
        crate::canonical::bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_protections', $pin) SET expires_at = 0;")
                .bind(("pin", second.selection().key().to_owned())),
        )
        .await
        .unwrap();
        assert_eq!(
            store.acknowledge_description(&description).await.unwrap(),
            Some(key.clone())
        );
        assert_eq!(
            store
                .publish_description(&second, &description)
                .await
                .unwrap(),
            key,
            "exact rooted settlement does not require another proposed revision or live pin"
        );
        let uncommitted = store
            .prepare_description(&second, product(&second, b"new body"))
            .unwrap();
        assert_eq!(
            store.acknowledge_description(&uncommitted).await.unwrap(),
            None
        );
        assert!(
            store
                .publish_description(&second, &uncommitted)
                .await
                .is_err(),
            "expired consumer protection cannot admit new material"
        );
        let deadline = tokio::time::Instant::now();
        assert!(
            matches!(
                store
                    .publish_description_until(&second, &description, deadline)
                    .await,
                Err(CanonicalError::Timeout)
            ),
            "an elapsed operation clock is never restarted"
        );
        store
            .drop_retained_root(
                &first_revision,
                &crate::canonical_retention::RetentionOwner::Product(key),
            )
            .await
            .unwrap();
        assert_eq!(
            store.acknowledge_description(&description).await.unwrap(),
            None,
            "cached bytes cannot revive a released exact root"
        );
        assert!(
            store
                .publish_description(&second, &description)
                .await
                .is_err()
        );
        store.release(first.selection()).await.unwrap();
        store.release(second.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn blocked_product_roundtrip_shared_roots_and_bounded_frontier_cleanup() {
        let store = fixture().await;
        let revision = store
            .edit(
                "p",
                None,
                "source",
                &[edit("Root", "root", "definition", vec![])],
            )
            .await
            .unwrap();
        let mut first = read(&store, revision.clone()).await;
        first
            .interpretation("large-dependency".into(), "d".repeat(4 * 1024 * 1024))
            .unwrap();
        let payload = vec![37u8; 34 * 1024 * 1024];
        assert_eq!(
            first.dependency_bytes().unwrap(),
            serde_json::to_vec(&first.dependencies()).unwrap().len()
        );
        let key = store
            .publish_product(&first, product(&first, &payload))
            .await
            .unwrap();
        let mut second = read(&store, revision.clone()).await;
        second
            .interpretation("large-dependency".into(), "d".repeat(4 * 1024 * 1024))
            .unwrap();
        let mut alternate = product(&second, &payload);
        alternate.request = b"second-request".to_vec().into();
        let second_key = store.publish_product(&second, alternate).await.unwrap();
        assert_ne!(key, second_key);
        let candidate = store
            .product_candidate(
                second.selection(),
                b"exact-request",
                "qualified-fixture", CanonicalProducerClass::General,
                "",
            )
            .await
            .unwrap()
            .unwrap();
        let other = store
            .product_candidate(
                second.selection(),
                b"second-request",
                "qualified-fixture", CanonicalProducerClass::General,
                "",
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            candidate.descriptor, other.descriptor,
            "publication pins/requests do not alter shared blob content"
        );
        assert!(candidate.retained_bytes() > payload.len());
        assert_eq!(current_head(&store).await, revision.key);
        let mut options =
            CanonicalOptions::from_state(Path::new(&std::env::var("PSE_SURREAL_STATE").unwrap()))
                .unwrap();
        options.database = store.database().to_owned();
        let reopened = crate::testing::canonical_fixture_peer(&store, &options).unwrap();
        reopened.open().await.unwrap();
        let mut selected = second.clone();
        assert_eq!(
            reopened
                .qualify_product(&mut selected, &candidate)
                .await
                .unwrap()
                .unwrap()
                .payload(),
            payload
        );
        // The immutable acknowledgment settles an activated stage; no restaging.
        assert_eq!(
            reopened
                .publish_product(&second, product(&second, &payload))
                .await
                .unwrap(),
            key
        );
        assert_eq!(
            reopened
                .product_acknowledged(&candidate.product)
                .await
                .unwrap(),
            Some(key.clone())
        );
        let dependencies = serde_json::to_vec(&first.dependencies()).unwrap();
        let descriptor = crate::canonical_staging::ProductBlob::from_parts(
            &payload,
            &dependencies,
            &candidate.product.producer,
        )
        .unwrap();
        let reserved_edit = ObjectEdit {
            logical: crate::canonical_staging::BODY_CONTENT_KIND.into(),
            scope: crate::canonical_staging::BODY_CONTENT_KIND.into(),
            name: crate::canonical_staging::BODY_CONTENT_KIND.into(),
            references: vec![],
            version: Some(pse_model::generated::runtime::canonical_versions::Row {
                key: descriptor.version.clone(),
                logical: crate::canonical_staging::BODY_CONTENT_KIND.into(),
                kind: crate::canonical_staging::BODY_CONTENT_KIND.into(),
                interpretation: wire::INTERPRETATION.into(),
                payload: payload.clone().into(),
            }),
        };
        assert_eq!(descriptor.version, candidate.descriptor.version);
        // Simulate concurrent discoveries which both staged before either observed
        // the equivalent root. The losing candidate is activated but remains an
        // orphan frontier; it must not steal the retained association from its winner.
        let mut raced = candidate.product.clone();
        raced.key = "raced-publication".into();
        let raced_stage = store
            .stage_product_parts("p", &raced.key, &descriptor, &payload, &dependencies)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            store
                .admit_product(first.selection(), &raced, &raced_stage)
                .await
                .unwrap(),
            key
        );
        assert_eq!(
            store
                .admit_product(first.selection(), &raced, &raced_stage)
                .await
                .unwrap(),
            key,
            "lost admission response settles before any stage replay"
        );
        assert!(
            store
                .edit("p", None, &key, std::slice::from_ref(&reserved_edit))
                .await
                .is_err()
        );
        assert!(
            store
                .edit("p", Some(&revision.key), &key, &[reserved_edit])
                .await
                .is_err()
        );
        assert_eq!(current_head(&store).await, revision.key);
        let rooted_page = store.reclaim_staging_page("p", "").await.unwrap();
        assert!(
            rooted_page
                .after
                .as_deref()
                .is_some_and(|cursor| !cursor.is_empty()),
            "rooted stage advances cursor"
        );
        assert_eq!(rooted_page.blocks, 0);
        store
            .drop_retained_root(
                &revision,
                &crate::canonical_retention::RetentionOwner::Product(key.clone()),
            )
            .await
            .unwrap();
        let mut cursor = String::new();
        for _ in 0..8 {
            let page = store.reclaim_staging_page("p", &cursor).await.unwrap();
            assert_eq!(
                page.blocks, 0,
                "remaining shared root preserves every block"
            );
            let Some(next) = page.after else {
                break;
            };
            cursor = next;
        }
        let mut surviving = second.clone();
        assert_eq!(
            store
                .qualify_product(&mut surviving, &other)
                .await
                .unwrap()
                .unwrap()
                .payload(),
            payload
        );
        store
            .drop_retained_root(
                &revision,
                &crate::canonical_retention::RetentionOwner::Product(second_key.clone()),
            )
            .await
            .unwrap();
        let mut blocks = 0usize;
        let mut cursor = String::new();
        for _ in 0..16 {
            let page = store.reclaim_staging_page("p", &cursor).await.unwrap();
            assert!(
                page.blocks <= 64,
                "each physical reclamation page remains bounded"
            );
            blocks += page.blocks;
            let Some(next) = page.after else {
                break;
            };
            cursor = next;
        }
        assert_eq!(
            blocks,
            descriptor
                .payload_bytes
                .div_ceil(crate::canonical_staging::SOURCE_BLOCK_BYTES)
                + descriptor
                    .dependency_bytes
                    .div_ceil(crate::canonical_staging::SOURCE_BLOCK_BYTES)
        );
        assert!(
            store
                .product_candidate(
                    second.selection(),
                    b"second-request",
                    "qualified-fixture", CanonicalProducerClass::General,
                    ""
                )
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .product_acknowledged(&candidate.product)
                .await
                .unwrap(),
            None,
            "released roots never resurrect through acknowledgment"
        );
        assert_eq!(current_head(&store).await, revision.key);
    }

    #[tokio::test]
    async fn passive_product_headers_are_bounded_protected_and_do_not_decode_payloads() {
        let store = fixture().await;
        let revision = store.edit("p", None, "source", &[]).await.unwrap();
        let selected = read(&store, revision.clone()).await;
        let mut expected = Vec::new();
        for index in 0..33 {
            let mut value = product(&selected, b"immutable recipe");
            value.request = format!("request-{index}").into_bytes().into();
            expected.push(store.publish_product(&selected, value).await.unwrap());
        }
        expected.sort();
        store
            .corrupt_product_descriptor(&expected[0])
            .await
            .unwrap();
        let page = store
            .product_headers(selected.selection(), None)
            .await
            .unwrap();
        assert_eq!(page.rows.len(), 32);
        assert_eq!(
            page.rows
                .iter()
                .map(|row| row.key.clone())
                .collect::<Vec<_>>(),
            expected[..32]
        );
        assert!(page.rows.iter().all(|row| row.revision == revision.key));
        let cursor = page.next.unwrap();
        let tail = store
            .product_headers(selected.selection(), Some(&cursor))
            .await
            .unwrap();
        assert_eq!(tail.rows.len(), 1);
        assert_eq!(tail.rows[0].key, expected[32]);
        assert!(tail.next.is_none());
        let mut foreign = cursor.clone();
        foreign.issuing_store = "foreign".into();
        assert!(
            store
                .product_headers(selected.selection(), Some(&foreign))
                .await
                .is_err()
        );
        let next_revision = store
            .edit("p", Some(&revision.key), "source-next", &[])
            .await
            .unwrap();
        let next = read(&store, next_revision).await;
        assert!(
            store
                .product_headers(next.selection(), None)
                .await
                .unwrap()
                .rows
                .is_empty()
        );
        assert!(
            store
                .product_headers(next.selection(), Some(&cursor))
                .await
                .is_err()
        );
        store.release(next.selection()).await.unwrap();
        store.release(selected.selection()).await.unwrap();
        assert!(
            store
                .product_headers(selected.selection(), None)
                .await
                .is_err()
        );
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn product_presence_does_not_decode_corrupt_descriptor() {
        let store = fixture().await;
        let revision = store.edit("p", None, "source", &[]).await.unwrap();
        let selected = read(&store, revision).await;
        let key = store
            .publish_product(&selected, product(&selected, b"immutable recipe"))
            .await
            .unwrap();
        assert!(
            store
                .product_candidate_presence(
                    selected.selection(),
                    b"exact-request",
                    "qualified-fixture", CanonicalProducerClass::General,
                    "",
                )
                .await
                .unwrap()
        );
        assert!(
            !store
                .product_candidate_presence(
                    selected.selection(),
                    b"other-request",
                    "qualified-fixture", CanonicalProducerClass::General,
                    "",
                )
                .await
                .unwrap()
        );
        store.corrupt_product_descriptor(&key).await.unwrap();
        assert!(
            store
                .product_candidate_presence(
                    selected.selection(),
                    b"exact-request",
                    "qualified-fixture", CanonicalProducerClass::General,
                    "",
                )
                .await
                .unwrap(),
            "presence uses protected row identity without decoding payload bytes"
        );
        assert!(
            store
                .product_candidate(
                    selected.selection(),
                    b"exact-request",
                    "qualified-fixture", CanonicalProducerClass::General,
                    ""
                )
                .await
                .is_err()
        );
        store.release(selected.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn blocked_product_corruption_pin_and_stage_fences_refuse_completion() {
        let store = fixture().await;
        let revision = store
            .edit(
                "p",
                None,
                "source",
                &[edit("Root", "root", "definition", vec![])],
            )
            .await
            .unwrap();
        let selected = read(&store, revision.clone()).await;
        let key = store
            .publish_product(&selected, product(&selected, &vec![83; 5 * 1024 * 1024]))
            .await
            .unwrap();
        let candidate = store
            .product_candidate(
                selected.selection(),
                b"exact-request",
                "qualified-fixture", CanonicalProducerClass::General,
                "",
            )
            .await
            .unwrap()
            .unwrap();
        let other_revision = store
            .edit(
                "other",
                None,
                "other-source",
                &[edit("Root", "root", "definition", vec![])],
            )
            .await
            .unwrap();
        let mut other = read(&store, other_revision).await;
        assert!(
            store.qualify_product(&mut other, &candidate).await.is_err(),
            "discovery cannot authorize a different problem"
        );
        assert!(
            store
                .publish_product(&other, product(&selected, &vec![83; 5 * 1024 * 1024]))
                .await
                .is_err(),
            "early exact acknowledgment cannot bypass publication selection"
        );
        store.corrupt_product_block(&key).await.unwrap();
        let mut attempt = selected.clone();
        assert!(
            store
                .qualify_product(&mut attempt, &candidate)
                .await
                .is_err()
        );
        assert_eq!(
            attempt.names, selected.names,
            "failed qualification does not contaminate premises"
        );
        crate::canonical::bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_product_associations', $key) SET payload = $pointer;")
                .bind(("key", key.clone()))
                .bind((
                    "pointer",
                    surrealdb::types::Bytes::from(b"unknown-inline-format".to_vec()),
                )),
        )
        .await
        .unwrap();
        assert!(
            store
                .product_candidate(
                    selected.selection(),
                    b"exact-request",
                    "qualified-fixture", CanonicalProducerClass::General,
                    ""
                )
                .await
                .is_err(),
            "unknown descriptor format fails closed"
        );
        crate::canonical::bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_product_associations', $key) SET payload = $pointer;")
                .bind(("key", key.clone()))
                .bind((
                    "pointer",
                    surrealdb::types::Bytes::from(candidate.product.payload.as_slice().to_vec()),
                )),
        )
        .await
        .unwrap();
        crate::canonical::bounded_query(store.db.query("UPDATE type::record('canonical_version_manifests', $key) SET kind = 'wrong-reserved-meaning';")
            .bind(("key", candidate.descriptor.version.clone()))).await.unwrap();
        assert!(
            store
                .product_candidate(
                    selected.selection(),
                    b"exact-request",
                    "qualified-fixture", CanonicalProducerClass::General,
                    ""
                )
                .await
                .is_err(),
            "descriptor cannot authorize a header with different meaning"
        );
        crate::canonical::bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_version_manifests', $key) SET kind = $kind;")
                .bind(("key", candidate.descriptor.version.clone()))
                .bind((
                    "kind",
                    crate::canonical_staging::BODY_CONTENT_KIND.to_owned(),
                )),
        )
        .await
        .unwrap();
        crate::canonical::bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_protections', $pin) SET expires_at = 0;")
                .bind(("pin", selected.selection().key().to_owned())),
        )
        .await
        .unwrap();
        assert!(
            store
                .qualify_product(&mut attempt, &candidate)
                .await
                .is_err()
        );
        assert!(
            store
                .product_candidate(
                    selected.selection(),
                    b"exact-request",
                    "qualified-fixture", CanonicalProducerClass::General,
                    ""
                )
                .await
                .is_err()
        );
        store.release(selected.selection()).await.unwrap();
        let fresh = read(&store, revision.clone()).await;
        let mut proposed = product(&fresh, b"fenced payload");
        proposed.key = "fenced-product".into();
        let dependencies = serde_json::to_vec(&fresh.dependencies()).unwrap();
        let payload = proposed.payload.clone();
        let descriptor = crate::canonical_staging::ProductBlob::from_parts(
            payload.as_slice(),
            &dependencies,
            &proposed.producer,
        )
        .unwrap();
        proposed.payload = serde_json::to_vec(&descriptor).unwrap().into();
        let stage = store
            .stage_product_parts(
                "p",
                &proposed.key,
                &descriptor,
                payload.as_slice(),
                &dependencies,
            )
            .await
            .unwrap()
            .unwrap();
        let mut fenced = stage.clone();
        fenced.generation += 1;
        assert!(
            store
                .admit_product(fresh.selection(), &proposed, &fenced)
                .await
                .is_err()
        );
        let mut wrong_request = stage.clone();
        wrong_request.request_digest.push('0');
        assert!(
            store
                .admit_product(fresh.selection(), &proposed, &wrong_request)
                .await
                .is_err()
        );
        crate::canonical::bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_stages', $stage) SET expires_at = 0;")
                .bind(("stage", proposed.key.clone())),
        )
        .await
        .unwrap();
        assert!(
            store
                .admit_product(fresh.selection(), &proposed, &stage)
                .await
                .is_err()
        );
        assert_eq!(store.product_acknowledged(&proposed).await.unwrap(), None);
        assert_eq!(current_head(&store).await, revision.key);
    }

    #[tokio::test]
    async fn exact_premises_unrelated_changes_and_native_inverse_selection() {
        let store = fixture().await;
        let first = store
            .edit(
                "p",
                None,
                "first",
                &[
                    edit("Root", "root1", "definition", Vec::new()),
                    edit(
                        "Data",
                        "data1",
                        "dataset",
                        vec![("p".into(), "Root".into())],
                    ),
                    edit("Other", "other1", "definition", Vec::new()),
                ],
            )
            .await
            .unwrap();
        let mut selected = read(&store, first.clone()).await;
        let mut forged = product(&selected, b"caller-made proof receipts");
        forged.producer = "caller-chosen-receipt".into();
        forged.producer_class = CanonicalProducerClass::LibraryMathematics;
        assert!(matches!(store.publish_product(&selected, forged).await,
                        Err(CanonicalError::Configuration(reason)) if reason.contains("impersonate")));
        assert_eq!(
            store
                .resolve_logicals(&mut selected, &["Root".into()])
                .await
                .unwrap()[0]
                .version,
            "root1"
        );
        assert!(
            store
                .resolve_names(&mut selected, "p", &["Missing".into()])
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            store
                .resolve_references(&mut selected, "p", "Root", "dataset")
                .await
                .unwrap()
                .iter()
                .map(|member| member.logical.as_str())
                .collect::<Vec<_>>(),
            ["Data"]
        );
        assert_eq!(
            store
                .resolve_kind_scope(&mut selected, Some("p"), "dataset")
                .await
                .unwrap()
                .len(),
            1
        );
        selected
            .interpretation("physical".into(), "physical-v1".into())
            .unwrap();
        assert_eq!(
            selected.dependency_bytes().unwrap(),
            serde_json::to_vec(&selected.dependencies()).unwrap().len(),
            "borrowed count projection matches all recorded premise fields"
        );
        let key = store
            .publish_product(&selected, product(&selected, b"portable-recipe"))
            .await
            .unwrap();
        assert_eq!(
            store
                .publish_product(&selected, product(&selected, b"portable-recipe"))
                .await
                .unwrap(),
            key
        );
        assert_eq!(
            store
                .product_acknowledged(&{
                    let mut product = product(&selected, b"portable-recipe");
                    product.key = key.clone();
                    product.dependencies =
                        serde_json::to_vec(&selected.dependencies()).unwrap().into();
                    stored_product(product)
                })
                .await
                .unwrap(),
            Some(key.clone())
        );
        store.release(selected.selection()).await.unwrap();
        let second = store
            .edit(
                "p",
                Some("first"),
                "unrelated",
                &[edit("Other", "other2", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let mut current = read(&store, second.clone()).await;
        current
            .interpretation("physical".into(), "physical-v1".into())
            .unwrap();
        assert_eq!(
            store
                .test_reuse_product(&mut current, b"exact-request", "qualified-fixture", "")
                .await
                .unwrap()
                .unwrap()
                .key(),
            key
        );
        // Exact rooted descriptions are shared without retagging their origin.
        // A released publication cannot be resurrected by fresh admission.
        let fresh_key = store
            .publish_product(&current, product(&current, b"portable-recipe"))
            .await
            .unwrap();
        assert_eq!(fresh_key, key);
        assert_eq!(
            store
                .publish_product(&current, product(&current, b"portable-recipe"))
                .await
                .unwrap(),
            fresh_key
        );
        let mut deduplicated_attempt = product(&current, b"portable-recipe");
        deduplicated_attempt.key = "proposed-key-never-stored".into();
        deduplicated_attempt.dependencies =
            serde_json::to_vec(&current.dependencies()).unwrap().into();
        let deduplicated_attempt = stored_product(deduplicated_attempt);
        store.release(current.selection()).await.unwrap();
        assert_eq!(
            store
                .product_acknowledged(&deduplicated_attempt)
                .await
                .unwrap(),
            Some(key.clone()),
            "lost deduplication acknowledgment settles the existing exact rooted product after pin release"
        );
        let mut mismatched_attempt = product(&current, b"different-admitted-recipe");
        mismatched_attempt.key = deduplicated_attempt.key.clone();
        mismatched_attempt.dependencies =
            serde_json::to_vec(&current.dependencies()).unwrap().into();
        let mismatched_attempt = stored_product(mismatched_attempt);
        assert_eq!(
            store
                .product_acknowledged(&mismatched_attempt)
                .await
                .unwrap(),
            None
        );
        let mut malformed_attempt = deduplicated_attempt.clone();
        malformed_attempt.payload = b"not-a-product-descriptor".to_vec().into();
        assert!(
            store
                .product_acknowledged(&malformed_attempt)
                .await
                .is_err()
        );
        let owner = crate::canonical_retention::RetentionOwner::Product(key.clone());
        assert!(
            store.retain_revision(&second, &owner).await.is_err(),
            "public root retention cannot rewrite a product's immutable origin"
        );
        store.drop_retained_root(&first, &owner).await.unwrap();
        assert_eq!(
            store
                .product_acknowledged(&deduplicated_attempt)
                .await
                .unwrap(),
            None,
            "settlement never resurrects an explicitly released product root"
        );
        let mut recompiled = read(&store, first.clone()).await;
        store
            .resolve_logicals(&mut recompiled, &["Root".into()])
            .await
            .unwrap();
        store
            .resolve_names(&mut recompiled, "p", &["Missing".into()])
            .await
            .unwrap();
        store
            .resolve_references(&mut recompiled, "p", "Root", "dataset")
            .await
            .unwrap();
        store
            .resolve_kind_scope(&mut recompiled, Some("p"), "dataset")
            .await
            .unwrap();
        recompiled
            .interpretation("physical".into(), "physical-v1".into())
            .unwrap();
        let new_key = store
            .publish_product(&recompiled, product(&recompiled, b"portable-recipe"))
            .await
            .unwrap();
        assert_ne!(
            new_key, key,
            "explicit release permits fresh admission without resurrecting the old root"
        );
        store.release(recompiled.selection()).await.unwrap();
        store.release(current.selection()).await.unwrap();
        let third = store
            .edit(
                "p",
                Some("unrelated"),
                "defined-missing",
                &[edit("Missing", "missing1", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let mut changed = read(&store, third).await;
        changed
            .interpretation("physical".into(), "physical-v1".into())
            .unwrap();
        assert!(
            store
                .test_reuse_product(&mut changed, b"exact-request", "qualified-fixture", "")
                .await
                .unwrap()
                .is_none()
        );
        store.release(changed.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn later_candidate_page_and_expired_selection_refuse_false_reuse() {
        let store = fixture().await;
        let revision = store
            .edit(
                "p",
                None,
                "first",
                &[edit("Root", "root1", "definition", Vec::new())],
            )
            .await
            .unwrap();
        let mut largest = (String::new(), String::new());
        for index in 0..24 {
            let mut selected = read(&store, revision.clone()).await;
            store
                .resolve_names(&mut selected, "p", &[format!("absent-{index}")])
                .await
                .unwrap();
            selected
                .interpretation("candidate".into(), index.to_string())
                .unwrap();
            let key = store
                .publish_product(&selected, product(&selected, index.to_string().as_bytes()))
                .await
                .unwrap();
            if key > largest.0 {
                largest = (key, index.to_string());
            }
            store.release(selected.selection()).await.unwrap();
        }
        let mut selected = read(&store, revision).await;
        selected
            .interpretation("candidate".into(), largest.1.clone())
            .unwrap();
        let hit = store
            .test_reuse_product(&mut selected, b"exact-request", "qualified-fixture", "")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(hit.key(), largest.0);
        assert_eq!(
            selected.names.len(),
            1,
            "rejected candidate premises must not contaminate the selected read"
        );
        store.release(selected.selection()).await.unwrap();
        assert!(
            store
                .test_reuse_product(&mut selected, b"exact-request", "qualified-fixture", "")
                .await
                .is_err()
        );
        store.remove_isolated_fixture().await.unwrap();
    }
}
