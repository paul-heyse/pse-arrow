// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Package data documents (ADR-0125).
//!
//! A data document reaches the kernel as a [`RowSet`]: named columns of equal length,
//! decoded by Arrow type by its consumer (`pse-runtime`), so this crate stays free of Arrow
//! and Parquet. A dataset declaration names its document by path; admission types the
//! document's columns once each, through the [`DocumentPlan`] of the dataset's table:
//!
//! - a magnitude is in the column's declared storage unit and converts to the column's
//!   canonical unit with one conversion per column; a unit the document states is refused
//!   unless it is the declared storage unit, so the declaration stays the only authority;
//! - an identity names an admitted entity, and an identifier-scheme value resolves through
//!   the identifier index to the entity holding it, never through a name;
//! - text, integers and Booleans are themselves, and a dictionary names enumeration
//!   members;
//! - column names equal the declared names of the keys and supplied columns.
//!
//! No cell is parsed or evaluated. The typed rows ([`DocumentTable`]) then enter the three
//! admission phases of the table like inline rows. [`Documents`] supplies the documents and
//! is the one owner of reuse: the compiler workspace answers [`Documents::admit`] through a
//! tracked query over the plan and the document input.
use crate::specialize::value::Value;
use crate::{DeclarationId, Result, invalid};
use pse_ids::{ContentHash, SemanticId};
use std::collections::BTreeMap;
use std::sync::Arc;

/// The values of one decoded column, by Arrow type; `None` is a null.
#[derive(Clone, Debug, PartialEq)]
pub enum Values {
    /// `Float64`: magnitudes in the column's declared storage unit.
    Magnitude(Vec<Option<f64>>),
    /// `FixedSizeBinary(16)`: declaration identities.
    Identity(Vec<Option<SemanticId>>),
    /// `Utf8`: identifier-scheme values or text.
    Text(Vec<Option<String>>),
    /// `Int64`.
    Integer(Vec<Option<i64>>),
    /// `Boolean`.
    Boolean(Vec<Option<bool>>),
    /// A dictionary: each row names an enumeration member by an index into `members`.
    Member {
        /// The dictionary's member names.
        members: Vec<String>,
        /// One dictionary index per row.
        keys: Vec<Option<u32>>,
    },
}
impl Values {
    /// Rows in the column.
    pub fn len(&self) -> usize {
        match self {
            Self::Magnitude(v) => v.len(),
            Self::Identity(v) => v.len(),
            Self::Text(v) => v.len(),
            Self::Integer(v) => v.len(),
            Self::Boolean(v) => v.len(),
            Self::Member { keys, .. } => keys.len(),
        }
    }
    /// Whether the column has no rows.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// The Arrow type the column was decoded from, as refusals name it.
    pub const fn arrow_type(&self) -> &'static str {
        match self {
            Self::Magnitude(_) => "Float64",
            Self::Identity(_) => "FixedSizeBinary(16)",
            Self::Text(_) => "Utf8",
            Self::Integer(_) => "Int64",
            Self::Boolean(_) => "Boolean",
            Self::Member { .. } => "Dictionary",
        }
    }
    fn retained_bytes(&self) -> usize {
        match self {
            Self::Magnitude(v) => v.capacity() * size_of::<Option<f64>>(),
            Self::Identity(v) => v.capacity() * size_of::<Option<SemanticId>>(),
            Self::Text(v) => {
                v.capacity() * size_of::<Option<String>>()
                    + v.iter().flatten().map(String::capacity).sum::<usize>()
            }
            Self::Integer(v) => v.capacity() * size_of::<Option<i64>>(),
            Self::Boolean(v) => v.capacity() * size_of::<Option<bool>>(),
            Self::Member { members, keys } => {
                keys.capacity() * size_of::<Option<u32>>()
                    + members
                        .iter()
                        .map(|m| m.capacity() + size_of::<String>())
                        .sum::<usize>()
            }
        }
    }
}

/// One named column of a data document.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentColumn {
    /// The column name, which must equal a declared key or column name.
    pub name: String,
    /// A unit the document states for the column, which is never an authority: it must be
    /// the declared storage unit.
    pub unit: Option<String>,
    /// The decoded values.
    pub values: Values,
}

/// A data document decoded by Arrow type: named, typed columns of equal length, in row
/// order (ADR-0125).
#[derive(Clone, Debug, PartialEq)]
pub struct RowSet {
    rows: usize,
    columns: Vec<DocumentColumn>,
}
impl RowSet {
    /// A row set over `columns`.
    ///
    /// # Errors
    /// Columns of different lengths, or a column name repeated.
    pub fn new(columns: Vec<DocumentColumn>) -> std::result::Result<Self, String> {
        let rows = columns.first().map_or(0, |c| c.values.len());
        let mut names = std::collections::BTreeSet::new();
        for column in &columns {
            if column.values.len() != rows {
                return Err(format!(
                    "column {} has {} rows where the document has {rows}",
                    column.name,
                    column.values.len()
                ));
            }
            if !names.insert(column.name.as_str()) {
                return Err(format!("column {} is named twice", column.name));
            }
        }
        Ok(Self { rows, columns })
    }
    /// Rows in every column.
    pub const fn rows(&self) -> usize {
        self.rows
    }
    /// The columns in document order.
    pub fn columns(&self) -> &[DocumentColumn] {
        &self.columns
    }
    /// Conservative owned storage.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .columns
                .iter()
                .map(|c| {
                    size_of::<DocumentColumn>()
                        + c.name.capacity()
                        + c.unit.as_ref().map_or(0, String::capacity)
                        + c.values.retained_bytes()
                })
                .sum::<usize>()
    }
}

/// One package data document: its identity, the hash of its exact bytes and its decoded
/// rows.
#[derive(Clone, Debug, PartialEq)]
pub struct DataDocument {
    /// `named_id(package_id, path)`, as every package document.
    pub id: SemanticId,
    /// The path within its package.
    pub path: String,
    /// The byte-level content hash, which enters the package checksum and the source
    /// revision (ADR-0123 Outcome 8).
    pub content_hash: ContentHash,
    /// The decoded rows.
    pub rows: RowSet,
}
impl DataDocument {
    /// Conservative owned storage.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>() + self.path.capacity() + self.rows.retained_bytes()
    }
}

/// The data documents of a package closure, and the package each source document belongs
/// to, which a dataset's document path resolves in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocumentInventory {
    /// The package identity of each source document declaring modeling rows.
    pub packages: BTreeMap<SemanticId, SemanticId>,
    /// Parser-derived declaration payload field locations, keyed by full structural field path.
    pub field_spans: BTreeMap<DeclarationId, BTreeMap<String, pse_authoring::SourceSpan>>,
    /// The data documents by identity.
    pub documents: BTreeMap<SemanticId, Arc<DataDocument>>,
}
impl DocumentInventory {
    /// The data document a declaration of source document `source` names by `path`: the
    /// document `named_id(package, path)` of the declaring package.
    pub fn resolve(&self, source: SemanticId, path: &str) -> Option<SemanticId> {
        let package = self.packages.get(&source)?;
        let id = pse_ids::named_id(*package, path);
        self.documents.contains_key(&id).then_some(id)
    }
    /// Conservative owned storage; documents are counted once each.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.packages.len() * (2 * size_of::<SemanticId>() + 32)
            + self
                .field_spans
                .values()
                .map(|fields| {
                    size_of::<DeclarationId>()
                        + size_of::<BTreeMap<String, pse_authoring::SourceSpan>>()
                        + 32
                        + fields
                            .keys()
                            .map(|path| {
                                size_of::<String>()
                                    + path.capacity()
                                    + size_of::<pse_authoring::SourceSpan>()
                                    + 32
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self
                .documents
                .values()
                .map(|d| d.retained_bytes() + size_of::<SemanticId>() + 32)
                .sum::<usize>()
    }
}

/// What one typed column of a document becomes (ADR-0125).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// A quantity: magnitudes in the declared storage unit, converted once per column.
    Quantity(pse_quantity::CanonicalConversionPlan),
    /// An entity of `kind` or a refinement, by identity or by a value of `scheme`.
    Entity {
        /// Identity projection forms keys first; full admission validates the reference.
        deferred: bool,
        /// The declared kind.
        kind: DeclarationId,
        /// Each admitted entity of the kind, or of a refinement, and its concrete kind.
        entities: Arc<BTreeMap<DeclarationId, DeclarationId>>,
        /// The identifier scheme a document names the entity by, and the entity holding
        /// each of its values.
        scheme: Option<(DeclarationId, Arc<BTreeMap<String, DeclarationId>>)>,
    },
    /// An opaque value of an identifier scheme.
    Identifier(DeclarationId),
    /// Text.
    Text,
    /// An exact integer.
    Integer,
    /// A Boolean.
    Boolean,
    /// A member of an enumeration, by name.
    Enum {
        /// The enumeration.
        enumeration: DeclarationId,
        /// Each member's identity by name.
        members: BTreeMap<String, SemanticId>,
    },
}

/// One declared key or supplied column a document must carry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Slot {
    /// The declared name, which the document's column must equal.
    pub name: String,
    /// The declared type as refusals name it.
    pub declared: String,
    /// Whether a null is admitted, as explicit absence.
    pub optional: bool,
    /// The declaration-owned value when a column is absent; null remains explicit absence.
    pub default: Option<Value>,
    /// What the column's values become.
    pub target: Target,
}

/// The admission plan of one document-supplied dataset: the typed slots of its table,
/// resolved from the declarations. Two equal plans over one document admit the same rows.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DocumentPlan {
    /// Identity admission projects keys first; full admission validates every column.
    pub keys_only: bool,
    /// The dataset declaration.
    pub dataset: DeclarationId,
    /// The dataset and table names as refusals name them.
    pub label: String,
    /// The keys, in key order.
    pub keys: Vec<Slot>,
    /// The supplied columns in column order, or the one `value` of a scalar table.
    pub values: Vec<Slot>,
}
impl DocumentPlan {
    /// Upper bound for the typed columns and row tuples simultaneously owned by admission.
    /// Includes expansion of declaration defaults and the document's variable-width payload.
    pub fn admission_bytes(&self, document: &DataDocument) -> usize {
        let rows = document.rows.rows();
        let slots = self.keys.iter().chain(&self.values);
        let mut cells = 0usize;
        let mut payload = 0usize;
        for slot in slots {
            cells = cells.saturating_add(1);
            if let Some(column) = document
                .rows
                .columns()
                .iter()
                .find(|column| column.name == slot.name)
            {
                if matches!(slot.target, Target::Text | Target::Identifier(_)) {
                    payload = payload.saturating_add(column.values.retained_bytes());
                }
            } else if let Some(default) = &slot.default {
                payload = payload.saturating_add(rows.saturating_mul(default.retained_bytes()));
            }
        }
        rows.saturating_mul(
            4 * size_of::<Vec<Value>>() + 2 * cells.saturating_mul(size_of::<Value>()),
        )
        .saturating_add(2 * payload)
    }
    /// Conservative owned storage; shared indexes are counted where they are held.
    pub fn retained_bytes(&self) -> usize {
        let slot = |s: &Slot| {
            size_of::<Slot>()
                + s.name.capacity()
                + s.declared.capacity()
                + s.default.as_ref().map_or(0, Value::retained_bytes)
                + match &s.target {
                    Target::Quantity(conversion) => conversion.heap_bytes(),
                    Target::Entity {
                        entities, scheme, ..
                    } => {
                        entities.len() * 64
                            + scheme.as_ref().map_or(0, |(_, values)| {
                                values.keys().map(|v| v.capacity() + 64).sum::<usize>()
                            })
                    }
                    Target::Enum { members, .. } => {
                        members.keys().map(|m| m.capacity() + 64).sum::<usize>()
                    }
                    _ => 0,
                }
        };
        size_of::<Self>()
            + self.label.capacity()
            + self.keys.iter().map(slot).sum::<usize>()
            + self.values.iter().map(slot).sum::<usize>()
    }
}

/// A document's rows admitted through a plan: each row's typed key values and supplied
/// values, in document order.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentTable {
    /// One typed key tuple per row, in key order.
    pub keys: Vec<Vec<Value>>,
    /// One tuple of supplied values per row, in the plan's value order.
    pub values: Vec<Vec<Value>>,
}
impl DocumentTable {
    /// Conservative owned storage.
    pub fn retained_bytes(&self) -> usize {
        let tuple = |t: &Vec<Value>| {
            t.capacity() * size_of::<Value>() + t.iter().map(Value::retained_bytes).sum::<usize>()
        };
        size_of::<Self>()
            + self.keys.iter().map(tuple).sum::<usize>()
            + self.values.iter().map(tuple).sum::<usize>()
    }
}

/// The package data documents admission reads (ADR-0125).
pub trait Documents {
    /// Exact source fields from the immutable source revision, when supplied by authoring.
    fn field_spans(
        &self,
        _declaration: DeclarationId,
    ) -> Option<&BTreeMap<String, pse_authoring::SourceSpan>> {
        None
    }

    /// Precharge owned expansion before allocating it. Standalone pure checks have no
    /// workspace owner; engine consumers enforce their workspace allowance here.
    ///
    /// # Errors
    /// Insufficient admission allowance or cancellation.
    fn preflight(&self, _: DeclarationId, _: usize) -> Result<()> {
        Ok(())
    }
    /// Number of rows in a resolved document, for record expansion before admission.
    fn rows(&self, _: SemanticId) -> Option<usize> {
        None
    }
    /// Variable-width input storage of a resolved document.
    fn bytes(&self, _: SemanticId) -> usize {
        0
    }
    /// The identity of the data document a declaration of source document `source` names
    /// by `path`.
    fn resolve(&self, source: SemanticId, path: &str) -> Option<SemanticId>;
    /// Document `document`'s rows admitted through `plan`. Equal plans over one unchanged
    /// document answer the same admitted rows; the implementation owns their reuse.
    ///
    /// # Errors
    /// The refusals of [`admit`], or cancellation.
    fn admit(&self, plan: &Arc<DocumentPlan>, document: SemanticId) -> Result<Arc<DocumentTable>>;
}

/// Declarations admitted without data documents: a dataset naming one is refused.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoDocuments;
impl Documents for NoDocuments {
    fn resolve(&self, _: SemanticId, _: &str) -> Option<SemanticId> {
        None
    }
    fn admit(&self, plan: &Arc<DocumentPlan>, _: SemanticId) -> Result<Arc<DocumentTable>> {
        Err(invalid(plan.dataset, "no data documents are supplied"))
    }
}

/// Admit `document`'s rows through `plan`: every declared name is a column and every column
/// a declared name; each column is typed once by its Arrow type against its declared type.
/// `quantities` resolves a unit the document states.
///
/// # Errors
/// A missing or undeclared column, a column whose Arrow type does not store its declared
/// type, a stated unit other than the declared storage unit, a null in a required column, a
/// nonfinite magnitude, an identity or identifier value naming no admitted entity of the
/// declared kind, or a member name outside its enumeration. Each refusal names the column
/// and its declared type.
pub fn admit(
    plan: &DocumentPlan,
    document: &DataDocument,
    quantities: &pse_quantity::QuantityRegistry,
) -> Result<DocumentTable> {
    let at = plan.dataset;
    let columns = document.rows.columns();
    let find = |slot: &Slot| {
        columns.iter().find(|c| c.name == slot.name).ok_or_else(|| {
            invalid(
                at,
                format!(
                    "data document {} of {} has no column {} (declared {}); column names equal the declared names",
                    document.path, plan.label, slot.name, slot.declared
                ),
            )
        })
    };
    for column in columns {
        if !plan.keys_only
            && !plan
                .keys
                .iter()
                .chain(&plan.values)
                .any(|s| s.name == column.name)
        {
            return Err(invalid(
                at,
                format!(
                    "data document {} of {} has column {}, which {} does not declare",
                    document.path, plan.label, column.name, plan.label
                ),
            ));
        }
    }
    let typed = |slots: &[Slot]| {
        slots
            .iter()
            .map(|slot| {
                if !columns.iter().any(|c| c.name == slot.name)
                    && let Some(default) = &slot.default
                {
                    return Ok(vec![default.clone(); document.rows.rows()]);
                }
                column(plan, document, slot, find(slot)?, quantities)
            })
            .collect::<Result<Vec<_>>>()
    };
    let keys = typed(&plan.keys)?;
    let values = typed(&plan.values)?;
    let rows = document.rows.rows();
    let transpose = |columns: Vec<Vec<Value>>| {
        let mut iterators = columns.into_iter().map(Vec::into_iter).collect::<Vec<_>>();
        (0..rows)
            .map(|_| {
                iterators
                    .iter_mut()
                    .map(|i| i.next().unwrap_or(Value::Missing))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    Ok(DocumentTable {
        keys: transpose(keys),
        values: transpose(values),
    })
}

/// One column typed against its slot.
fn column(
    plan: &DocumentPlan,
    document: &DataDocument,
    slot: &Slot,
    column: &DocumentColumn,
    quantities: &pse_quantity::QuantityRegistry,
) -> Result<Vec<Value>> {
    let at = plan.dataset;
    let refuse = |message: String| {
        invalid(
            at,
            format!(
                "column {} of data document {} ({}) declared {}: {message}",
                slot.name, document.path, plan.label, slot.declared
            ),
        )
    };
    let mismatch = |stored: &str| {
        refuse(format!(
            "the document stores it as {}, where {stored} is expected",
            column.values.arrow_type()
        ))
    };
    if let Some(stated) = &column.unit {
        let Target::Quantity(conversion) = &slot.target else {
            return Err(refuse(format!(
                "the document states unit {stated}, but only a quantity column has one"
            )));
        };
        let agrees =
            stated_unit(stated, quantities).is_some_and(|unit| unit == conversion.source_unit());
        if !agrees {
            return Err(refuse(format!(
                "the document states unit {stated}, which disagrees with the declared storage unit; units come only from the declaration"
            )));
        }
    }
    let missing = |row: usize| {
        if slot.optional {
            Ok(Value::Missing)
        } else {
            Err(refuse(format!(
                "row {row} is null where a value is required"
            )))
        }
    };
    let values = match (&slot.target, &column.values) {
        (Target::Quantity(conversion), Values::Magnitude(values)) => {
            values
                .iter()
                .enumerate()
                .map(|(row, v)| match v {
                    None => missing(row),
                    Some(v) => conversion.apply(*v)
                        .map(|value| Value::Number {
                            bits: value.bits(),
                            quantity: value.quantity(),
                        })
                        .map_err(|error| refuse(format!("row {row}: {error}"))),
                })
                .collect::<Result<Vec<_>>>()?
        }
        (Target::Quantity(_), _) => return Err(mismatch("Float64 magnitudes")),
        (Target::Entity { entities, kind, deferred, .. }, Values::Identity(values)) => values
            .iter()
            .enumerate()
            .map(|(row, v)| match v {
                None => missing(row),
                Some(id) => {
                    let id = DeclarationId::from(*id);
                    entities
                        .get(&id)
                        .map(|kind| Value::Entity { id, kind: *kind })
                        .or_else(|| deferred.then_some(Value::Entity { id, kind: *kind }))
                        .ok_or_else(|| {
                            refuse(format!("row {row} names {id}, which is no admitted entity of the declared kind"))
                        })
                }
            })
            .collect::<Result<Vec<_>>>()?,
        (
            Target::Entity {
                entities,
                scheme: Some((_, index)),
                ..
            },
            Values::Text(values),
        ) => values
            .iter()
            .enumerate()
            .map(|(row, v)| match v {
                None => missing(row),
                Some(value) => index
                    .get(value)
                    .and_then(|id| entities.get(id).map(|kind| Value::Entity { id: *id, kind: *kind }))
                    .ok_or_else(|| {
                        refuse(format!(
                            "row {row} names \"{value}\", which no admitted entity of the declared kind holds"
                        ))
                    }),
            })
            .collect::<Result<Vec<_>>>()?,
        (Target::Entity { scheme: None, .. }, Values::Text(_)) => {
            return Err(refuse(
                "the document names entities by text, but the column declares no identifier scheme (`by scheme`)"
                    .into(),
            ));
        }
        (Target::Entity { .. }, _) => {
            return Err(mismatch("FixedSizeBinary(16) identities or Utf8 identifier values"));
        }
        (Target::Identifier(scheme), Values::Text(values)) => values
            .iter()
            .enumerate()
            .map(|(row, v)| match v {
                None => missing(row),
                Some(value) => Ok(Value::Identifier {
                    scheme: *scheme,
                    value: value.clone(),
                }),
            })
            .collect::<Result<Vec<_>>>()?,
        (Target::Text, Values::Text(values)) => values
            .iter()
            .enumerate()
            .map(|(row, v)| v.as_ref().map_or_else(|| missing(row), |v| Ok(Value::Text(v.clone()))))
            .collect::<Result<Vec<_>>>()?,
        (Target::Identifier(_) | Target::Text, _) => return Err(mismatch("Utf8")),
        (Target::Integer, Values::Integer(values)) => values
            .iter()
            .enumerate()
            .map(|(row, v)| v.map_or_else(|| missing(row), |v| Ok(Value::Integer(v))))
            .collect::<Result<Vec<_>>>()?,
        (Target::Integer, _) => return Err(mismatch("Int64")),
        (Target::Boolean, Values::Boolean(values)) => values
            .iter()
            .enumerate()
            .map(|(row, v)| v.map_or_else(|| missing(row), |v| Ok(Value::Boolean(v))))
            .collect::<Result<Vec<_>>>()?,
        (Target::Boolean, _) => return Err(mismatch("Boolean")),
        (
            Target::Enum {
                enumeration,
                members,
            },
            Values::Member {
                members: names,
                keys,
            },
        ) => {
            let dictionary = names
                .iter()
                .map(|name| {
                    members.get(name).copied().ok_or_else(|| {
                        refuse(format!("the dictionary names {name}, which is not a member"))
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            keys.iter()
                .enumerate()
                .map(|(row, key)| match key {
                    None => missing(row),
                    Some(key) => dictionary
                        .get(*key as usize)
                        .map(|member| Value::Enum {
                            enumeration: *enumeration,
                            member: *member,
                        })
                        .ok_or_else(|| refuse(format!("row {row} indexes outside its dictionary"))),
                })
                .collect::<Result<Vec<_>>>()?
        }
        (Target::Enum { .. }, _) => return Err(mismatch("a dictionary of member names")),
    };
    Ok(values)
}

/// The unit a document states, by its spelling in the unit literal grammar.
fn stated_unit(
    spelling: &str,
    quantities: &pse_quantity::QuantityRegistry,
) -> Option<pse_quantity::UnitId> {
    let expression = pse_authoring::dsl::parse_expr(&format!("1{{{spelling}}}")).ok()?;
    let pse_authoring::dsl::ExprKind::Number(number) = expression.kind else {
        return None;
    };
    Some(quantities.compose(number.unit.as_ref()?).ok()?.id)
}
