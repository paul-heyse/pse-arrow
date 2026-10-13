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
}
impl std::ops::Deref for LocalSourceObject {
    type Target = SourceObjectValue;
    fn deref(&self) -> &Self::Target { &self.value }
}
impl PartialEq for LocalSourceObject {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Eq for LocalSourceObject {}
impl Hash for LocalSourceObject {
    fn hash<H: Hasher>(&self, state: &mut H) { self.value.hash(state); }
}
impl LocalSourceObject {
    pub(crate) fn retained_bytes(&self) -> usize { self.lease.size() }
    fn from_edit(runtime: &Runtime, edit: ObjectEdit) -> Result<Option<Arc<Self>>, WorkflowError> {
        let Some(version) = edit.version else { return Ok(None); };
        let extent = edit.logical.capacity() + edit.scope.capacity() + edit.name.capacity()
            + version.owned_bytes() + edit.references.iter().map(|(s,n)| s.capacity()+n.capacity()+128).sum::<usize>()
            + size_of::<Self>() + 256;
        let lease = runtime.shared.math().reserve("modeling:owned-source-object", extent)?;
        if version.logical != edit.logical || version.interpretation != pse_operations::generated::surreal::INTERPRETATION {
            return Err(contract("owned source object interpretation or logical identity differs"));
        }
        let value = SourceObjectValue {
            logical: edit.logical, scope: edit.scope, name: edit.name, version: version.key,
            kind: version.kind, payload: version.payload.into_vec(), references: edit.references,
        };
        Ok(Some(Arc::new(Self { value, lease })))
    }
    pub(crate) fn edit(&self) -> ObjectEdit {
        ObjectEdit {
            logical: self.logical.clone(), scope: self.scope.clone(), name: self.name.clone(),
            version: Some(pse_model::generated::runtime::canonical_versions::Row {
                key: self.version.clone(), logical: self.logical.clone(), kind: self.kind.clone(),
                payload: self.payload.clone().into(), interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            }), references: self.references.clone(),
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
        Self { logical: object.logical.clone(), scope: object.scope.clone(), name: object.name.clone(), version: object.version.clone() }
    }
}
/// Complete immutable local source closure. Raw data bytes are owned here;
/// decoding remains restricted to the suppliers selected by each consumer.
#[derive(Debug)]
pub(crate) struct AdmittedModelRevision {
    pub(crate) objects: BTreeMap<String, Arc<LocalSourceObject>>,
    _metadata: Arc<pse_columnar::AllocationLease>,
    _validation_owner: pse_engine::session::BoundModel,
}
impl AdmittedModelRevision {
    pub(crate) fn retained_bytes(&self) -> usize {
        self._metadata.size() + self.objects.values().map(|o| o.retained_bytes()).sum::<usize>()
    }
    pub(super) fn apply(
        runtime: &Runtime,
        previous: Option<&Self>,
        edits: &[ObjectEdit],
    ) -> Result<Arc<Self>, WorkflowError> {
        let count = previous.map_or(0, |p| p.objects.len()).checked_add(edits.len())
            .ok_or_else(|| contract("owned source inventory extent"))?;
        let metadata = runtime.shared.math().reserve("modeling:owned-source-inventory", count.saturating_mul(256).saturating_add(size_of::<Self>()+256))?;
        let mut objects = previous.map_or_else(BTreeMap::new, |p| p.objects.clone());
        for edit in edits {
            match &edit.version {
                None => { objects.remove(&edit.logical); }
                Some(version) if objects.get(&edit.logical).is_some_and(|old| {
                    old.scope == edit.scope && old.name == edit.name && old.version == version.key
                    && old.kind == version.kind && old.payload.as_slice() == version.payload.as_slice()
                    && old.references == edit.references
                }) => {}
                Some(_) => { if let Some(object) = LocalSourceObject::from_edit(runtime, edit.clone())? { objects.insert(edit.logical.clone(), object); } }
            }
        }
        Ok(Arc::new(Self { objects, _metadata: metadata, _validation_owner: runtime.model.clone() }))
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
    fn matches(&self, object: &LocalSourceObject) -> bool {
        match self {
            Self::Scope(scope) => &object.scope == scope,
            Self::Kind(scope, kind) => &object.kind == kind && scope.as_ref().is_none_or(|scope| &object.scope == scope),
            Self::Name(scope,name) => &object.scope == scope && &object.name == name,
            Self::References(scope,name,kind) => &object.kind == kind && object.references.contains(&(scope.clone(),name.clone())),
        }
    }
}
/// Exact payload/lookup premises. Hashes accelerate lookup but equality checks
/// all bytes, metadata and positive/absent namespace observations.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct LocalSourcePremises {
    logicals: BTreeMap<String, Option<Arc<LocalSourceObject>>>,
    inventories: BTreeMap<LocalInventory, Vec<Arc<LocalSourceObject>>>,
}
impl PartialOrd for LocalInventory {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}
impl Ord for LocalInventory {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn key(value: &LocalInventory) -> (u8,Option<&str>,&str,&str) { match value {
            LocalInventory::Scope(s)=>(0,None,s,""), LocalInventory::Kind(s,k)=>(1,s.as_deref(),k,""),
            LocalInventory::Name(s,n)=>(2,None,s,n), LocalInventory::References(s,n,k)=>(3,Some(k),s,n),
        }}
        key(self).cmp(&key(other))
    }
}
impl LocalSourcePremises {
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>() + self.logicals.keys().map(|k| k.capacity()+size_of::<Option<Arc<LocalSourceObject>>>()+128).sum::<usize>()
            + self.inventories.iter().map(|(query,v)| {
                let query_bytes = match query {
                    LocalInventory::Scope(s) => s.capacity(),
                    LocalInventory::Kind(s,k) => s.as_ref().map_or(0,String::capacity)+k.capacity(),
                    LocalInventory::Name(s,n) => s.capacity()+n.capacity(),
                    LocalInventory::References(s,n,k) => s.capacity()+n.capacity()+k.capacity(),
                };
                query_bytes+v.capacity()*size_of::<Arc<LocalSourceObject>>()+256
            }).sum::<usize>()
    }
    pub(crate) fn matches(&self, source: &AdmittedModelRevision) -> bool {
        self.logicals.iter().all(|(logical,old)| source.objects.get(logical) == old.as_ref())
            && self.inventories.iter().all(|(query,old)| {
                let current = source.objects.values().filter(|object|query.matches(object)).collect::<Vec<_>>();
                current.len() == old.len() && current.iter().zip(old).all(|(a,b)| *a == b)
            })
    }
    pub(super) fn logical(&mut self, source: &AdmittedModelRevision, key: &str) -> Option<Arc<LocalSourceObject>> {
        let object = source.objects.get(key).cloned(); self.logicals.insert(key.into(),object.clone()); object
    }
    pub(super) fn inventory(&mut self, source: &AdmittedModelRevision, query: LocalInventory) -> Vec<Arc<LocalSourceObject>> {
        let objects = source.objects.values().filter(|object|query.matches(object)).cloned().collect::<Vec<_>>();
        self.inventories.insert(query,objects.clone()); objects
    }
    pub(crate) fn versions(&self) -> BTreeMap<String,String> {
        self.logicals.values().flatten().chain(self.inventories.values().flatten()).map(|o|(o.logical.clone(),o.version.clone())).collect()
    }
}
#[derive(Debug)]
pub(super) struct LocalHead { pub(super) revision: String, pub(super) generation: i64 }
