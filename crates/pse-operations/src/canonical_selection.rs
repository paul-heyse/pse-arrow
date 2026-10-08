// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Protected selected reads and semantic premises for durable compilation products.

use crate::{
    canonical::{
        CanonicalError, CanonicalStore, PROTECTED_BEGIN, ProtectedSelection, protected_query,
    },
    generated::surreal as wire,
};
use pse_model::generated::runtime::{
    canonical_memberships::Row as Membership, canonical_products::Row as Product,
    canonical_versions::Row as ObjectVersion,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use surrealdb::types::Object;

/// Reserved writer role for compiler-issued descriptions eligible for scientific replay.
pub const SCIENTIFIC_PRODUCER_PREFIX: &str = "pse.qualified-math-producer.v1:";

/// Reserved independently observed exact deployment-local replay admission.
pub const LOCAL_RUNTIME_PRODUCER_PREFIX: &str = "pse.local-runtime.v1:";

/// Both namespaces require the controlled scientific writer; generic safe publication
/// cannot confer either guarantee from caller-selected producer bytes.
pub fn is_replay_producer(producer: &str) -> bool {
    producer.starts_with(SCIENTIFIC_PRODUCER_PREFIX)
        || producer.starts_with(LOCAL_RUNTIME_PRODUCER_PREFIX)
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

/// A compiler's protected read session. Recorded premises include absent names and
/// explicitly enumerated scopes, rather than only positive object references.
#[derive(Clone, Debug)]
pub struct SelectedRead {
    selection: ProtectedSelection,
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
    /// Record a consumed compiler/provider/configuration interpretation.
    pub fn interpretation(&mut self, role: String, identity: String) -> Result<(), CanonicalError> {
        if let Some(previous) = self.interpretations.get(&role)
            && previous != &identity
        {
            return Err(CanonicalError::Configuration(
                "one role consumed incompatible interpretations".into(),
            ));
        }
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
        insert_delta(&mut self.names, delta.names);
        insert_delta(&mut self.scopes, delta.scopes);
        insert_delta(&mut self.interpretations, delta.interpretations);
        insert_delta(&mut self.logicals, delta.logicals);
        insert_delta(&mut self.kinds, delta.kinds);
        insert_delta(&mut self.references, delta.references);
        Ok(())
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

/// Small discovery receipt. Its private header permits budgeting, never replay authority.
#[derive(Debug)]
pub struct ProductCandidate {
    product: Product,
    descriptor: crate::canonical_staging::ProductBlob,
    retained_bytes: usize,
}
impl ProductCandidate {
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
    /// Exact producer interpretation used for eligibility.
    pub fn producer(&self) -> &str {
        &self.product.producer
    }
}

impl CanonicalStore {
    /// Resolve exact authored identities, including inspected absent identities.
    pub async fn resolve_logicals(
        &self,
        read: &mut SelectedRead,
        logicals: &[String],
    ) -> Result<Vec<Membership>, CanonicalError> {
        if logicals.len() > 256 {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut response = protected_query("canonical_selection::resolve_logicals", || Ok(self.db.query(format!("{PROTECTED_BEGIN}\nSELECT * FROM canonical_memberships WHERE problem = $problem AND logical IN $logicals AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence);\nCOMMIT;"))
            .bind(("problem", read.selection.revision().problem.clone())).bind(("revision", read.selection.revision().key.clone())).bind(("protection", read.selection.key().to_owned())).bind(("logicals", logicals.to_vec())).bind(("sequence", crate::canonical_codec::encode_uint(read.selection.revision().sequence)?)))).await?;
        let rows: Vec<Object> = response.take(response.num_statements().saturating_sub(2))?;
        let members = rows
            .into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<Vec<_>, _>>()?;
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
    }
    /// Enumerate a requested declaration kind without hydrating unrelated source objects.
    pub async fn resolve_kind_scope(
        &self,
        read: &mut SelectedRead,
        scope: Option<&str>,
        source_kind: &str,
    ) -> Result<Vec<Membership>, CanonicalError> {
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
        let mut cursor = self.reference_pages(read, target_scope, target_name, source_kind)?;
        let mut members = Vec::new();
        loop {
            let page = self.next_membership_page(read, &mut cursor).await?;
            if page.is_empty() {
                break;
            }
            members.extend(page);
        }
        Ok(members)
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
        let mut pages = self
            .next_membership_pages(read, std::slice::from_mut(cursor))
            .await?;
        Ok(pages.remove(0))
    }
    /// Group exact inventory cursors in one protected transaction. The combined
    /// response contains at most 64 memberships, regardless of cursor count.
    pub async fn next_membership_pages(
        &self,
        read: &mut SelectedRead,
        cursors: &mut [SelectedMemberships],
    ) -> Result<Vec<Vec<Membership>>, CanonicalError> {
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
        for index in 0..cursors.len() {
            sql.push_str(&format!("\nSELECT * FROM canonical_memberships WHERE problem = $problem AND ($scope{index} = NONE OR scope = $scope{index}) AND key > $after{index} AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) AND ($kind{index} = NONE OR (out.kind = $kind{index} AND out.closed = true)) AND ($target_scope{index} = NONE OR (SELECT VALUE key FROM canonical_edges WHERE source_version = $parent.version AND target_scope = $target_scope{index} AND target_name = $target_name{index} LIMIT 1) != []) ORDER BY key LIMIT {limit};"));
        }
        sql.push_str("\nCOMMIT;");
        let mut response = protected_query("canonical_selection::next_membership_pages", || {
            let mut query = self
                .db
                .query(sql.clone())
                .bind(("problem", read.selection.revision().problem.clone()))
                .bind(("revision", read.selection.revision().key.clone()))
                .bind(("protection", read.selection.key().to_owned()))
                .bind((
                    "sequence",
                    crate::canonical_codec::encode_uint(read.selection.revision().sequence)?,
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
        })
        .await?;
        let first = response
            .num_statements()
            .checked_sub(cursors.len() + 1)
            .ok_or(CanonicalError::IncompleteResponse)?;
        // Decode the complete response before moving any cursor's premise.
        let mut pages = Vec::with_capacity(cursors.len());
        for index in first..first + cursors.len() {
            let rows: Vec<Object> = response.take(index)?;
            pages.push(
                rows.into_iter()
                    .map(wire::decode_canonical_memberships)
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        for (cursor, page) in cursors.iter_mut().zip(&pages) {
            self.accept_membership_page(read, cursor, page)?;
        }
        Ok(pages)
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
        let mut response = protected_query("canonical_selection::selection_membership_page", || Ok(self.db.query(format!("{PROTECTED_BEGIN}\nSELECT * FROM canonical_memberships WHERE problem = $problem AND ($scope = NONE OR scope = $scope) AND key > $after AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) AND ($source_kind = NONE OR (out.kind = $source_kind AND out.closed = true)) AND ($target_scope = NONE OR (SELECT VALUE key FROM canonical_edges WHERE source_version = $parent.version AND target_scope = $target_scope AND target_name = $target_name LIMIT 1) != []) ORDER BY key LIMIT 64;\nCOMMIT;"))
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
        if expected.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Ok(false);
        }
        let mut seen = vec![false; expected.len()];
        let mut count = 0usize;
        let mut after = String::new();
        loop {
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
        let members = self.select_names(&read.selection, scope, names).await?;
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
    }
    /// Resolve an exact sparse set of lexical pairs, preserving absent pairs
    /// without admitting the cross product of independently grouped scopes/names.
    pub async fn resolve_name_pairs(
        &self,
        read: &mut SelectedRead,
        pairs: &[(String, String)],
    ) -> Result<Vec<Membership>, CanonicalError> {
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
            "{PROTECTED_BEGIN}\nSELECT * FROM canonical_memberships WHERE problem = $problem AND ({predicate}) AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence);\nCOMMIT;"
        );
        let mut response = protected_query("canonical_selection::resolve_name_pairs", || {
            let mut query = self
                .db
                .query(sql.clone())
                .bind(("problem", read.selection.revision().problem.clone()))
                .bind(("revision", read.selection.revision().key.clone()))
                .bind(("protection", read.selection.key().to_owned()))
                .bind((
                    "sequence",
                    crate::canonical_codec::encode_uint(read.selection.revision().sequence)?,
                ));
            for (index, (scope, name)) in pairs.iter().enumerate() {
                query = query
                    .bind((format!("scope{index}"), scope.clone()))
                    .bind((format!("name{index}"), name.clone()));
            }
            Ok(query)
        })
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
        for pair in pairs {
            read.names.insert(pair.clone(), present.get(pair).cloned());
        }
        Ok(members)
    }
    /// One bounded inventory page; no cursor exposes unprotected subsequent reads.
    pub async fn membership_page(
        &self,
        selection: &ProtectedSelection,
        scope: Option<&str>,
        after: &str,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let mut response = protected_query("canonical_selection::membership_page", || Ok(self.db.query(format!("{PROTECTED_BEGIN}\nSELECT * FROM canonical_memberships WHERE problem = $problem AND ($scope = NONE OR scope = $scope) AND key > $after AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) ORDER BY key LIMIT 64;\nCOMMIT;"))
            .bind(("problem", selection.revision().problem.clone())).bind(("revision", selection.revision().key.clone())).bind(("protection", selection.key().to_owned())).bind(("scope", scope.map(str::to_owned))).bind(("after", after.to_owned())).bind(("sequence", crate::canonical_codec::encode_uint(selection.revision().sequence)?)))).await?;
        let rows: Vec<Object> = response.take(response.num_statements().saturating_sub(2))?;
        rows.into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
    /// Enumerate an explicitly required scope, preserving its complete membership premise.
    pub async fn resolve_scope(
        &self,
        read: &mut SelectedRead,
        scope: &str,
    ) -> Result<Vec<Membership>, CanonicalError> {
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
    }
    /// Fetch an exact immutable object within a live selection. Independent pages
    /// and payloads remain below the transport's individual-message bound.
    pub async fn selected_object(
        &self,
        selection: &ProtectedSelection,
        version: &str,
    ) -> Result<Option<ObjectVersion>, CanonicalError> {
        self.assemble_selected_object(selection, version).await
    }
    /// Publish the kernel's description together with the actual selected read premises.
    pub async fn publish_product(
        &self,
        read: &SelectedRead,
        product: Product,
    ) -> Result<String, CanonicalError> {
        if is_replay_producer(&product.producer) {
            return Err(CanonicalError::Configuration(
                "generic product publication cannot impersonate scientific admission".into(),
            ));
        }
        self.publish_product_inner(read, product).await
    }
    /// Publish a description issued by the scientific admission owner.
    ///
    /// # Safety
    /// The payload and request must come from the owning compiler's successfully admitted
    /// immutable body, with its exact construction receipts and scientific interpretation.
    /// The reserved producer must be qualified for that implementation. Arbitrary decoded
    /// DTOs, caller-created receipts and claimed hashes are insufficient. This operation
    /// establishes the controlled writer role trusted by strict scientific reconstruction.
    ///
    /// # Errors
    /// Missing reserved producer role or ordinary guarded publication refusal.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 explicit inter-crate scientific writer trust assertion; no unsafe memory operation"
    )]
    pub async unsafe fn publish_scientific_product(
        &self,
        read: &SelectedRead,
        product: Product,
    ) -> Result<String, CanonicalError> {
        if !is_replay_producer(&product.producer) {
            return Err(CanonicalError::Configuration(
                "scientific publication requires its reserved qualified producer".into(),
            ));
        }
        self.publish_product_inner(read, product).await
    }
    async fn publish_product_inner(
        &self,
        read: &SelectedRead,
        mut product: Product,
    ) -> Result<String, CanonicalError> {
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
        if product.payload.len() > 65 * 1024 * 1024 + 4096
            || product.request.len() > crate::canonical::PAYLOAD_BYTES
        {
            return Err(CanonicalError::PayloadLimit);
        }
        read.dependency_bytes()?;
        product.dependencies = serde_json::to_vec(&BorrowedDependencies(read))
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?
            .into();
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalProductV1);
        hash.str(&product.key)
            .str(read.selection.key())
            .str(&product.problem)
            .str(&product.interpretation)
            .str(&product.producer)
            .part(product.request.as_slice())
            .part(product.payload.as_slice())
            .part(product.dependencies.as_slice());
        product.key = hash.finish_hash().to_hex();
        let (descriptor, edit) = crate::canonical_staging::product_blob(
            product.payload.as_slice(),
            product.dependencies.as_slice(),
        )?;
        product.payload = serde_json::to_vec(&descriptor)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?
            .into();
        product.dependencies = Vec::new().into();
        if product.request.len().saturating_add(product.payload.len())
            > crate::canonical::PAYLOAD_BYTES
        {
            return Err(CanonicalError::PayloadLimit);
        }
        // A committed exact acknowledgment settles before touching an activated stage.
        if let Some(key) = self.product_acknowledged(&product).await? {
            return Ok(key);
        }
        let stage = self
            .stage_product_blob(&product.problem, &product.key, edit)
            .await?;
        if let Some(stage) = stage {
            return self.admit_product(&read.selection, &product, &stage).await;
        }
        // Native acquisition identified the exact live/activated writer. No
        // second writer token escapes; only its immutable acknowledgment settles.
        tokio::time::timeout(crate::canonical::REQUEST_TIMEOUT, async {
            loop {
                self.ensure_writes()?;
                if let Some(key) = self.product_acknowledged(&product).await? {
                    return Ok(key);
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_| CanonicalError::Timeout)?
    }
    /// Corrupt only a disposable native fixture's first immutable product block.
    #[cfg(feature = "canonical-tests")]
    pub async fn corrupt_product_block(&self, key: &str) -> Result<(), CanonicalError> {
        if !self.database().starts_with("canonical_test_") {
            return Err(CanonicalError::Configuration(
                "corruption requires isolated canonical fixture".into(),
            ));
        }
        let row = crate::canonical::request(self.db.select(("canonical_products", key)))
            .await?
            .ok_or_else(|| CanonicalError::Configuration("fixture product missing".into()))?;
        let product = wire::decode_canonical_products(row)?;
        let descriptor = crate::canonical_staging::ProductBlob::decode(product.payload.as_slice())?;
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
    }
    /// Discover one eligible request/producer header without hydrating its scientific bytes.
    pub async fn product_candidate(
        &self,
        selection: &ProtectedSelection,
        request: &[u8],
        producer: &str,
        after: &str,
    ) -> Result<Option<ProductCandidate>, CanonicalError> {
        if request.len() > crate::canonical::PAYLOAD_BYTES
            || producer.len() > crate::canonical_staging::IDENTITY_BYTES
            || after.len() > crate::canonical_staging::IDENTITY_BYTES
        {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut response = protected_query("canonical_selection::product_candidate", || Ok(self.db.query(format!("{PROTECTED_BEGIN}\nSELECT * FROM canonical_products WHERE problem = $problem AND producer = $producer AND request = $request AND key > $after AND interpretation = $interpretation AND type::record('canonical_roots', key).owner_kind = 'product' AND type::record('canonical_roots', key).owner = key AND type::record('canonical_roots', key).problem = problem AND type::record('canonical_roots', key).revision = revision ORDER BY key LIMIT 1;\nCOMMIT;"))
            .bind(("problem", selection.revision().problem.clone())).bind(("revision", selection.revision().key.clone())).bind(("sequence", crate::canonical_codec::encode_uint(selection.revision().sequence)?))
            .bind(("protection", selection.key().to_owned())).bind(("producer", producer.to_owned())).bind(("request", surrealdb::types::Bytes::from(request.to_vec()))).bind(("after", after.to_owned())).bind(("interpretation", wire::INTERPRETATION.to_owned())))).await?;
        let rows: Vec<Object> = response.take(response.num_statements().saturating_sub(2))?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let product = wire::decode_canonical_products(row)?;
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
    }
    /// Hydrate only after the caller reserved the issued bound; check every premise
    /// against the protected revision without contaminating rejected read sessions.
    pub async fn qualify_product(
        &self,
        read: &mut SelectedRead,
        candidate: &ProductCandidate,
    ) -> Result<Option<ReusableProduct>, CanonicalError> {
        read.complete()?;
        if candidate.product.problem != read.selection().revision().problem {
            return Err(CanonicalError::Configuration(
                "product candidate belongs to a different problem".into(),
            ));
        }
        let Some(blob) = self
            .assemble_product_blob(read.selection(), &candidate.product, &candidate.descriptor)
            .await?
        else {
            return Ok(None);
        };
        let (payload, dependencies_bytes) = candidate.descriptor.split(blob.payload.as_slice())?;
        let dependencies: Dependencies = serde_json::from_slice(&dependencies_bytes)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        if dependencies.interpretation != wire::INTERPRETATION {
            return Ok(None);
        }
        let mut product = candidate.product.clone();
        product.payload = payload.into();
        product.dependencies = dependencies_bytes.into();
        if self.recheck_dependencies(read, &dependencies).await? {
            Ok(Some(ReusableProduct { product }))
        } else {
            Ok(None)
        }
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
        let mut eligible = true;
        let mut candidate_read = SelectedRead::new(read.selection.clone());
        // Exact positive AND absent lookup premises share the existing bounded
        // acquisition routes. Reuse must not replace hydration with singleton RPCs.
        let mut logicals = dependencies
            .premises
            .iter()
            .filter_map(|premise| match premise {
                Premise::Logical { logical, .. } => Some(logical.clone()),
                _ => None,
            });
        loop {
            let window = logicals.by_ref().take(256).collect::<Vec<_>>();
            if window.is_empty() {
                break;
            }
            self.resolve_logicals(&mut candidate_read, &window).await?;
        }
        let mut names = dependencies
            .premises
            .iter()
            .filter_map(|premise| match premise {
                Premise::Name { scope, name, .. } => Some((scope.clone(), name.clone())),
                _ => None,
            });
        loop {
            let window = names.by_ref().take(64).collect::<Vec<_>>();
            if window.is_empty() {
                break;
            }
            self.resolve_name_pairs(&mut candidate_read, &window)
                .await?;
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
            .product_candidate(read.selection(), request, producer, &cursor)
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
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        store
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
            interpretation: wire::INTERPRETATION.into(),
        }
    }
    async fn current_head(store: &CanonicalStore) -> String {
        let row: Option<Object> =
            crate::canonical::request(store.db.select(("canonical_problems", "p")))
                .await
                .unwrap();
        wire::decode_canonical_problems(row.unwrap()).unwrap().head
    }
    fn stored_product(mut product: Product) -> Product {
        let (descriptor, _) = crate::canonical_staging::product_blob(
            product.payload.as_slice(),
            product.dependencies.as_slice(),
        )
        .unwrap();
        product.payload = serde_json::to_vec(&descriptor).unwrap().into();
        product.dependencies = Vec::new().into();
        product
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
            .product_candidate(old.selection(), b"exact-request", "qualified-fixture", "")
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
                "qualified-fixture",
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
        let proposed = product(&selected, &payload);
        let start = tokio::sync::Barrier::new(2);
        let first = async {
            start.wait().await;
            store.publish_product(&selected, proposed.clone()).await
        };
        let second = async {
            start.wait().await;
            store.publish_product(&selected, proposed.clone()).await
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
                "qualified-fixture",
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
            store.publish_product(&selected, proposed).await.unwrap(),
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
                "qualified-fixture",
                "",
            )
            .await
            .unwrap()
            .unwrap();
        let other = store
            .product_candidate(
                second.selection(),
                b"second-request",
                "qualified-fixture",
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
        let reopened = CanonicalStore::connect(&options).await.unwrap();
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
        let (descriptor, reserved_edit) = crate::canonical_staging::product_blob(
            &payload,
            &serde_json::to_vec(&first.dependencies()).unwrap(),
        )
        .unwrap();
        assert_eq!(descriptor.version, candidate.descriptor.version);
        // Simulate concurrent discoveries which both staged before either observed
        // the equivalent root. The losing candidate is activated but remains an
        // orphan frontier; it must not steal the retained association from its winner.
        let mut raced = candidate.product.clone();
        raced.key = "raced-publication".into();
        let raced_stage = store
            .stage_product_blob("p", &raced.key, reserved_edit.clone())
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
                .bytes
                .div_ceil(crate::canonical_staging::SOURCE_BLOCK_BYTES)
        );
        assert!(
            store
                .product_candidate(
                    second.selection(),
                    b"second-request",
                    "qualified-fixture",
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
                "qualified-fixture",
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
                .query("UPDATE type::record('canonical_products', $key) SET payload = $pointer;")
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
                    "qualified-fixture",
                    ""
                )
                .await
                .is_err(),
            "unknown descriptor format fails closed"
        );
        crate::canonical::bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_products', $key) SET payload = $pointer;")
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
                    "qualified-fixture",
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
                    crate::canonical_staging::PRODUCT_BLOB_KIND.to_owned(),
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
                    "qualified-fixture",
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
        let (descriptor, edit) =
            crate::canonical_staging::product_blob(proposed.payload.as_slice(), &dependencies)
                .unwrap();
        proposed.payload = serde_json::to_vec(&descriptor).unwrap().into();
        let stage = store
            .stage_product_blob("p", &proposed.key, edit)
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
        forged.producer = format!("{SCIENTIFIC_PRODUCER_PREFIX}claimed-compiler");
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
        let mut mismatched_attempt = deduplicated_attempt.clone();
        mismatched_attempt.payload = b"different-admitted-recipe".to_vec().into();
        assert_eq!(
            store
                .product_acknowledged(&mismatched_attempt)
                .await
                .unwrap(),
            None
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
