// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared complete source binding for P3 and identity-preserving rename.

mod paths;
mod rewrite;
pub use rewrite::rename as rename_expression;
mod walk;

use super::{DocumentBundle, load::contract, value::Value};
use crate::{AuthoringError, SourceSpan, dsl};
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use pse_schema::{
    Registry,
    model::{Cell, ExtensionUse, LogicalType},
};
use std::collections::{BTreeMap, BTreeSet};

/// A declared or lexical path meaning; composite declaration keys stay composite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathMeaning {
    /// A bound symbol declaration and its template.
    Symbol {
        /// Owning template.
        template_id: SemanticId,
        /// Symbol declaration.
        symbol_id: SemanticId,
    },
    /// A bound equation declaration.
    Equation {
        /// Owning template.
        template_id: SemanticId,
        /// Equation declaration.
        equation_id: SemanticId,
    },
    /// A port's actual composite primary key.
    Port {
        /// Owning template.
        template_id: SemanticId,
        /// Port key.
        name: String,
    },
    /// An explicit registered entity.
    Entity(SemanticId),
    /// A template-local domain declaration, before instance realization.
    Domain {
        /// Owning template.
        template_id: SemanticId,
        /// Domain key.
        name: String,
    },
    /// A declared template parameter.
    Parameter {
        /// Owning template.
        template_id: SemanticId,
        /// Parameter key.
        name: String,
    },
    /// A declared template feature.
    Feature {
        /// Owning template.
        template_id: SemanticId,
        /// Feature key.
        name: String,
    },
    /// An actual declared unit in a conversion target expression.
    Unit {
        /// Exact admitted unit identity.
        unit_id: SemanticId,
    },
    /// A predicate literal admitted under the actual compared declaration's enum.
    EnumLiteral {
        /// Exact registered enum identity.
        enum_id: SemanticId,
        /// Exact declared member spelling.
        member: String,
    },
    /// A lexically bound reduction/let/index variable.
    Local(String),
}
/// Every parsed path is accounted for, including lexical variables and nested indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundPath {
    /// Byte range relative to the decoded DSL string.
    pub span: dsl::Span,
    /// Resolved declared or lexical meaning.
    pub meaning: PathMeaning,
    /// Identity of each named path segment where one exists; used by rename.
    pub segment_entities: Vec<Option<SemanticId>>,
}
/// The grammar admitted for one source field.
#[derive(Clone, Debug, PartialEq)]
pub enum ParsedExpression {
    /// Arithmetic expression.
    Expr(dsl::Expr),
    /// Equation, including conditional equations.
    Equation(dsl::Equation),
    /// Predicate.
    Predicate(dsl::Predicate),
}
/// One fully bound original expression field.
#[derive(Clone, Debug)]
pub struct SourceExpression {
    /// Owning original document.
    pub document_id: SemanticId,
    /// Original field path, including nested list indices.
    pub document_path: String,
    /// Exact registered relation.
    pub relation_id: SemanticId,
    /// Complete actual primary key of the source row.
    pub row_key: Vec<Cell>,
    /// Explicit template owner from the document declaration.
    pub owner_template_id: SemanticId,
    /// Ordered declared index variables visible in this source row.
    pub indexed_by: Vec<String>,
    /// Original source field range.
    pub source_span: SourceSpan,
    /// Exact decoded DSL string.
    pub text: String,
    /// Parsed structure, never an authored row authority.
    pub parsed: ParsedExpression,
    /// Exhaustive path binding inventory.
    pub paths: Vec<BoundPath>,
}
/// Only construction through [`bind_sources`] establishes complete source coverage.
#[derive(Clone, Debug)]
pub struct SourceBindings {
    expressions: Vec<SourceExpression>,
}
impl SourceBindings {
    /// Every source field, in document and declaration order.
    pub fn expressions(&self) -> &[SourceExpression] {
        &self.expressions
    }
}

pub(super) struct Context {
    targets: crate::targets::TargetContext,
    templates: Vec<authored::templates::Row>,
    params: Vec<authored::template_params::Row>,
    features: Vec<authored::template_features::Row>,
    submodels: Vec<authored::template_submodels::Row>,
    units: Vec<pse_relations::generated::reference::units::Row>,
    enums: BTreeMap<SemanticId, BTreeSet<String>>,
}

/// Bind every declared DSL field against a complete actual base/candidate row inventory.
/// # Errors
/// Missing ownership, malformed grammar, unbound or ambiguous paths, duplicate lexical
/// declarations, cyclic submodel paths or missing source ranges are typed failures.
pub fn bind_sources(
    bundles: &[DocumentBundle],
    rows: &super::Rows,
    registry: &Registry,
) -> Result<SourceBindings, AuthoringError> {
    let parsed = reparse(bundles, registry)?;
    bind_parsed(&parsed, rows, registry)
}

/// Bind immutable accounted loader results without reparsing their original source.
/// # Errors
/// Actual source/binding disagreement, cancellation or insufficient workspace.
pub fn bind_sources_owned(
    bundles: &super::OwnedDocumentSet,
    rows: &super::Rows,
    registry: &Registry,
    reserver: &dyn pse_ids::MemoryReserver,
    cancel: &pse_ids::CancellationToken,
) -> Result<OwnedSourceBindings, AuthoringError> {
    bundles.validate_registry(registry)?;
    cancel.checkpoint()?;
    let mut work = reserver.open("authoring:source-bindings");
    work.try_grow(crate::work::add(
        crate::work::sources(bundles.bundles())?,
        crate::work::mul(crate::work::rows(rows)?, 4)?,
    )?)?;
    let bindings = bind_parsed(bundles.bundles(), rows, registry)?;
    cancel.checkpoint()?;
    Ok(OwnedSourceBindings(std::sync::Arc::new(BindingOwner {
        bindings,
        _lease: pse_ids::ReservationLease::new(work),
    })))
}
#[derive(Debug)]
struct BindingOwner {
    bindings: SourceBindings,
    _lease: std::sync::Arc<pse_ids::ReservationLease>,
}
/// Immutable complete binding inventory retaining its construction reservation.
#[derive(Clone, Debug)]
pub struct OwnedSourceBindings(std::sync::Arc<BindingOwner>);
impl OwnedSourceBindings {
    /// Every source field after actual lexical and declaration binding.
    pub fn expressions(&self) -> &[SourceExpression] {
        self.0.bindings.expressions()
    }
}

pub(crate) fn bind_parsed(
    parsed: &[DocumentBundle],
    rows: &super::Rows,
    registry: &Registry,
) -> Result<SourceBindings, AuthoringError> {
    let context = Context::new(rows, registry)?;
    let mut expressions = Vec::new();
    for bundle in parsed {
        for document in &bundle.documents {
            let decoded = crate::generated::documents::rows_from_document(
                document.declaration.name,
                document.value.clone(),
            )
            .map_err(|error| contract(None, &error.to_string()))?;
            for section in &document.declaration.sections {
                let Some(owner) = section.expression_owner_column else {
                    continue;
                };
                let relation = registry
                    .relation(section.relation)
                    .ok_or_else(|| contract(None, "missing source relation"))?;
                let Some(Value::List(values)) = document.value.get(section.key) else {
                    continue;
                };
                for (ordinal, value) in values.iter().enumerate() {
                    let owner = value
                        .value
                        .get(owner)
                        .and_then(Value::text)
                        .ok_or_else(|| contract(None, "missing declared expression owner"))?;
                    let owner = crate::ids::parse_id(owner, SourceSpan::head(document.id))?;
                    if context
                        .templates
                        .iter()
                        .filter(|template| template.template_id == owner)
                        .count()
                        != 1
                    {
                        return Err(contract(
                            None,
                            "expression owner must resolve to exactly one declared template",
                        ));
                    }
                    let row = decoded
                        .get(&relation.id)
                        .and_then(|rows| rows.get(ordinal))
                        .ok_or_else(|| {
                            contract(None, "source row missing from exact generated document")
                        })?;
                    let indexed_by = index_names(relation.id, row, &value.value)?;
                    let key = checked_key(relation, row, rows)?;
                    let mut fields = Vec::new();
                    for column in &relation.columns {
                        if let Some(value) = value.value.get(column.name) {
                            fields_of(
                                value,
                                &column.logical_type,
                                &format!("/{}/{ordinal}/{}", section.key, column.name),
                                column.name,
                                &mut fields,
                            );
                        }
                    }
                    for (path, typed_path, text) in fields {
                        let span = document.spans.span(&path).ok_or_else(|| {
                            contract(None, "missing original expression field range")
                        })?;
                        let syntax = section
                            .expression_fields
                            .iter()
                            .find(|(path, _)| *path == typed_path)
                            .map(|(_, syntax)| *syntax)
                            .ok_or_else(|| contract(Some(span), "DSL field grammar undeclared"))?;
                        let parsed = parse(text, span, syntax)?;
                        let mut expression = SourceExpression {
                            document_id: document.id,
                            document_path: path,
                            relation_id: relation.id,
                            row_key: key.clone(),
                            owner_template_id: owner,
                            indexed_by: indexed_by.clone(),
                            source_span: span,
                            text: text.to_owned(),
                            parsed,
                            paths: Vec::new(),
                        };
                        expression.paths =
                            walk::bind(&expression.parsed, owner, &context, span, &indexed_by)?;
                        expressions.push(expression);
                    }
                }
            }
        }
    }
    Ok(SourceBindings { expressions })
}

fn index_names(
    relation: SemanticId,
    row: &[Cell],
    value: &Value,
) -> Result<Vec<String>, AuthoringError> {
    if relation == authored::template_property_requirements::RELATION_ID {
        let requirement = authored::template_property_requirements::Row::from_cells(row.to_vec())
            .map_err(|error| contract(None, &error.to_string()))?;
        return Ok(requirement
            .index_domain_bindings
            .into_iter()
            .map(|binding| binding.index_name)
            .collect());
    }
    match value.get("indexed_by") {
        Some(Value::List(values)) => values
            .iter()
            .map(|value| {
                value
                    .value
                    .text()
                    .map(str::to_owned)
                    .ok_or_else(|| contract(None, "index variable must be a name"))
            })
            .collect(),
        _ => Ok(Vec::new()),
    }
}

fn reparse(
    bundles: &[DocumentBundle],
    registry: &Registry,
) -> Result<Vec<DocumentBundle>, AuthoringError> {
    bundles
        .iter()
        .map(|bundle| {
            let mut texts = BTreeMap::new();
            for document in &bundle.documents {
                if texts
                    .insert(document.path.clone(), document.text.clone())
                    .is_some()
                {
                    return Err(contract(None, "duplicate original document path"));
                }
            }
            super::load_package_texts(texts, registry, crate::ParseBudget::default())
        })
        .collect()
}
fn checked_key(
    relation: &pse_schema::model::RelationSpec,
    row: &[Cell],
    rows: &super::Rows,
) -> Result<Vec<Cell>, AuthoringError> {
    let columns = relation
        .primary_key
        .iter()
        .map(|name| {
            relation
                .columns
                .iter()
                .position(|column| column.name == *name)
                .ok_or_else(|| contract(None, "missing declared source key"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let actual = rows
        .get(&relation.id)
        .into_iter()
        .flatten()
        .filter(|candidate| {
            columns.iter().all(|&index| {
                candidate.get(index).map(Cell::literal_spec)
                    == row.get(index).map(Cell::literal_spec)
            })
        })
        .collect::<Vec<_>>();
    if actual.len() != 1
        || actual[0].iter().map(Cell::literal_spec).collect::<Vec<_>>()
            != row.iter().map(Cell::literal_spec).collect::<Vec<_>>()
    {
        return Err(contract(
            None,
            "original expression source row differs from the actual bound input",
        ));
    }
    Ok(columns
        .into_iter()
        .map(|index| row[index].clone())
        .collect())
}

fn fields_of<'a>(
    value: &'a Value,
    ty: &LogicalType,
    path: &str,
    typed_path: &str,
    fields: &mut Vec<(String, String, &'a str)>,
) {
    match (value, ty) {
        (Value::Text(text), LogicalType::Ext(ExtensionUse::ExprDsl)) => {
            fields.push((path.to_owned(), typed_path.to_owned(), text));
        }
        (Value::List(values), LogicalType::List(element) | LogicalType::FixedList(element, _)) => {
            for (ordinal, value) in values.iter().enumerate() {
                fields_of(
                    &value.value,
                    element,
                    &format!("{path}/{ordinal}"),
                    &format!("{typed_path}[]"),
                    fields,
                );
            }
        }
        (value @ Value::Map(_), LogicalType::Struct(columns)) => {
            for (name, ty, _) in columns {
                if let Some(value) = value.get(name) {
                    fields_of(
                        value,
                        ty,
                        &format!("{path}/{name}"),
                        &format!("{typed_path}.{name}"),
                        fields,
                    );
                }
            }
        }
        _ => {}
    }
}
fn parse(
    text: &str,
    at: SourceSpan,
    syntax: pse_schema::model::DslSyntax,
) -> Result<ParsedExpression, AuthoringError> {
    use pse_schema::model::DslSyntax;
    match syntax {
        DslSyntax::Expression => dsl::parse_expr(text).map(ParsedExpression::Expr),
        DslSyntax::Predicate => dsl::parse_predicate(text).map(ParsedExpression::Predicate),
        DslSyntax::Equation => dsl::parse_equation(text).map(ParsedExpression::Equation),
    }
    .map_err(|error| contract(Some(at), &error.to_string()))
}

pub(super) fn validate_document(
    document: &super::Document,
    registry: &Registry,
) -> Result<(), AuthoringError> {
    for section in &document.declaration.sections {
        if section.expression_fields.is_empty() {
            continue;
        }
        let relation = registry
            .relation(section.relation)
            .ok_or_else(|| contract(None, "document relation undeclared"))?;
        let Some(Value::List(values)) = document.value.get(section.key) else {
            continue;
        };
        for (ordinal, value) in values.iter().enumerate() {
            let mut fields = Vec::new();
            for column in &relation.columns {
                if let Some(value) = value.value.get(column.name) {
                    fields_of(
                        value,
                        &column.logical_type,
                        &format!("/{}/{ordinal}/{}", section.key, column.name),
                        column.name,
                        &mut fields,
                    );
                }
            }
            for (path, typed_path, text) in fields {
                let span = document
                    .spans
                    .span(&path)
                    .ok_or_else(|| contract(None, "DSL field source range absent"))?;
                let syntax = section
                    .expression_fields
                    .iter()
                    .find(|(path, _)| *path == typed_path)
                    .map(|(_, syntax)| *syntax)
                    .ok_or_else(|| contract(Some(span), "DSL field grammar undeclared"))?;
                parse(text, span, syntax)?;
            }
        }
    }
    Ok(())
}
impl Context {
    fn new(rows: &super::Rows, registry: &Registry) -> Result<Self, AuthoringError> {
        fn decode<T>(
            rows: &BTreeMap<SemanticId, Vec<Vec<Cell>>>,
            id: SemanticId,
            convert: fn(Vec<Cell>) -> Result<T, pse_relations::RelationError>,
        ) -> Result<Vec<T>, AuthoringError> {
            rows.get(&id)
                .into_iter()
                .flatten()
                .cloned()
                .map(convert)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| contract(None, &error.to_string()))
        }
        Ok(Self {
            enums: registry
                .enums()
                .iter()
                .map(|definition| {
                    (
                        definition.id,
                        definition
                            .members
                            .iter()
                            .map(|member| member.name.to_owned())
                            .collect(),
                    )
                })
                .collect(),
            targets: crate::targets::TargetContext::from_rows(rows)?,
            templates: decode(
                rows,
                authored::templates::RELATION_ID,
                authored::templates::Row::from_cells,
            )?,
            params: decode(
                rows,
                authored::template_params::RELATION_ID,
                authored::template_params::Row::from_cells,
            )?,
            features: decode(
                rows,
                authored::template_features::RELATION_ID,
                authored::template_features::Row::from_cells,
            )?,
            units: decode(
                rows,
                pse_relations::generated::reference::units::RELATION_ID,
                pse_relations::generated::reference::units::Row::from_cells,
            )?,
            submodels: decode(
                rows,
                authored::template_submodels::RELATION_ID,
                authored::template_submodels::Row::from_cells,
            )?,
        })
    }
}
