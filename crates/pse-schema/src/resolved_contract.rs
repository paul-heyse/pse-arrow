// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native semantic contracts, exact graph admission and registry-owned proofs.
//! Descriptions never establish array, relational or publication validity.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use arrow_schema::Field;
use pse_ids::{ContentHash, SemanticId};

use crate::model::{EnumSpec, ExtensionUse, FieldContract, RelationSpec};
use crate::{Registry, SchemaError};

/// Resolved extension definition; parameters remain on the native field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedExtensionContract {
    /// Registered extension name.
    pub name: String,
    /// Exact physical storage including nested dictionary ordering.
    pub storage: FieldContract,
    /// Declared metadata encoding.
    pub metadata_kind: String,
    /// Metadata version.
    pub version: u32,
}

/// Native local declaration plus edges into shared resolved definitions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRelationContract {
    /// Complete native declaration, including prose and its legacy fingerprint.
    /// The separate semantic projection lives in fingerprint::semantic_relation.
    pub declaration: RelationSpec,
    /// Resolved native execution fields, with semantic metadata.
    pub fields: Vec<FieldContract>,
    /// Referenced relation identities, sorted and unique.
    pub references: Vec<SemanticId>,
    /// Referenced enum identities, sorted and unique.
    pub enums: Vec<SemanticId>,
    /// Referenced extension names, sorted and unique.
    pub extensions: Vec<String>,
}

/// Frozen generated expectations. This description cannot mint an admitted handle.
#[derive(Debug)]
pub struct GeneratedContracts {
    pub(crate) relations: BTreeMap<SemanticId, ResolvedRelationContract>,
    pub(crate) enums: BTreeMap<SemanticId, EnumSpec>,
    pub(crate) extensions: BTreeMap<String, ResolvedExtensionContract>,
}

impl GeneratedContracts {
    /// Complete relation contracts in semantic identity order.
    pub fn relations(&self) -> impl Iterator<Item = &ResolvedRelationContract> {
        self.relations.values()
    }
    /// Complete enum contracts in semantic identity order.
    pub fn enums(&self) -> impl Iterator<Item = &EnumSpec> {
        self.enums.values()
    }
    /// Complete extension contracts in name order.
    pub fn extensions(&self) -> impl Iterator<Item = &ResolvedExtensionContract> {
        self.extensions.values()
    }

    /// Assemble frozen descriptions generated independently of the runtime registry.
    /// # Errors
    /// Duplicate identities, missing definitions or ambiguous native fields.
    pub fn new(
        relations: Vec<ResolvedRelationContract>,
        enums: Vec<EnumSpec>,
        extensions: Vec<ResolvedExtensionContract>,
    ) -> Result<Self, SchemaError> {
        let mut result = Self {
            relations: BTreeMap::new(),
            enums: BTreeMap::new(),
            extensions: BTreeMap::new(),
        };
        for relation in relations {
            crate::field_contract::declaration(&arrow_schema::Schema::new(
                relation
                    .fields
                    .iter()
                    .map(|field| field.field().clone())
                    .collect::<Vec<_>>(),
            ))?;
            if result
                .relations
                .insert(relation.declaration.id, relation)
                .is_some()
            {
                return Err(invalid("duplicate resolved relation"));
            }
        }
        for value in enums {
            if result.enums.insert(value.id, value).is_some() {
                return Err(invalid("duplicate resolved enum"));
            }
        }
        for value in extensions {
            if result
                .extensions
                .insert(value.name.clone(), value)
                .is_some()
            {
                return Err(invalid("duplicate resolved extension"));
            }
        }
        for relation in result.relations.values() {
            if relation
                .references
                .iter()
                .any(|id| !result.relations.contains_key(id))
                || relation
                    .enums
                    .iter()
                    .any(|id| !result.enums.contains_key(id))
                || relation
                    .extensions
                    .iter()
                    .any(|name| !result.extensions.contains_key(name))
            {
                return Err(invalid("unresolved contract graph edge"));
            }
        }
        Ok(result)
    }
}

#[derive(Debug)]
struct GeneratedProof {
    expected: &'static GeneratedContracts,
    relations: BTreeSet<SemanticId>,
}

/// One immutable registry's semantic graph and successful generated bindings.
#[derive(Debug)]
pub(crate) struct ContractArena {
    pub(crate) graph: GeneratedContracts,
    schemas: BTreeMap<SemanticId, arrow_schema::SchemaRef>,
    proofs: Mutex<Vec<GeneratedProof>>,
    #[cfg(test)]
    comparisons: std::sync::atomic::AtomicUsize,
}

impl ContractArena {
    pub(crate) fn resolve(registry: &Registry) -> Result<Arc<Self>, SchemaError> {
        let relations = registry
            .relations()
            .iter()
            .map(|spec| resolve_relation(registry, spec))
            .collect::<Result<Vec<_>, _>>()?;
        let enum_ids: BTreeSet<_> = relations
            .iter()
            .flat_map(|r| r.enums.iter().copied())
            .collect();
        let extension_names: BTreeSet<_> = relations
            .iter()
            .flat_map(|r| r.extensions.iter().cloned())
            .collect();
        let enums = registry
            .enums()
            .iter()
            .filter(|e| enum_ids.contains(&e.id))
            .cloned()
            .map(|mut e| {
                for member in &mut e.members {
                    member.doc = "";
                }
                e
            })
            .collect();
        let extensions = crate::model::EXTENSION_TYPES
            .iter()
            .filter(|e| extension_names.contains(e.name))
            .map(|e| {
                Ok(ResolvedExtensionContract {
                    name: e.name.to_owned(),
                    storage: FieldContract::from_field(semantic_field(&Field::new(
                        "item",
                        e.storage(),
                        false,
                    ))?),
                    metadata_kind: e.metadata.to_string(),
                    version: e.metadata_version,
                })
            })
            .collect::<Result<_, SchemaError>>()?;
        Ok(Arc::new(Self {
            graph: GeneratedContracts::new(relations, enums, extensions)?,
            schemas: registry
                .relations()
                .iter()
                .map(|spec| {
                    Ok((
                        spec.id,
                        Arc::new(crate::arrow::uncached_relation_schema(registry, spec)?),
                    ))
                })
                .collect::<Result<_, SchemaError>>()?,
            proofs: Mutex::new(Vec::new()),
            #[cfg(test)]
            comparisons: std::sync::atomic::AtomicUsize::new(0),
        }))
    }

    pub(crate) fn handle(
        self: &Arc<Self>,
        id: SemanticId,
    ) -> Result<RelationContractHandle, SchemaError> {
        if !self.graph.relations.contains_key(&id) {
            return Err(invalid("relation absent from contract arena"));
        }
        Ok(RelationContractHandle {
            arena: Arc::clone(self),
            id,
        })
    }
}

/// Sealed semantic admission bound to the actual immutable registry arena.
/// Keeping a handle alive retains its definitions, not a process-global cache.
#[derive(Clone, Debug)]
pub struct RelationContractHandle {
    arena: Arc<ContractArena>,
    id: SemanticId,
}

impl RelationContractHandle {
    /// Exact retained native schema belonging to this admitted declaration.
    pub fn schema(&self) -> &arrow_schema::SchemaRef {
        &self.arena.schemas[&self.id]
    }

    /// Semantic relation identity; identity alone cannot construct a handle.
    pub const fn relation_id(&self) -> SemanticId {
        self.id
    }

    /// Borrow this handle's complete semantic declaration.
    pub fn resolved(&self) -> &ResolvedRelationContract {
        // Construction checks membership and the graph cannot change thereafter.
        &self.arena.graph.relations[&self.id]
    }

    /// Admit an independent owner exactly; local handles take the pointer fast path.
    /// # Errors
    /// Any reachable semantic declaration differs.
    pub fn require_equivalent(&self, other: &Self) -> Result<(), SchemaError> {
        if self.id != other.id {
            return Err(invalid("relation identities differ"));
        }
        if Arc::ptr_eq(&self.arena, &other.arena) {
            return Ok(());
        }
        compare(
            &self.arena.graph,
            &other.arena.graph,
            self.id,
            &BTreeSet::new(),
        )
        .map(|_| ())
    }

    /// Admit frozen generated expectations once for this actual owner.
    /// Successful subgraph proofs are reused across generated relation roots.
    /// # Errors
    /// A complete generated expectation differs, or proof synchronization failed.
    pub fn require_generated(
        &self,
        expected: &'static GeneratedContracts,
    ) -> Result<(), SchemaError> {
        let mut proofs = self
            .arena
            .proofs
            .lock()
            .map_err(|_| invalid("contract admission proof lock poisoned"))?;
        let existing = proofs
            .iter()
            .position(|proof| std::ptr::eq(proof.expected, expected));
        if existing.is_some_and(|index| proofs[index].relations.contains(&self.id)) {
            return Ok(());
        }
        #[cfg(test)]
        self.arena
            .comparisons
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let empty = BTreeSet::new();
        let known = existing.map_or(&empty, |index| &proofs[index].relations);
        let admitted = compare(&self.arena.graph, expected, self.id, known)?;
        if let Some(index) = existing {
            proofs[index].relations.extend(admitted);
        } else {
            proofs.push(GeneratedProof {
                expected,
                relations: admitted,
            });
        }
        Ok(())
    }
}

fn compare(
    actual: &GeneratedContracts,
    expected: &GeneratedContracts,
    root: SemanticId,
    known: &BTreeSet<SemanticId>,
) -> Result<BTreeSet<SemanticId>, SchemaError> {
    // IDs select candidates only. Every newly visited pair is compared in full.
    let mut pending = vec![root];
    let mut visited = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if known.contains(&id) || !visited.insert(id) {
            continue;
        }
        let left = actual
            .relations
            .get(&id)
            .ok_or_else(|| invalid("missing actual relation"))?;
        let right = expected
            .relations
            .get(&id)
            .ok_or_else(|| invalid("missing expected relation"))?;
        if left != right {
            return Err(invalid(format!(
                "resolved relation {} differs",
                left.declaration.key
            )));
        }
        for domain in &left.enums {
            if actual.enums.get(domain) != expected.enums.get(domain) {
                return Err(invalid("resolved enum domain differs"));
            }
        }
        for extension in &left.extensions {
            if actual.extensions.get(extension) != expected.extensions.get(extension) {
                return Err(invalid("resolved extension definition differs"));
            }
        }
        pending.extend(left.references.iter().copied());
    }
    Ok(visited)
}

pub(crate) fn resolve_relation(
    registry: &Registry,
    spec: &RelationSpec,
) -> Result<ResolvedRelationContract, SchemaError> {
    let mut declaration = spec.clone();
    declaration.doc = "";
    declaration.fingerprint = ContentHash::NIL;
    declaration.columns = spec
        .columns
        .iter()
        .map(|f| semantic_field(f.field()).map(FieldContract::from_field))
        .collect::<Result<_, _>>()?;
    let fields = spec
        .columns
        .iter()
        .map(|f| {
            crate::arrow::field_for(registry, f)
                .and_then(|f| semantic_field(&f))
                .map(FieldContract::from_field)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut references = BTreeSet::new();
    let mut enums = BTreeSet::new();
    let mut extensions = BTreeSet::new();
    let mut pending: Vec<_> = spec.columns.iter().chain(&fields).cloned().collect();
    while let Some(field) = pending.pop() {
        if let Some(reference) = crate::model::ReferenceContract::for_contract(&field)? {
            references.insert(target(registry, &reference.relation)?);
        }
        if let Some(use_) = field.extension() {
            extensions.insert(use_.spec().name.to_owned());
            match use_ {
                ExtensionUse::Enum(name) => {
                    enums.insert(
                        registry
                            .enum_spec(name)
                            .ok_or_else(|| invalid("missing enum definition"))?
                            .id,
                    );
                }
                ExtensionUse::OrdinalRef { target: name } => {
                    references.insert(target(registry, name)?);
                }
                _ => {}
            }
        }
        if let Some(id) = field.field().metadata().get(crate::arrow::KEY_ENUM) {
            enums.insert(
                registry
                    .enums()
                    .iter()
                    .find(|e| e.id.to_hex() == *id)
                    .ok_or_else(|| invalid("missing physical enum domain"))?
                    .id,
            );
        }
        if let Some(name) = field
            .field()
            .metadata()
            .get(crate::arrow::KEY_EXTENSION_NAME)
            && crate::model::EXTENSION_TYPES.iter().any(|e| e.name == name)
        {
            extensions.insert(name.clone());
        }
        pending.extend(field.children());
    }
    Ok(ResolvedRelationContract {
        declaration,
        fields,
        references: references.into_iter().collect(),
        enums: enums.into_iter().collect(),
        extensions: extensions.into_iter().collect(),
    })
}

fn target(registry: &Registry, name: &str) -> Result<SemanticId, SchemaError> {
    registry
        .relation(name)
        .map(|s| s.id)
        .ok_or_else(|| invalid(format!("missing relation target {name}")))
}

/// Only explicitly classified prose is omitted. Full original fields remain elsewhere.
fn semantic_field(field: &Field) -> Result<Field, SchemaError> {
    pse_columnar::native_field::project(
        field,
        pse_columnar::native_field::MetadataPurpose::ExecutionIdentity,
    )
    .map_err(|error| invalid(error.to_string()))
}

fn invalid(reason: impl Into<String>) -> SchemaError {
    SchemaError::InvalidDeclaration {
        context: "resolved contract admission".into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod consolidation_unit;

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    #[test]
    fn registry_schema_is_retained_and_foreign_meaning_is_not_inferred_from_id() {
        let registry = crate::catalog::assemble().unwrap();
        let spec = registry.relation("reference.dimensions").unwrap();
        let first = crate::arrow::relation_schema_ref(&registry, spec).unwrap();
        for _ in 0..8 {
            assert!(Arc::ptr_eq(
                &first,
                &crate::arrow::relation_schema_ref(&registry, spec).unwrap()
            ));
        }
        let mut changed = spec.clone();
        changed.checks.insert("hostile".into(), "false".into());
        assert!(crate::arrow::relation_schema_ref(&registry, &changed).is_err());
        let independent = crate::catalog::assemble().unwrap();
        let second = crate::arrow::relation_schema_ref(
            &independent,
            independent.relation_by_id(spec.id).unwrap(),
        )
        .unwrap();
        assert_eq!(first, second);
        assert!(!Arc::ptr_eq(&first, &second));
    }
}
