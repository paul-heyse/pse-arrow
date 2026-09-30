// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed provenance and test-only taint (ADR-0123 Outcome 5, Plan 23 KR6).
//!
//! Every dataset and constant names a source entity, a role and its lineage, each by path;
//! a test may name the source entity of its expected values as its oracle. A source is an
//! entity whose kind, or an ancestor kind, carries the provenance facet. A role is a member
//! of a package enumeration; the kernel acts only on the data facets the member declares:
//! `test_only` data is read by test fixtures only, and data whose role `requires_lineage`
//! names a nonempty lineage. Lineage entries name datasets and sources, and the lineage
//! between datasets is acyclic. Everything else about a role is package knowledge, so a new
//! role is a package edit.
//!
//! Test-only taint is the one authority for "production reads no test-only data". Each
//! row's origin role is the role of the dataset that supplied it, which the kernel
//! provides; it is not a declared attribute. Data a test-only role supplies is test-only,
//! and so is data derived from it: a dataset or constant whose lineage reaches test-only
//! data, through datasets or a test-only source, is test-only whatever its own role, so a
//! derived dataset cannot launder oracle data into production. The taint propagates along
//! resolved references: an entity, keyed row, table row or constant that references
//! test-only data is itself test-only. A relation holding rows of several datasets is
//! tainted row by row, so its production rows stay readable. A specialization root outside a
//! test fixture that reads test-only data is refused where it reads it ([`Reader`]): a
//! lookup whose keys are literal resolves its row statically, any other lookup once
//! specialization binds its keys.
use crate::specialize::value::Value;
use crate::{CheckedPackage, DeclarationId, ModelingError, Result, Type, invalid};
use pse_authoring::language::ModelingProvenance;
use pse_ids::SemanticId;
use pse_model::generated::enums::{
    ModelingDataFacet as Facet, ModelingDeclarationKind as K, ModelingLineageKind as Lineage,
};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

/// A role: a member of a package enumeration and the data facets it declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Role {
    /// The enumeration declaring the member.
    pub enumeration: DeclarationId,
    /// The member's identity.
    pub member: SemanticId,
    /// The data facets the member declares.
    pub facets: BTreeSet<Facet>,
}

/// The checked provenance of a dataset or a constant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provenance {
    /// The source entity.
    pub source: DeclarationId,
    /// The role, with its facets.
    pub role: Role,
    /// What the data derive from: datasets and source entities, in authored order.
    pub lineage: Vec<(Lineage, DeclarationId)>,
}
impl Provenance {
    /// Whether the role makes the data test-only.
    pub fn test_only(&self) -> bool {
        self.role.facets.contains(&Facet::TestOnly)
    }
}

impl CheckedPackage {
    /// The checked provenance of a dataset or a constant.
    pub fn provenance(&self, data: DeclarationId) -> Option<&Provenance> {
        self.provenance.get(&data)
    }
    /// The role of the dataset or constant that supplied data: a row's origin role, which
    /// the kernel provides rather than a declared attribute.
    pub fn origin_role(&self, origin: DeclarationId) -> Option<&Role> {
        self.provenance.get(&origin).map(|p| &p.role)
    }
    /// The source entity a test names as the source of its expected values.
    pub fn oracle(&self, test: DeclarationId) -> Option<DeclarationId> {
        self.oracles.get(&test).copied()
    }
    /// Whether an admitted entity, keyed row or constant is test-only.
    pub fn is_test_only(&self, id: DeclarationId) -> bool {
        self.test_only.contains(&id)
    }
    /// Whether a dataset or constant supplies test-only data: its role is test-only, or its
    /// lineage reaches test-only data.
    pub fn supplies_test_only(&self, origin: DeclarationId) -> bool {
        self.test_only_data.contains(&origin)
    }
    /// Whether `value` references test-only data: a test-only entity or keyed row, through
    /// sets, tuples and materialized rows.
    pub(crate) fn references_test_only(&self, value: &Value) -> bool {
        match value {
            Value::Entity { id, .. } => self.test_only.contains(id),
            Value::Set(values) | Value::Tuple(values) => {
                values.iter().any(|v| self.references_test_only(v))
            }
            Value::Row { fields, .. } => fields.iter().any(|v| self.references_test_only(v)),
            _ => false,
        }
    }
}

/// Who reads admitted data through an evaluation.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Reader<'a> {
    /// Admission reads any data; a derived value records that it read test-only data.
    Admission(Option<&'a Cell<bool>>),
    /// A test fixture's specialization reads any data.
    Fixture,
    /// The specialization of a root outside a test fixture reads no test-only data.
    Production(DeclarationId),
}
impl Reader<'_> {
    /// The reader of `root`'s specialization: a test is a fixture; every other root is
    /// production.
    pub(crate) fn of(p: &CheckedPackage, root: DeclarationId) -> Reader<'static> {
        if p.declarations
            .get(&root)
            .is_some_and(|row| row.value.kind == K::Test)
        {
            Reader::Fixture
        } else {
            Reader::Production(root)
        }
    }
    /// Record a read at `at` of data that is test-only when `test_only` holds; `data` names
    /// it in a refusal.
    pub(crate) fn read(
        self,
        p: &CheckedPackage,
        at: DeclarationId,
        test_only: bool,
        data: impl FnOnce() -> String,
    ) -> Result<()> {
        if !test_only {
            return Ok(());
        }
        match self {
            Self::Admission(seen) => {
                if let Some(seen) = seen {
                    seen.set(true);
                }
                Ok(())
            }
            Self::Fixture => Ok(()),
            Self::Production(root) => Err(ModelingError::TestOnly {
                declaration: at.as_id(),
                root: root.as_id(),
                data: format!(
                    "{}, read by root {}",
                    data(),
                    p.declarations
                        .get(&root)
                        .map_or_else(|| root.to_string(), |row| row.name.clone())
                ),
            }),
        }
    }
}
/// How a refusal names test-only data supplied by `origin`: its dataset or constant and
/// the role that makes it test-only.
pub(crate) fn supplied_by(p: &CheckedPackage, origin: DeclarationId) -> String {
    let name = |id: &DeclarationId| {
        p.declarations
            .get(id)
            .map_or_else(|| id.to_string(), |row| row.name.clone())
    };
    match p.provenance.get(&origin) {
        Some(provenance) if provenance.test_only() => format!(
            "supplied by {} with role {}",
            name(&origin),
            member_name(p, &provenance.role)
        ),
        Some(provenance) if p.supplies_test_only(origin) => format!(
            "supplied by {} with role {}, whose lineage reaches test-only data",
            name(&origin),
            member_name(p, &provenance.role)
        ),
        Some(provenance) => format!(
            "supplied by {} with role {}, referencing test-only data",
            name(&origin),
            member_name(p, &provenance.role)
        ),
        None => format!("referencing test-only data through {}", name(&origin)),
    }
}
fn member_name(p: &CheckedPackage, role: &Role) -> String {
    p.declarations[&role.enumeration]
        .value
        .enumeration
        .as_ref()
        .and_then(|e| e.members.iter().find(|m| m.member_id == role.member))
        .map_or_else(|| role.member.to_string(), |m| m.name.clone())
}

/// Resolve the provenance of every dataset and constant and the oracle of every test, then
/// taint the admitted entities, keyed rows and constants. Runs after entities and constants
/// are admitted and before tables are, which taint their rows as they admit them.
pub(crate) fn admit(p: &mut CheckedPackage) -> Result<()> {
    let mut provenance = BTreeMap::new();
    let mut oracles = BTreeMap::new();
    for row in p.declarations.values() {
        let id = row.declaration_id;
        let declared = row
            .value
            .dataset
            .as_ref()
            .map(|d| &d.provenance)
            .or_else(|| row.value.constant.as_ref().map(|c| &c.provenance));
        if let Some(declared) = declared {
            provenance.insert(id, resolve(p, id, declared)?);
        }
        if let Some(path) = row.value.scope.as_ref().and_then(|s| s.oracle.as_ref()) {
            oracles.insert(id, source(p, id, path, "an oracle")?);
        }
    }
    acyclic(p, &provenance)?;
    p.provenance = provenance;
    p.oracles = oracles;
    taint(p);
    Ok(())
}

/// A dataset's or constant's provenance, each path resolved once.
fn resolve(p: &CheckedPackage, at: DeclarationId, declared: &ModelingProvenance) -> Result<Provenance> {
    let origin = source(p, at, &declared.source, "a provenance source")?;
    let role = role(p, at, &declared.role)?;
    let lineage = declared
        .lineage
        .iter()
        .map(|entry| {
            let target = match entry.kind {
                Lineage::Dataset => {
                    let name = entry.path.join(".");
                    p.resolve(at, &name)
                        .filter(|id| p.declarations[id].value.dataset.is_some())
                        .ok_or_else(|| {
                            invalid(at, format!("lineage entry {name} is not a dataset"))
                        })?
                }
                Lineage::Source => source(p, at, &entry.path, "a lineage source")?,
            };
            Ok((entry.kind, target))
        })
        .collect::<Result<Vec<_>>>()?;
    if role.facets.contains(&Facet::RequiresLineage) && lineage.is_empty() {
        return Err(invalid(
            at,
            format!(
                "role {} requires lineage; {} names none",
                declared.role.join("."),
                p.declarations[&at].name
            ),
        ));
    }
    Ok(Provenance {
        source: origin,
        role,
        lineage,
    })
}

/// A source: a declared entity whose kind or an ancestor kind carries the provenance facet.
fn source(p: &CheckedPackage, at: DeclarationId, path: &[String], what: &str) -> Result<DeclarationId> {
    let name = path.join(".");
    let id = p
        .resolve(at, &name)
        .ok_or_else(|| invalid(at, format!("{what} names unknown entity {name}")))?;
    match (p.declarations[&id].value.kind, p.types.get(&id)) {
        (K::Entity, Some(Type::Entity(kind))) if p.kinds.get(kind).is_some_and(|k| k.provenance) => {
            Ok(id)
        }
        (K::Entity, Some(Type::Entity(kind))) => Err(invalid(
            at,
            format!(
                "{what} is an entity whose kind carries the provenance facet; {name} is a {}, which does not",
                p.declarations[kind].name
            ),
        )),
        _ => Err(invalid(
            at,
            format!("{what} is an entity whose kind carries the provenance facet; {name} is not an entity"),
        )),
    }
}

/// A role: `enumeration.member`, the member of a package enumeration.
fn role(p: &CheckedPackage, at: DeclarationId, path: &[String]) -> Result<Role> {
    let name = path.join(".");
    let (member, owner) = path
        .split_last()
        .filter(|(_, owner)| !owner.is_empty())
        .ok_or_else(|| invalid(at, format!("a role names its enumeration and member; {name} does not")))?;
    let enumeration = p
        .resolve(at, &owner.join("."))
        .filter(|id| p.declarations[id].value.enumeration.is_some())
        .ok_or_else(|| {
            invalid(at, format!("role {name} is not a member of a declared enumeration"))
        })?;
    let declared = p.declarations[&enumeration]
        .value
        .enumeration
        .as_ref()
        .and_then(|e| e.members.iter().find(|m| m.name == *member))
        .ok_or_else(|| {
            invalid(
                at,
                format!(
                    "{member} is not a member of {}",
                    p.declarations[&enumeration].name
                ),
            )
        })?;
    Ok(Role {
        enumeration,
        member: declared.member_id,
        facets: declared.facets.iter().copied().collect(),
    })
}

/// The lineage between datasets is acyclic; a cycle is refused with its datasets named.
fn acyclic(p: &CheckedPackage, provenance: &BTreeMap<DeclarationId, Provenance>) -> Result<()> {
    let mut graph = petgraph::graph::DiGraph::<DeclarationId, ()>::new();
    let nodes = provenance
        .keys()
        .map(|id| (*id, graph.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    for (id, data) in provenance {
        for (kind, target) in &data.lineage {
            if *kind == Lineage::Dataset
                && let Some(from) = nodes.get(target)
            {
                graph.add_edge(*from, nodes[id], ());
            }
        }
    }
    if let Some(cycle) = petgraph::algo::kosaraju_scc(&graph)
        .into_iter()
        .find(|c| c.len() > 1 || c.first().is_some_and(|n| graph.contains_edge(*n, *n)))
    {
        let mut names = cycle
            .iter()
            .map(|n| p.declarations[&graph[*n]].name.clone())
            .collect::<Vec<_>>();
        names.sort();
        let first = cycle.iter().map(|n| graph[*n]).min().unwrap_or(graph[cycle[0]]);
        return Err(invalid(
            first,
            format!("lineage is cyclic: {} derive from one another", names.join(", ")),
        ));
    }
    Ok(())
}

/// Taint the test-only data: the datasets and constants whose role is test-only or whose
/// lineage reaches test-only data, the rows they supply, and every entity or constant that
/// references test-only data. A lineage source is test-only data when its entity is, so the
/// two closures are taken together until neither grows.
fn taint(p: &mut CheckedPackage) {
    let mut data = p
        .provenance
        .iter()
        .filter(|(_, provenance)| provenance.test_only())
        .map(|(id, _)| *id)
        .collect::<BTreeSet<_>>();
    loop {
        let tainted = referenced(p, &data);
        let derived = p
            .provenance
            .iter()
            .filter(|(id, provenance)| {
                !data.contains(*id)
                    && provenance.lineage.iter().any(|(kind, target)| match kind {
                        Lineage::Dataset => data.contains(target),
                        Lineage::Source => tainted.contains(target),
                    })
            })
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        if derived.is_empty() {
            p.test_only = tainted;
            p.test_only_data = data;
            return;
        }
        data.extend(derived);
    }
}

/// Taint the entities whose derived attributes read test-only data (Plan 23 D0), then every
/// entity, constant and table row that references them.
pub(crate) fn taint_derived(p: &mut CheckedPackage, derived: BTreeSet<DeclarationId>) {
    let mut pending = derived
        .into_iter()
        .filter(|id| p.test_only.insert(*id))
        .collect::<Vec<_>>();
    if pending.is_empty() {
        return;
    }
    let mut referrers = BTreeMap::<DeclarationId, Vec<DeclarationId>>::new();
    let values = p
        .entities
        .iter()
        .flat_map(|(id, record)| record.values.values().map(move |v| (*id, v)))
        .chain(p.constants.iter().map(|(id, c)| (*id, &c.value)));
    for (id, value) in values {
        let mut targets = Vec::new();
        entity_references(value, &mut targets);
        for target in targets {
            referrers.entry(target).or_default().push(id);
        }
    }
    while let Some(id) = pending.pop() {
        for referrer in referrers.get(&id).into_iter().flatten() {
            if p.test_only.insert(*referrer) {
                pending.push(*referrer);
            }
        }
    }
    let test_only = p.test_only.clone();
    let references = |value: &Value| {
        let mut targets = Vec::new();
        entity_references(value, &mut targets);
        targets.iter().any(|id| test_only.contains(id))
    };
    for table in p.tables.values_mut() {
        for row in table.rows.values_mut() {
            if !row.test_only && row.cells.iter().any(&references) {
                row.test_only = true;
            }
        }
    }
}

/// The entities and keyed rows a value references, through sets, tuples and rows.
fn entity_references(value: &Value, out: &mut Vec<DeclarationId>) {
    match value {
        Value::Entity { id, .. } => out.push(*id),
        Value::Set(values) | Value::Tuple(values) => {
            for v in values {
                entity_references(v, out);
            }
        }
        Value::Row { fields, .. } => {
            for v in fields.iter() {
                entity_references(v, out);
            }
        }
        _ => {}
    }
}

/// The entities, keyed rows and constants that `data` supplies or that reference test-only
/// data.
fn referenced(p: &CheckedPackage, data: &BTreeSet<DeclarationId>) -> BTreeSet<DeclarationId> {
    let mut tainted = BTreeSet::new();
    let mut referrers = BTreeMap::<DeclarationId, Vec<DeclarationId>>::new();
    let references = entity_references;
    let supplied = |origin: &DeclarationId| data.contains(origin);
    for (id, record) in &p.entities {
        if supplied(&record.origin) {
            tainted.insert(*id);
        }
        let mut targets = Vec::new();
        for value in record.values.values() {
            references(value, &mut targets);
        }
        for target in targets {
            referrers.entry(target).or_default().push(*id);
        }
    }
    for (id, constant) in &p.constants {
        if supplied(id) {
            tainted.insert(*id);
        }
        let mut targets = Vec::new();
        references(&constant.value, &mut targets);
        for target in targets {
            referrers.entry(target).or_default().push(*id);
        }
    }
    let mut pending = tainted.iter().copied().collect::<Vec<_>>();
    while let Some(id) = pending.pop() {
        for referrer in referrers.get(&id).into_iter().flatten() {
            if tainted.insert(*referrer) {
                pending.push(*referrer);
            }
        }
    }
    tainted
}
