// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned source bytes and exact local lookup premises, without store permission.
use super::*;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SourceObjectValue {
    pub(crate) logical: String,
    pub(crate) scope: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) kind: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) references: Vec<(String, String)>,
}
#[derive(Debug)]
pub(crate) struct LocalSourceObject {
    pub(crate) value: SourceObjectValue,
    pub(crate) lease: Arc<pse_columnar::AllocationLease>,
    pub(super) declaration: Option<SourcePayload>,
    pub(super) parsed: Option<Arc<pse_modeling::expression::occurrences::ParsedFields>>,
}
impl std::ops::Deref for LocalSourceObject {
    type Target = SourceObjectValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl PartialEq for LocalSourceObject {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl Eq for LocalSourceObject {}
impl Hash for LocalSourceObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
impl LocalSourceObject {
    pub(crate) fn retained_bytes(&self) -> usize {
        self.lease.size()
    }
    fn from_edit(runtime: &Runtime, edit: ObjectEdit) -> Result<Option<Arc<Self>>, WorkflowError> {
        let Some(version) = edit.version else {
            return Ok(None);
        };
        let extent = edit.logical.capacity()
            + edit.scope.capacity()
            + edit.name.capacity()
            + version.owned_bytes()
            + edit
                .references
                .iter()
                .map(|(s, n)| s.capacity() + n.capacity() + 128)
                .sum::<usize>()
            + size_of::<Self>()
            + 256;
        let declaration_extent = if version.kind.starts_with("modeling:") && version.kind != CONTEXT
        {
            version
                .payload
                .len()
                .checked_mul(32)
                .and_then(|n| n.checked_add(8192))
                .ok_or_else(|| contract("owned declaration extent"))?
        } else {
            0
        };
        let lease = runtime.shared.math().reserve(
            "modeling:owned-source-object",
            extent
                .checked_add(declaration_extent)
                .ok_or_else(|| contract("owned source object extent"))?,
        )?;
        if version.logical != edit.logical
            || version.interpretation != pse_operations::generated::surreal::INTERPRETATION
        {
            return Err(contract(
                "owned source object interpretation or logical identity differs",
            ));
        }
        let value = SourceObjectValue {
            logical: edit.logical,
            scope: edit.scope,
            name: edit.name,
            version: version.key,
            kind: version.kind,
            payload: version.payload.into_vec(),
            references: edit.references,
        };
        let declaration = if value.kind.starts_with("modeling:") && value.kind != CONTEXT {
            Some(decode::<SourcePayload>(&value.payload)?)
        } else {
            None
        };
        let parsed = declaration
            .as_ref()
            .map(|(row, fields)| parsed_fields(runtime, row, fields.as_ref()))
            .transpose()?;
        Ok(Some(Arc::new(Self {
            value,
            lease,
            declaration,
            parsed,
        })))
    }
    #[cfg(test)]
    pub(crate) fn edit(&self) -> ObjectEdit {
        ObjectEdit {
            logical: self.logical.clone(),
            scope: self.scope.clone(),
            name: self.name.clone(),
            version: Some(pse_model::generated::runtime::canonical_versions::Row {
                key: self.version.clone(),
                logical: self.logical.clone(),
                kind: self.kind.clone(),
                payload: self.payload.clone().into(),
                interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            }),
            references: self.references.clone(),
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct SourceMember {
    pub(crate) logical: String,
    pub(crate) scope: String,
    pub(crate) name: String,
    pub(crate) version: String,
}
impl From<&LocalSourceObject> for SourceMember {
    fn from(object: &LocalSourceObject) -> Self {
        Self {
            logical: object.logical.clone(),
            scope: object.scope.clone(),
            name: object.name.clone(),
            version: object.version.clone(),
        }
    }
}
/// Complete immutable local source closure. Raw data bytes are owned here;
/// decoding remains restricted to the suppliers selected by each consumer.
#[derive(Debug)]
pub(crate) struct AdmittedModelRevision {
    pub(crate) objects: Arc<BTreeMap<String, Arc<LocalSourceObject>>>,
    inventories: Arc<BTreeMap<LocalInventory, Arc<InventoryBucket>>>,
    _metadata: Arc<pse_columnar::AllocationLease>,
    _receiver_metadata: Option<Arc<pse_columnar::AllocationLease>>,
    _validation_owner: pse_engine::session::BoundModel,
    _validation_context: Arc<pse_relations::validate::ValidationContext>,
}
#[derive(Debug)]
struct InventoryBucket {
    members: Vec<Arc<LocalSourceObject>>,
    _metadata: Arc<pse_columnar::AllocationLease>,
}
impl PartialEq for InventoryBucket {
    fn eq(&self, other: &Self) -> bool { self.members == other.members }
}
impl Eq for InventoryBucket {}
impl Hash for InventoryBucket {
    fn hash<H: Hasher>(&self, state: &mut H) { self.members.hash(state); }
}
impl AdmittedModelRevision {
    pub(super) fn validation_context(&self) -> &Arc<pse_relations::validate::ValidationContext> {
        &self._validation_context
    }
    /// Caller has established receiver-specific source admission. Shared maps and
    /// original leases remain owned, rather than transferred between pools.
    pub(super) fn rebind_for(
        &self, runtime: &Runtime,
        context: Arc<pse_relations::validate::ValidationContext>,
    ) -> Result<Arc<Self>, WorkflowError> {
        let metadata = runtime.shared.math().reserve("modeling:receiver-source-view", 4096 + size_of::<Self>())?;
        Ok(Arc::new(Self {
            objects: self.objects.clone(), inventories: self.inventories.clone(),
            _metadata: self._metadata.clone(), _receiver_metadata: Some(metadata),
            _validation_owner: runtime.model.clone(), _validation_context: context,
        }))
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self._metadata.size()
            + self._receiver_metadata.as_ref().map_or(0, |owner| owner.size())
            + self.inventories.values().map(|bucket| bucket._metadata.size()).sum::<usize>()
            + self.objects.values().map(|o| o.retained_bytes()).sum::<usize>()
    }
    pub(super) fn apply(
        runtime: &Runtime,
        previous: Option<&Self>,
        edits: &[ObjectEdit],
        cancel: Option<&crate::CancelSource>,
    ) -> Result<Arc<Self>, WorkflowError> {
        let check = || cancel.map_or(Ok(()), checkpoint);
        check()?;
        // Reserve copied map keys and temporary update views before cloning. A
        // bucket's separate lease owns its retained member-vector allocation.
        let old_bytes = previous.map_or(0, |p| {
            p.objects.keys().map(|key| key.capacity() + 256).sum::<usize>()
                + p.inventories.keys().map(|key| key.retained_bytes() + 256).sum::<usize>()
        });
        let edit_bytes = edits.iter().try_fold(0usize, |bytes, edit| {
            let kind = edit.version.as_ref().map_or("", |version| version.kind.as_str());
            let added = 4 * (edit.scope.len() + edit.name.len() + kind.len() + 512)
                + edit.references.iter().map(|(scope, name)| scope.len() + name.len() + kind.len() + 512).sum::<usize>();
            bytes.checked_add(added)?.checked_add(edit.logical.capacity() + 512)
        }).ok_or_else(|| contract("owned indexed source extent"))?;
        let staged_edit_bytes = edit_bytes.checked_mul(4).ok_or_else(|| contract("indexed staging extent"))?;
        let metadata = runtime.shared.math().reserve("modeling:owned-source-inventory",
            old_bytes.checked_mul(3).and_then(|old| old.checked_add(staged_edit_bytes))
                .and_then(|bytes| bytes.checked_add(size_of::<Self>() + 2048))
                .ok_or_else(|| contract("owned indexed source extent"))?)?;
        let mut objects = previous.map_or_else(BTreeMap::new, |p| p.objects.as_ref().clone());
        let mut inventories = previous.map_or_else(BTreeMap::new, |p| p.inventories.as_ref().clone());
        // Count possible insertions so each affected vector is reserved before
        // copying it, even when several edits touch the same bucket.
        let mut insertions = BTreeMap::<LocalInventory, usize>::new();
        for edit in edits {
            check()?;
            if let Some(version) = &edit.version {
                for key in LocalInventory::keys(&edit.scope, &edit.name, &version.kind, &edit.references) {
                    *insertions.entry(key).or_default() += 1;
                }
            }
        }
        let mut changed = BTreeMap::<LocalInventory, (Vec<Arc<LocalSourceObject>>, Arc<pse_columnar::AllocationLease>)>::new();
        for edit in edits {
            check()?;
            let old = objects.get(&edit.logical).cloned();
            if edit.version.as_ref().is_some_and(|version| old.as_ref().is_some_and(|old| {
                old.scope == edit.scope && old.name == edit.name && old.version == version.key
                    && old.kind == version.kind && old.payload.as_slice() == version.payload.as_slice()
                    && old.references == edit.references
            })) { continue; }
            let new = LocalSourceObject::from_edit(runtime, edit.clone())?;
            for object in old.iter().chain(new.iter()) {
                for query in LocalInventory::keys(&object.scope, &object.name, &object.kind, &object.references) {
                    if !changed.contains_key(&query) {
                        let existing = inventories.get(&query);
                        let capacity = existing.map_or(0, |bucket| bucket.members.len())
                            .checked_add(*insertions.get(&query).unwrap_or(&0))
                            .ok_or_else(|| contract("source bucket capacity"))?;
                        let lease = runtime.shared.math().reserve("modeling:source-inventory-bucket",
                            capacity.checked_mul(size_of::<Arc<LocalSourceObject>>())
                                .and_then(|bytes| bytes.checked_add(size_of::<InventoryBucket>() + 256))
                                .ok_or_else(|| contract("source bucket extent"))?)?;
                        let mut members = Vec::with_capacity(capacity);
                        if let Some(existing) = existing { members.extend(existing.members.iter().cloned()); }
                        changed.insert(query.clone(), (members, lease));
                    }
                    let (members, _) = changed.get_mut(&query).ok_or_else(|| contract("staged bucket absent"))?;
                    let position = members.binary_search_by(|member| member.logical.cmp(&edit.logical));
                    if new.as_ref().is_some_and(|new| Arc::ptr_eq(object, new)) {
                        match position { Ok(index) => members[index] = object.clone(), Err(index) => members.insert(index, object.clone()) }
                    } else if let Ok(index) = position { members.remove(index); }
                }
            }
            if let Some(new) = new { objects.insert(edit.logical.clone(), new); }
            else { objects.remove(&edit.logical); }
        }
        for (query, (members, metadata)) in changed {
            check()?;
            if members.is_empty() { inventories.remove(&query); }
            else { inventories.insert(query, Arc::new(InventoryBucket { members, _metadata: metadata })); }
        }
        let retained = objects.keys().map(|key| key.capacity() + 256).sum::<usize>()
            + inventories.keys().map(|key| key.retained_bytes() + 256).sum::<usize>()
            + size_of::<Self>() + 512;
        let metadata = metadata.partition(&[retained]).map_err(|_| contract("indexed source retained extent"))?
            .pop().ok_or_else(|| contract("indexed source metadata absent"))?;
        check()?;
        Ok(Arc::new(Self {
            objects: Arc::new(objects), inventories: Arc::new(inventories),
            _metadata: metadata, _receiver_metadata: None,
            _validation_owner: runtime.model.clone(), _validation_context: runtime.admission_context(),
        }))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum LocalInventory {
    Scope(String),
    Kind(Option<String>, String),
    Name(String, String),
    References(String, String, String),
}
impl LocalInventory {
    fn retained_bytes(&self) -> usize {
        size_of::<Self>() + match self {
            Self::Scope(scope) => scope.capacity(),
            Self::Kind(scope, kind) => scope.as_ref().map_or(0, String::capacity) + kind.capacity(),
            Self::Name(scope, name) => scope.capacity() + name.capacity(),
            Self::References(scope, name, kind) => scope.capacity() + name.capacity() + kind.capacity(),
        }
    }
    fn keys(scope: &str, name: &str, kind: &str, references: &[(String, String)]) -> BTreeSet<Self> {
        let mut keys = BTreeSet::from([
            Self::Scope(scope.into()), Self::Kind(Some(scope.into()), kind.into()),
            Self::Kind(None, kind.into()), Self::Name(scope.into(), name.into()),
        ]);
        keys.extend(references.iter().map(|(scope, name)| Self::References(scope.clone(), name.clone(), kind.into())));
        keys
    }
    #[cfg(test)]
    fn matches(&self, object: &LocalSourceObject) -> bool {
        match self {
            Self::Scope(scope) => &object.scope == scope,
            Self::Kind(scope, kind) => {
                &object.kind == kind && scope.as_ref().is_none_or(|scope| &object.scope == scope)
            }
            Self::Name(scope, name) => &object.scope == scope && &object.name == name,
            Self::References(scope, name, kind) => {
                &object.kind == kind && object.references.contains(&(scope.clone(), name.clone()))
            }
        }
    }
}
/// Exact payload/lookup premises. Hashes accelerate lookup but equality checks
/// all bytes, metadata and positive/absent namespace observations.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct LocalSourcePremises {
    logicals: BTreeMap<String, Option<Arc<LocalSourceObject>>>,
    inventories: BTreeMap<LocalInventory, Option<Arc<InventoryBucket>>>,
}
impl PartialOrd for LocalInventory {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for LocalInventory {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn key(value: &LocalInventory) -> (u8, Option<&str>, &str, &str) {
            match value {
                LocalInventory::Scope(s) => (0, None, s, ""),
                LocalInventory::Kind(s, k) => (1, s.as_deref(), k, ""),
                LocalInventory::Name(s, n) => (2, None, s, n),
                LocalInventory::References(s, n, k) => (3, Some(k), s, n),
            }
        }
        key(self).cmp(&key(other))
    }
}
impl LocalSourcePremises {
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.logicals.keys().map(|key| key.capacity() + 256).sum::<usize>()
            + self.inventories.keys().map(|key| key.retained_bytes() + 256).sum::<usize>()
    }
    #[cfg(test)]
    pub(crate) fn matches(&self, source: &AdmittedModelRevision) -> bool {
        self.matches_scoped(source, &crate::CancelSource::new()).unwrap_or(false)
    }
    pub(super) fn matches_scoped(&self, source: &AdmittedModelRevision, cancel: &crate::CancelSource) -> Result<bool, WorkflowError> {
        for (logical, old) in &self.logicals {
            checkpoint(cancel)?;
            if source.objects.get(logical) != old.as_ref() { return Ok(false); }
        }
        for (query, old) in &self.inventories {
            checkpoint(cancel)?;
            let current = source.inventories.get(query);
            match (old, current) {
                (None, None) => {},
                (Some(old), Some(current)) if Arc::ptr_eq(old, current) => {},
                (Some(old), Some(current)) if old.members.len() == current.members.len() => {
                    for (old, current) in old.members.iter().zip(&current.members) {
                        checkpoint(cancel)?;
                        if old != current { return Ok(false); }
                    }
                },
                _ => return Ok(false),
            }
        }
        Ok(true)
    }
    pub(super) fn logical(
        &mut self,
        source: &AdmittedModelRevision,
        key: &str,
    ) -> Option<Arc<LocalSourceObject>> {
        let object = source.objects.get(key).cloned();
        self.logicals.insert(key.into(), object.clone());
        object
    }
    pub(super) fn inventory(
        &mut self,
        source: &AdmittedModelRevision,
        query: LocalInventory,
    ) -> Vec<Arc<LocalSourceObject>> {
        let bucket = source.inventories.get(&query).cloned();
        let objects = bucket.as_ref().map_or_else(Vec::new, |bucket| bucket.members.clone());
        self.inventories.insert(query, bucket);
        objects
    }
}
#[derive(Debug)]
pub(super) struct LocalHead {
    pub(super) revision: String,
    pub(super) generation: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn object(logical: &str, scope: &str, name: &str, kind: &str, body: &[u8], references: &[(&str, &str)]) -> ObjectEdit {
        edit(logical.into(), scope.into(), name.into(), kind.into(), body.to_vec(),
            references.iter().map(|(scope, name)| ((*scope).into(), (*name).into())).collect())
    }
    fn oracle(source: &AdmittedModelRevision, query: &LocalInventory) -> Vec<Arc<LocalSourceObject>> {
        source.objects.values().filter(|object| query.matches(object)).cloned().collect()
    }
    fn queries(source: &AdmittedModelRevision) -> BTreeSet<LocalInventory> {
        let mut queries = BTreeSet::from([LocalInventory::Scope("absent".into()),
            LocalInventory::Name("scope".into(), "missing".into()),
            LocalInventory::References("target".into(), "missing".into(), "alpha".into())]);
        for object in source.objects.values() {
            queries.extend(LocalInventory::keys(&object.scope, &object.name, &object.kind, &object.references));
        }
        queries
    }
    #[test]
    fn plan39_local_inventory_exact_edits_match_independent_scan() {
        let runtime = crate::workflow::tests::runtime();
        let mut edits = (0..1024).map(|index| object(&format!("unrelated:{index}"), "other", &format!("n{index}"), "other", b"owned", &[])).collect::<Vec<_>>();
        edits.extend([
            object("a", "scope", "same", "alpha", b"a", &[("target", "x"), ("target", "x")]),
            object("b", "scope", "same", "beta", b"b", &[("target", "x")]),
            object("c", "scope", "c", "alpha", b"c", &[]),
        ]);
        let first = AdmittedModelRevision::apply(&runtime, None, &edits, None).unwrap();
        let mut premises = LocalSourcePremises::default();
        for query in queries(&first) {
            assert_eq!(premises.inventory(&first, query.clone()), oracle(&first, &query));
        }
        let no_op = AdmittedModelRevision::apply(&runtime, Some(&first), &[first.objects["a"].edit()], None).unwrap();
        assert!(Arc::ptr_eq(&first.objects["a"], &no_op.objects["a"]));
        assert!(first.inventories.iter().all(|(query, bucket)| Arc::ptr_eq(bucket, &no_op.inventories[query])));
        assert!(premises.matches(&no_op));
        let mut deletion = first.objects["c"].edit(); deletion.version = None;
        let changes = [
            object("a", "moved", "renamed", "beta", b"new", &[("target", "y")]),
            object("b", "scope", "same", "beta", b"changed payload", &[("target", "x")]),
            object("d", "scope", "same", "alpha", b"near shadow", &[]), deletion,
        ];
        let changed = AdmittedModelRevision::apply(&runtime, Some(&first), &changes, None).unwrap();
        assert!(!premises.matches(&changed));
        assert!(Arc::ptr_eq(&first.inventories[&LocalInventory::Scope("other".into())], &changed.inventories[&LocalInventory::Scope("other".into())]));
        assert!(!Arc::ptr_eq(&first.objects["b"], &changed.objects["b"]));
        for query in queries(&first).union(&queries(&changed)) {
            let mut observed = LocalSourcePremises::default();
            assert_eq!(observed.inventory(&changed, query.clone()), oracle(&changed, query));
        }
        assert_eq!(first.objects["a"].payload, b"a");
    }
    #[test]
    fn plan39_local_inventory_cancel_and_exact_absence_preserve_predecessor() {
        let runtime = crate::workflow::tests::runtime();
        let first = AdmittedModelRevision::apply(&runtime, None,
            &[object("a", "scope", "a", "alpha", b"original", &[])], None).unwrap();
        let mut observed = LocalSourcePremises::default();
        observed.logical(&first, "missing");
        observed.inventory(&first, LocalInventory::Name("scope".into(), "shadow".into()));
        let cancelled = crate::CancelSource::new(); cancelled.cancel();
        assert!(observed.matches_scoped(&first, &cancelled).is_err());
        assert!(AdmittedModelRevision::apply(&runtime, Some(&first),
            &[object("a", "scope", "a", "alpha", b"replacement", &[])], Some(&cancelled)).is_err());
        assert_eq!(first.objects["a"].payload, b"original");
        let changed = AdmittedModelRevision::apply(&runtime, Some(&first),
            &[object("missing", "scope", "shadow", "alpha", b"present", &[])], None).unwrap();
        assert!(!observed.matches(&changed));
    }
}
