// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Assembly: declarations in, an immutable [`Registry`] out (blueprint §4.1).
//!
//! The builder is where "declared once" becomes enforceable. Every cross-reference — a
//! foreign key, an enumeration, a per-row quantity contract's sibling column, a derived
//! input port's producer — is resolved here, once, against the whole set of declarations,
//! so a catalog module can name something another module declares without either module
//! knowing the other exists.
//!
//! Identity comes from ADR-0050 and nowhere else:
//! `REGISTRY_PACKAGE_ID = named_id(SemanticId::NIL, "pse.schema")`, and within it
//! `relation:<ns>.<name>@<v>`, `enum:<Name>`, `logical_type:<rendered type>`,
//! `invariant:<relation>:<name>` and `pass:<P>@<v>`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use pse_ids::{ContentHash, SemanticId, named_id};

use crate::error::SchemaError;
use crate::ext_metadata;
use crate::model::{
    AlgorithmDecl, AlgorithmSpec, DocumentSpec, EnumDecl, EnumSpec, ExtensionUse, FieldContract,
    FieldTypeRow, InvariantDecl, InvariantSpec, MigrationSpec, RelationDecl, RelationKey,
    RelationSpec, render_data_type,
};

mod integrity;

mod native;

/// Native self-description tables; arrays are shared when a projection is cloned.
pub type SchemaBatches = Vec<(RelationKey, arrow_array::RecordBatch)>;

/// The registry package's qualified name (ADR-0050).
pub const REGISTRY_PACKAGE_NAME: &str = "pse.schema";

/// The registry's own package identity: `named_id(SemanticId::NIL, "pse.schema")`
/// (ADR-0050).
///
/// Every registry identity is derived under this one, which is what makes a relation ID a
/// function of the declaration rather than of the order the catalog modules ran in.
pub static REGISTRY_PACKAGE_ID: LazyLock<SemanticId> =
    LazyLock::new(|| named_id(SemanticId::NIL, REGISTRY_PACKAGE_NAME));

/// The identity of a registry item with the given qualified name (ADR-0050).
pub fn registry_id(qualified_name: &str) -> SemanticId {
    named_id(*REGISTRY_PACKAGE_ID, qualified_name)
}

/// The identity a column's declared quantity contract resolves to.
///
/// A quantity type is a *reference package* entity, not a registry declaration, so the
/// registry cannot read its identity out of a table it does not hold. It mints a contract
/// identity from the qualified name instead, under the same named policy (§5.1): the
/// column's contract is then part of the relation fingerprint, and a column that silently
/// changed which quantity type it carries changes its fingerprint rather than keeping it.
/// Packet A-1 declares `reference.quantity_types` and reconciles the two.
pub fn quantity_type_id(qualified_name: &str) -> SemanticId {
    registry_id(&format!("quantity_type:{qualified_name}"))
}

/// The `reference.schema_*` and `reference.pass_*` relations, in the
/// fixed order [`Registry::schema_batches`] emits them.
///
/// The names and version 1 are fixed by blueprint §4.1 and §6.11, so a table is emitted
/// whether or not the relation describing it is declared yet. That matters: the registry
/// fingerprint is the digest of these rows, and a registry that emitted nothing until it
/// declared its own catalog would give every early registry the same fingerprint — a
/// silent equality between two different schemas, which is the failure mode a fingerprint
/// exists to prevent. A declared relation supplies its own key, so a future version bump
/// is picked up rather than overridden.
const SELF_DESCRIBING_RELATIONS: [&str; 14] = [
    "reference.schema_relations",
    "reference.schema_columns",
    "reference.schema_logical_types",
    "reference.schema_enums",
    "reference.schema_enum_types",
    "reference.schema_invariants",
    "reference.schema_migrations",
    "reference.algorithm_specs",
    "reference.algorithm_arguments",
    "reference.algorithm_results",
    "reference.function_capabilities",
    "reference.schema_documents",
    "reference.schema_document_sections",
    "reference.artifact_profiles",
];

/// The assembled, immutable registry.
///
/// Every collection is sorted and every lookup is a `BTreeMap`: a `HashMap` reaching
/// output would make the registry fingerprint depend on a hash seed, and the failure would
/// look like a logic bug rather than a nondeterminism (blueprint §5.3 step 8).
#[derive(Debug)]
pub struct Registry {
    /// Owner-local derived implementations. This never stores declarations or admission proofs.
    implementations: crate::implementation_cache::ImplementationCache,
    /// `named_id(SemanticId::NIL, "pse.schema")`.
    package_id: SemanticId,
    /// The digest of [`Registry::schema_batches`].
    fingerprint: ContentHash,
    /// Sorted by `(namespace spelling, name, version)`.
    relations: Vec<RelationSpec>,
    /// Qualified name to the index of its highest declared version.
    relation_index: BTreeMap<String, usize>,
    /// Relation identity to index.
    relation_by_id: BTreeMap<SemanticId, usize>,
    /// Sorted by name.
    enums: Vec<EnumSpec>,
    /// Sorted by name.
    logical_types: Vec<FieldTypeRow>,
    /// Resolved native Arrow storage in the same name order; compiled once.
    logical_storage: Vec<arrow_schema::DataType>,
    /// Sorted by `<relation>:<name>`.
    invariants: Vec<InvariantSpec>,
    /// Sorted by `<relation>@<from>-><to>`.
    migrations: Vec<MigrationSpec>,
    /// Sorted by `<name>@<version>`.
    algorithms: Vec<AlgorithmSpec>,
    /// Sorted by name.
    documents: Vec<DocumentSpec>,
    /// Required member declarations by artifact kind, independent of execution order.
    artifact_profiles: BTreeMap<String, BTreeSet<SemanticId>>,
    /// Immutable projection of the completed declarations, materialized during assembly.
    self_description: SchemaBatches,
    /// Immutable resolved graph; installed after declaration assembly.
    contracts: Option<std::sync::Arc<crate::resolved_contract::ContractArena>>,
}

impl Registry {
    pub(crate) fn contracts_ready(&self) -> bool {
        self.contracts.is_some()
    }

    /// Memoize a derived implementation for this immutable registry only. Consumers
    /// use private key types; session-dependent implementations require their own owner.
    /// # Errors
    /// Preparation failure or a poisoned owner lock.
    pub fn derived_implementation<T: std::any::Any + Send + Sync>(
        &self,
        build: impl FnOnce() -> Result<T, SchemaError>,
    ) -> Result<std::sync::Arc<T>, SchemaError> {
        self.implementations.get(build)
    }
    /// Opaque lifetime identity for derived implementation contexts, not semantic equality.
    pub fn implementation_owner(&self) -> std::sync::Arc<()> {
        self.implementations.owner()
    }

    /// Complete required relation identities for a declared artifact kind. Empty
    /// profiles still require explicit member inventory and all member obligations.
    pub fn artifact_profile(&self, name: &str) -> Option<&BTreeSet<SemanticId>> {
        self.artifact_profiles.get(name)
    }
    /// The registry's package identity (ADR-0050).
    pub const fn package_id(&self) -> SemanticId {
        self.package_id
    }

    /// The registry fingerprint: the digest of every self-describing row
    /// (blueprint §5.3, ADR-0050).
    /// It identifies the admitted declarations; it does not establish the validity of
    /// any rows, artifact, cached pass result or external dependency using them.
    pub const fn fingerprint(&self) -> ContentHash {
        self.fingerprint
    }

    /// Every relation, sorted by `(namespace, name, version)`.
    pub fn relations(&self) -> &[RelationSpec] {
        &self.relations
    }

    /// The relation of that qualified name, for example `authored.entities`.
    ///
    /// When several versions are declared, the highest is returned: a foreign key and a
    /// port name a relation, and the registry owns which version is current.
    pub fn relation(&self, qualified_name: &str) -> Option<&RelationSpec> {
        self.relation_index
            .get(qualified_name)
            .and_then(|index| self.relations.get(*index))
    }

    /// The relation with that identity.
    pub fn relation_by_id(&self, id: SemanticId) -> Option<&RelationSpec> {
        self.relation_by_id
            .get(&id)
            .and_then(|index| self.relations.get(*index))
    }

    /// Resolve a local declaration assertion against this actual registry.
    /// Foreign registry transfers must carry a handle with their source definitions.
    /// # Errors
    /// Missing relation, unresolved graph or a changed semantic declaration.
    pub fn contract(
        &self,
        spec: &RelationSpec,
    ) -> Result<crate::resolved_contract::RelationContractHandle, SchemaError> {
        let actual = self
            .relation_by_id(spec.id)
            .ok_or_else(|| SchemaError::UnknownReference {
                context: "contract admission".into(),
                reference: spec.qualified_name(),
            })?;
        let handle = self.contract_arena()?.handle(actual.id)?;
        if !std::ptr::eq(actual, spec)
            && crate::resolved_contract::resolve_relation(self, spec)? != *handle.resolved()
        {
            return Err(SchemaError::InvalidDeclaration {
                context: spec.qualified_name(),
                reason: "candidate differs from the authoritative resolved contract".into(),
            });
        }
        Ok(handle)
    }

    /// Admit a contract from an independent owner and return this owner's handle.
    /// # Errors
    /// Any reachable semantic definition differs.
    pub fn admit_contract(
        &self,
        foreign: &crate::resolved_contract::RelationContractHandle,
    ) -> Result<crate::resolved_contract::RelationContractHandle, SchemaError> {
        let local = self.contract_arena()?.handle(foreign.relation_id())?;
        local.require_equivalent(foreign)?;
        Ok(local)
    }

    /// Complete resolved declarations for pure code generation.
    /// # Errors
    /// Registry assembly has not completed.
    pub fn generated_contracts(
        &self,
    ) -> Result<&crate::resolved_contract::GeneratedContracts, SchemaError> {
        Ok(&self.contract_arena()?.graph)
    }

    pub(crate) fn contract_arena(
        &self,
    ) -> Result<&std::sync::Arc<crate::resolved_contract::ContractArena>, SchemaError> {
        self.contracts
            .as_ref()
            .ok_or_else(|| SchemaError::InvalidDeclaration {
                context: "contract arena".into(),
                reason: "registry assembly has not completed".into(),
            })
    }

    /// The exact declared relation version, without selecting the current alias.
    pub fn relation_by_key(&self, key: RelationKey) -> Option<&RelationSpec> {
        self.relations
            .binary_search_by_key(&key, |relation| relation.key)
            .ok()
            .and_then(|index| self.relations.get(index))
    }

    /// Every declared enumeration, sorted by name.
    pub fn enums(&self) -> &[EnumSpec] {
        &self.enums
    }

    /// The enumeration of that name.
    pub fn enum_spec(&self, name: &str) -> Option<&EnumSpec> {
        self.enums
            .binary_search_by(|candidate| candidate.name.cmp(name))
            .ok()
            .and_then(|index| self.enums.get(index))
    }

    /// The logical-type catalog, sorted by name (blueprint §4.5).
    pub fn logical_types(&self) -> &[FieldTypeRow] {
        &self.logical_types
    }

    fn compile_logical_storage(&self) -> Result<Vec<arrow_schema::DataType>, SchemaError> {
        self.logical_types
            .iter()
            .map(|logical| {
                let storage: arrow_schema::DataType = serde_json::from_str(&logical.arrow_storage)
                    .map_err(|error| crate::checks::invalid(&logical.name, error.to_string()))?;
                if logical.extension_name.as_ref().is_some_and(|name| {
                    crate::model::EXTENSION_TYPES
                        .iter()
                        .any(|spec| spec.name == name)
                }) {
                    Ok(storage)
                } else {
                    crate::arrow::bind_type(self, &storage, &logical.name)
                }
            })
            .collect()
    }

    /// The resolved native Arrow storage for a registered logical type.
    /// JSON catalog rows are an interchange representation, not a runtime parser.
    pub fn logical_storage(&self, name: &str) -> Option<&arrow_schema::DataType> {
        self.logical_types
            .binary_search_by(|row| row.name.as_str().cmp(name))
            .ok()
            .and_then(|index| self.logical_storage.get(index))
    }

    /// The logical type of that registry name, for example `enum:Namespace`.
    pub fn logical_type(&self, name: &str) -> Option<&FieldTypeRow> {
        self.logical_types
            .binary_search_by(|candidate| candidate.name.as_str().cmp(name))
            .ok()
            .and_then(|index| self.logical_types.get(index))
    }

    /// Every declared invariant, sorted by `<relation>:<name>`.
    pub fn invariants(&self) -> &[InvariantSpec] {
        &self.invariants
    }

    /// The invariants of one qualified relation.
    pub fn invariants_for(&self, relation: &str) -> Vec<&InvariantSpec> {
        self.invariants
            .iter()
            .filter(|invariant| invariant.relation == relation)
            .collect()
    }

    /// Every declared migration.
    pub fn migrations(&self) -> &[MigrationSpec] {
        &self.migrations
    }

    /// Every declared pass, sorted by `<name>@<version>`.
    pub fn algorithms(&self) -> &[AlgorithmSpec] {
        &self.algorithms
    }

    /// The pass of that name (`P2`) or qualified name (`P2@1`).
    pub fn algorithm(&self, name: &str) -> Option<&AlgorithmSpec> {
        let mut candidates = self
            .algorithms
            .iter()
            .filter(|pass| pass.name == name || pass.qualified_name() == name);
        let candidate = candidates.next()?;
        candidates.next().is_none().then_some(candidate)
    }

    /// Every declared authoring document shape (blueprint §22.1).
    pub fn documents(&self) -> &[DocumentSpec] {
        &self.documents
    }

    /// Immutable native self-description, sorted by each declaration's primary key.
    /// Empty and custom registries use the same intrinsic declarations as the platform.
    pub fn schema_batches(&self) -> &[(RelationKey, arrow_array::RecordBatch)] {
        &self.self_description
    }

    /// The identity of the invariant named `<relation>:<name>`.
    pub fn invariant_id(&self, qualified_name: &str) -> Option<SemanticId> {
        self.invariants
            .iter()
            .find(|invariant| invariant.qualified_name() == qualified_name)
            .map(|invariant| invariant.id)
    }
}

/// The accumulator a catalog module declares into.
///
/// Declarations are collected, not checked, as they arrive: a catalog module may name a
/// relation a later module declares, so nothing can be resolved until
/// [`RegistryBuilder::build`] has them all.
#[derive(Debug, Default)]
pub struct RegistryBuilder {
    /// Relations, in declaration order.
    relations: Vec<RelationDecl>,
    /// Enumerations, in declaration order.
    enums: Vec<EnumDecl>,
    /// Invariants, in declaration order.
    invariants: Vec<InvariantDecl>,
    /// Passes, in declaration order.
    algorithms: Vec<AlgorithmDecl>,
    /// Migrations, in declaration order.
    migrations: Vec<MigrationSpec>,
    /// Authoring document shapes, in declaration order.
    documents: Vec<DocumentSpec>,
    artifact_profiles: Vec<(String, BTreeSet<String>)>,
}

impl RegistryBuilder {
    /// Declare artifact completeness once, referencing authoritative relation names.
    pub fn declare_artifact_profile(&mut self, name: &str, members: BTreeSet<String>) -> &mut Self {
        self.artifact_profiles.push((name.into(), members));
        self
    }
    /// Native signatures can supply profile requirements without restating outputs.
    pub fn declared_algorithms(&self) -> &[AlgorithmDecl] {
        &self.algorithms
    }
    /// Project declared field/relation integrity into the same executable rule catalog.
    /// Exact existing projections are retained; conflicting declarations remain errors.
    pub(crate) fn derive_integrity(&mut self) {
        integrity::declare(self);
    }
    /// Read the authoritative declarations while deriving additional catalog contracts.
    pub fn declared_relations(&self) -> &[RelationDecl] {
        &self.relations
    }

    /// Read declared invariants while deriving pass preconditions and postconditions.
    pub fn declared_invariants(&self) -> &[InvariantDecl] {
        &self.invariants
    }

    /// Read the sole document-to-relation inventory when declaring authoring pass ports.
    pub fn declared_documents(&self) -> &[DocumentSpec] {
        &self.documents
    }
    /// An empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Declares a relation.
    pub fn declare_relation(&mut self, decl: RelationDecl) -> &mut Self {
        self.relations.push(decl);
        self
    }

    /// Declares an enumeration.
    pub fn declare_enum(&mut self, decl: EnumDecl) -> &mut Self {
        self.enums.push(decl);
        self
    }

    /// Declares an invariant.
    pub fn declare_invariant(&mut self, decl: InvariantDecl) -> &mut Self {
        self.invariants.push(decl);
        self
    }

    /// Declares a pass.
    pub fn declare_algorithm(&mut self, decl: AlgorithmDecl) -> &mut Self {
        self.algorithms.push(decl);
        self
    }

    /// Declares a migration.
    pub fn declare_migration(&mut self, spec: MigrationSpec) -> &mut Self {
        self.migrations.push(spec);
        self
    }

    /// Declares an authoring document shape.
    pub fn declare_document(&mut self, spec: DocumentSpec) -> &mut Self {
        self.documents.push(spec);
        self
    }

    /// Resolves every declaration into an immutable [`Registry`].
    ///
    /// # Errors
    ///
    /// [`SchemaError::DuplicateDeclaration`] when two declarations claim one name;
    /// [`SchemaError::UnknownReference`] for a dangling foreign key, enumeration,
    /// per-row-quantity sibling, primary-key column, invariant relation;
    /// [`SchemaError::MissingGranularity`] for a derived relation without one;
    pub fn build(mut self) -> Result<Registry, SchemaError> {
        let mut registry = self.resolve_base()?;
        self.derive_integrity();

        check_relation_references(&registry)?;
        registry.invariants = resolve_invariants(self.invariants, &registry)?;
        registry
            .invariants
            .sort_by_key(InvariantSpec::qualified_name);
        registry.algorithms = resolve_algorithms(self.algorithms, &registry)?;
        registry
            .algorithms
            .sort_by_key(AlgorithmSpec::qualified_name);
        registry.migrations = resolve_migrations(self.migrations, &registry)?;
        registry.documents = resolve_documents(self.documents, &registry)?;
        for (name, members) in self.artifact_profiles {
            let members = members
                .into_iter()
                .map(|relation| {
                    registry
                        .relation(&relation)
                        .map(|spec| spec.id)
                        .ok_or_else(|| SchemaError::UnknownReference {
                            context: format!("artifact profile {name}"),
                            reference: relation,
                        })
                })
                .collect::<Result<_, _>>()?;
            if registry
                .artifact_profiles
                .insert(name.clone(), members)
                .is_some()
            {
                return Err(SchemaError::DuplicateDeclaration {
                    kind: "artifact profile",
                    name,
                });
            }
        }
        crate::checks::documents(&registry.documents, &registry)?;

        let fingerprints: Vec<ContentHash> = registry
            .relations
            .iter()
            .map(|spec| crate::fingerprint::relation(&registry, spec))
            .collect::<Result<_, _>>()?;
        for (spec, fingerprint) in registry.relations.iter_mut().zip(fingerprints) {
            spec.fingerprint = fingerprint;
        }
        registry.contracts = Some(crate::resolved_contract::ContractArena::resolve(&registry)?);
        registry.self_description = native::materialize(&registry)?;
        registry.fingerprint = crate::fingerprint::registry(registry.schema_batches())?;
        Ok(registry)
    }

    /// Resolve only declarations. Bootstrap consumers do not recursively build a registry.
    fn resolve_base(&self) -> Result<Registry, SchemaError> {
        let relations = self.resolve_relations()?;
        let relation_index = index_relations(&relations);
        let relation_by_id = relations
            .iter()
            .enumerate()
            .map(|(index, spec)| (spec.id, index))
            .collect();
        let enums = self.resolve_enums()?;
        let logical_types = collect_logical_types(&relations, &enums)?;

        let mut registry = Registry {
            implementations: crate::implementation_cache::ImplementationCache::default(),
            package_id: *REGISTRY_PACKAGE_ID,
            fingerprint: ContentHash::NIL,
            relations,
            relation_index,
            relation_by_id,
            enums,
            logical_types,
            logical_storage: Vec::new(),
            invariants: Vec::new(),
            migrations: Vec::new(),
            algorithms: Vec::new(),
            documents: Vec::new(),
            artifact_profiles: BTreeMap::new(),
            self_description: Vec::new(),
            contracts: None,
        };

        registry.logical_storage = registry.compile_logical_storage()?;

        Ok(registry)
    }

    /// Assigns identities to the relation declarations and sorts them.
    ///
    /// # Errors
    ///
    /// [`SchemaError::DuplicateDeclaration`] or [`SchemaError::MissingGranularity`].
    fn resolve_relations(&self) -> Result<Vec<RelationSpec>, SchemaError> {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut out = Vec::with_capacity(self.relations.len());
        for decl in &self.relations {
            let name = decl.key.to_string();
            if !seen.insert(name.clone()) {
                return Err(SchemaError::DuplicateDeclaration {
                    kind: "relation",
                    name,
                });
            }
            if decl.authority == crate::model::Authority::Derived
                && decl.derivation_granularity.is_none()
            {
                return Err(SchemaError::MissingGranularity { relation: name });
            }
            crate::checks::relation_declaration(decl)?;
            out.push(RelationSpec {
                id: registry_id(&format!("relation:{name}")),
                key: decl.key,
                authority: decl.authority,
                snapshot_class: decl.snapshot_class,
                derivation_granularity: decl.derivation_granularity,
                stability: decl.stability,
                primary_key: decl.primary_key.clone().ok_or_else(|| {
                    crate::checks::invalid(&name, "missing primary key declaration")
                })?,
                columns: decl.columns.clone(),
                checks: decl.checks.clone(),
                delta_properties: decl.delta_properties.clone(),
                doc: decl.doc,
                fingerprint: ContentHash::NIL,
            });
        }
        out.sort_by_key(|spec| spec.key);
        Ok(out)
    }

    /// Assigns identities to the enumeration declarations and sorts them.
    ///
    /// # Errors
    ///
    /// [`SchemaError::DuplicateDeclaration`].
    fn resolve_enums(&self) -> Result<Vec<EnumSpec>, SchemaError> {
        let mut seen: BTreeSet<&'static str> = BTreeSet::new();
        let mut out = Vec::with_capacity(self.enums.len());
        for decl in &self.enums {
            if !seen.insert(decl.name) {
                return Err(SchemaError::DuplicateDeclaration {
                    kind: "enum",
                    name: decl.name.to_owned(),
                });
            }
            let mut members = BTreeSet::new();
            for member in &decl.members {
                if !members.insert(member.name) {
                    return Err(SchemaError::DuplicateDeclaration {
                        kind: "enum member",
                        name: format!("{}.{}", decl.name, member.name),
                    });
                }
            }
            out.push(EnumSpec {
                id: registry_id(&format!("enum:{}", decl.name)),
                name: decl.name,
                idaes_source: decl.idaes_source,
                members: decl.members.clone(),
            });
        }
        out.sort_by_key(|spec| spec.name);
        Ok(out)
    }
}

/// Maps each qualified relation name to the index of its highest declared version.
fn index_relations(relations: &[RelationSpec]) -> BTreeMap<String, usize> {
    let mut index = BTreeMap::new();
    for (position, spec) in relations.iter().enumerate() {
        index
            .entry(spec.qualified_name())
            .and_modify(|current: &mut usize| {
                if relations[*current].key.version < spec.key.version {
                    *current = position;
                }
            })
            .or_insert(position);
    }
    index
}

/// Records `ty` and every type reachable from it in the catalog under construction.
fn record_logical_type(
    ty: &FieldContract,
    types: &mut BTreeMap<String, FieldContract>,
) -> Result<(), SchemaError> {
    let mut reachable = Vec::new();
    ty.walk(&mut reachable);
    for child in reachable {
        types.insert(child.type_name()?, child.value_type());
    }
    Ok(())
}

/// The logical-type catalog: the §4.5 scalars, the §4.4 extension types, every declared
/// enumeration and every type reachable from a declared column.
///
/// # Errors
///
/// The errors of [`render_data_type`] for a storage the registry cannot render.
fn collect_logical_types(
    relations: &[RelationSpec],
    enums: &[EnumSpec],
) -> Result<Vec<FieldTypeRow>, SchemaError> {
    let mut types: BTreeMap<String, FieldContract> = BTreeMap::new();
    for scalar in [
        FieldContract::native(arrow_schema::DataType::Float64),
        FieldContract::native(arrow_schema::DataType::Int64),
        FieldContract::native(arrow_schema::DataType::Int32),
        FieldContract::native(arrow_schema::DataType::UInt8),
        FieldContract::native(arrow_schema::DataType::UInt16),
        FieldContract::native(arrow_schema::DataType::UInt32),
        FieldContract::native(arrow_schema::DataType::UInt64),
        FieldContract::native(arrow_schema::DataType::Boolean),
        FieldContract::native(arrow_schema::DataType::Utf8),
        FieldContract::native(crate::model::extension::timestamp_storage()),
        FieldContract::source_support(),
    ] {
        record_logical_type(&scalar, &mut types)?;
    }
    for use_ in [
        ExtensionUse::SemanticId,
        ExtensionUse::ContentHash,
        ExtensionUse::DimensionVector,
        ExtensionUse::QuantityValue,
        ExtensionUse::Bound,
        ExtensionUse::IndexTuple,
        ExtensionUse::SourceSpan,
        ExtensionUse::ExprDsl,
        ExtensionUse::TargetPath,
    ] {
        record_logical_type(&FieldContract::extended(use_), &mut types)?;
    }
    for spec in enums {
        record_logical_type(&FieldContract::enumeration(spec.name), &mut types)?;
    }
    for relation in relations {
        for column in &relation.columns {
            record_logical_type(&column.value_type(), &mut types)?;
        }
    }

    let mut rows = Vec::with_capacity(types.len());
    for (name, ty) in types {
        let extension = ty.extension();
        rows.push(FieldTypeRow {
            id: registry_id(&format!("logical_type:{name}")),
            arrow_storage: render_data_type(&ty.data_type())?,
            extension_name: extension
                .map(|use_| use_.extension_name().to_owned())
                .or_else(|| {
                    ty.field()
                        .metadata()
                        .get(crate::arrow::KEY_EXTENSION_NAME)
                        .cloned()
                }),
            metadata_schema: extension.map(|use_| {
                let spec = use_.spec();
                let resolved = match use_ {
                    ExtensionUse::Enum(name) => enums
                        .iter()
                        .find(|enumeration| enumeration.name == name)
                        .map(|enumeration| enumeration.id),
                    ExtensionUse::OrdinalRef { target } => relations
                        .iter()
                        .filter(|relation| relation.qualified_name() == target)
                        .max_by_key(|relation| relation.key.version)
                        .map(|relation| relation.id),
                    _ => None,
                };
                resolved.map_or_else(
                    || ext_metadata::json_schema(spec.metadata, spec.metadata_version),
                    |id| {
                        ext_metadata::specialized_json_schema(
                            spec.metadata,
                            spec.metadata_version,
                            id,
                        )
                    },
                )
            }),
            name,
        });
    }
    Ok(rows)
}

/// Resolves every reference a column makes.
///
/// # Errors
///
/// [`SchemaError::UnknownReference`] for a dangling foreign key, enumeration, ordinal-ref
/// target, per-row-quantity sibling or primary-key column.
fn check_relation_references(registry: &Registry) -> Result<(), SchemaError> {
    for spec in &registry.relations {
        for name in &spec.primary_key {
            if spec.column(name).is_none() {
                return Err(SchemaError::UnknownReference {
                    context: format!("primary key of {}", spec.key),
                    reference: (*name).to_owned(),
                });
            }
        }
        for column in &spec.columns {
            check_column(registry, spec, column)?;
        }
    }
    Ok(())
}

/// Resolves one column's references.
///
/// # Errors
///
/// [`SchemaError::UnknownReference`].
fn check_column(
    registry: &Registry,
    spec: &RelationSpec,
    column: &FieldContract,
) -> Result<(), SchemaError> {
    for name in column.enum_domains() {
        if registry.enum_spec(&name).is_none() {
            return Err(SchemaError::UnknownReference {
                context: format!("column {}.{}", spec.key, column.name()),
                reference: format!("enum:{name}"),
            });
        }
    }
    check_field(
        registry,
        column,
        &format!("column {}.{}", spec.key, column.name()),
    )?;
    let field = crate::arrow::field_for(registry, column)?;
    crate::field_contract::declaration(&arrow_schema::Schema::new(vec![field]))
}

fn check_alternative(
    registry: &Registry,
    column: &FieldContract,
    context: &str,
) -> Result<(), SchemaError> {
    if let Some(alternative) = crate::model::TaggedAlternative::from_field(column.field())? {
        let children = column.children();
        if let Some(tag) = children
            .iter()
            .find(|child| child.name() == alternative.discriminator)
            && let Some(name) = tag.enum_name()
        {
            let declared = registry.enum_spec(name).ok_or_else(|| {
                crate::checks::invalid(context, "unknown alternative tag enumeration")
            })?;
            if declared.members.len() != alternative.arms.len()
                || declared
                    .members
                    .iter()
                    .any(|member| !alternative.arms.contains_key(member.name))
            {
                return Err(crate::checks::invalid(
                    context,
                    "alternative arms must cover the discriminator enumeration exactly",
                ));
            }
        }
    }
    Ok(())
}

fn check_field(
    registry: &Registry,
    column: &FieldContract,
    context: &str,
) -> Result<(), SchemaError> {
    column.validate_facets(context)?;
    crate::model::IntegerRange::from_field(column.field())?;
    crate::model::CollectionContract::from_field(column.field())?;
    check_alternative(registry, column, context)?;
    let context = context.to_owned();
    if let Some(reference) = crate::model::ReferenceContract::for_contract(column)? {
        let target = registry.relation(&reference.relation).ok_or_else(|| {
            SchemaError::UnknownReference {
                context: context.clone(),
                reference: reference.relation.clone(),
            }
        })?;
        for mapping in &reference.columns {
            let source = crate::model::reference::source_field(column.field(), &mapping.source)?;
            let target_column =
                target
                    .column(&mapping.target)
                    .ok_or_else(|| SchemaError::UnknownReference {
                        context: context.clone(),
                        reference: format!("{}.{}", reference.relation, mapping.target),
                    })?;
            if FieldContract::from_field(source.clone()).value_type() != target_column.value_type()
            {
                return Err(crate::checks::invalid(
                    &context,
                    format!(
                        "foreign key {}.{} has incompatible component types",
                        reference.relation, mapping.target
                    ),
                ));
            }
        }
    }
    match column.extension() {
        Some(ExtensionUse::Enum(name)) if registry.enum_spec(name).is_none() => {
            return Err(SchemaError::UnknownReference {
                context: context.clone(),
                reference: format!("enum:{name}"),
            });
        }
        Some(ExtensionUse::OrdinalRef { target }) if registry.relation(target).is_none() => {
            return Err(SchemaError::UnknownReference {
                context: context.clone(),
                reference: target.to_owned(),
            });
        }
        _ => {}
    }
    if column.extension().is_none() {
        let children = column.children();
        for child in &children {
            check_field(registry, child, &format!("{context}.{}", child.name()))?;
        }
    }
    Ok(())
}

/// Assigns identities to the invariant declarations and checks their references.
///
/// # Errors
///
/// [`SchemaError::DuplicateDeclaration`] or [`SchemaError::UnknownReference`].
fn resolve_invariants(
    decls: Vec<InvariantDecl>,
    registry: &Registry,
) -> Result<Vec<InvariantSpec>, SchemaError> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = Vec::with_capacity(decls.len());
    for decl in decls {
        let name = format!("{}:{}", decl.relation, decl.name);
        if !seen.insert(name.clone()) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "invariant",
                name,
            });
        }
        if registry.relation(&decl.relation).is_none() {
            return Err(SchemaError::UnknownReference {
                context: format!("invariant {name}"),
                reference: decl.relation.clone(),
            });
        }
        if decl.query.trim().is_empty() || !decl.inputs.contains(&decl.relation) {
            return Err(crate::checks::invalid(
                format!("invariant {name}"),
                "native query must bind its constrained relation",
            ));
        }
        for input in &decl.inputs {
            if registry.relation(input).is_none() {
                return Err(SchemaError::UnknownReference {
                    context: format!("invariant {name}"),
                    reference: input.clone(),
                });
            }
        }
        let target = registry
            .relation(&decl.relation)
            .ok_or_else(|| crate::checks::invalid(format!("invariant {name}"), "unknown target"))?;
        if decl
            .key_columns
            .iter()
            .any(|key| target.column(key).is_none())
        {
            return Err(crate::checks::invalid(
                format!("invariant {name}"),
                "unknown offending key",
            ));
        }
        out.push(InvariantSpec {
            id: registry_id(&format!("invariant:{name}")),
            name: decl.name,
            relation: decl.relation,
            kind: decl.kind,
            query: decl.query,
            inputs: decl.inputs,
            key_columns: decl.key_columns,
            severity: decl.severity,
            doc: decl.doc,
        });
    }
    Ok(out)
}

/// Assign identities and validate native argument, result and obligation declarations.
///
/// # Errors
///
/// [`SchemaError::DuplicateDeclaration`], [`SchemaError::UnknownReference`] or
/// Invalid declaration or unresolved relation contract.
fn resolve_algorithms(
    decls: Vec<AlgorithmDecl>,
    registry: &Registry,
) -> Result<Vec<AlgorithmSpec>, SchemaError> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = Vec::with_capacity(decls.len());
    for decl in decls {
        let name = format!("{}@{}", decl.name, decl.version);
        if !seen.insert(name.clone()) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "algorithm",
                name,
            });
        }
        let mut ports: BTreeSet<&str> = BTreeSet::new();
        for output in &decl.outputs {
            if !ports.insert(output.port.as_str()) {
                return Err(SchemaError::DuplicateDeclaration {
                    kind: "output port",
                    name: format!("{name}.{}", output.port),
                });
            }
            registry
                .relation(&output.relation)
                .ok_or_else(|| SchemaError::UnknownReference {
                    context: format!("result {name}.{}", output.port),
                    reference: output.relation.clone(),
                })?;
        }
        let mut input_ports = BTreeSet::new();
        for input in &decl.inputs {
            if !input_ports.insert(input.port.as_str()) {
                return Err(SchemaError::DuplicateDeclaration {
                    kind: "input port",
                    name: format!("{name}.{}", input.port),
                });
            }
            if registry.relation(&input.relation).is_none() {
                return Err(SchemaError::UnknownReference {
                    context: format!("input port {name}.{}", input.port),
                    reference: input.relation.clone(),
                });
            }
            if let crate::model::algorithm::InputConsumption::Columns(columns) = &input.consumption
            {
                let relation = registry
                    .relation(&input.relation)
                    .ok_or_else(|| crate::checks::invalid(&name, "argument relation absent"))?;
                if columns
                    .iter()
                    .any(|name| !relation.columns.iter().any(|field| field.name() == name))
                    || relation
                        .primary_key
                        .iter()
                        .any(|name| !columns.contains(*name))
                {
                    return Err(crate::checks::invalid(
                        &name,
                        "argument projection must name existing root fields and include every primary key",
                    ));
                }
            }
        }
        for conditions in [&decl.preconditions, &decl.postconditions] {
            let mut seen_conditions = BTreeSet::new();
            for condition in conditions {
                if !seen_conditions.insert(condition) {
                    return Err(SchemaError::DuplicateDeclaration {
                        kind: "pass condition",
                        name: format!("{name}:{condition}"),
                    });
                }
                if registry.invariant_id(condition).is_none() {
                    return Err(SchemaError::UnknownReference {
                        context: format!("condition of {name}"),
                        reference: (*condition).clone(),
                    });
                }
            }
        }
        check_algorithm_diagnostics(&decl, registry)?;
        out.push(AlgorithmSpec {
            id: registry_id(&format!("algorithm:{name}")),
            name: decl.name,
            version: decl.version,
            inputs: decl.inputs,
            outputs: decl.outputs,
            preconditions: decl.preconditions,
            postconditions: decl.postconditions,
            determinism: decl.determinism,
            diagnostics: decl.diagnostics,
            effects: decl.effects,
        });
    }
    Ok(out)
}

fn check_algorithm_diagnostics(
    decl: &AlgorithmDecl,
    registry: &Registry,
) -> Result<(), SchemaError> {
    let mut seen = BTreeSet::new();
    for diagnostic in &decl.diagnostics {
        if !seen.insert(diagnostic) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "pass diagnostic",
                name: format!("{}@{}:{diagnostic}", decl.name, decl.version),
            });
        }
        if registry
            .enum_spec("FailureClass")
            .is_none_or(|enumeration| {
                !enumeration
                    .members
                    .iter()
                    .any(|member| member.name == *diagnostic)
            })
        {
            return Err(SchemaError::UnknownReference {
                context: format!("diagnostic of {}@{}", decl.name, decl.version),
                reference: format!("FailureClass.{diagnostic}"),
            });
        }
    }
    Ok(())
}

/// Checks every migration's relation.
///
/// # Errors
///
/// [`SchemaError::DuplicateDeclaration`] or [`SchemaError::UnknownReference`].
fn resolve_migrations(
    specs: Vec<MigrationSpec>,
    registry: &Registry,
) -> Result<Vec<MigrationSpec>, SchemaError> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = Vec::with_capacity(specs.len());
    for spec in specs {
        let name = spec.qualified_name();
        if !seen.insert(name.clone()) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "migration",
                name,
            });
        }
        if registry.relation(spec.relation).is_none() {
            return Err(SchemaError::UnknownReference {
                context: format!("migration {name}"),
                reference: spec.relation.to_owned(),
            });
        }
        crate::checks::migration(&spec, registry)?;
        out.push(spec);
    }
    out.sort_by_key(MigrationSpec::qualified_name);
    Ok(out)
}

/// Checks every document section's relation.
///
/// # Errors
///
/// [`SchemaError::DuplicateDeclaration`] or [`SchemaError::UnknownReference`].
fn resolve_documents(
    specs: Vec<DocumentSpec>,
    registry: &Registry,
) -> Result<Vec<DocumentSpec>, SchemaError> {
    let mut seen: BTreeSet<&'static str> = BTreeSet::new();
    let mut out = Vec::with_capacity(specs.len());
    for spec in specs {
        if !seen.insert(spec.name) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "document",
                name: spec.name.to_owned(),
            });
        }
        let mut sections = BTreeSet::new();
        for section in &spec.sections {
            if !sections.insert(section.key) {
                return Err(SchemaError::DuplicateDeclaration {
                    kind: "document section",
                    name: format!("{}.{}", spec.name, section.key),
                });
            }
            if registry.relation(section.relation).is_none() {
                return Err(SchemaError::UnknownReference {
                    context: format!("document {}.{}", spec.name, section.key),
                    reference: section.relation.to_owned(),
                });
            }
        }
        out.push(spec);
    }
    out.sort_by_key(|spec| spec.name);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Authority, DerivationGranularity, EnumMember, Namespace, SnapshotClass};

    /// A one-column relation with a semantic-ID primary key.
    fn simple(name: &'static str) -> RelationDecl {
        RelationDecl::new(
            Namespace::Authored,
            name,
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "a fixture relation",
        )
        .pk(&["id"])
        .columns(vec![FieldContract::key(
            "id",
            FieldContract::id(),
            "the identity",
        )])
    }

    #[test]
    fn argument_projection_refuses_unknown_fields_and_missing_keys() {
        use crate::model::algorithm::InputConsumption;
        let build = |columns: &[&str]| {
            let mut builder = RegistryBuilder::new();
            builder.declare_relation(simple("argument"));
            builder.declare_algorithm(
                AlgorithmDecl::new("projection", "1", crate::model::Determinism::Deterministic)
                    .inputs(vec![crate::model::ArgumentSpec {
                        port: "values".into(),
                        relation: "authored.argument".into(),
                        required: true,
                        consumption: InputConsumption::Columns(
                            columns.iter().map(|name| (*name).into()).collect(),
                        ),
                    }]),
            );
            builder.build()
        };
        assert!(build(&["id"]).is_ok());
        assert!(build(&[]).is_err());
        assert!(build(&["id", "unknown"]).is_err());
    }

    #[test]
    fn native_table_policy_changes_identity_and_survives_cold_metadata() {
        let build = |value: &str| {
            let mut builder = RegistryBuilder::new();
            builder.declare_relation(
                simple("policy")
                    .delta_properties([("delta.logRetentionDuration".into(), value.into())]),
            );
            builder.build().unwrap()
        };
        let before = build("interval 30 days");
        let after = build("interval 60 days");
        let spec = before.relation("authored.policy").unwrap();
        assert_ne!(
            spec.fingerprint,
            after.relation("authored.policy").unwrap().fingerprint
        );
        let schema = crate::arrow::relation_schema(&before, spec).unwrap();
        assert_eq!(
            crate::arrow::delta_properties(&schema).unwrap(),
            spec.delta_properties
        );
        let mut metadata = schema.metadata().clone();
        metadata.insert(crate::arrow::KEY_DELTA_PROPERTIES.into(),
            "{\"delta.logRetentionDuration\":\"30 days\",\"delta.logRetentionDuration\":\"60 days\"}".into());
        assert!(crate::arrow::delta_properties(&schema.with_metadata(metadata)).is_err());
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(
            simple("bad").delta_properties([("delta.constraints.shadow".into(), "true".into())]),
        );
        assert!(builder.build().is_err());
    }

    #[test]
    fn the_package_id_is_the_frozen_vector() {
        assert_eq!(
            REGISTRY_PACKAGE_ID.to_hex(),
            "a409aa6be295e9f374cf4b829f193f49"
        );
        assert_eq!(
            registry_id("relation:authored.stoichiometry@1").to_hex(),
            "4af05b3e65e854a58198776b3cddaa8d"
        );
    }

    #[test]
    fn a_duplicate_relation_is_rejected() {
        let mut builder = RegistryBuilder::new();
        builder
            .declare_relation(simple("a"))
            .declare_relation(simple("a"));
        assert!(matches!(
            builder.build().unwrap_err(),
            SchemaError::DuplicateDeclaration {
                kind: "relation",
                ..
            }
        ));
    }

    #[test]
    fn artifact_profiles_are_declared_reflected_and_identity_bearing() {
        let build = |required: BTreeSet<String>| {
            let mut builder = RegistryBuilder::new();
            builder.declare_relation(simple("a"));
            builder.declare_artifact_profile("test", required);
            builder.build()
        };
        let full = build(["authored.a".into()].into_iter().collect()).unwrap();
        let empty = build(BTreeSet::new()).unwrap();
        assert_ne!(full.fingerprint(), empty.fingerprint());
        assert_eq!(
            full.artifact_profile("test").unwrap(),
            &[full.relation("authored.a").unwrap().id]
                .into_iter()
                .collect()
        );
        assert!(
            full.schema_batches()
                .iter()
                .any(|(key, rows)| key.name == "artifact_profiles" && rows.num_rows() == 1)
        );
        assert!(build(["authored.absent".into()].into_iter().collect()).is_err());
        let mut duplicate = RegistryBuilder::new();
        duplicate.declare_artifact_profile("test", BTreeSet::new());
        duplicate.declare_artifact_profile("test", BTreeSet::new());
        assert!(duplicate.build().is_err());
    }

    #[test]
    fn a_duplicate_enum_is_rejected() {
        let mut builder = RegistryBuilder::new();
        let decl = || EnumDecl::platform("X", vec![EnumMember::new("a", "a member")]);
        builder.declare_enum(decl()).declare_enum(decl());
        assert!(matches!(
            builder.build().unwrap_err(),
            SchemaError::DuplicateDeclaration { kind: "enum", .. }
        ));
    }

    #[test]
    fn a_dangling_foreign_key_is_rejected() {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(simple("a").columns(vec![
            FieldContract::key("id", FieldContract::id(), "the identity"),
            FieldContract::reference("other_id", FieldContract::id(), "a reference")
                .with_fk("authored.nowhere", "id"),
        ]));
        assert!(matches!(
            builder.build().unwrap_err(),
            SchemaError::UnknownReference { .. }
        ));
    }

    #[test]
    fn a_foreign_key_to_a_missing_column_is_rejected() {
        let mut builder = RegistryBuilder::new();
        builder
            .declare_relation(simple("a"))
            .declare_relation(simple("b").columns(vec![
                FieldContract::key("id", FieldContract::id(), "the identity"),
                FieldContract::reference("a_id", FieldContract::id(), "a reference")
                    .with_fk("authored.a", "not_a_column"),
            ]));
        assert!(matches!(
            builder.build().unwrap_err(),
            SchemaError::UnknownReference { .. }
        ));
    }

    #[test]
    fn an_undeclared_enum_is_rejected() {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(simple("a").columns(vec![
            FieldContract::key("id", FieldContract::id(), "the identity"),
            FieldContract::label("kind", FieldContract::enumeration("Nowhere"), "a label"),
        ]));
        assert!(matches!(
            builder.build().unwrap_err(),
            SchemaError::UnknownReference { .. }
        ));
    }

    #[test]
    fn a_derived_relation_without_granularity_is_rejected() {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(RelationDecl::new(
            Namespace::Inferred,
            "a",
            1,
            Authority::Derived,
            SnapshotClass::Derived,
            "a derived fixture",
        ));
        assert!(matches!(
            builder.build().unwrap_err(),
            SchemaError::MissingGranularity { .. }
        ));
    }

    #[test]
    fn a_derived_relation_with_granularity_is_accepted() {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Inferred,
                "a",
                1,
                Authority::Derived,
                SnapshotClass::Derived,
                "a derived fixture",
            )
            .granularity(DerivationGranularity::Row)
            .pk(&["id"])
            .columns(vec![FieldContract::key(
                "id",
                FieldContract::id(),
                "identity",
            )]),
        );
        assert!(builder.build().is_ok());
    }

    #[test]
    fn a_primary_key_naming_no_column_is_rejected() {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(simple("a").pk(&["nope"]));
        assert!(matches!(
            builder.build().unwrap_err(),
            SchemaError::UnknownReference { .. }
        ));
    }

    #[test]
    fn the_logical_type_catalog_always_carries_the_declared_scalars() {
        let registry = RegistryBuilder::new().build().unwrap();
        for name in [
            "f64", "i64", "i32", "u8", "u16", "u32", "u64", "bool", "text", "ts",
        ] {
            assert!(
                registry.logical_type(name).is_some(),
                "the §4.5 catalog is declared, not inferred from use: {name} is missing"
            );
        }
        assert_eq!(
            registry
                .logical_type("semantic_id")
                .map(|row| row.arrow_storage.as_str()),
            Some(r#"{"FixedSizeBinary":16}"#)
        );
    }
}
