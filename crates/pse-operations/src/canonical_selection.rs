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

/// A semantic premise actually inspected while resolving an immutable selection.
/// Conflict generations are deliberately separate from these immutable meanings.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Dependencies {
    interpretation: String,
    premises: Vec<Premise>,
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
        let mut counter = DependencyCounter(0);
        serde_json::to_writer(&mut counter, &BorrowedDependencies(self))
            .map_err(|_| CanonicalError::PayloadLimit)?;
        Ok(counter.0)
    }
    /// Allocation-free bound for dependency cloning, escaping, JSON and blob copies.
    /// The caller additionally reserves its portable payload's assembly copies.
    pub fn publication_scratch_bytes(&self) -> Result<usize, CanonicalError> {
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
    #[cfg(all(test, feature = "canonical-tests"))]
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
        let mut response = protected_query(|| Ok(self.db.query(format!("{PROTECTED_BEGIN}\nSELECT * FROM canonical_memberships WHERE problem = $problem AND logical IN $logicals AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence);\nCOMMIT;"))
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
        let members = self
            .filtered_memberships(&read.selection, scope, source_kind, None)
            .await?;
        read.kinds.insert(
            (scope.map(str::to_owned), source_kind.into()),
            membership_meaning(&members),
        );
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
        let members = self
            .filtered_memberships(
                &read.selection,
                None,
                source_kind,
                Some((target_scope, target_name)),
            )
            .await?;
        read.references.insert(
            (target_scope.into(), target_name.into(), source_kind.into()),
            membership_meaning(&members),
        );
        Ok(members)
    }
    async fn filtered_memberships(
        &self,
        selection: &ProtectedSelection,
        scope: Option<&str>,
        source_kind: &str,
        target: Option<(&str, &str)>,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let mut after = String::new();
        let mut members = Vec::new();
        loop {
            let page = self
                .selection_membership_page(selection, scope, Some(source_kind), target, &after)
                .await?;
            let Some(last) = page.last() else {
                break;
            };
            after = last.key.clone();
            members.extend(page);
        }
        Ok(members)
    }
    async fn selection_membership_page(
        &self,
        selection: &ProtectedSelection,
        scope: Option<&str>,
        source_kind: Option<&str>,
        target: Option<(&str, &str)>,
        after: &str,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let mut response = protected_query(|| Ok(self.db.query(format!("{PROTECTED_BEGIN}\nSELECT * FROM canonical_memberships WHERE problem = $problem AND ($scope = NONE OR scope = $scope) AND key > $after AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) AND ($source_kind = NONE OR (out.kind = $source_kind AND out.closed = true)) AND ($target_scope = NONE OR (SELECT VALUE key FROM canonical_edges WHERE source_version = $parent.version AND target_scope = $target_scope AND target_name = $target_name LIMIT 1) != []) ORDER BY key LIMIT 64;\nCOMMIT;"))
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
    /// One bounded inventory page; no cursor exposes unprotected subsequent reads.
    pub async fn membership_page(
        &self,
        selection: &ProtectedSelection,
        scope: Option<&str>,
        after: &str,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let mut response = protected_query(|| Ok(self.db.query(format!("{PROTECTED_BEGIN}\nSELECT * FROM canonical_memberships WHERE problem = $problem AND ($scope = NONE OR scope = $scope) AND key > $after AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) ORDER BY key LIMIT 64;\nCOMMIT;"))
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
        let mut after = String::new();
        let mut members = Vec::new();
        loop {
            let page = self
                .membership_page(&read.selection, Some(scope), &after)
                .await?;
            let Some(last) = page.last() else {
                break;
            };
            after = last.key.clone();
            members.extend(page);
        }
        let mut premise = members
            .iter()
            .map(|member| (member.name.clone(), member.version.clone()))
            .collect::<Vec<_>>();
        premise.sort();
        read.scopes.insert(scope.into(), premise);
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
        if product.producer.starts_with(SCIENTIFIC_PRODUCER_PREFIX) {
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
        if !product.producer.starts_with(SCIENTIFIC_PRODUCER_PREFIX) {
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
        self.admit_product(&read.selection, &product, &stage).await
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
        let mut response = protected_query(|| Ok(self.db.query(format!("{PROTECTED_BEGIN}\nSELECT * FROM canonical_products WHERE problem = $problem AND producer = $producer AND request = $request AND key > $after AND interpretation = $interpretation AND type::record('canonical_roots', key).owner_kind = 'product' AND type::record('canonical_roots', key).owner = key AND type::record('canonical_roots', key).problem = problem AND type::record('canonical_roots', key).revision = revision ORDER BY key LIMIT 1;\nCOMMIT;"))
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
        let mut eligible = true;
        let mut candidate_read = SelectedRead::new(read.selection.clone());
        for premise in dependencies.premises {
            match premise {
                Premise::Name {
                    scope,
                    name,
                    version,
                } => {
                    let members = self
                        .resolve_names(&mut candidate_read, &scope, &[name])
                        .await?;
                    eligible &= members.first().map(|member| &member.version) == version.as_ref();
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
                    let members = self
                        .resolve_logicals(&mut candidate_read, &[logical])
                        .await?;
                    eligible &= members.first().map(|member| &member.version) == version.as_ref();
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
            Ok(Some(ReusableProduct { product }))
        } else {
            Ok(None)
        }
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

fn membership_meaning(members: &[Membership]) -> Vec<(String, String)> {
    let mut meaning = members
        .iter()
        .map(|member| (member.logical.clone(), member.version.clone()))
        .collect::<Vec<_>>();
    meaning.sort();
    meaning
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_server_unit {
    use super::*;
    use crate::canonical::{CanonicalOptions, ObjectEdit};
    use std::{path::Path, time::Duration};

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
