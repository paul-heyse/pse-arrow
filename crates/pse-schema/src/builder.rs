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
//! `invariant:<relation>:<name>`, `pass:<P>@<v>` and `rule:<name>@<v>`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use pse_ids::{ContentHash, SemanticId, named_id};

use crate::error::SchemaError;
use crate::ext_metadata;
use crate::model::{
    Cell, ColumnSpec, DocumentSpec, EnumDecl, EnumSpec, ExtensionUse, InvariantDecl, InvariantSpec,
    LogicalType, LogicalTypeRow, ManifestSpec, MigrationSpec, NON_DERIVABLE_NAMESPACES, Namespace,
    PassDecl, PassSpec, PortSource, QuantityContract, RelationDecl, RelationKey, RelationSpec,
    RuleDecl, RuleDependency, RulePlan, RuleSpec, render_data_type,
};

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

/// The `reference.schema_*` and `reference.pass_*`/`reference.rule_*` relations, in the
/// fixed order [`Registry::schema_rows`] emits them.
///
/// The names and version 1 are fixed by blueprint §4.1 and §6.11, so a table is emitted
/// whether or not the relation describing it is declared yet. That matters: the registry
/// fingerprint is the digest of these rows, and a registry that emitted nothing until it
/// declared its own catalog would give every early registry the same fingerprint — a
/// silent equality between two different schemas, which is the failure mode a fingerprint
/// exists to prevent. A declared relation supplies its own key, so a future version bump
/// is picked up rather than overridden.
const SELF_DESCRIBING_RELATIONS: [&str; 16] = [
    "reference.schema_relations",
    "reference.schema_columns",
    "reference.schema_logical_types",
    "reference.schema_enums",
    "reference.schema_invariants",
    "reference.schema_migrations",
    "reference.pass_specs",
    "reference.pass_input_ports",
    "reference.pass_output_ports",
    "reference.rule_specs",
    "reference.rule_plan_nodes",
    "reference.rule_aggregates",
    "reference.rule_group_keys",
    "reference.rule_unnest",
    "reference.rule_plan_edges",
    "reference.rule_dependencies",
];

/// The assembled, immutable registry.
///
/// Every collection is sorted and every lookup is a `BTreeMap`: a `HashMap` reaching
/// output would make the registry fingerprint depend on a hash seed, and the failure would
/// look like a logic bug rather than a nondeterminism (blueprint §5.3 step 8).
#[derive(Debug)]
pub struct Registry {
    /// `named_id(SemanticId::NIL, "pse.schema")`.
    package_id: SemanticId,
    /// The digest of [`Registry::schema_rows`].
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
    logical_types: Vec<LogicalTypeRow>,
    /// Sorted by `<relation>:<name>`.
    invariants: Vec<InvariantSpec>,
    /// Sorted by `<relation>@<from>-><to>`.
    migrations: Vec<MigrationSpec>,
    /// Sorted by `<name>@<version>`.
    passes: Vec<PassSpec>,
    /// Sorted by `<name>@<version>`.
    rules: Vec<RuleSpec>,
    /// Derived from the rule plans at assembly, sorted.
    rule_dependencies: Vec<RuleDependency>,
    /// Sorted by name.
    documents: Vec<DocumentSpec>,
    /// The declared manifest envelope, when a catalog module declared one.
    manifest: Option<ManifestSpec>,
}

impl Registry {
    /// The registry's package identity (ADR-0050).
    pub const fn package_id(&self) -> SemanticId {
        self.package_id
    }

    /// The registry fingerprint: the digest of every self-describing row
    /// (blueprint §5.3, ADR-0050).
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
    pub fn logical_types(&self) -> &[LogicalTypeRow] {
        &self.logical_types
    }

    /// The logical type of that registry name, for example `enum:Namespace`.
    pub fn logical_type(&self, name: &str) -> Option<&LogicalTypeRow> {
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
    pub fn passes(&self) -> &[PassSpec] {
        &self.passes
    }

    /// The pass of that name (`P2`) or qualified name (`P2@1`).
    pub fn pass(&self, name: &str) -> Option<&PassSpec> {
        self.passes
            .iter()
            .find(|pass| pass.name == name || pass.qualified_name() == name)
    }

    /// Every declared rule, sorted by `<name>@<version>`.
    pub fn rules(&self) -> &[RuleSpec] {
        &self.rules
    }

    /// The rule of that name or qualified name.
    pub fn rule(&self, name: &str) -> Option<&RuleSpec> {
        self.rules
            .iter()
            .find(|rule| rule.name == name || rule.qualified_name() == name)
    }

    /// The `reference.rule_dependencies` facts derived from the rule plans
    /// (blueprint §6.11).
    pub fn rule_dependencies(&self) -> &[RuleDependency] {
        &self.rule_dependencies
    }

    /// Every declared authoring document shape (blueprint §22.1).
    pub fn documents(&self) -> &[DocumentSpec] {
        &self.documents
    }

    /// The declared `pse.manifest.v2` envelope (blueprint §20.2).
    pub const fn manifest(&self) -> Option<&ManifestSpec> {
        self.manifest.as_ref()
    }

    /// The registry as its own rows: `reference.schema_*`, `reference.pass_*` and
    /// `reference.rule_*`, each primary-key sorted with its cells in column order
    /// (blueprint §4.1).
    ///
    /// This is the preimage of [`Registry::fingerprint`] and the input
    /// `pse-relations` materializes into `RecordBatch`es, so the two cannot disagree
    /// about what the registry says.
    pub fn schema_rows(&self) -> Vec<(RelationKey, Vec<Vec<Cell>>)> {
        let mut out = Vec::new();
        for qualified in SELF_DESCRIBING_RELATIONS {
            let spec = self.relation(qualified);
            let rows = match qualified {
                "reference.schema_relations" => self.schema_relations_rows(),
                "reference.schema_columns" => self.schema_columns_rows(),
                "reference.schema_logical_types" => self.schema_logical_types_rows(),
                "reference.schema_enums" => self.schema_enums_rows(),
                "reference.schema_invariants" => self.schema_invariants_rows(),
                "reference.schema_migrations" => self.schema_migrations_rows(),
                "reference.pass_specs" => self.pass_specs_rows(),
                "reference.pass_input_ports" => self.pass_input_ports_rows(),
                "reference.pass_output_ports" => self.pass_output_ports_rows(),
                "reference.rule_specs" => self.rule_specs_rows(),
                "reference.rule_plan_nodes" => self.rule_plan_nodes_rows(),
                "reference.rule_aggregates" => self.rule_aggregates_rows(),
                "reference.rule_group_keys" => self.rule_group_keys_rows(),
                "reference.rule_unnest" => self.rule_unnest_rows(),
                "reference.rule_plan_edges" => self.rule_plan_edges_rows(),
                _ => self.rule_dependencies_rows(),
            };
            let key = spec.map_or_else(
                || RelationKey::new(Namespace::Reference, bare_name(qualified), 1),
                |spec| spec.key,
            );
            out.push((key, sort_by_primary_key(spec, rows)));
        }
        out
    }

    /// One `reference.schema_relations` row.
    pub(crate) fn schema_relations_row(spec: &RelationSpec) -> Vec<Cell> {
        vec![
            Cell::Id(spec.id),
            Cell::Enum(spec.key.namespace.as_str()),
            Cell::text(spec.key.name),
            Cell::U64(u64::from(spec.key.version)),
            Cell::Enum(spec.authority.as_str()),
            Cell::Enum(spec.snapshot_class.as_str()),
            Cell::List(
                spec.primary_key
                    .iter()
                    .map(|name| Cell::text(*name))
                    .collect(),
            ),
            Cell::opt_enum(
                spec.derivation_granularity
                    .map(crate::model::DerivationGranularity::as_str),
            ),
            Cell::Enum(spec.stability.as_str()),
            Cell::text(spec.doc),
        ]
    }

    /// Every `reference.schema_relations` row.
    fn schema_relations_rows(&self) -> Vec<Vec<Cell>> {
        self.relations
            .iter()
            .map(Self::schema_relations_row)
            .collect()
    }

    /// The `reference.schema_columns` rows of one relation.
    pub(crate) fn schema_columns_rows_of(&self, spec: &RelationSpec) -> Vec<Vec<Cell>> {
        spec.columns
            .iter()
            .enumerate()
            .map(|(ordinal, column)| {
                let logical_type_id = self
                    .logical_type(&column.logical_type.name())
                    .map(|row| row.id);
                vec![
                    Cell::Id(spec.id),
                    Cell::U64(count(ordinal)),
                    Cell::text(column.name),
                    Cell::opt_id(logical_type_id),
                    Cell::Bool(column.nullable),
                    Cell::opt_id(match column.quantity {
                        QuantityContract::Column(name) => Some(quantity_type_id(name)),
                        QuantityContract::None | QuantityContract::PerRow => None,
                    }),
                    Cell::Bool(matches!(column.quantity, QuantityContract::PerRow)),
                    Cell::opt_id(
                        column
                            .fk
                            .and_then(|fk| self.relation(fk.relation).map(|target| target.id)),
                    ),
                    Cell::opt_text(column.fk.map(|fk| fk.column)),
                    Cell::Enum(column.role.as_str()),
                    Cell::text(column.doc),
                ]
            })
            .collect()
    }

    /// Every `reference.schema_columns` row.
    fn schema_columns_rows(&self) -> Vec<Vec<Cell>> {
        self.relations
            .iter()
            .flat_map(|spec| self.schema_columns_rows_of(spec))
            .collect()
    }

    /// One `reference.schema_logical_types` row.
    pub(crate) fn schema_logical_types_row(row: &LogicalTypeRow) -> Vec<Cell> {
        vec![
            Cell::Id(row.id),
            Cell::text(row.name.clone()),
            Cell::text(row.arrow_storage.clone()),
            Cell::opt_text(row.extension_name),
            Cell::opt_text(row.metadata_schema.as_deref()),
        ]
    }

    /// Every `reference.schema_logical_types` row.
    fn schema_logical_types_rows(&self) -> Vec<Vec<Cell>> {
        self.logical_types
            .iter()
            .map(Self::schema_logical_types_row)
            .collect()
    }

    /// The `reference.schema_enums` rows of one enumeration.
    pub(crate) fn schema_enums_rows_of(spec: &EnumSpec) -> Vec<Vec<Cell>> {
        spec.members
            .iter()
            .enumerate()
            .map(|(ordinal, member)| {
                vec![
                    Cell::Id(spec.id),
                    Cell::U64(count(ordinal)),
                    Cell::text(member.name),
                    Cell::opt_text(member.idaes_name),
                    Cell::Bool(member.deprecated),
                    Cell::text(member.doc),
                ]
            })
            .collect()
    }

    /// Every `reference.schema_enums` row.
    fn schema_enums_rows(&self) -> Vec<Vec<Cell>> {
        self.enums
            .iter()
            .flat_map(Self::schema_enums_rows_of)
            .collect()
    }

    /// Every `reference.schema_invariants` row.
    fn schema_invariants_rows(&self) -> Vec<Vec<Cell>> {
        self.invariants
            .iter()
            .map(|invariant| {
                vec![
                    Cell::Id(invariant.id),
                    Cell::opt_id(self.relation(invariant.relation).map(|spec| spec.id)),
                    Cell::Enum(invariant.kind.as_str()),
                    Cell::opt_id(self.rule(invariant.rule).map(|rule| rule.id)),
                    Cell::Enum(invariant.severity.as_str()),
                    Cell::text(invariant.doc),
                ]
            })
            .collect()
    }

    /// Every `reference.schema_migrations` row.
    fn schema_migrations_rows(&self) -> Vec<Vec<Cell>> {
        self.migrations
            .iter()
            .map(|migration| {
                vec![
                    Cell::opt_id(self.relation(migration.relation).map(|spec| spec.id)),
                    Cell::U64(u64::from(migration.from_version)),
                    Cell::U64(u64::from(migration.to_version)),
                    Cell::text(migration.plan_spec()),
                    Cell::text(migration.doc),
                ]
            })
            .collect()
    }

    /// Every `reference.pass_specs` row.
    fn pass_specs_rows(&self) -> Vec<Vec<Cell>> {
        self.passes
            .iter()
            .map(|pass| {
                vec![
                    Cell::Id(pass.id),
                    Cell::text(pass.name),
                    Cell::text(pass.version),
                    Cell::List(
                        pass.preconditions
                            .iter()
                            .map(|name| Cell::opt_id(self.invariant_id(name)))
                            .collect(),
                    ),
                    Cell::List(
                        pass.postconditions
                            .iter()
                            .map(|name| Cell::opt_id(self.invariant_id(name)))
                            .collect(),
                    ),
                    Cell::Enum(pass.determinism.as_str()),
                    Cell::List(pass.diagnostics.iter().copied().map(Cell::Enum).collect()),
                ]
            })
            .collect()
    }

    /// Every `reference.pass_input_ports` row.
    fn pass_input_ports_rows(&self) -> Vec<Vec<Cell>> {
        self.passes
            .iter()
            .flat_map(|pass| {
                pass.inputs.iter().map(move |input| {
                    let (source_pass, source_port) = match input.source {
                        PortSource::Pinned => (None, None),
                        PortSource::Derived { pass, port } => (Some(pass), Some(port)),
                    };
                    vec![
                        Cell::Id(pass.id),
                        Cell::text(input.port),
                        Cell::opt_id(self.relation(input.relation).map(|spec| spec.id)),
                        Cell::opt_id(
                            source_pass
                                .and_then(|name| self.pass(name))
                                .map(|spec| spec.id),
                        ),
                        Cell::opt_text(source_port),
                        Cell::Bool(input.required),
                    ]
                })
            })
            .collect()
    }

    /// Every `reference.pass_output_ports` row.
    fn pass_output_ports_rows(&self) -> Vec<Vec<Cell>> {
        self.passes
            .iter()
            .flat_map(|pass| {
                pass.outputs.iter().map(move |output| {
                    vec![
                        Cell::Id(pass.id),
                        Cell::text(output.port),
                        Cell::opt_id(self.relation(output.relation).map(|spec| spec.id)),
                    ]
                })
            })
            .collect()
    }

    /// Every `reference.rule_specs` row.
    fn rule_specs_rows(&self) -> Vec<Vec<Cell>> {
        self.rules
            .iter()
            .map(|rule| {
                vec![
                    Cell::Id(rule.id),
                    Cell::text(rule.version),
                    Cell::U64(u64::from(rule.stratum)),
                    Cell::opt_id(self.relation(rule.head.relation()).map(|spec| spec.id)),
                    Cell::Id(rule_node_id(rule, ROOT_NODE_PATH)),
                    Cell::Enum(rule.negation.as_str()),
                    Cell::Bool(rule.monotonic),
                    Cell::Enum(rule.conflict_policy.as_str()),
                ]
            })
            .collect()
    }

    /// Every `reference.rule_plan_nodes` row.
    fn rule_plan_nodes_rows(&self) -> Vec<Vec<Cell>> {
        let mut rows = Vec::new();
        for rule in &self.rules {
            for (path, node) in walk_plan(&rule.plan) {
                let (relation, port) = match node {
                    RulePlan::Scan { relation, port } => (Some(*relation), Some(*port)),
                    _ => (None, None),
                };
                let keys = match node {
                    RulePlan::EquiJoin { keys, .. } | RulePlan::AntiJoin { keys, .. } => Some(keys),
                    _ => None,
                };
                let predicate = match node {
                    RulePlan::Filter { .. } => Some(rule_expr_id(rule, &path, "predicate")),
                    _ => None,
                };
                let projection = match node {
                    RulePlan::Project { columns, .. } => Some(Cell::List(
                        columns
                            .iter()
                            .enumerate()
                            .map(|(index, (name, _))| {
                                Cell::Struct(vec![
                                    Cell::text(*name),
                                    Cell::Id(rule_expr_id(
                                        rule,
                                        &path,
                                        &format!("project:{index}"),
                                    )),
                                ])
                            })
                            .collect(),
                    )),
                    _ => None,
                };
                rows.push(vec![
                    Cell::Id(rule_node_id(rule, &path)),
                    Cell::Id(rule.id),
                    Cell::Enum(node.op()),
                    Cell::opt_id(
                        relation
                            .and_then(|name| self.relation(name))
                            .map(|spec| spec.id),
                    ),
                    Cell::opt_text(port),
                    keys.map_or(Cell::Null, |keys| {
                        Cell::List(
                            keys.iter()
                                .map(|(left, right)| {
                                    Cell::Struct(vec![Cell::text(*left), Cell::text(*right)])
                                })
                                .collect(),
                        )
                    }),
                    Cell::opt_id(predicate),
                    projection.unwrap_or(Cell::Null),
                    Cell::Null,
                    Cell::text(""),
                ]);
            }
        }
        rows
    }

    /// Every `reference.rule_aggregates` row.
    fn rule_aggregates_rows(&self) -> Vec<Vec<Cell>> {
        let mut rows = Vec::new();
        for rule in &self.rules {
            for (path, node) in walk_plan(&rule.plan) {
                let RulePlan::Aggregate { aggregates, .. } = node else {
                    continue;
                };
                for (ordinal, aggregate) in aggregates.iter().enumerate() {
                    rows.push(vec![
                        Cell::Id(rule_node_id(rule, &path)),
                        Cell::U64(count(ordinal)),
                        Cell::Enum(aggregate.function.as_str()),
                        Cell::opt_id(
                            aggregate.input.as_ref().map(|_| {
                                rule_expr_id(rule, &path, &format!("aggregate:{ordinal}"))
                            }),
                        ),
                        Cell::text(aggregate.output_name),
                        Cell::List(
                            aggregate
                                .order_by
                                .iter()
                                .map(|(column, ascending)| {
                                    Cell::Struct(vec![Cell::text(*column), Cell::Bool(*ascending)])
                                })
                                .collect(),
                        ),
                        Cell::Enum(aggregate.null_policy.as_str()),
                        Cell::Enum(aggregate.empty_policy.as_str()),
                    ]);
                }
            }
        }
        rows
    }

    /// Every `reference.rule_group_keys` row.
    fn rule_group_keys_rows(&self) -> Vec<Vec<Cell>> {
        let mut rows = Vec::new();
        for rule in &self.rules {
            for (path, node) in walk_plan(&rule.plan) {
                let RulePlan::Aggregate { group, .. } = node else {
                    continue;
                };
                for (ordinal, column) in group.iter().enumerate() {
                    rows.push(vec![
                        Cell::Id(rule_node_id(rule, &path)),
                        Cell::U64(count(ordinal)),
                        Cell::text(*column),
                    ]);
                }
            }
        }
        rows
    }

    /// Every `reference.rule_unnest` row.
    fn rule_unnest_rows(&self) -> Vec<Vec<Cell>> {
        let mut rows = Vec::new();
        for rule in &self.rules {
            for (path, node) in walk_plan(&rule.plan) {
                let RulePlan::Unnest {
                    column,
                    value_name,
                    null_list,
                    empty_list,
                    ..
                } = node
                else {
                    continue;
                };
                rows.push(vec![
                    Cell::Id(rule_node_id(rule, &path)),
                    Cell::text(*column),
                    Cell::text(*value_name),
                    Cell::Enum(null_list.as_str()),
                    Cell::Enum(empty_list.as_str()),
                ]);
            }
        }
        rows
    }

    /// Every `reference.rule_plan_edges` row.
    fn rule_plan_edges_rows(&self) -> Vec<Vec<Cell>> {
        let mut rows = Vec::new();
        for rule in &self.rules {
            for (path, node) in walk_plan(&rule.plan) {
                for (ordinal, _) in node.children().iter().enumerate() {
                    rows.push(vec![
                        Cell::Id(rule_node_id(rule, &path)),
                        Cell::U64(count(ordinal)),
                        Cell::Id(rule_node_id(rule, &child_path(&path, ordinal))),
                    ]);
                }
            }
        }
        rows
    }

    /// Every `reference.rule_dependencies` row.
    fn rule_dependencies_rows(&self) -> Vec<Vec<Cell>> {
        self.rule_dependencies
            .iter()
            .map(|dependency| {
                vec![
                    Cell::opt_id(self.rule(dependency.rule).map(|rule| rule.id)),
                    Cell::opt_id(self.relation(dependency.relation).map(|spec| spec.id)),
                    Cell::opt_text(dependency.input_port),
                    Cell::Enum(dependency.mode.as_str()),
                    Cell::U64(u64::from(dependency.stratum)),
                    Cell::Null,
                ]
            })
            .collect()
    }

    /// The identity of the invariant named `<relation>:<name>`.
    fn invariant_id(&self, qualified_name: &str) -> Option<SemanticId> {
        self.invariants
            .iter()
            .find(|invariant| invariant.qualified_name() == qualified_name)
            .map(|invariant| invariant.id)
    }
}

/// The path of the root plan node.
const ROOT_NODE_PATH: &str = "0";

/// An ordinal as a row count cell.
///
/// The saturating conversion is unreachable on every supported target; it exists because
/// the crate's panic policy has no room for an `expect` only a 128-bit address space could
/// reach.
fn count(ordinal: usize) -> u64 {
    u64::try_from(ordinal).unwrap_or(u64::MAX)
}

/// The path of the `ordinal`-th child of the node at `parent`.
fn child_path(parent: &str, ordinal: usize) -> String {
    format!("{parent}.{ordinal}")
}

/// Every `(preorder path, node)` of a rule plan.
fn walk_plan(plan: &RulePlan) -> Vec<(String, &RulePlan)> {
    let mut out = Vec::new();
    walk_plan_into(plan, ROOT_NODE_PATH.to_owned(), &mut out);
    out
}

/// Appends `plan` and its descendants to `out` under `path`.
fn walk_plan_into<'a>(plan: &'a RulePlan, path: String, out: &mut Vec<(String, &'a RulePlan)>) {
    for (ordinal, child) in plan.children().into_iter().enumerate() {
        walk_plan_into(child, child_path(&path, ordinal), out);
    }
    out.push((path, plan));
}

/// The identity of a rule plan node: `rule_node:<rule>@<v>:<preorder path>` (ADR-0050).
pub fn rule_node_id(rule: &RuleSpec, path: &str) -> SemanticId {
    registry_id(&format!("rule_node:{}:{path}", rule.qualified_name()))
}

/// The identity of an expression graph attached to a rule plan node.
///
/// `slot` names which expression of the node it is — `predicate`, `project:<n>` or
/// `aggregate:<n>` — because one node can carry several. Packet A-3's `expr_family`
/// serializer emits the `reference.rule_expr_*` rows under these identities.
pub fn rule_expr_id(rule: &RuleSpec, path: &str, slot: &str) -> SemanticId {
    registry_id(&format!(
        "rule_expr:{}:{path}:{slot}",
        rule.qualified_name()
    ))
}

/// Sorts `rows` by the relation's declared primary key (blueprint §5.3 step 1).
fn sort_by_primary_key(spec: Option<&RelationSpec>, mut rows: Vec<Vec<Cell>>) -> Vec<Vec<Cell>> {
    let key_positions: Vec<usize> = spec.map_or_else(Vec::new, |spec| {
        spec.primary_key
            .iter()
            .filter_map(|name| spec.columns.iter().position(|column| column.name == *name))
            .collect()
    });
    rows.sort_by_cached_key(|row| {
        // The declared key first, then the whole row. The tail is a total, deterministic
        // tiebreak; it is also what orders a table whose describing relation is not
        // declared yet, and it agrees with the key order because every self-describing
        // relation's primary key is a prefix of its row.
        let mut key: Vec<Vec<u8>> = key_positions
            .iter()
            .filter_map(|position| row.get(*position))
            .map(sort_key)
            .collect();
        key.extend(row.iter().map(sort_key));
        key
    });
    rows
}

/// The relation name inside the `reference` namespace, for example `schema_relations`.
fn bare_name(qualified: &'static str) -> &'static str {
    qualified
        .split_once('.')
        .map_or(qualified, |(_, name)| name)
}

/// A byte string whose ordering is the declared one: text by UTF-8 bytes, identities by
/// raw bytes, numbers numerically (blueprint §5.3 step 7).
fn sort_key(cell: &Cell) -> Vec<u8> {
    let mut out = Vec::new();
    push_sort_key(cell, &mut out);
    out
}

/// Appends `cell`'s sort key to `out`.
fn push_sort_key(cell: &Cell, out: &mut Vec<u8>) {
    match cell {
        Cell::Null => out.push(0x00),
        Cell::Bool(value) => out.extend_from_slice(&[0x01, u8::from(*value)]),
        Cell::I64(value) => {
            out.push(0x02);
            // Flip the sign bit so that two's-complement order is byte order.
            let unsigned = u64::from_be_bytes(value.to_be_bytes()) ^ (1_u64 << 63);
            out.extend_from_slice(&unsigned.to_be_bytes());
        }
        Cell::U64(value) => {
            out.push(0x03);
            out.extend_from_slice(&value.to_be_bytes());
        }
        Cell::F64(value) => {
            out.push(0x04);
            out.extend_from_slice(&pse_ids::canonical_f64_bits(*value).to_be_bytes());
        }
        Cell::Text(value) => {
            out.push(0x05);
            out.extend_from_slice(value.as_bytes());
            out.push(0x00);
        }
        Cell::Id(value) => {
            out.push(0x06);
            out.extend_from_slice(value.as_bytes());
        }
        Cell::Hash(value) => {
            out.push(0x07);
            out.extend_from_slice(value.as_bytes());
        }
        Cell::Enum(value) => {
            out.push(0x08);
            out.extend_from_slice(value.as_bytes());
            out.push(0x00);
        }
        Cell::List(elements) | Cell::Struct(elements) => {
            out.push(0x09);
            for element in elements {
                push_sort_key(element, out);
            }
            out.push(0x00);
        }
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
    /// Rules, in declaration order.
    rules: Vec<RuleDecl>,
    /// Passes, in declaration order.
    passes: Vec<PassDecl>,
    /// Migrations, in declaration order.
    migrations: Vec<MigrationSpec>,
    /// Authoring document shapes, in declaration order.
    documents: Vec<DocumentSpec>,
    /// The manifest envelope, if declared.
    manifest: Option<ManifestSpec>,
}

impl RegistryBuilder {
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

    /// Declares a rule.
    pub fn declare_rule(&mut self, decl: RuleDecl) -> &mut Self {
        self.rules.push(decl);
        self
    }

    /// Declares a pass.
    pub fn declare_pass(&mut self, decl: PassDecl) -> &mut Self {
        self.passes.push(decl);
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

    /// Declares the `pse.manifest.v2` envelope (blueprint §20.2).
    pub fn declare_manifest(&mut self, spec: ManifestSpec) -> &mut Self {
        self.manifest = Some(spec);
        self
    }

    /// Resolves every declaration into an immutable [`Registry`].
    ///
    /// # Errors
    ///
    /// [`SchemaError::DuplicateDeclaration`] when two declarations claim one name;
    /// [`SchemaError::UnknownReference`] for a dangling foreign key, enumeration,
    /// per-row-quantity sibling, primary-key column, invariant relation or rule head;
    /// [`SchemaError::MissingGranularity`] for a derived relation without one;
    /// [`SchemaError::StageGraph`] for a duplicate output port, an unresolved derived
    /// input port, or a compiler output port targeting `authored` or `reference`;
    /// [`SchemaError::RuleFloatKey`] for a rule that keys on an `f64` column.
    pub fn build(self) -> Result<Registry, SchemaError> {
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
            package_id: *REGISTRY_PACKAGE_ID,
            fingerprint: ContentHash::NIL,
            relations,
            relation_index,
            relation_by_id,
            enums,
            logical_types,
            invariants: Vec::new(),
            migrations: Vec::new(),
            passes: Vec::new(),
            rules: Vec::new(),
            rule_dependencies: Vec::new(),
            documents: Vec::new(),
            manifest: self.manifest,
        };

        check_relation_references(&registry)?;
        registry.rules = resolve_rules(self.rules, &registry)?;
        registry.rules.sort_by_key(RuleSpec::qualified_name);
        registry.rule_dependencies = crate::rule_deps::derive(&registry.rules);
        registry.invariants = resolve_invariants(self.invariants, &registry)?;
        registry
            .invariants
            .sort_by_key(InvariantSpec::qualified_name);
        registry.passes = resolve_passes(self.passes, &registry)?;
        registry.passes.sort_by_key(PassSpec::qualified_name);
        registry.migrations = resolve_migrations(self.migrations, &registry)?;
        registry.documents = resolve_documents(self.documents, &registry)?;
        crate::checks::run(&registry)?;

        let fingerprints: Vec<ContentHash> = registry
            .relations
            .iter()
            .map(|spec| crate::fingerprint::relation(&registry, spec))
            .collect();
        for (spec, fingerprint) in registry.relations.iter_mut().zip(fingerprints) {
            spec.fingerprint = fingerprint;
        }
        registry.fingerprint = crate::fingerprint::registry(&registry.schema_rows());
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
            out.push(RelationSpec {
                id: registry_id(&format!("relation:{name}")),
                key: decl.key,
                authority: decl.authority,
                snapshot_class: decl.snapshot_class,
                derivation_granularity: decl.derivation_granularity,
                stability: decl.stability,
                primary_key: decl.primary_key.clone(),
                columns: decl.columns.clone(),
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
fn record_logical_type(ty: &LogicalType, types: &mut BTreeMap<String, LogicalType>) {
    let mut reachable = Vec::new();
    ty.walk(&mut reachable);
    for child in reachable {
        types.insert(child.name(), child);
    }
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
) -> Result<Vec<LogicalTypeRow>, SchemaError> {
    let mut types: BTreeMap<String, LogicalType> = BTreeMap::new();
    for scalar in [
        LogicalType::F64,
        LogicalType::I64,
        LogicalType::I32,
        LogicalType::U8,
        LogicalType::U16,
        LogicalType::U32,
        LogicalType::U64,
        LogicalType::Bool,
        LogicalType::Text,
        LogicalType::Timestamp,
    ] {
        record_logical_type(&scalar, &mut types);
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
        record_logical_type(&LogicalType::Ext(use_), &mut types);
    }
    for spec in enums {
        record_logical_type(&LogicalType::enumeration(spec.name), &mut types);
    }
    for relation in relations {
        for column in &relation.columns {
            record_logical_type(&column.logical_type, &mut types);
        }
    }

    let mut rows = Vec::with_capacity(types.len());
    for (name, ty) in types {
        let extension = ty.extension();
        rows.push(LogicalTypeRow {
            id: registry_id(&format!("logical_type:{name}")),
            arrow_storage: render_data_type(&ty.data_type())?,
            extension_name: extension.map(ExtensionUse::extension_name),
            metadata_schema: extension.map(|use_| {
                let spec = use_.spec();
                ext_metadata::json_schema(spec.metadata, spec.metadata_version)
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
    column: &ColumnSpec,
) -> Result<(), SchemaError> {
    let context = format!("column {}.{}", spec.key, column.name);
    if let Some(fk) = column.fk {
        let target =
            registry
                .relation(fk.relation)
                .ok_or_else(|| SchemaError::UnknownReference {
                    context: context.clone(),
                    reference: fk.relation.to_owned(),
                })?;
        if target.column(fk.column).is_none() {
            return Err(SchemaError::UnknownReference {
                context: context.clone(),
                reference: fk.to_string(),
            });
        }
    }
    if column.quantity == QuantityContract::PerRow
        && spec.column(&column.per_row_quantity_sibling()).is_none()
    {
        return Err(SchemaError::UnknownReference {
            context: context.clone(),
            reference: column.per_row_quantity_sibling(),
        });
    }
    let mut reachable = Vec::new();
    column.logical_type.walk(&mut reachable);
    for ty in reachable {
        match ty.extension() {
            Some(ExtensionUse::Enum(name)) if registry.enum_spec(name).is_none() => {
                return Err(SchemaError::UnknownReference {
                    context: context.clone(),
                    reference: format!("enum:{name}"),
                });
            }
            Some(ExtensionUse::OrdinalRef { target }) if registry.relation(target).is_none() => {
                return Err(SchemaError::UnknownReference {
                    context: context.clone(),
                    reference: (*target).to_owned(),
                });
            }
            _ => {}
        }
    }
    Ok(())
}

/// Assigns identities to the rule declarations and checks their references.
///
/// # Errors
///
/// [`SchemaError::DuplicateDeclaration`], [`SchemaError::UnknownReference`] or
/// [`SchemaError::RuleFloatKey`].
fn resolve_rules(decls: Vec<RuleDecl>, registry: &Registry) -> Result<Vec<RuleSpec>, SchemaError> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = Vec::with_capacity(decls.len());
    for decl in decls {
        let name = format!("{}@{}", decl.name, decl.version);
        if !seen.insert(name.clone()) {
            return Err(SchemaError::DuplicateDeclaration { kind: "rule", name });
        }
        if registry.relation(decl.head.relation()).is_none() {
            return Err(SchemaError::UnknownReference {
                context: format!("head of rule {name}"),
                reference: decl.head.relation().to_owned(),
            });
        }
        for (relation, _, _) in decl.plan.dependencies() {
            if registry.relation(relation).is_none() {
                return Err(SchemaError::UnknownReference {
                    context: format!("body of rule {name}"),
                    reference: relation.to_owned(),
                });
            }
        }
        check_rule_keys(&name, &decl.plan, registry)?;
        out.push(RuleSpec {
            id: registry_id(&format!("rule:{name}")),
            name: decl.name,
            version: decl.version,
            stratum: decl.stratum,
            head: decl.head,
            plan: decl.plan,
            negation: decl.negation,
            monotonic: decl.monotonic,
            conflict_policy: decl.conflict_policy,
        });
    }
    Ok(out)
}

/// Rejects a rule that joins, groups or deduplicates on an `f64` column
/// (blueprint §14.2 rule 7).
///
/// # Errors
///
/// [`SchemaError::RuleFloatKey`].
fn check_rule_keys(rule: &str, plan: &RulePlan, registry: &Registry) -> Result<(), SchemaError> {
    for (_, node) in walk_plan(plan) {
        let keys: Vec<&'static str> = match node {
            RulePlan::EquiJoin { keys, .. } | RulePlan::AntiJoin { keys, .. } => keys
                .iter()
                .flat_map(|(left, right)| [*left, *right])
                .collect(),
            RulePlan::Aggregate { group, .. } => group.clone(),
            _ => Vec::new(),
        };
        for key in keys {
            if scanned_column_is_float(node, key, registry) {
                return Err(SchemaError::RuleFloatKey {
                    rule: rule.to_owned(),
                    column: key.to_owned(),
                });
            }
        }
    }
    Ok(())
}

/// Whether `column` resolves, through a scan reachable from `node`, to an `f64` column.
///
/// Resolution stops at what the plan scans: a projected expression's type is the rule
/// compiler's business, and this check deliberately reports only what the registry alone
/// can prove.
fn scanned_column_is_float(node: &RulePlan, column: &str, registry: &Registry) -> bool {
    walk_plan(node).into_iter().any(|(_, child)| {
        matches!(child, RulePlan::Scan { relation, .. }
            if registry
                .relation(relation)
                .and_then(|spec| spec.column(column))
                .is_some_and(|spec| spec.logical_type == LogicalType::F64))
    })
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
        if registry.relation(decl.relation).is_none() {
            return Err(SchemaError::UnknownReference {
                context: format!("invariant {name}"),
                reference: decl.relation.to_owned(),
            });
        }
        if registry.rule(decl.rule).is_none() {
            return Err(SchemaError::UnknownReference {
                context: format!("invariant {name}"),
                reference: decl.rule.to_owned(),
            });
        }
        out.push(InvariantSpec {
            id: registry_id(&format!("invariant:{name}")),
            name: decl.name,
            relation: decl.relation,
            kind: decl.kind,
            rule: decl.rule,
            severity: decl.severity,
            doc: decl.doc,
        });
    }
    Ok(out)
}

/// Assigns identities to the pass declarations and checks the port graph.
///
/// # Errors
///
/// [`SchemaError::DuplicateDeclaration`], [`SchemaError::UnknownReference`] or
/// [`SchemaError::StageGraph`].
fn resolve_passes(decls: Vec<PassDecl>, registry: &Registry) -> Result<Vec<PassSpec>, SchemaError> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = Vec::with_capacity(decls.len());
    for decl in decls {
        let name = format!("{}@{}", decl.name, decl.version);
        if !seen.insert(name.clone()) {
            return Err(SchemaError::DuplicateDeclaration { kind: "pass", name });
        }
        let mut ports: BTreeSet<&'static str> = BTreeSet::new();
        for output in &decl.outputs {
            if !ports.insert(output.port) {
                return Err(SchemaError::DuplicateDeclaration {
                    kind: "output port",
                    name: format!("{name}.{}", output.port),
                });
            }
            let target = registry.relation(output.relation).ok_or_else(|| {
                SchemaError::UnknownReference {
                    context: format!("output port {name}.{}", output.port),
                    reference: output.relation.to_owned(),
                }
            })?;
            if pass_ordinal(decl.name).is_some_and(|ordinal| ordinal >= 3)
                && NON_DERIVABLE_NAMESPACES.contains(&target.key.namespace)
            {
                return Err(SchemaError::StageGraph {
                    reason: format!(
                        "{name} outputs {} into `{}`; compilation writes neither authored nor reference relations (blueprint §14.1)",
                        output.port, target.key.namespace
                    ),
                });
            }
        }
        for input in &decl.inputs {
            if registry.relation(input.relation).is_none() {
                return Err(SchemaError::UnknownReference {
                    context: format!("input port {name}.{}", input.port),
                    reference: input.relation.to_owned(),
                });
            }
        }
        out.push(PassSpec {
            id: registry_id(&format!("pass:{name}")),
            name: decl.name,
            version: decl.version,
            inputs: decl.inputs,
            outputs: decl.outputs,
            preconditions: decl.preconditions,
            postconditions: decl.postconditions,
            determinism: decl.determinism,
            diagnostics: decl.diagnostics,
            executes_plans: decl.executes_plans,
        });
    }
    check_derived_ports(&out)?;
    Ok(out)
}

/// The `n` of a pass named `P<n>`, when the name has that shape (blueprint §14.1).
fn pass_ordinal(name: &str) -> Option<u16> {
    name.strip_prefix('P').and_then(|rest| rest.parse().ok())
}

/// Resolves every derived input port against its producing pass.
///
/// # Errors
///
/// [`SchemaError::StageGraph`].
fn check_derived_ports(passes: &[PassSpec]) -> Result<(), SchemaError> {
    for pass in passes {
        for input in &pass.inputs {
            let PortSource::Derived { pass: from, port } = input.source else {
                continue;
            };
            let producer = passes
                .iter()
                .find(|candidate| candidate.name == from)
                .ok_or_else(|| SchemaError::StageGraph {
                    reason: format!(
                        "{}.{} reads {from}.{port}, and no pass named {from} is declared",
                        pass.name, input.port
                    ),
                })?;
            if producer.output(port).is_none() {
                return Err(SchemaError::StageGraph {
                    reason: format!(
                        "{}.{} reads {from}.{port}, and {from} declares no such output port",
                        pass.name, input.port
                    ),
                });
            }
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
        for section in &spec.sections {
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
    use crate::model::{
        Authority, ColumnRole, DerivationGranularity, EnumMember, Namespace, SnapshotClass,
    };

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
        .columns(vec![ColumnSpec::key(
            "id",
            LogicalType::id(),
            "the identity",
        )])
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
            ColumnSpec::key("id", LogicalType::id(), "the identity"),
            ColumnSpec::reference("other_id", LogicalType::id(), "a reference")
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
                ColumnSpec::key("id", LogicalType::id(), "the identity"),
                ColumnSpec::reference("a_id", LogicalType::id(), "a reference")
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
            ColumnSpec::key("id", LogicalType::id(), "the identity"),
            ColumnSpec::label("kind", LogicalType::enumeration("Nowhere"), "a label"),
        ]));
        assert!(matches!(
            builder.build().unwrap_err(),
            SchemaError::UnknownReference { .. }
        ));
    }

    #[test]
    fn a_per_row_quantity_without_its_sibling_is_rejected() {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(simple("a").columns(vec![
            ColumnSpec::key("id", LogicalType::id(), "the identity"),
            ColumnSpec::new(
                "value",
                LogicalType::F64,
                false,
                ColumnRole::Measure,
                "a measure",
            )
            .with_per_row_quantity(),
        ]));
        let error = builder.build().unwrap_err();
        assert!(matches!(
            &error,
            SchemaError::UnknownReference { reference, .. } if reference == "value_quantity_type_id"
        ));
    }

    #[test]
    fn a_per_row_quantity_with_its_sibling_is_accepted() {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(simple("a").columns(vec![
            ColumnSpec::key("id", LogicalType::id(), "the identity"),
            ColumnSpec::new(
                "value",
                LogicalType::F64,
                false,
                ColumnRole::Measure,
                "a measure",
            )
            .with_per_row_quantity(),
            ColumnSpec::reference(
                "value_quantity_type_id",
                LogicalType::id(),
                "the per-row contract",
            ),
        ]));
        assert!(builder.build().is_ok());
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
            .granularity(DerivationGranularity::Row),
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
            Some("FixedSizeBinary(16)")
        );
    }
}
