// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned source bytes and exact local lookup premises, without store permission.
use super::*;
use rpds::RedBlackTreeMapSync;
use std::hash::{Hash, Hasher};

#[cfg(test)]
thread_local! {
    static CONSTRUCTION_DECODES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Issued together with the exact canonical edit. This is local construction
/// input, not a receiving-validation receipt or an alternative source model.
#[derive(Default)]
pub(super) struct TypedSourceInputs {
    declarations: BTreeMap<String, TypedDeclarationInput>,
    objects: BTreeMap<String, (String, TypedObjectPayload)>,
}
#[derive(Clone, Debug)]
pub(super) enum TypedObjectPayload {
    Fit(Arc<pse_model::generated::authored::fit_cases::Row>),
    Document(Arc<pse_model::generated::authored::documents::Row>),
    Package(Arc<pse_model::generated::authored::packages::Row>),
}
struct TypedDeclarationInput {
    version: String,
    payload: SourcePayload,
    parsed: Arc<pse_modeling::expression::occurrences::ParsedFields>,
}
impl TypedSourceInputs {
    pub(super) fn fit(
        &mut self,
        row: pse_model::generated::authored::fit_cases::Row,
    ) -> Result<ObjectEdit, WorkflowError> {
        let encoded = edit(
            format!("fit:{}", row.fit_id),
            "fits".into(),
            row.fit_id.to_string(),
            "fit".into(),
            payload(&row)?,
            Vec::new(),
        );
        self.object(encoded, TypedObjectPayload::Fit(Arc::new(row)))
    }
    pub(super) fn document(
        &mut self,
        row: pse_model::generated::authored::documents::Row,
    ) -> Result<ObjectEdit, WorkflowError> {
        let encoded = edit(
            format!("document:{}", row.document_id),
            "documents".into(),
            row.document_id.to_string(),
            "document".into(),
            payload(&row)?,
            Vec::new(),
        );
        self.object(encoded, TypedObjectPayload::Document(Arc::new(row)))
    }
    pub(super) fn package(
        &mut self,
        row: pse_model::generated::authored::packages::Row,
    ) -> Result<ObjectEdit, WorkflowError> {
        let encoded = edit(
            format!("package:{}", row.package_id),
            "packages".into(),
            row.package_id.to_string(),
            "package".into(),
            payload(&row)?,
            Vec::new(),
        );
        self.object(encoded, TypedObjectPayload::Package(Arc::new(row)))
    }
    fn object(
        &mut self,
        encoded: ObjectEdit,
        source: TypedObjectPayload,
    ) -> Result<ObjectEdit, WorkflowError> {
        let version = encoded
            .version
            .as_ref()
            .ok_or_else(|| contract("typed source object version absent"))?;
        self.objects
            .insert(encoded.logical.clone(), (version.key.clone(), source));
        Ok(encoded)
    }
    pub(super) fn source(&self, logical: &str) -> Option<&SourcePayload> {
        self.declarations.get(logical).map(|input| &input.payload)
    }
    pub(super) fn declaration(
        &mut self,
        row: &Declaration,
        fields: Option<FieldSpans>,
        parsed: Arc<pse_modeling::expression::occurrences::ParsedFields>,
        references: Vec<(String, String)>,
    ) -> Result<ObjectEdit, WorkflowError> {
        let source = (row.clone(), fields);
        let encoded = edit(
            logical(row.declaration_id),
            scope(row.parent_id),
            row.name.clone(),
            kind(row),
            payload(&source)?,
            references,
        );
        self.decoded(&encoded, source, parsed)?;
        Ok(encoded)
    }
    /// Actual imports have decoded and authenticated these bytes already.
    pub(super) fn decoded(
        &mut self,
        edit: &ObjectEdit,
        source: SourcePayload,
        parsed: Arc<pse_modeling::expression::occurrences::ParsedFields>,
    ) -> Result<(), WorkflowError> {
        let version = edit
            .version
            .as_ref()
            .ok_or_else(|| contract("typed source version absent"))?;
        if edit.logical != logical(source.0.declaration_id)
            || edit.scope != scope(source.0.parent_id)
            || edit.name != source.0.name
            || version.kind != kind(&source.0)
            || !parsed.matches(&source.0, source.1.as_ref())
        {
            return Err(contract(
                "typed source payload or parsed fields differ from membership",
            ));
        }
        self.declarations.insert(
            edit.logical.clone(),
            TypedDeclarationInput {
                version: version.key.clone(),
                payload: source,
                parsed,
            },
        );
        Ok(())
    }
}

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
    pub(super) typed: Option<TypedObjectPayload>,
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
    fn from_edit(
        services: &crate::workflow::PreparationServices,
        edit: ObjectEdit,
        input: Option<&TypedDeclarationInput>,
        typed: Option<&(String, TypedObjectPayload)>,
    ) -> Result<Option<Arc<Self>>, WorkflowError> {
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
        let declaration_extent = if (version.kind.starts_with("modeling:")
            && version.kind != CONTEXT)
            || typed.is_some()
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
        let lease = services.math().reserve(
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
        let (declaration, parsed) = if let Some(input) = input {
            if input.version != value.version {
                return Err(contract(
                    "typed source input belongs to another immutable version",
                ));
            }
            (Some(input.payload.clone()), Some(input.parsed.clone()))
        } else if value.kind.starts_with("modeling:") && value.kind != CONTEXT {
            #[cfg(test)]
            CONSTRUCTION_DECODES.with(|count| count.set(count.get() + 1));
            let source = decode::<SourcePayload>(&value.payload)?;
            let parsed = parsed_fields(services, &source.0, source.1.as_ref())?;
            (Some(source), Some(parsed))
        } else {
            (None, None)
        };
        let typed = typed
            .map(|(version, source)| {
                if version != &value.version {
                    Err(contract(
                        "typed object input belongs to another immutable version",
                    ))
                } else {
                    Ok(source.clone())
                }
            })
            .transpose()?;
        Ok(Some(Arc::new(Self {
            value,
            lease,
            declaration,
            parsed,
            typed,
        })))
    }
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
    pub(crate) objects: Arc<RedBlackTreeMapSync<String, Arc<LocalSourceObject>>>,
    inventories: Arc<RedBlackTreeMapSync<LocalInventory, Arc<InventoryBucket>>>,
    /// Conservative complete-root charge, not unique shared-node usage. Each
    /// live root covers its reachable nodes even after another root is released.
    map_bytes: usize,
    _metadata: Arc<pse_columnar::AllocationLease>,
    _receiver_metadata: Option<Arc<pse_columnar::AllocationLease>>,
    _validation_context: Arc<pse_relations::validate::ValidationContext>,
}
#[derive(Debug)]
struct InventoryBucket {
    members: Vec<Arc<LocalSourceObject>>,
    _metadata: Arc<pse_columnar::AllocationLease>,
}
impl PartialEq for InventoryBucket {
    fn eq(&self, other: &Self) -> bool {
        self.members == other.members
    }
}
impl Eq for InventoryBucket {}
impl Hash for InventoryBucket {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.members.hash(state);
    }
}
impl AdmittedModelRevision {
    pub(in crate::workflow::modeling) fn validation_context(
        &self,
    ) -> &Arc<pse_relations::validate::ValidationContext> {
        &self._validation_context
    }
    /// Caller has established receiver-specific source admission. Shared maps and
    /// original leases remain owned, rather than transferred between pools.
    pub(super) fn rebind_for(
        &self,
        context: Arc<pse_relations::validate::ValidationContext>,
        metadata: Arc<pse_columnar::AllocationLease>,
    ) -> Result<Arc<Self>, WorkflowError> {
        Ok(Arc::new(Self {
            objects: self.objects.clone(),
            inventories: self.inventories.clone(),
            map_bytes: self.map_bytes,
            _metadata: self._metadata.clone(),
            _receiver_metadata: Some(metadata),

            _validation_context: context,
        }))
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self._metadata.size()
            + self
                ._receiver_metadata
                .as_ref()
                .map_or(0, |owner| owner.size())
            + self
                .inventories
                .values()
                .map(|bucket| bucket._metadata.size())
                .sum::<usize>()
            + self
                .objects
                .values()
                .map(|o| o.retained_bytes())
                .sum::<usize>()
    }
    #[cfg(test)]
    pub(super) fn apply(
        services: &crate::workflow::PreparationServices,
        previous: Option<&Self>,
        edits: &[ObjectEdit],
        cancel: Option<&crate::CancelSource>,
    ) -> Result<Arc<Self>, WorkflowError> {
        Self::apply_inputs(
            services,
            previous,
            edits,
            &TypedSourceInputs::default(),
            cancel,
        )
    }
    pub(super) fn apply_inputs(
        services: &crate::workflow::PreparationServices,
        previous: Option<&Self>,
        edits: &[ObjectEdit],
        inputs: &TypedSourceInputs,
        cancel: Option<&crate::CancelSource>,
    ) -> Result<Arc<Self>, WorkflowError> {
        let check = || cancel.map_or(Ok(()), checkpoint);
        check()?;
        // rpds 1.2.1 shares both tree nodes and entries; cloning a root does not
        // traverse keys. Cover full old/new roots plus mutation/staging overlap
        // conservatively; individual hidden shared nodes are not reported.
        let old_bytes = previous.map_or(0, |p| p.map_bytes);
        let edit_bytes = edits
            .iter()
            .try_fold(0usize, |bytes, edit| {
                let kind = edit
                    .version
                    .as_ref()
                    .map_or("", |version| version.kind.as_str());
                let added = 4 * (edit.scope.len() + edit.name.len() + kind.len() + 512)
                    + edit
                        .references
                        .iter()
                        .map(|(scope, name)| scope.len() + name.len() + kind.len() + 512)
                        .sum::<usize>();
                bytes
                    .checked_add(added)?
                    .checked_add(edit.logical.capacity() + 512)
            })
            .ok_or_else(|| contract("owned indexed source extent"))?;
        let staged_edit_bytes = edit_bytes
            .checked_mul(4)
            .ok_or_else(|| contract("indexed staging extent"))?;
        let metadata = services.math().reserve(
            "modeling:owned-source-inventory",
            old_bytes
                .checked_mul(3)
                .and_then(|old| old.checked_add(staged_edit_bytes))
                .and_then(|bytes| bytes.checked_add(size_of::<Self>() + 2048))
                .ok_or_else(|| contract("owned indexed source extent"))?,
        )?;
        let mut objects = previous.map_or_else(RedBlackTreeMapSync::new_sync, |p| {
            p.objects.as_ref().clone()
        });
        let mut inventories = previous.map_or_else(RedBlackTreeMapSync::new_sync, |p| {
            p.inventories.as_ref().clone()
        });
        let mut map_bytes = old_bytes;
        // Count possible insertions so each affected vector is reserved before
        // copying it, even when several edits touch the same bucket.
        let mut insertions = BTreeMap::<LocalInventory, usize>::new();
        for edit in edits {
            check()?;
            if let Some(version) = &edit.version {
                for key in
                    LocalInventory::keys(&edit.scope, &edit.name, &version.kind, &edit.references)
                {
                    *insertions.entry(key).or_default() += 1;
                }
            }
        }
        let mut changed = BTreeMap::<
            LocalInventory,
            (
                Vec<Arc<LocalSourceObject>>,
                Arc<pse_columnar::AllocationLease>,
            ),
        >::new();
        for edit in edits {
            check()?;
            if edit.version.as_ref().is_some_and(|version| {
                version.logical != edit.logical
                    || version.interpretation != pse_operations::generated::surreal::INTERPRETATION
            }) {
                return Err(contract(
                    "owned source object interpretation or logical identity differs",
                ));
            }
            let old = objects.get(&edit.logical).cloned();
            if edit.version.as_ref().is_some_and(|version| {
                old.as_ref().is_some_and(|old| {
                    old.scope == edit.scope
                        && old.name == edit.name
                        && old.version == version.key
                        && old.kind == version.kind
                        && old.payload.as_slice() == version.payload.as_slice()
                        && old.references == edit.references
                })
            }) {
                continue;
            }
            let new = LocalSourceObject::from_edit(
                services,
                edit.clone(),
                inputs.declarations.get(&edit.logical),
                inputs.objects.get(&edit.logical),
            )?;
            for object in old.iter().chain(new.iter()) {
                for query in LocalInventory::keys(
                    &object.scope,
                    &object.name,
                    &object.kind,
                    &object.references,
                ) {
                    if !changed.contains_key(&query) {
                        let existing = inventories.get(&query);
                        let capacity = existing
                            .map_or(0, |bucket| bucket.members.len())
                            .checked_add(*insertions.get(&query).unwrap_or(&0))
                            .ok_or_else(|| contract("source bucket capacity"))?;
                        let lease = services.math().reserve(
                            "modeling:source-inventory-bucket",
                            capacity
                                .checked_mul(size_of::<Arc<LocalSourceObject>>())
                                .and_then(|bytes| {
                                    bytes.checked_add(size_of::<InventoryBucket>() + 256)
                                })
                                .ok_or_else(|| contract("source bucket extent"))?,
                        )?;
                        let mut members = Vec::with_capacity(capacity);
                        if let Some(existing) = existing {
                            members.extend(existing.members.iter().cloned());
                        }
                        changed.insert(query.clone(), (members, lease));
                    }
                    let (members, _) = changed
                        .get_mut(&query)
                        .ok_or_else(|| contract("staged bucket absent"))?;
                    let position =
                        members.binary_search_by(|member| member.logical.cmp(&edit.logical));
                    if new.as_ref().is_some_and(|new| Arc::ptr_eq(object, new)) {
                        match position {
                            Ok(index) => members[index] = object.clone(),
                            Err(index) => members.insert(index, object.clone()),
                        }
                    } else if let Ok(index) = position {
                        members.remove(index);
                    }
                }
            }
            if let Some(new) = new {
                if let Some((key, _)) = objects.get_key_value(&edit.logical) {
                    map_bytes = map_bytes
                        .checked_sub(key.capacity() + size_of::<String>() + 256)
                        .ok_or_else(|| contract("source map extent"))?;
                }
                let key = edit.logical.clone();
                map_bytes = map_bytes
                    .checked_add(key.capacity() + size_of::<String>() + 256)
                    .ok_or_else(|| contract("source map extent"))?;
                objects.insert_mut(key, new);
            } else {
                if let Some((key, _)) = objects.get_key_value(&edit.logical) {
                    map_bytes = map_bytes
                        .checked_sub(key.capacity() + size_of::<String>() + 256)
                        .ok_or_else(|| contract("source map extent"))?;
                }
                objects.remove_mut(&edit.logical);
            }
        }
        for (query, (members, metadata)) in changed {
            check()?;
            if members.is_empty() {
                if let Some((key, _)) = inventories.get_key_value(&query) {
                    map_bytes = map_bytes
                        .checked_sub(key.retained_bytes() + 256)
                        .ok_or_else(|| contract("source inventory extent"))?;
                }
                inventories.remove_mut(&query);
            } else {
                if let Some((key, _)) = inventories.get_key_value(&query) {
                    map_bytes = map_bytes
                        .checked_sub(key.retained_bytes() + 256)
                        .ok_or_else(|| contract("source inventory extent"))?;
                }
                map_bytes = map_bytes
                    .checked_add(query.retained_bytes() + 256)
                    .ok_or_else(|| contract("source inventory extent"))?;
                inventories.insert_mut(
                    query,
                    Arc::new(InventoryBucket {
                        members,
                        _metadata: metadata,
                    }),
                );
            }
        }
        let retained = map_bytes
            .checked_add(size_of::<Self>() + 512)
            .ok_or_else(|| contract("indexed retained root extent"))?;
        let metadata = metadata
            .partition(&[retained])
            .map_err(|_| contract("indexed source retained extent"))?
            .pop()
            .ok_or_else(|| contract("indexed source metadata absent"))?;
        check()?;
        Ok(Arc::new(Self {
            objects: Arc::new(objects),
            inventories: Arc::new(inventories),
            map_bytes,
            _metadata: metadata,
            _receiver_metadata: None,

            _validation_context: services.admission_context(),
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
        size_of::<Self>()
            + match self {
                Self::Scope(scope) => scope.capacity(),
                Self::Kind(scope, kind) => {
                    scope.as_ref().map_or(0, String::capacity) + kind.capacity()
                }
                Self::Name(scope, name) => scope.capacity() + name.capacity(),
                Self::References(scope, name, kind) => {
                    scope.capacity() + name.capacity() + kind.capacity()
                }
            }
    }
    fn keys(
        scope: &str,
        name: &str,
        kind: &str,
        references: &[(String, String)],
    ) -> BTreeSet<Self> {
        let mut keys = BTreeSet::from([
            Self::Scope(scope.into()),
            Self::Kind(Some(scope.into()), kind.into()),
            Self::Kind(None, kind.into()),
            Self::Name(scope.into(), name.into()),
        ]);
        keys.extend(
            references
                .iter()
                .map(|(scope, name)| Self::References(scope.clone(), name.clone(), kind.into())),
        );
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
            + self
                .logicals
                .keys()
                .map(|key| key.capacity() + 256)
                .sum::<usize>()
            + self
                .inventories
                .keys()
                .map(|key| key.retained_bytes() + 256)
                .sum::<usize>()
    }
    #[cfg(test)]
    pub(crate) fn matches(&self, source: &AdmittedModelRevision) -> bool {
        self.matches_scoped(source, &crate::CancelSource::new())
            .unwrap_or(false)
    }
    pub(crate) fn matches_scoped(
        &self,
        source: &AdmittedModelRevision,
        cancel: &crate::CancelSource,
    ) -> Result<bool, WorkflowError> {
        for (logical, old) in &self.logicals {
            checkpoint(cancel)?;
            if source.objects.get(logical) != old.as_ref() {
                return Ok(false);
            }
        }
        for (query, old) in &self.inventories {
            checkpoint(cancel)?;
            let current = source.inventories.get(query);
            match (old, current) {
                (None, None) => {}
                (Some(old), Some(current)) if Arc::ptr_eq(old, current) => {}
                (Some(old), Some(current)) if old.members.len() == current.members.len() => {
                    for (old, current) in old.members.iter().zip(&current.members) {
                        checkpoint(cancel)?;
                        if old != current {
                            return Ok(false);
                        }
                    }
                }
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
        let objects = bucket
            .as_ref()
            .map_or_else(Vec::new, |bucket| bucket.members.clone());
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
    fn object(
        logical: &str,
        scope: &str,
        name: &str,
        kind: &str,
        body: &[u8],
        references: &[(&str, &str)],
    ) -> ObjectEdit {
        edit(
            logical.into(),
            scope.into(),
            name.into(),
            kind.into(),
            body.to_vec(),
            references
                .iter()
                .map(|(scope, name)| ((*scope).into(), (*name).into()))
                .collect(),
        )
    }
    fn oracle(
        source: &AdmittedModelRevision,
        query: &LocalInventory,
    ) -> Vec<Arc<LocalSourceObject>> {
        source
            .objects
            .values()
            .filter(|object| query.matches(object))
            .cloned()
            .collect()
    }
    fn queries(source: &AdmittedModelRevision) -> BTreeSet<LocalInventory> {
        let mut queries = BTreeSet::from([
            LocalInventory::Scope("absent".into()),
            LocalInventory::Name("scope".into(), "missing".into()),
            LocalInventory::References("target".into(), "missing".into(), "alpha".into()),
        ]);
        for object in source.objects.values() {
            queries.extend(LocalInventory::keys(
                &object.scope,
                &object.name,
                &object.kind,
                &object.references,
            ));
        }
        queries
    }
    #[test]
    fn local_typed_source_preserves_encoded_identity_and_exact_parser_owner() {
        let runtime = crate::workflow::tests::runtime();
        let row = pse_authoring::language::parse(
            "package p {param a:Scalar=1;}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap()
        .remove(0);
        let parsed = parsed_fields(&runtime.preparation_services(), &row, None).unwrap();
        let mut inputs = TypedSourceInputs::default();
        let edit = inputs
            .declaration(&row, None, parsed.clone(), Vec::new())
            .unwrap();
        CONSTRUCTION_DECODES.with(|count| count.set(0));
        let typed = AdmittedModelRevision::apply_inputs(
            &runtime.preparation_services(),
            None,
            std::slice::from_ref(&edit),
            &inputs,
            None,
        )
        .unwrap();
        assert_eq!(CONSTRUCTION_DECODES.with(std::cell::Cell::get), 0);
        let imported = AdmittedModelRevision::apply(
            &runtime.preparation_services(),
            None,
            std::slice::from_ref(&edit),
            None,
        )
        .unwrap();
        assert_eq!(CONSTRUCTION_DECODES.with(std::cell::Cell::get), 1);
        let object = &typed.objects[&edit.logical];
        assert_eq!(object.value, imported.objects[&edit.logical].value);
        assert_eq!(
            object.declaration,
            imported.objects[&edit.logical].declaration
        );
        // A cache hit alone does not establish carry-through: the exact caller's
        // parser owner is retained by the newly constructed source object.
        assert!(Arc::ptr_eq(object.parsed.as_ref().unwrap(), &parsed));
        let mut other = edit.clone();
        other.version.as_mut().unwrap().key.push_str("-different");
        assert!(
            AdmittedModelRevision::apply_inputs(
                &runtime.preparation_services(),
                None,
                &[other],
                &inputs,
                None
            )
            .is_err()
        );
    }
    #[test]
    fn local_shared_indexes_keep_old_roots_and_charge_complete_live_roots() {
        let runtime = crate::workflow::tests::runtime();
        let edits = (0..128)
            .map(|i| {
                object(
                    &format!("object:{i}"),
                    "scope",
                    &format!("name{i}"),
                    "item",
                    b"initial",
                    &[],
                )
            })
            .collect::<Vec<_>>();
        let old = AdmittedModelRevision::apply(&runtime.preparation_services(), None, &edits, None)
            .unwrap();
        let same =
            AdmittedModelRevision::apply(&runtime.preparation_services(), Some(&old), &[], None)
                .unwrap();
        assert!(old.objects.ptr_eq(same.objects.as_ref()));
        assert!(old.inventories.ptr_eq(same.inventories.as_ref()));
        let changed = AdmittedModelRevision::apply(
            &runtime.preparation_services(),
            Some(&old),
            &[object(
                "object:1",
                "scope",
                "name1",
                "item",
                b"changed",
                &[],
            )],
            None,
        )
        .unwrap();
        assert_eq!(old.objects["object:1"].payload, b"initial");
        assert_eq!(changed.objects["object:1"].payload, b"changed");
        assert!(Arc::ptr_eq(
            &old.objects["object:2"],
            &changed.objects["object:2"]
        ));
        assert_eq!(old.map_bytes, changed.map_bytes);
        let expected = changed
            .objects
            .keys()
            .map(|key| key.capacity() + size_of::<String>() + 256)
            .sum::<usize>()
            + changed
                .inventories
                .keys()
                .map(|key| key.retained_bytes() + 256)
                .sum::<usize>();
        assert_eq!(changed.map_bytes, expected);
        assert!(changed._metadata.size() >= expected);
        drop(old);
        drop(same);
        assert_eq!(changed.objects["object:2"].payload, b"initial");
    }
    #[test]
    fn plan39_local_inventory_exact_edits_match_independent_scan() {
        let runtime = crate::workflow::tests::runtime();
        let mut edits = (0..1024)
            .map(|index| {
                object(
                    &format!("unrelated:{index}"),
                    "other",
                    &format!("n{index}"),
                    "other",
                    b"owned",
                    &[],
                )
            })
            .collect::<Vec<_>>();
        edits.extend([
            object(
                "a",
                "scope",
                "same",
                "alpha",
                b"a",
                &[("target", "x"), ("target", "x")],
            ),
            object("b", "scope", "same", "beta", b"b", &[("target", "x")]),
            object("c", "scope", "c", "alpha", b"c", &[]),
        ]);
        let first =
            AdmittedModelRevision::apply(&runtime.preparation_services(), None, &edits, None)
                .unwrap();
        let mut premises = LocalSourcePremises::default();
        for query in queries(&first) {
            assert_eq!(
                premises.inventory(&first, query.clone()),
                oracle(&first, &query)
            );
        }
        let no_op = AdmittedModelRevision::apply(
            &runtime.preparation_services(),
            Some(&first),
            &[first.objects["a"].edit()],
            None,
        )
        .unwrap();
        assert!(Arc::ptr_eq(&first.objects["a"], &no_op.objects["a"]));
        assert!(
            first
                .inventories
                .iter()
                .all(|(query, bucket)| Arc::ptr_eq(bucket, &no_op.inventories[query]))
        );
        assert!(premises.matches(&no_op));
        for corrupt_logical in [true, false] {
            let mut forged = first.objects["a"].edit();
            let version = forged.version.as_mut().unwrap();
            if corrupt_logical {
                version.logical = "foreign".into();
            } else {
                version.interpretation = "foreign".into();
            }
            assert!(
                AdmittedModelRevision::apply(
                    &runtime.preparation_services(),
                    Some(&first),
                    &[forged],
                    None
                )
                .is_err()
            );
        }
        let mut deletion = first.objects["c"].edit();
        deletion.version = None;
        let changes = [
            object("a", "moved", "renamed", "beta", b"new", &[("target", "y")]),
            object(
                "b",
                "scope",
                "same",
                "beta",
                b"changed payload",
                &[("target", "x")],
            ),
            object("d", "scope", "same", "alpha", b"near shadow", &[]),
            deletion,
        ];
        let changed = AdmittedModelRevision::apply(
            &runtime.preparation_services(),
            Some(&first),
            &changes,
            None,
        )
        .unwrap();
        assert!(!premises.matches(&changed));
        assert!(Arc::ptr_eq(
            &first.inventories[&LocalInventory::Scope("other".into())],
            &changed.inventories[&LocalInventory::Scope("other".into())]
        ));
        assert!(!Arc::ptr_eq(&first.objects["b"], &changed.objects["b"]));
        for query in queries(&first).union(&queries(&changed)) {
            let mut observed = LocalSourcePremises::default();
            assert_eq!(
                observed.inventory(&changed, query.clone()),
                oracle(&changed, query)
            );
        }
        assert_eq!(first.objects["a"].payload, b"a");
    }
    #[test]
    fn plan39_local_inventory_cancel_and_exact_absence_preserve_predecessor() {
        let runtime = crate::workflow::tests::runtime();
        let first = AdmittedModelRevision::apply(
            &runtime.preparation_services(),
            None,
            &[object("a", "scope", "a", "alpha", b"original", &[])],
            None,
        )
        .unwrap();
        let mut observed = LocalSourcePremises::default();
        observed.logical(&first, "missing");
        observed.inventory(
            &first,
            LocalInventory::Name("scope".into(), "shadow".into()),
        );
        let cancelled = crate::CancelSource::new();
        cancelled.cancel();
        assert!(observed.matches_scoped(&first, &cancelled).is_err());
        assert!(
            AdmittedModelRevision::apply(
                &runtime.preparation_services(),
                Some(&first),
                &[object("a", "scope", "a", "alpha", b"replacement", &[])],
                Some(&cancelled)
            )
            .is_err()
        );
        assert_eq!(first.objects["a"].payload, b"original");
        let changed = AdmittedModelRevision::apply(
            &runtime.preparation_services(),
            Some(&first),
            &[object(
                "missing",
                "scope",
                "shadow",
                "alpha",
                b"present",
                &[],
            )],
            None,
        )
        .unwrap();
        assert!(!observed.matches(&changed));
    }
}
