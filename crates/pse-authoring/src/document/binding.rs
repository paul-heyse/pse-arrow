// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared complete source binding for P3 and identity-preserving rename.

mod indices;
mod instance_path;
mod native;
mod paths;
pub use instance_path::{BoundInstancePath, InstancePathSegment};
mod rewrite;
pub use rewrite::rename as rename_expression;
mod walk;

use super::{DocumentBundle, load::contract, value::Value};
use crate::{AuthoringError, SourceSpan, dsl};
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use pse_schema::{
    Registry,
    model::{Cell, ExpressionOwnerKind, ExtensionUse, FieldContract},
};
use std::collections::{BTreeMap, BTreeSet};

/// A declared or lexical path meaning; composite declaration keys stay composite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathMeaning {
    /// Exact instance-relative source path, resolved independently in every actual context.
    InstancePath(BoundInstancePath),
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
    /// A Boolean configuration literal, distinct from a numerical graph constant.
    BooleanLiteral(bool),
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
    /// Explicit owner kind from the document declaration.
    pub owner_kind: ExpressionOwnerKind,
    /// Actual source owner identity; never substituted with its template context.
    pub owner_id: SemanticId,
    /// Resolved template binding context, which need not own an instance equation.
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
/// Constructed from the retained parser fields and their native declaration joins.
#[derive(Clone, Debug)]
pub struct SourceBindings {
    expressions: Vec<SourceExpression>,
    lookup: native::Lookup,
}
impl SourceBindings {
    /// Every source field, in document and declaration order.
    pub fn expressions(&self) -> &[SourceExpression] {
        &self.expressions
    }
    /// Exact source key computed by the retained native source-row projection.
    pub fn source_key(&self, expression: &SourceExpression) -> Option<&str> {
        self.lookup
            .source_keys
            .get(&(expression.document_id, expression.document_path.clone()))
            .map(String::as_str)
    }
}

pub(super) struct Context {
    entities: Vec<authored::entities::Row>,
    symbols: Vec<authored::template_symbols::Row>,
    template_domains: Vec<authored::template_domains::Row>,
    instances: Vec<authored::instances::Row>,
    templates: Vec<authored::templates::Row>,
    params: Vec<authored::template_params::Row>,
    features: Vec<authored::template_features::Row>,
    submodels: Vec<authored::template_submodels::Row>,
    prospective: Vec<pse_relations::generated::normalized::instance_bindings::Row>,
    semantic_id_type: SemanticId,
    index_companions: indices::IndexCompanions,
    lookup: native::Lookup,
    enums: BTreeMap<SemanticId, BTreeSet<String>>,
    enum_names: BTreeMap<String, SemanticId>,
}

/// Bind cached DSL syntax through native occurrence/declaration plans.
/// # Errors
/// Ownership/path ambiguity, malformed grammar, cancellation or insufficient workspace.
pub async fn bind_sources_owned(
    bundles: &super::OwnedDocumentSet,
    batches: &super::Batches,
    session: &pse_catalog::session::SnapshotSession,
    cancel: &pse_ids::CancellationToken,
) -> Result<OwnedSourceBindings, AuthoringError> {
    bundles.validate_registry(session.registry())?;
    cancel.checkpoint()?;
    let mut work = session.reserver().open("authoring:source-bindings");
    let decoded = [
        authored::entities::RELATION_ID,
        authored::instances::RELATION_ID,
        authored::templates::RELATION_ID,
        authored::template_params::RELATION_ID,
        authored::template_features::RELATION_ID,
        authored::template_submodels::RELATION_ID,
        authored::template_symbols::RELATION_ID,
        authored::template_equations::RELATION_ID,
        authored::template_ports::RELATION_ID,
        authored::template_domains::RELATION_ID,
        authored::domains::RELATION_ID,
        authored::domain_members::RELATION_ID,
        authored::instance_domain_bindings::RELATION_ID,
        pse_relations::generated::normalized::instance_bindings::RELATION_ID,
    ];
    let columns = decoded
        .iter()
        .filter_map(|id| batches.get(id))
        .try_fold(0_usize, |sum, batch| {
            crate::work::add(sum, pse_ids::validation_extent(batch.batch())?)
        })?;
    work.try_grow(crate::work::add(
        crate::work::sources(bundles.bundles())?,
        crate::work::mul(columns, 4)?,
    )?)?;
    let mut lookup = native::prepare(bundles.bundles(), batches, session, cancel).await?;
    let indices = indices::load(batches, session, cancel, &mut lookup.completed).await?;
    let context = Context::new(batches, session.registry(), lookup, indices)?;
    let bindings = bind_parsed(bundles.bundles(), session.registry(), context)?;
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
    /// Exact source row key produced by this inventory's native binding computation.
    pub fn source_key(&self, source: &SourceExpression) -> Option<&str> {
        self.0.bindings.source_key(source)
    }
    /// Actual completed native binding pieces, retaining their frozen providers.
    pub fn completions(&self) -> &[std::sync::Arc<pse_catalog::session::CompletedComputation>] {
        &self.0.bindings.lookup.completed
    }
    /// Actual parser occurrence batches bound by the prepared native computation.
    pub fn occurrences(&self) -> &[pse_relations::columnar::FieldCheckedBatch] {
        &self.0.bindings.lookup.occurrences
    }
    /// Complete matching declaration rows, including ambiguity multiplicity.
    pub fn resolved_occurrences(&self) -> &[pse_relations::columnar::FieldCheckedBatch] {
        &self.0.bindings.lookup.resolved
    }
}

fn bind_parsed(
    parsed: &[DocumentBundle],
    registry: &Registry,
    context: Context,
) -> Result<SourceBindings, AuthoringError> {
    let mut expressions = Vec::new();
    for bundle in parsed {
        for document in &bundle.documents {
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
                    let owner_id = crate::ids::parse_id(owner, SourceSpan::head(document.id))?;
                    let owner_kind = section
                        .expression_owner_kind
                        .ok_or_else(|| contract(None, "expression owner kind absent"))?;
                    let owner = owner_template(&context, owner_kind, owner_id)?;
                    let key = source_row_key(document, relation, ordinal, registry)?;
                    let mut fields = Vec::new();
                    for column in &relation.columns {
                        if let Some(value) = value.value.get(column.name()) {
                            fields_of(
                                value,
                                &column.value_type(),
                                &format!("/{}/{ordinal}/{}", section.key, column.name()),
                                column.name(),
                                &mut fields,
                            );
                        }
                    }
                    for (path, _typed_path, text) in fields {
                        let span = document.spans.span(&path).ok_or_else(|| {
                            contract(None, "missing original expression field range")
                        })?;
                        let source_key = context
                            .lookup
                            .source_keys
                            .get(&(document.id, path.clone()))
                            .ok_or_else(|| {
                                contract(Some(span), "native source-key binding absent")
                            })?;
                        let indexed_by =
                            index_names(relation.id, source_key, &value.value, &context)?;
                        let parsed = document
                            .expressions
                            .get(&path)
                            .ok_or_else(|| {
                                contract(Some(span), "parsed source expression missing")
                            })?
                            .clone();
                        let mut expression = SourceExpression {
                            document_id: document.id,
                            document_path: path,
                            relation_id: relation.id,
                            row_key: key.clone(),
                            owner_kind,
                            owner_id,
                            owner_template_id: owner,
                            indexed_by: indexed_by.clone(),
                            source_span: span,
                            text: text.to_owned(),
                            parsed,
                            paths: Vec::new(),
                        };
                        expression.paths = walk::bind(
                            &expression.parsed,
                            owner,
                            &context,
                            span,
                            &indexed_by,
                            relation.id == authored::template_submodels::RELATION_ID,
                        )?;
                        expressions.push(expression);
                    }
                }
            }
        }
    }
    Ok(SourceBindings {
        expressions,
        lookup: context.lookup,
    })
}

fn index_names(
    relation: SemanticId,
    source_key: &str,
    value: &Value,
    context: &Context,
) -> Result<Vec<String>, AuthoringError> {
    if let Some(values) = context.index_companions.get(&relation) {
        return values
            .get(source_key)
            .cloned()
            .ok_or_else(|| contract(None, "source index companion lookup absent"));
    }
    if relation == authored::template_property_requirements::RELATION_ID {
        let Some(Value::List(bindings)) = value.get("index_domain_bindings") else {
            return Ok(Vec::new());
        };
        return bindings
            .iter()
            .map(|binding| {
                binding
                    .value
                    .get("index_name")
                    .and_then(Value::text)
                    .map(str::to_owned)
                    .ok_or_else(|| contract(None, "requirement index name absent"))
            })
            .collect();
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

fn fields_of<'a>(
    value: &'a Value,
    ty: &FieldContract,
    path: &str,
    typed_path: &str,
    fields: &mut Vec<(String, String, &'a str)>,
) {
    match (value, ty.extension(), ty.data_type()) {
        (Value::Text(text), Some(ExtensionUse::ExprDsl), _) => {
            fields.push((path.to_owned(), typed_path.to_owned(), text));
        }
        (
            Value::List(values),
            None,
            datafusion::arrow::datatypes::DataType::List(element)
            | datafusion::arrow::datatypes::DataType::FixedSizeList(element, _),
        ) => {
            let element = &FieldContract::from_field((*element).clone());
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
        (value @ Value::Map(_), None, datafusion::arrow::datatypes::DataType::Struct(columns)) => {
            for field in &columns {
                let name = field.name();
                let ty = &FieldContract::from_field((**field).clone());
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

pub(super) fn parse_document(
    document: &super::Document,
    registry: &Registry,
) -> Result<BTreeMap<String, ParsedExpression>, AuthoringError> {
    let mut parsed = BTreeMap::new();
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
                if let Some(value) = value.value.get(column.name()) {
                    fields_of(
                        value,
                        &column.value_type(),
                        &format!("/{}/{ordinal}/{}", section.key, column.name()),
                        column.name(),
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
                parsed.insert(path, parse(text, span, syntax)?);
            }
        }
    }
    Ok(parsed)
}
impl Context {
    fn new(
        batches: &super::Batches,
        registry: &Registry,
        lookup: native::Lookup,
        index_companions: indices::IndexCompanions,
    ) -> Result<Self, AuthoringError> {
        macro_rules! rows {
            ($module:path) => {{
                use $module as relation;
                batches
                    .get(&relation::RELATION_ID)
                    .map(|batch| relation::View::from_checked(batch)?.rows())
                    .transpose()?
                    .unwrap_or_default()
            }};
        }
        Ok(Self {
            index_companions,
            lookup,
            prospective: rows!(pse_relations::generated::normalized::instance_bindings),
            semantic_id_type: registry
                .logical_type(
                    &FieldContract::id()
                        .type_name()
                        .map_err(|e| contract(None, &e.to_string()))?,
                )
                .ok_or_else(|| contract(None, "semantic-ID logical type declaration absent"))?
                .id,
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
            enum_names: registry
                .enums()
                .iter()
                .map(|definition| (definition.name.to_owned(), definition.id))
                .collect(),
            entities: rows!(authored::entities),
            symbols: rows!(authored::template_symbols),
            template_domains: rows!(authored::template_domains),
            instances: rows!(authored::instances),
            templates: rows!(authored::templates),
            params: rows!(authored::template_params),
            features: rows!(authored::template_features),
            submodels: rows!(authored::template_submodels),
        })
    }
}

fn owner_template(
    context: &Context,
    owner_kind: ExpressionOwnerKind,
    owner_id: SemanticId,
) -> Result<SemanticId, AuthoringError> {
    let owner = match owner_kind {
        ExpressionOwnerKind::Template => owner_id,
        ExpressionOwnerKind::Instance => {
            let mut matches = context
                .instances
                .iter()
                .filter(|instance| instance.instance_id == owner_id);
            let template = matches
                .next()
                .ok_or_else(|| contract(None, "instance expression owner absent"))?
                .template_id;
            if matches.next().is_some() {
                return Err(contract(None, "instance expression owner is ambiguous"));
            }
            template
        }
    };
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
    Ok(owner)
}

fn source_row_key(
    document: &super::Document,
    relation: &pse_schema::model::RelationSpec,
    ordinal: usize,
    registry: &Registry,
) -> Result<Vec<Cell>, AuthoringError> {
    let source = document
        .batches
        .get(&relation.id)
        .ok_or_else(|| contract(None, "source batch absent"))?;
    let positions = relation
        .primary_key
        .iter()
        .map(|name| {
            source
                .batch()
                .schema()
                .index_of(name)
                .map_err(|error| contract(None, &error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let key_batch = source
        .batch()
        .slice(ordinal, 1)
        .project(&positions)
        .map_err(|error| contract(None, &error.to_string()))?;
    let mut keys = pse_relations::cells::decode_columns(registry, &key_batch)?;
    let key = keys
        .pop()
        .ok_or_else(|| contract(None, "source key absent"))?;
    Ok(key)
}
