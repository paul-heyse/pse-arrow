// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Relations with constraints (ADR-0123 Outcome 3, Plan 23 KR5).
//!
//! A table declares typed keys, supplied and derived columns or one value type, its absence
//! policy and the constraints its rows satisfy. Datasets supply rows as cells, each typed
//! once. Admission runs in three phases, none of which depends on declaration order:
//!
//! 1. Row identities are formed from keys: key cells are typed, an integer-range key is
//!    checked against its range, and a symmetric key pair takes its canonical orientation.
//! 2. Order-free checks run: row and entity references must name admitted rows, completeness
//!    claims hold over their declared sets, a symmetric pair is written once, and uniqueness
//!    constraints hold.
//! 3. Derived columns are evaluated once per row: tables in the order of their value
//!    dependencies, and a table's rows in the order of their references to one another. A
//!    cycle is refused with its tables or rows named. Row requirements are evaluated last,
//!    and a refusal names the row and the clause.
//!
//! A required table admits a lookup only inside its completeness, which admission has
//! verified, so a lookup outside it is refused before any row is read. Rows are positional
//! cells; a row value carries its column names for member access and framing.
//!
//! Each row carries its test-only taint (ADR-0123 Outcome 5): its dataset's role is
//! test-only or its dataset's lineage reaches test-only data, or it references a test-only
//! entity or row, or a derived column read test-only data. A lookup by a root outside a test
//! fixture refuses a test-only row.
//!
//! A table may declare validity envelopes as data (ADR-0123 Outcome 4): an axis, its
//! quantity type and the two typed columns of each row that bound it.
use crate::entity::{Rows, typed};
use crate::provenance::Reader;
use crate::specialize::value::{Environment, Evaluator, Value, conforms};
use crate::{CheckedPackage, DeclarationId, Result, Type, TypeContext, invalid};
use petgraph::algo::{kosaraju_scc, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use pse_authoring::dsl;
use pse_authoring::language::{Cell, CellSelected, ModelingCompleteness};
use pse_model::generated::enums::{
    ModelingDiagonalPolicy as Diagonal, ModelingMissingPolicy as Missing,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The key tuples one completeness claim may enumerate at admission.
const CLAIM_LIMIT: usize = 1_000_000;
/// The static evaluation allowance of one derived value or requirement.
const EVALUATION_LIMIT: usize = 100_000;

/// The finite values one key ranges over in a completeness claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Domain {
    /// The members of a declared set or enumeration.
    Set(BTreeSet<Value>),
    /// An inclusive integer range.
    Range(i64, i64),
}
impl Domain {
    /// Whether `value` is a member.
    pub fn contains(&self, value: &Value) -> bool {
        match (self, value) {
            (Self::Set(members), value) => members.contains(value),
            (Self::Range(lower, upper), Value::Integer(v)) => lower <= v && v <= upper,
            (Self::Range(..), _) => false,
        }
    }
    fn size(&self) -> usize {
        match self {
            Self::Set(members) => members.len(),
            Self::Range(lower, upper) => {
                usize::try_from(upper.saturating_sub(*lower).saturating_add(1)).unwrap_or(usize::MAX)
            }
        }
    }
    fn members(&self) -> Vec<Value> {
        match self {
            Self::Set(members) => members.iter().cloned().collect(),
            Self::Range(lower, upper) => (*lower..=*upper).map(Value::Integer).collect(),
        }
    }
}

/// One completeness claim: the table or dataset that declares it and one domain per key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    /// The declaring table, or the dataset claiming a table's open keys.
    pub origin: DeclarationId,
    /// One finite domain per key, in key order.
    pub domains: Vec<Domain>,
}

/// What a lookup of a key without a row answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Absence {
    /// A lookup is admitted only inside the claims, whose rows admission verified.
    Required(Vec<Claim>),
    /// A lookup answers explicit absence, which a consumer must guard.
    Optional,
    /// A lookup answers the declared typed default.
    Default(Value),
}

/// A typed key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    /// Key name, bound in derived columns and requirements.
    pub name: String,
    /// Key type.
    pub ty: Type,
    /// The inclusive range of an integer-range key.
    pub range: Option<(i64, i64)>,
}

/// A typed column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    /// Column name.
    pub name: String,
    /// Column type.
    pub ty: Type,
    /// Whether admission derives the column rather than a dataset supplying it.
    pub derived: bool,
}

/// A symmetric key pair: each unordered pair is stored once, in canonical orientation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Symmetry {
    /// Position of the first key of the pair.
    pub first: usize,
    /// Position of the second key of the pair.
    pub second: usize,
    /// Whether a row may have equal pair keys.
    pub diagonal: Diagonal,
}

/// An admitted row: positional cells, one per column or the one value of a scalar table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// Values in column order.
    pub cells: Arc<[Value]>,
    /// The dataset that supplied the row; its role is the row's origin role.
    pub origin: DeclarationId,
    /// Whether the row is test-only: its origin role is, or it references test-only data.
    pub test_only: bool,
}

/// An admitted relation. Authored declarations remain the durable authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    /// Ordered typed keys.
    pub keys: Vec<Key>,
    /// Columns in source order; empty for a table of one value type.
    pub columns: Vec<Column>,
    /// Column names in column order, shared by every row value.
    pub names: Arc<[String]>,
    /// The whole scalar or row type a lookup answers.
    pub result: Type,
    /// What a lookup of a key without a row answers.
    pub absence: Absence,
    /// A symmetric key pair.
    pub symmetry: Option<Symmetry>,
    /// Admitted rows by canonical key.
    pub rows: BTreeMap<Vec<Value>, Row>,
    /// Validity envelopes, each bounded by two typed columns of every row (ADR-0123
    /// Outcome 4).
    pub envelopes: Vec<crate::envelope::Envelope>,
    /// Whether every row is admitted; a table is read only once it is.
    pub(crate) complete: bool,
}
impl Table {
    /// Whether a lookup answers explicit absence.
    pub fn optional(&self) -> bool {
        matches!(self.absence, Absence::Optional)
    }
    /// The canonical orientation of a key tuple and whether it was swapped.
    fn canonical(&self, mut keys: Vec<Value>) -> (Vec<Value>, bool) {
        if let Some(pair) = self.symmetry
            && keys[pair.first] > keys[pair.second]
        {
            keys.swap(pair.first, pair.second);
            return (keys, true);
        }
        (keys, false)
    }
    fn excluded_diagonal(&self, keys: &[Value]) -> bool {
        self.symmetry.is_some_and(|pair| {
            pair.diagonal == Diagonal::Excluded && keys[pair.first] == keys[pair.second]
        })
    }
    /// Whether a canonical key tuple lies inside a claim, in either orientation of a
    /// symmetric pair.
    fn completes(&self, claims: &[Claim], keys: &[Value]) -> bool {
        if self.excluded_diagonal(keys) {
            return false;
        }
        let within = |keys: &[Value]| {
            claims.iter().any(|claim| {
                claim
                    .domains
                    .iter()
                    .zip(keys)
                    .all(|(domain, key)| domain.contains(key))
            })
        };
        within(keys)
            || self.symmetry.is_some_and(|pair| {
                let mut swapped = keys.to_vec();
                swapped.swap(pair.first, pair.second);
                within(&swapped)
            })
    }
    /// A row's value: the one value of a scalar table, else the row of its columns.
    pub fn value(&self, table: DeclarationId, row: &Row) -> Value {
        if self.columns.is_empty() {
            row.cells.first().cloned().unwrap_or(Value::Missing)
        } else {
            Value::Row {
                table,
                names: Arc::clone(&self.names),
                fields: Arc::clone(&row.cells),
            }
        }
    }
    /// The value at `keys`, looked up at `at` by `reader`. A symmetric pair answers both
    /// orders. A required table refuses a key outside its completeness before reading any
    /// row; a root outside a test fixture refuses a test-only row (ADR-0123 Outcome 5).
    pub(crate) fn lookup(
        &self,
        p: &CheckedPackage,
        table: DeclarationId,
        at: DeclarationId,
        keys: Vec<Value>,
        reader: Reader<'_>,
    ) -> Result<Value> {
        let name = || p.declarations[&table].name.clone();
        if !self.complete {
            return Err(invalid(
                at,
                format!(
                    "table {} is read before its rows are admitted; rows of one relation read one another through reference columns",
                    name()
                ),
            ));
        }
        if keys.len() != self.keys.len() {
            return Err(invalid(at, "table key arity"));
        }
        for (key, value) in self.keys.iter().zip(&keys) {
            if let (Some((lower, upper)), Value::Integer(v)) = (key.range, value)
                && !(lower..=upper).contains(v)
            {
                return Err(invalid(
                    at,
                    format!(
                        "key {} = {v} of table {} is outside its declared range {lower}..{upper}",
                        key.name,
                        name()
                    ),
                ));
            }
        }
        let (keys, _) = self.canonical(keys);
        let row = self.rows.get(&keys);
        if let Some(row) = row {
            reader.read(p, at, row.test_only, || {
                format!(
                    "row {}[{}] {}",
                    name(),
                    display_keys(p, &keys),
                    crate::provenance::supplied_by(p, row.origin)
                )
            })?;
        }
        match &self.absence {
            Absence::Required(claims) => {
                if !self.completes(claims, &keys) {
                    return Err(invalid(
                        at,
                        format!(
                            "lookup {}[{}] is outside the completeness of {}: a required table admits a lookup only over the sets its completeness declares",
                            name(),
                            display_keys(p, &keys),
                            name()
                        ),
                    ));
                }
                row.map(|row| self.value(table, row)).ok_or_else(|| {
                    invalid(table, "an admitted completeness claim lacks one of its rows")
                })
            }
            Absence::Optional => Ok(row.map_or(Value::Missing, |row| self.value(table, row))),
            Absence::Default(value) => {
                Ok(row.map_or_else(|| value.clone(), |row| self.value(table, row)))
            }
        }
    }
    /// The admitted key tuples, canonical, in key order.
    pub(crate) fn keys_read(
        &self,
        p: &CheckedPackage,
        table: DeclarationId,
        at: DeclarationId,
    ) -> Result<impl Iterator<Item = &Vec<Value>>> {
        if !self.complete {
            return Err(invalid(
                at,
                format!(
                    "table {} is read before its rows are admitted",
                    p.declarations[&table].name
                ),
            ));
        }
        Ok(self.rows.keys())
    }
    /// Conservative owned storage.
    pub(crate) fn retained_bytes(&self, ty: impl Fn(&Type) -> usize) -> usize {
        let value = |v: &Value| v.retained_bytes();
        self.keys
            .iter()
            .map(|k| k.name.capacity() + ty(&k.ty) + size_of::<Key>())
            .sum::<usize>()
            + self
                .columns
                .iter()
                .map(|c| c.name.capacity() + ty(&c.ty) + size_of::<Column>())
                .sum::<usize>()
            + self.names.iter().map(|n| n.capacity() + size_of::<String>()).sum::<usize>()
            + self
                .envelopes
                .iter()
                .map(|e| {
                    size_of::<crate::envelope::Envelope>()
                        + e.axis.capacity()
                        + e.lower.capacity()
                        + e.upper.capacity()
                        + ty(&e.ty)
                })
                .sum::<usize>()
            + ty(&self.result)
            + match &self.absence {
                Absence::Required(claims) => claims
                    .iter()
                    .flat_map(|c| &c.domains)
                    .map(|d| match d {
                        Domain::Set(members) => members.iter().map(value).sum::<usize>(),
                        Domain::Range(..) => 16,
                    })
                    .sum::<usize>(),
                Absence::Optional => 0,
                Absence::Default(v) => value(v),
            }
            + self
                .rows
                .iter()
                .map(|(k, r)| {
                    k.capacity() * size_of::<Value>()
                        + k.iter().map(value).sum::<usize>()
                        + r.cells.iter().map(value).sum::<usize>()
                        + size_of::<Row>()
                        + 64
                })
                .sum::<usize>()
    }
}

/// A value as a refusal names it: an entity by name, a keyed row by its kind and keys, a
/// member by name and a scalar as written.
pub(crate) fn display(p: &CheckedPackage, value: &Value) -> String {
    match value {
        Value::Entity { id, kind } => match (p.declarations.get(id), p.entities.get(id)) {
            (Some(row), _) => row.name.clone(),
            (None, Some(record)) => {
                let keys = p
                    .keys(*kind)
                    .map(|(_, keys)| {
                        keys.iter()
                            .filter_map(|(name, _, _)| record.values.get(name))
                            .map(|v| display(p, v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                format!("{}[{keys}]", p.declarations[kind].name)
            }
            (None, None) => format!("{} row {id}", p.declarations[kind].name),
        },
        Value::Enum {
            enumeration,
            member,
        } => p.declarations[enumeration]
            .value
            .enumeration
            .as_ref()
            .and_then(|e| e.members.iter().find(|m| m.member_id == *member))
            .map_or_else(|| member.to_string(), |m| m.name.clone()),
        Value::Boolean(v) => v.to_string(),
        Value::Integer(v) => v.to_string(),
        Value::Number { bits, .. } | Value::Coordinate { bits, .. } => {
            format!("{:?}", f64::from_bits(*bits))
        }
        Value::Text(v) => format!("{v:?}"),
        Value::Identifier { scheme, value } => {
            format!("Id<{}>({value:?})", p.declarations[scheme].name)
        }
        Value::Missing => "missing".into(),
        other => format!("{other:?}"),
    }
}
fn display_keys(p: &CheckedPackage, keys: &[Value]) -> String {
    keys.iter()
        .map(|v| display(p, v))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A cell of a row before phase 3.
#[derive(Clone, Debug)]
enum Pending {
    /// A typed value.
    Value(Value),
    /// A reference to the row of `table` with canonical `keys`, materialized in phase 3.
    Row {
        table: DeclarationId,
        keys: Vec<Value>,
    },
    /// A derived column, evaluated in phase 3.
    Derived,
}
/// A dataset row after phase 1: its canonical keys, origin and orientation.
#[derive(Clone, Debug)]
struct Staged {
    origin: DeclarationId,
    position: usize,
    swapped: bool,
    cells: Vec<Pending>,
}
/// A key or supplied column a uniqueness constraint names.
#[derive(Clone, Copy, Debug)]
enum Slot {
    Key(usize),
    Column(usize),
}
/// A table declaration resolved for admission.
struct Declared {
    id: DeclarationId,
    table: Table,
    /// Derived columns and their expressions, in evaluation order.
    derived: Vec<(usize, dsl::Expr)>,
    /// Requirement clauses as written and parsed.
    requirements: Vec<(String, dsl::Predicate)>,
    /// Uniqueness constraints as written and resolved.
    unique: Vec<(Vec<String>, Vec<Slot>)>,
    /// Completeness entries in key order; `None` for an open key.
    completeness: Vec<Option<ModelingCompleteness>>,
}
impl Declared {
    fn open(&self) -> Vec<usize> {
        self.completeness
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.is_none())
            .map(|(position, _)| position)
            .collect()
    }
    fn name<'p>(&self, p: &'p CheckedPackage) -> &'p str {
        &p.declarations[&self.id].name
    }
}

/// Admit every table: schemas, then the three phases (ADR-0123 Outcome 3).
pub(crate) fn admit(p: &mut CheckedPackage, c: &TypeContext<'_>) -> Result<()> {
    let tables = p
        .declarations
        .values()
        .filter(|r| r.value.table.is_some())
        .map(|r| r.declaration_id)
        .collect::<Vec<_>>();
    let admitted_rows = p
        .entities
        .iter()
        .map(|(id, record)| (*id, record.kind))
        .collect::<BTreeMap<_, _>>();
    let rows = Rows::Admitted(&admitted_rows);
    // Every schema precedes every expression check: a derived column, a requirement or a
    // reference column may name any table.
    let mut declared = BTreeMap::new();
    for id in &tables {
        let table = schema(p, c, *id, rows)?;
        p.tables.insert(*id, table.table.clone());
        declared.insert(*id, table);
    }
    for table in declared.values() {
        expressions(p, c, table)?;
    }
    // Datasets name a table or a keyed kind; only a keyed kind's dataset binds keys.
    let mut datasets = BTreeMap::<DeclarationId, Vec<DeclarationId>>::new();
    for row in p.declarations.values() {
        let Some(dataset) = &row.value.dataset else {
            continue;
        };
        let target = p
            .resolve(row.declaration_id, &dataset.target)
            .ok_or_else(|| invalid(row.declaration_id, "unknown dataset target"))?;
        if declared.contains_key(&target) {
            if !dataset.bindings.is_empty() {
                return Err(invalid(
                    row.declaration_id,
                    "only a dataset of a keyed kind binds keys",
                ));
            }
            datasets.entry(target).or_default().push(row.declaration_id);
        } else if !p.kinds.contains_key(&target) {
            return Err(invalid(
                row.declaration_id,
                "dataset target is neither a table nor a keyed entity kind",
            ));
        }
    }
    // Phase 1: row identities from keys.
    let mut staged = BTreeMap::<DeclarationId, BTreeMap<Vec<Value>, Staged>>::new();
    let mut positions = BTreeMap::<DeclarationId, Vec<Vec<Value>>>::new();
    for (table, sources) in &datasets {
        let rows_of = staged.entry(*table).or_default();
        for dataset in sources {
            positions.insert(
                *dataset,
                identities(p, c, &declared[table], *dataset, rows, rows_of)?,
            );
        }
    }
    // Phase 2: references, completeness, symmetry and uniqueness; none needs an order.
    let mut typed_cells = Vec::new();
    for (table, sources) in &datasets {
        for dataset in sources {
            typed_cells.push((
                *table,
                *dataset,
                values(p, c, &declared, *table, *dataset, rows, &staged)?,
            ));
        }
    }
    for (table, dataset, cells) in typed_cells {
        let rows_of = staged
            .get_mut(&table)
            .ok_or_else(|| invalid(table, "staged rows"))?;
        for (keys, cells) in positions[&dataset].iter().zip(cells) {
            if let Some(row) = rows_of.get_mut(keys) {
                row.cells = cells;
            }
        }
    }
    let empty = BTreeMap::new();
    let mut claims = BTreeMap::new();
    for (id, table) in &declared {
        let rows_of = staged.get(id).unwrap_or(&empty);
        let sources = datasets.get(id).map_or(&[][..], Vec::as_slice);
        claims.insert(*id, completeness(p, table, sources, rows_of)?);
        uniqueness(p, table, rows_of)?;
    }
    // Phase 3: derived columns in value-dependency order; each table is published whole.
    for id in table_order(p, &declared, &staged)? {
        let table = &declared[&id];
        let mut result = table.table.clone();
        if matches!(result.absence, Absence::Required(_)) {
            result.absence = Absence::Required(claims.remove(&id).unwrap_or_default());
        }
        let rows_of = staged.get(&id).unwrap_or(&empty);
        for keys in row_order(p, table, rows_of)? {
            let row = &rows_of[&keys];
            let (cells, test_only) = derive(p, table, &keys, row, &result)?;
            result.rows.insert(
                keys,
                Row {
                    cells: cells.into(),
                    origin: row.origin,
                    test_only,
                },
            );
        }
        result.complete = true;
        p.tables.insert(id, result);
    }
    for table in declared.values() {
        requirements(p, table)?;
        envelopes(p, table.id)?;
    }
    Ok(())
}

/// Every row orders the bounds of each envelope of its table (ADR-0123 Outcome 4).
fn envelopes(p: &CheckedPackage, id: DeclarationId) -> Result<()> {
    let table = &p.tables[&id];
    for envelope in &table.envelopes {
        let position = |name: &str| table.columns.iter().position(|c| c.name == name);
        let (Some(lower), Some(upper)) = (position(&envelope.lower), position(&envelope.upper))
        else {
            return Err(invalid(id, "envelope bounds are columns"));
        };
        for (keys, row) in &table.rows {
            crate::envelope::ordered(envelope, &row.cells[lower], &row.cells[upper], row.origin, || {
                format!(
                    "row {}[{}]",
                    p.declarations[&id].name,
                    display_keys(p, keys)
                )
            })?;
        }
    }
    Ok(())
}

/// Resolve a table declaration: keys, columns, absence policy, symmetry, uniqueness,
/// requirements and the shape of its completeness.
fn schema(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    id: DeclarationId,
    rows: Rows<'_>,
) -> Result<Declared> {
    let declaration = p.declarations[&id]
        .value
        .table
        .as_ref()
        .ok_or_else(|| invalid(id, "table contract"))?;
    let table_name = &p.declarations[&id].name;
    let names = p.named_types(id);
    let variables = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let keys = declaration
        .keys
        .iter()
        .map(|key| {
            if !seen.insert(key.name.as_str()) {
                return Err(invalid(id, "duplicate table key name"));
            }
            let ty = c.resolve(&key.r#type, &variables, &names, id)?;
            let range = key.range.as_ref().map(|r| (r.lower, r.upper));
            if let Some((lower, upper)) = range
                && (ty != Type::Integer || lower > upper)
            {
                return Err(invalid(
                    id,
                    format!("integer-range key {} declares an integer range lower..upper with lower at most upper", key.name),
                ));
            }
            Ok(Key {
                name: key.name.clone(),
                ty,
                range,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut derived = Vec::new();
    let columns = declaration
        .columns
        .iter()
        .enumerate()
        .map(|(position, column)| {
            if !seen.insert(column.name.as_str()) {
                return Err(invalid(
                    id,
                    format!("column {} repeats a key or column name", column.name),
                ));
            }
            let ty = c.resolve(&column.r#type, &variables, &names, id)?;
            // A reference column names a row of a table with columns.
            let mut inner = &ty;
            while let Type::Optional(value) = inner {
                inner = value;
            }
            if let Type::Row(target) = inner
                && p.declarations[target]
                    .value
                    .table
                    .as_ref()
                    .is_some_and(|t| t.columns.is_empty())
            {
                return Err(invalid(
                    id,
                    format!(
                        "column {} references rows of table {}, which has no columns",
                        column.name, p.declarations[target].name
                    ),
                ));
            }
            if let Some(expression) = &column.derived {
                derived.push((
                    position,
                    dsl::parse_expr(expression).map_err(|e| invalid(id, e.to_string()))?,
                ));
            }
            Ok(Column {
                name: column.name.clone(),
                ty,
                derived: column.derived.is_some(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    // ADR-0123 Outcome 4: an envelope is data, bounded by two typed columns of each row.
    let mut axes = BTreeSet::new();
    let envelopes = declaration
        .envelopes
        .iter()
        .map(|e| {
            if !axes.insert(e.name.as_str()) {
                return Err(invalid(
                    id,
                    format!("table {table_name} declares envelope {} twice", e.name),
                ));
            }
            crate::envelope::resolve(
                c,
                id,
                &format!("table {table_name}"),
                &e.name,
                c.resolve(&e.r#type, &variables, &names, id)?,
                &e.lower,
                &e.upper,
                |name| columns.iter().find(|col| col.name == name).map(|col| col.ty.clone()),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let result = match &declaration.value_type {
        Some(value) if columns.is_empty() => c.resolve(value, &variables, &names, id)?,
        None if !columns.is_empty() => Type::Row(id),
        _ => {
            return Err(invalid(
                id,
                "a table declares a value type exactly when it has no columns",
            ));
        }
    };
    let absence = match (
        declaration.missing_policy,
        &declaration.default_value,
        declaration.complete_over.is_empty(),
    ) {
        (Missing::Required, None, false) => Absence::Required(Vec::new()),
        (Missing::Required, None, true) => {
            return Err(invalid(
                id,
                format!(
                    "table {table_name} is required but declares no completeness: a required table declares complete_over, or its absence is missing optional or missing default"
                ),
            ));
        }
        (Missing::Optional, None, true) => Absence::Optional,
        (Missing::Default, Some(cell), true) => {
            if !columns.is_empty() {
                return Err(invalid(id, "a default is one value; a table with columns has none"));
            }
            let value = typed(p, c, id, cell, &result, rows)?;
            if value.uncertainty.is_some() {
                return Err(invalid(id, "a table default carries no uncertainty"));
            }
            Absence::Default(value.value)
        }
        (Missing::Optional | Missing::Default, _, false) => {
            return Err(invalid(
                id,
                "completeness belongs to a required table; an optional or defaulted one declares none",
            ));
        }
        _ => return Err(invalid(id, "the default policy exactly carries its value")),
    };
    let position = |name: &str| keys.iter().position(|k| k.name == name);
    let symmetry = declaration
        .symmetry
        .as_ref()
        .map(|pair| {
            let (Some(first), Some(second)) = (position(&pair.first), position(&pair.second))
            else {
                return Err(invalid(id, "a symmetric pair names two keys of the table"));
            };
            if first == second || keys[first].ty != keys[second].ty || keys[first].range != keys[second].range {
                return Err(invalid(
                    id,
                    "a symmetric pair names two distinct keys of one type",
                ));
            }
            Ok(Symmetry {
                first,
                second,
                diagonal: pair.diagonal,
            })
        })
        .transpose()?;
    let unique = declaration
        .unique
        .iter()
        .map(|constraint| {
            if constraint.names.is_empty() {
                return Err(invalid(id, "a uniqueness constraint names a key or column"));
            }
            let slots = constraint
                .names
                .iter()
                .map(|name| match (position(name), columns.iter().position(|c| c.name == *name)) {
                    (Some(key), _) => Ok(Slot::Key(key)),
                    (None, Some(column)) if !columns[column].derived => Ok(Slot::Column(column)),
                    _ => Err(invalid(
                        id,
                        format!("unique({name}) names no key or supplied column of {table_name}"),
                    )),
                })
                .collect::<Result<Vec<_>>>()?;
            Ok((constraint.names.clone(), slots))
        })
        .collect::<Result<Vec<_>>>()?;
    let requirements = declaration
        .requirements
        .iter()
        .map(|text| {
            dsl::parse_predicate(text)
                .map(|predicate| (text.clone(), predicate))
                .map_err(|e| invalid(id, e.to_string()))
        })
        .collect::<Result<Vec<_>>>()?;
    // ADR-0123 Outcome 3: completeness names every key once, over a set, a range or open.
    let mut completeness = vec![None::<ModelingCompleteness>; keys.len()];
    let mut named = vec![false; keys.len()];
    for entry in &declaration.complete_over {
        let key = position(&entry.key).ok_or_else(|| {
            invalid(id, format!("complete_over names {}, not a key of {table_name}", entry.key))
        })?;
        if std::mem::replace(&mut named[key], true) {
            return Err(invalid(id, format!("complete_over names key {} twice", entry.key)));
        }
        if entry.set.is_some() || entry.range.is_some() {
            completeness[key] = Some(entry.clone());
        }
    }
    if !declaration.complete_over.is_empty() && named.contains(&false) {
        return Err(invalid(
            id,
            format!("complete_over of {table_name} names every key, over a set, a range or open"),
        ));
    }
    let mut table = Declared {
        id,
        table: Table {
            names: columns.iter().map(|c| c.name.clone()).collect(),
            keys,
            columns,
            result,
            absence,
            symmetry,
            rows: BTreeMap::new(),
            envelopes,
            complete: false,
        },
        derived,
        requirements,
        unique,
        completeness,
    };
    table.derived = derived_order(p, &table)?;
    Ok(table)
}

/// Derived columns ordered by the columns of their own row that they read.
fn derived_order(p: &CheckedPackage, table: &Declared) -> Result<Vec<(usize, dsl::Expr)>> {
    let mut graph = DiGraph::<usize, ()>::new();
    let nodes = table
        .derived
        .iter()
        .map(|(position, _)| (*position, graph.add_node(*position)))
        .collect::<BTreeMap<_, _>>();
    for (position, expression) in &table.derived {
        for name in crate::expression::references(expression) {
            if let Some(read) = table
                .table
                .columns
                .iter()
                .position(|c| c.derived && c.name == name)
            {
                graph.add_edge(nodes[&read], nodes[position], ());
            }
        }
    }
    let order = toposort(&graph, None).map_err(|_| {
        let mut names = cycle(&graph)
            .into_iter()
            .map(|n| table.table.columns[graph[n]].name.clone())
            .collect::<Vec<_>>();
        names.sort();
        invalid(
            table.id,
            format!(
                "derived columns {} of {} derive from one another",
                names.join(", "),
                table.name(p)
            ),
        )
    })?;
    let by_position = table.derived.iter().cloned().collect::<BTreeMap<_, _>>();
    Ok(order
        .into_iter()
        .map(|n| (graph[n], by_position[&graph[n]].clone()))
        .collect())
}

/// The nodes of one strongly connected cycle, sorted; `kosaraju_scc` names them.
fn cycle<N>(graph: &DiGraph<N, ()>) -> Vec<NodeIndex> {
    let mut nodes = kosaraju_scc(graph)
        .into_iter()
        .find(|c| c.len() > 1 || c.first().is_some_and(|n| graph.contains_edge(*n, *n)))
        .unwrap_or_default();
    nodes.sort();
    nodes
}

/// The names and types a row binds: keys, columns, and `value` for a table of one value.
fn row_types(table: &Declared) -> BTreeMap<String, Type> {
    let mut env = table
        .table
        .keys
        .iter()
        .map(|k| (k.name.clone(), k.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    env.extend(table.table.columns.iter().map(|c| (c.name.clone(), c.ty.clone())));
    if table.table.columns.is_empty() {
        env.insert("value".into(), table.table.result.clone());
    }
    env
}

/// A contract failure prefixed with the table part it concerns.
fn context(error: crate::ModelingError, part: &str) -> crate::ModelingError {
    match error {
        crate::ModelingError::Contract {
            declaration,
            message,
        } => crate::ModelingError::Contract {
            declaration,
            message: format!("{part}: {message}"),
        },
        other => other,
    }
}

/// Type-check every derived column and requirement before any row is evaluated, so a
/// table without rows is checked too.
fn expressions(p: &CheckedPackage, c: &TypeContext<'_>, table: &Declared) -> Result<()> {
    let env = row_types(table);
    for (position, expression) in &table.derived {
        let column = &table.table.columns[*position];
        let actual = crate::expression::infer(expression, &env, p, c, table.id, Some(&column.ty))
            .map_err(|e| {
                context(e, &format!("derived column {} of {}", column.name, table.name(p)))
            })?;
        if !p.subsumes(&column.ty, &actual) {
            return Err(invalid(
                table.id,
                format!(
                    "derived column {} of {} is {actual:?}, not its declared {:?}",
                    column.name,
                    table.name(p),
                    column.ty
                ),
            ));
        }
    }
    for (clause, predicate) in &table.requirements {
        crate::expression::predicate(predicate, &env, p, c, table.id)
            .map_err(|e| context(e, &format!("requirement `{clause}` of {}", table.name(p))))?;
    }
    Ok(())
}

/// Phase 1: every row of `dataset` keyed by its canonical key tuple. An integer-range key
/// lies in its range, a symmetric pair is written in one orientation, and a table
/// excluding its diagonal has no diagonal row.
fn identities(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    table: &Declared,
    dataset: DeclarationId,
    rows: Rows<'_>,
    staged: &mut BTreeMap<Vec<Value>, Staged>,
) -> Result<Vec<Vec<Value>>> {
    let data = p.declarations[&dataset]
        .value
        .dataset
        .as_ref()
        .ok_or_else(|| invalid(dataset, "dataset"))?;
    let name = table.name(p);
    let mut canonical = Vec::with_capacity(data.rows.len());
    for (position, entry) in data.rows.iter().enumerate() {
        if entry.keys.len() != table.table.keys.len() {
            return Err(invalid(dataset, "dataset key arity"));
        }
        let keys = entry
            .keys
            .iter()
            .zip(&table.table.keys)
            .map(|(cell, key)| {
                let value = scalar(p, c, dataset, cell, &key.ty, rows)?;
                if let (Some((lower, upper)), Value::Integer(v)) = (key.range, &value)
                    && !(lower..=upper).contains(v)
                {
                    return Err(invalid(
                        dataset,
                        format!(
                            "row {position} of dataset {}: key {} = {v} of {name} is outside its declared range {lower}..{upper}",
                            p.declarations[&dataset].name, key.name
                        ),
                    ));
                }
                Ok(value)
            })
            .collect::<Result<Vec<_>>>()?;
        let (keys, swapped) = table.table.canonical(keys);
        if table.table.excluded_diagonal(&keys) {
            return Err(invalid(
                dataset,
                format!(
                    "row {name}[{}] of dataset {} lies on the diagonal, which {name} excludes",
                    display_keys(p, &keys),
                    p.declarations[&dataset].name
                ),
            ));
        }
        if let Some(previous) = staged.get(&keys) {
            let both = |row: &Staged| {
                format!(
                    "row {} of dataset {}",
                    row.position, p.declarations[&row.origin].name
                )
            };
            let here = Staged {
                origin: dataset,
                position,
                swapped,
                cells: Vec::new(),
            };
            return Err(invalid(
                dataset,
                if previous.swapped == swapped {
                    format!(
                        "{name}[{}] is supplied twice: {} and {}",
                        display_keys(p, &keys),
                        both(previous),
                        both(&here)
                    )
                } else {
                    format!(
                        "symmetric pair {name}[{}] is written in both orientations, by {} and {}; a symmetric pair is written once and answers both orders",
                        display_keys(p, &keys),
                        both(previous),
                        both(&here)
                    )
                },
            ));
        }
        staged.insert(
            keys.clone(),
            Staged {
                origin: dataset,
                position,
                swapped,
                cells: Vec::new(),
            },
        );
        canonical.push(keys);
    }
    Ok(canonical)
}

/// A typed cell that carries no uncertainty.
fn scalar(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
    cell: &Cell,
    ty: &Type,
    rows: Rows<'_>,
) -> Result<Value> {
    let typed = typed(p, c, at, cell, ty, rows)?;
    if typed.uncertainty.is_some() {
        return Err(invalid(at, "a table row carries no uncertainty"));
    }
    Ok(typed.value)
}

/// Phase 2: the value cells of `dataset`'s rows. An entity reference names an admitted
/// entity or keyed row; a row reference `table[keys]` names an admitted row of that table.
fn values(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    declared: &BTreeMap<DeclarationId, Declared>,
    table: DeclarationId,
    dataset: DeclarationId,
    rows: Rows<'_>,
    staged: &BTreeMap<DeclarationId, BTreeMap<Vec<Value>, Staged>>,
) -> Result<Vec<Vec<Pending>>> {
    let data = p.declarations[&dataset]
        .value
        .dataset
        .as_ref()
        .ok_or_else(|| invalid(dataset, "dataset"))?;
    let schema = &declared[&table].table;
    let supplied = schema.columns.iter().filter(|c| !c.derived).count();
    let mut output = Vec::with_capacity(data.rows.len());
    for (position, entry) in data.rows.iter().enumerate() {
        let cells = if schema.columns.is_empty() {
            if entry.values.len() != 1 {
                return Err(invalid(dataset, "scalar table row arity"));
            }
            vec![Pending::Value(scalar(p, c, dataset, &entry.values[0], &schema.result, rows)?)]
        } else {
            if entry.values.len() != supplied {
                return Err(invalid(
                    dataset,
                    format!(
                        "row {position} of dataset {}: {} value cells where {} supplies {supplied} columns",
                        p.declarations[&dataset].name,
                        entry.values.len(),
                        p.declarations[&table].name
                    ),
                ));
            }
            let mut cells = entry.values.iter();
            schema
                .columns
                .iter()
                .map(|column| {
                    if column.derived {
                        return Ok(Pending::Derived);
                    }
                    let cell = cells.next().ok_or_else(|| invalid(dataset, "row arity"))?;
                    reference(p, c, declared, dataset, cell, &column.ty, rows, staged)
                })
                .collect::<Result<Vec<_>>>()?
        };
        output.push(cells);
    }
    Ok(output)
}

/// A value cell of a table row: a row reference to a table with columns, or any cell.
#[allow(clippy::too_many_arguments, reason = "the admission state of one phase")]
fn reference(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    declared: &BTreeMap<DeclarationId, Declared>,
    at: DeclarationId,
    cell: &Cell,
    ty: &Type,
    rows: Rows<'_>,
    staged: &BTreeMap<DeclarationId, BTreeMap<Vec<Value>, Staged>>,
) -> Result<Pending> {
    let mut target = ty;
    while let Type::Optional(inner) = target {
        target = inner;
    }
    let selected = cell.value.selected().map_err(|e| invalid(at, e.to_string()))?;
    let (Type::Row(table), CellSelected::Row(row)) = (target, selected) else {
        return Ok(Pending::Value(scalar(p, c, at, cell, ty, rows)?));
    };
    if cell.uncertainty.is_some() {
        return Err(invalid(at, "a table row carries no uncertainty"));
    }
    let path = row.target.join(".");
    if p.resolve(at, &path) != Some(*table) {
        return Err(invalid(
            at,
            format!(
                "{path}[...] is not a row of table {}",
                p.declarations[table].name
            ),
        ));
    }
    let schema = &declared[table].table;
    if row.keys.len() != schema.keys.len() {
        return Err(invalid(
            at,
            format!("a {path} row is referenced by its {} keys", schema.keys.len()),
        ));
    }
    let values = row
        .keys
        .iter()
        .zip(&schema.keys)
        .map(|(key, declared)| {
            let cell = pse_authoring::language::key_cell_value(key)
                .map_err(|e| invalid(at, e.to_string()))?;
            scalar(p, c, at, &cell, &declared.ty, rows)
        })
        .collect::<Result<Vec<_>>>()?;
    let (values, _) = schema.canonical(values);
    if !staged
        .get(table)
        .is_some_and(|rows_of| rows_of.contains_key(&values))
    {
        return Err(invalid(
            at,
            format!(
                "row reference {path}[{}] names no admitted row of {path}",
                display_keys(p, &values)
            ),
        ));
    }
    Ok(Pending::Row {
        table: *table,
        keys: values,
    })
}

/// The domain of one completeness entry: a declared set, an enumeration or a range.
fn domain(
    p: &CheckedPackage,
    at: DeclarationId,
    entry: &ModelingCompleteness,
    key: &Key,
) -> Result<Domain> {
    match (&entry.set, &entry.range) {
        (None, Some(range)) => {
            if key.ty != Type::Integer
                || key
                    .range
                    .is_some_and(|(lower, upper)| range.lower < lower || range.upper > upper)
                || range.lower > range.upper
            {
                return Err(invalid(
                    at,
                    format!(
                        "completeness of key {} over {}..{} lies outside the key's integers",
                        key.name, range.lower, range.upper
                    ),
                ));
            }
            Ok(Domain::Range(range.lower, range.upper))
        }
        (Some(path), None) => {
            let name = path.join(".");
            let id = p
                .resolve(at, &name)
                .ok_or_else(|| invalid(at, format!("completeness names unknown set {name}")))?;
            let members = if let Some(enumeration) = &p.declarations[&id].value.enumeration {
                enumeration
                    .members
                    .iter()
                    .map(|m| Value::Enum {
                        enumeration: id,
                        member: m.member_id,
                    })
                    .collect::<Vec<_>>()
            } else if matches!(p.types.get(&id), Some(Type::Set(_))) {
                let env = Environment::new();
                let physical = p.context();
                let mut evaluator = Evaluator {
                    package: p,
                    physical: &physical,
                    at,
                    env: &env,
                    limit: CLAIM_LIMIT,
                    stack: Vec::new(),
                    reader: Reader::Admission(None),
                };
                let Value::Set(members) = evaluator.text(&name, None)? else {
                    return Err(invalid(at, format!("{name} is not a finite set")));
                };
                members
            } else {
                return Err(invalid(
                    at,
                    format!("completeness names {name}, which is neither a declared set nor an enumeration"),
                ));
            };
            if let Some(member) = members.iter().find(|m| !conforms(m, &key.ty, p)) {
                return Err(invalid(
                    at,
                    format!(
                        "member {} of {name} is not a key {} of type {:?}",
                        display(p, member),
                        key.name,
                        key.ty
                    ),
                ));
            }
            Ok(Domain::Set(members.into_iter().collect()))
        }
        _ => Err(invalid(at, "a completeness entry is over a set or a range")),
    }
}

/// The completeness claims of a required table, each verified at admission: the table's
/// own claim when it closes every key, else one claim per dataset that claims the open
/// keys, verified against that dataset's rows.
fn completeness(
    p: &CheckedPackage,
    table: &Declared,
    datasets: &[DeclarationId],
    rows: &BTreeMap<Vec<Value>, Staged>,
) -> Result<Vec<Claim>> {
    let name = table.name(p);
    let open = table.open();
    let required = matches!(table.table.absence, Absence::Required(_));
    let mut claims = Vec::new();
    let closed = table
        .completeness
        .iter()
        .zip(&table.table.keys)
        .map(|(entry, key)| entry.as_ref().map(|e| domain(p, table.id, e, key)).transpose())
        .collect::<Result<Vec<_>>>()?;
    if required && open.is_empty() {
        let domains = closed.into_iter().flatten().collect::<Vec<_>>();
        claims.push(Claim {
            origin: table.id,
            domains,
        });
        for dataset in datasets {
            if !p.declarations[dataset]
                .value
                .dataset
                .as_ref()
                .is_some_and(|d| d.complete_over.is_empty())
            {
                return Err(invalid(
                    *dataset,
                    format!("{name} declares its completeness itself; a dataset claims only open keys"),
                ));
            }
        }
    } else {
        for dataset in datasets {
            let data = p.declarations[dataset]
                .value
                .dataset
                .as_ref()
                .ok_or_else(|| invalid(*dataset, "dataset"))?;
            if data.complete_over.is_empty() {
                continue;
            }
            if !required {
                return Err(invalid(
                    *dataset,
                    format!("{name} is not required; its datasets claim no completeness"),
                ));
            }
            let mut domains = closed.clone();
            for entry in &data.complete_over {
                let key = table
                    .table
                    .keys
                    .iter()
                    .position(|k| k.name == entry.key)
                    .filter(|key| open.contains(key))
                    .ok_or_else(|| {
                        invalid(
                            *dataset,
                            format!("complete_over names {}, which is not an open key of {name}", entry.key),
                        )
                    })?;
                if domains[key].is_some() {
                    return Err(invalid(*dataset, format!("complete_over names key {} twice", entry.key)));
                }
                domains[key] = Some(domain(p, *dataset, entry, &table.table.keys[key])?);
            }
            let domains = domains.into_iter().collect::<Option<Vec<_>>>().ok_or_else(|| {
                invalid(
                    *dataset,
                    format!("complete_over of dataset {} names every open key of {name}", p.declarations[dataset].name),
                )
            })?;
            claims.push(Claim {
                origin: *dataset,
                domains,
            });
        }
    }
    for claim in &claims {
        verify(p, table, claim, rows)?;
    }
    Ok(claims)
}

/// Every key tuple of a claim has a row: any row for the table's own claim, a row of the
/// claiming dataset for a dataset's.
fn verify(
    p: &CheckedPackage,
    table: &Declared,
    claim: &Claim,
    rows: &BTreeMap<Vec<Value>, Staged>,
) -> Result<()> {
    let size = claim
        .domains
        .iter()
        .try_fold(1_usize, |n, d| n.checked_mul(d.size()))
        .filter(|n| *n <= CLAIM_LIMIT)
        .ok_or_else(|| {
            invalid(
                claim.origin,
                format!("the completeness of {} exceeds {CLAIM_LIMIT} key tuples", table.name(p)),
            )
        })?;
    if size == 0 {
        return Ok(());
    }
    let members = claim.domains.iter().map(Domain::members).collect::<Vec<_>>();
    let mut odometer = vec![0_usize; members.len()];
    loop {
        let keys = odometer
            .iter()
            .zip(&members)
            .map(|(i, m)| m[*i].clone())
            .collect::<Vec<_>>();
        let (keys, _) = table.table.canonical(keys);
        let present = rows
            .get(&keys)
            .is_some_and(|row| claim.origin == table.id || row.origin == claim.origin);
        if !table.table.excluded_diagonal(&keys) && !present {
            let supplier = if claim.origin == table.id {
                String::new()
            } else {
                format!(" of dataset {}", p.declarations[&claim.origin].name)
            };
            return Err(invalid(
                claim.origin,
                format!(
                    "{} is declared complete over its sets by {}, but no row{supplier} has the keys [{}]",
                    table.name(p),
                    p.declarations[&claim.origin].name,
                    display_keys(p, &keys)
                ),
            ));
        }
        let mut digit = odometer.len();
        loop {
            if digit == 0 {
                return Ok(());
            }
            digit -= 1;
            odometer[digit] += 1;
            if odometer[digit] < members[digit].len() {
                break;
            }
            odometer[digit] = 0;
        }
    }
}

/// A uniqueness constraint holds: no two rows share the named keys and supplied columns;
/// an absent value never collides.
fn uniqueness(
    p: &CheckedPackage,
    table: &Declared,
    rows: &BTreeMap<Vec<Value>, Staged>,
) -> Result<()> {
    for (names, slots) in &table.unique {
        let mut seen = BTreeMap::<Vec<Value>, &Vec<Value>>::new();
        for (keys, row) in rows {
            let tuple = slots
                .iter()
                .map(|slot| match slot {
                    Slot::Key(k) => keys[*k].clone(),
                    Slot::Column(c) => match &row.cells[*c] {
                        Pending::Value(v) => v.clone(),
                        Pending::Row { keys, .. } => Value::Tuple(keys.clone()),
                        Pending::Derived => Value::Missing,
                    },
                })
                .collect::<Vec<_>>();
            if tuple.contains(&Value::Missing) {
                continue;
            }
            if let Some(previous) = seen.insert(tuple.clone(), keys) {
                return Err(invalid(
                    row.origin,
                    format!(
                        "rows {name}[{}] and {name}[{}] share ({}) = ({}), which unique({}) declares unique",
                        display_keys(p, previous),
                        display_keys(p, keys),
                        names.join(", "),
                        display_keys(p, &tuple),
                        names.join(", "),
                        name = table.name(p)
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// The tables a declaration's text reads, directly or through the functions and static
/// bindings it names.
fn tables_read(p: &CheckedPackage, owner: DeclarationId, text: &str) -> BTreeSet<DeclarationId> {
    let mut read = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut pending = vec![(owner, text.to_owned())];
    while let Some((owner, text)) = pending.pop() {
        for id in crate::check::dependency_paths(p, owner, &text) {
            if !visited.insert(id) {
                continue;
            }
            if p.tables.contains_key(&id) {
                read.insert(id);
            }
            let row = &p.declarations[&id];
            if let Some(function) = &row.value.function {
                pending.extend(
                    function
                        .body
                        .iter()
                        .chain(&function.validity)
                        .map(|t| (id, t.clone())),
                );
            }
            if let Some(binding) = &row.value.binding {
                pending.extend(binding.expression.iter().map(|t| (id, t.clone())));
            }
        }
    }
    read
}

/// Tables in the order of their value dependencies: a table follows every other table
/// its reference columns name or its derived columns read. A cycle is refused with its
/// tables named.
fn table_order(
    p: &CheckedPackage,
    declared: &BTreeMap<DeclarationId, Declared>,
    staged: &BTreeMap<DeclarationId, BTreeMap<Vec<Value>, Staged>>,
) -> Result<Vec<DeclarationId>> {
    let mut graph = DiGraph::<DeclarationId, ()>::new();
    let nodes = declared
        .keys()
        .map(|id| (*id, graph.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    for id in declared.keys() {
        let mut reads = BTreeSet::new();
        for rows in staged.get(id).into_iter().flat_map(BTreeMap::values) {
            for cell in &rows.cells {
                if let Pending::Row { table, .. } = cell {
                    reads.insert(*table);
                }
            }
        }
        if let Some(declaration) = &p.declarations[id].value.table {
            for column in declaration.columns.iter().filter_map(|c| c.derived.as_ref()) {
                reads.extend(tables_read(p, *id, column));
            }
        }
        reads.remove(id);
        for read in reads.iter().filter_map(|r| nodes.get(r)) {
            graph.add_edge(*read, nodes[id], ());
        }
    }
    let order = toposort(&graph, None).map_err(|_| {
        let mut names = cycle(&graph)
            .into_iter()
            .map(|n| p.declarations[&graph[n]].name.clone())
            .collect::<Vec<_>>();
        names.sort();
        invalid(
            graph[cycle(&graph)[0]],
            format!("tables {} derive their values from one another", names.join(", ")),
        )
    })?;
    Ok(order.into_iter().map(|n| graph[n]).collect())
}

/// A table's rows in the order of their references to one another: a row follows every
/// row of its own table it references. A self-referential relation admits when this is
/// acyclic; a cycle is refused with its rows named.
fn row_order(
    p: &CheckedPackage,
    table: &Declared,
    rows: &BTreeMap<Vec<Value>, Staged>,
) -> Result<Vec<Vec<Value>>> {
    let mut graph = DiGraph::<&Vec<Value>, ()>::new();
    let nodes = rows
        .keys()
        .map(|keys| (keys, graph.add_node(keys)))
        .collect::<BTreeMap<_, _>>();
    for (keys, row) in rows {
        for cell in &row.cells {
            if let Pending::Row { table: target, keys: to } = cell
                && *target == table.id
                && let Some(from) = nodes.get(to)
            {
                graph.add_edge(*from, nodes[keys], ());
            }
        }
    }
    let order = toposort(&graph, None).map_err(|_| {
        let mut names = cycle(&graph)
            .into_iter()
            .map(|n| format!("{}[{}]", table.name(p), display_keys(p, graph[n])))
            .collect::<Vec<_>>();
        names.sort();
        invalid(
            rows[graph[cycle(&graph)[0]]].origin,
            format!("rows reference one another in a cycle: {}", names.join(", ")),
        )
    })?;
    Ok(order.into_iter().map(|n| graph[n].clone()).collect())
}

/// Phase 3 for one row: reference columns take the referenced row, then each derived
/// column is evaluated once, with the row's keys and columns bound. The row is test-only
/// when its origin role is, or it references or derives from test-only data.
fn derive(
    p: &CheckedPackage,
    table: &Declared,
    keys: &[Value],
    row: &Staged,
    partial: &Table,
) -> Result<(Vec<Value>, bool)> {
    let tainted = std::cell::Cell::new(
        p.supplies_test_only(row.origin) || keys.iter().any(|key| p.references_test_only(key)),
    );
    let mut cells = row
        .cells
        .iter()
        .map(|cell| match cell {
            Pending::Value(value) => Ok(value.clone()),
            Pending::Row { table: target, keys } => {
                let source = if *target == table.id {
                    partial
                } else {
                    p.tables
                        .get(target)
                        .filter(|t| t.complete)
                        .ok_or_else(|| invalid(table.id, "referenced table is not admitted"))?
                };
                let referenced = source
                    .rows
                    .get(keys)
                    .ok_or_else(|| invalid(table.id, "referenced row is not admitted"))?;
                if referenced.test_only {
                    tainted.set(true);
                }
                Ok(source.value(*target, referenced))
            }
            Pending::Derived => Ok(Value::Missing),
        })
        .collect::<Result<Vec<_>>>()?;
    if cells.iter().any(|cell| p.references_test_only(cell)) {
        tainted.set(true);
    }
    if table.derived.is_empty() {
        return Ok((cells, tainted.get()));
    }
    let mut env = Environment::new();
    for (key, value) in table.table.keys.iter().zip(keys) {
        env.insert(key.name.clone(), value.clone());
    }
    for (column, value) in table.table.columns.iter().zip(&cells) {
        if !column.derived {
            env.insert(column.name.clone(), value.clone());
        }
    }
    let physical = p.context();
    for (position, expression) in &table.derived {
        let column = &table.table.columns[*position];
        let value = Evaluator {
            package: p,
            physical: &physical,
            at: table.id,
            env: &env,
            limit: EVALUATION_LIMIT,
            stack: Vec::new(),
            reader: Reader::Admission(Some(&tainted)),
        }
        .expr(expression, Some(&column.ty), 0)?;
        if !conforms(&value, &column.ty, p) {
            return Err(invalid(
                row.origin,
                format!(
                    "derived column {} of {}[{}] is not a {:?}",
                    column.name,
                    table.name(p),
                    display_keys(p, keys),
                    column.ty
                ),
            ));
        }
        env.insert(column.name.clone(), value.clone());
        cells[*position] = value;
    }
    Ok((cells, tainted.get()))
}

/// Every row satisfies every requirement of its table; a refusal names the row, its
/// dataset and the clause.
fn requirements(p: &CheckedPackage, table: &Declared) -> Result<()> {
    if table.requirements.is_empty() {
        return Ok(());
    }
    let admitted = &p.tables[&table.id];
    let physical = p.context();
    for (keys, row) in &admitted.rows {
        let mut env = Environment::new();
        for (key, value) in admitted.keys.iter().zip(keys) {
            env.insert(key.name.clone(), value.clone());
        }
        for (column, value) in admitted.columns.iter().zip(row.cells.iter()) {
            env.insert(column.name.clone(), value.clone());
        }
        if admitted.columns.is_empty() {
            env.insert("value".into(), admitted.value(table.id, row));
        }
        for (clause, predicate) in &table.requirements {
            let holds = Evaluator {
                package: p,
                physical: &physical,
                at: table.id,
                env: &env,
                limit: EVALUATION_LIMIT,
                stack: Vec::new(),
                reader: Reader::Admission(None),
            }
            .predicate(predicate)?;
            if !holds {
                return Err(invalid(
                    row.origin,
                    format!(
                        "row {}[{}] of dataset {} violates the requirement `{clause}`",
                        table.name(p),
                        display_keys(p, keys),
                        p.declarations[&row.origin].name
                    ),
                ));
            }
        }
    }
    Ok(())
}
