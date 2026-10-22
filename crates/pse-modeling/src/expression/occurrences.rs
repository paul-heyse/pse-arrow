// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked source syntax with declaration-local roles and immutable lexical attribution.
use crate::{CheckedPackage, Declaration, DeclarationId, Result, invalid};
use pse_authoring::{
    SourceSpan, dsl,
    language::{self, StaticValue},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// A field role and ordinal distinguish repeated syntax in one declaration.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OccurrenceKey {
    /// Source declaration, never a mathematical-body identity.
    pub declaration: DeclarationId,
    /// Authored field path, relative to the declaration payload.
    pub role: String,
    /// Position in a repeated field.
    pub position: usize,
}

/// Syntax admitted by the source grammar; AST handles are local to the checked package.
#[derive(Clone, Debug, PartialEq)]
pub enum Syntax {
    /// An arithmetic expression or reference.
    Expression(dsl::Expr),
    /// A source predicate.
    Predicate(dsl::Predicate),
    /// A relation, including conditional equations.
    Equation(dsl::Equation),
    /// Logic with retained source-bearing atoms and cardinality count expressions.
    Logic(dsl::Proposition),
    /// Compile-time collections and named applications embed the expression grammar.
    Static(StaticValue),
}

/// Attribution and dependencies for one checked source field.
#[derive(Clone, Debug)]
pub struct CheckedExpression {
    /// Exact parser field range when available; otherwise the programmatic declaration range.
    pub source: SourceSpan,
    /// Exact authored field text, independent of normalized mathematical syntax.
    pub text: String,
    /// Lexical declaration chain, innermost first, retaining import context.
    pub context: Vec<DeclarationId>,
    /// Parsed once before semantic checking and retained for consumers.
    pub syntax: Arc<Syntax>,
    /// Source-local allocation lifetime retained by detached checked fields.
    pub source_owner: Option<Arc<dyn crate::document::DataOwner>>,
    /// The declaration's checked contract, where the declaration owns a value type.
    pub declared_type: Option<crate::Type>,
    /// Concrete operation admissions and generic obligations present in this field's AST.
    pub physical_admissions: super::admission::ExpressionAdmissions,
    /// Resolved declaration dependencies from this occurrence's lexical environment.
    pub dependencies: BTreeSet<DeclarationId>,
    /// Index binders await concrete members at specialization, even when their types check.
    pub index_obligations: Vec<(String, String)>,
    /// Free references whose binding is supplied by specialization rather than the package.
    pub unresolved_references: BTreeSet<String>,
}

impl PartialEq for CheckedExpression {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.text == other.text
            && self.context == other.context
            && self.syntax == other.syntax
            && self.declared_type == other.declared_type
            && self.physical_admissions == other.physical_admissions
            && self.dependencies == other.dependencies
            && self.index_obligations == other.index_obligations
            && self.unresolved_references == other.unresolved_references
    }
}

/// Immutable grammar products for one exact declaration and its parser-owned
/// field ranges. Contextual type/binding/physical admissions are never cached here.
#[derive(Clone, Debug)]
pub struct ParsedFields {
    row: Declaration,
    fields: Option<BTreeMap<String, SourceSpan>>,
    expressions: BTreeMap<OccurrenceKey, CheckedExpression>,
    allocation_owner: Option<Arc<dyn crate::document::DataOwner>>,
}
impl PartialEq for ParsedFields {
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row
            && self.fields == other.fields
            && self.expressions == other.expressions
    }
}
impl ParsedFields {
    /// Parse through the same role/grammar/binder collector as checked admission.
    /// The caller precharges parsing and retains the returned allocation owner.
    pub fn parse(row: &Declaration, fields: Option<&BTreeMap<String, SourceSpan>>) -> Result<Self> {
        struct Fields<'a>(Option<&'a BTreeMap<String, SourceSpan>>);
        impl crate::document::Documents for Fields<'_> {
            fn field_spans(&self, _: DeclarationId) -> Option<&BTreeMap<String, SourceSpan>> {
                self.0
            }
            fn resolve(&self, _: pse_ids::SemanticId, _: &str) -> Option<pse_ids::SemanticId> {
                None
            }
            fn admit(
                &self,
                plan: &Arc<crate::document::DocumentPlan>,
                document: pse_ids::SemanticId,
            ) -> Result<Arc<crate::document::DocumentTable>> {
                crate::document::Documents::admit(&crate::document::NoDocuments, plan, document)
            }
        }
        Ok(Self {
            row: row.clone(),
            fields: fields.cloned(),
            expressions: collect_fresh(std::slice::from_ref(row), &Fields(fields))?,
            allocation_owner: None,
        })
    }
    /// Exact bytes, roles, grammar-bearing declaration and span owner must agree.
    pub fn matches(
        &self,
        row: &Declaration,
        fields: Option<&BTreeMap<String, SourceSpan>>,
    ) -> bool {
        &self.row == row && self.fields.as_ref() == fields
    }
    /// Borrow unbound grammar fields for source dependency extraction.
    pub fn expressions(&self) -> &BTreeMap<OccurrenceKey, CheckedExpression> {
        &self.expressions
    }
    /// Exact signature syntax without byte attribution; a relocated unchanged field
    /// does not invalidate its declaration's expensive type derivation.
    pub fn signature_syntax(&self) -> Self {
        let mut value = self.clone();
        value.row.source_start = 0;
        value.row.source_end = 0;
        value.fields = None;
        for expression in value.expressions.values_mut() {
            expression.source = SourceSpan::head(pse_ids::SemanticId::NIL);
        }
        value
    }
    /// Attach the source-local allocation lifetime, independently of cache residency.
    pub fn with_owner(mut self, owner: Arc<dyn crate::document::DataOwner>) -> Self {
        for expression in self.expressions.values_mut() {
            expression.source_owner = Some(owner.clone());
        }
        self.allocation_owner = Some(owner);
        self
    }
    /// Conservative immutable source/syntax extent for retention accounting.
    pub fn retained_bytes(&self) -> usize {
        use pse_model::HeapUsage;
        self.row.owned_bytes()
            + self.fields.as_ref().map_or(0, |fields| {
                fields
                    .keys()
                    .map(|role| role.capacity() + size_of::<SourceSpan>() + 128)
                    .sum::<usize>()
            })
            + retained_bytes_mode(&self.expressions, true)
            + size_of::<Self>()
            + 256
    }
}

pub(crate) type Occurrences = BTreeMap<OccurrenceKey, CheckedExpression>;
pub(super) enum StaticPart<'a> {
    Expression(&'a dsl::Expr),
    Predicate(&'a dsl::Predicate),
    Name(&'a str),
}
pub(super) fn static_parts<'a>(value: &'a StaticValue, visit: &mut impl FnMut(StaticPart<'a>)) {
    match value {
        StaticValue::Expression(expression) => visit(StaticPart::Expression(expression)),
        StaticValue::Set(values) | StaticValue::Tuple(values) => {
            for value in values {
                static_parts(value, visit);
            }
        }
        StaticValue::Apply { name, arguments } => {
            visit(StaticPart::Name(name));
            for (_, value) in arguments {
                static_parts(value, visit);
            }
        }
        StaticValue::Comprehension {
            body,
            bindings,
            filter,
        } => {
            for (_, domain) in bindings {
                static_parts(domain, visit);
            }
            if let Some(filter) = filter {
                visit(StaticPart::Predicate(filter));
            }
            static_parts(body, visit);
        }
        StaticValue::Text(_) => {}
    }
}

impl CheckedPackage {
    /// Look up a precise authored field occurrence without parsing at use.
    pub fn expression_occurrence(
        &self,
        declaration: DeclarationId,
        role: &str,
        position: usize,
    ) -> Option<&CheckedExpression> {
        self.expressions.get(&OccurrenceKey {
            declaration,
            role: role.into(),
            position,
        })
    }
    /// All retained authored fields and their declaration-local keys.
    pub fn expression_occurrences(
        &self,
    ) -> impl Iterator<Item = (&OccurrenceKey, &CheckedExpression)> {
        self.expressions.iter()
    }
    /// Retained expression at its exact declaration field and repeated-field position.
    /// # Errors
    /// This field is absent or owns another syntax grammar.
    pub fn expression_at(
        &self,
        declaration: DeclarationId,
        role: &str,
        position: usize,
    ) -> Result<&dsl::Expr> {
        match self
            .expression_occurrence(declaration, role, position)
            .map(|value| value.syntax.as_ref())
        {
            Some(
                Syntax::Expression(expression)
                | Syntax::Static(StaticValue::Expression(expression)),
            ) => Ok(expression),
            _ => Err(invalid(
                declaration,
                format!("checked expression occurrence absent: {role}[{position}]"),
            )),
        }
    }
    /// Retained equation at its exact declaration field and repeated-field position.
    /// # Errors
    /// This field is absent or owns another syntax grammar.
    pub fn equation_at(
        &self,
        declaration: DeclarationId,
        role: &str,
        position: usize,
    ) -> Result<&dsl::Equation> {
        match self
            .expression_occurrence(declaration, role, position)
            .map(|value| value.syntax.as_ref())
        {
            Some(Syntax::Equation(equation)) => Ok(equation),
            _ => Err(invalid(
                declaration,
                format!("checked equation occurrence absent: {role}[{position}]"),
            )),
        }
    }
    /// Retained predicate at an exact declaration field and repeated-field position.
    /// # Errors
    /// The field is absent or uses another grammar.
    pub fn predicate_at(
        &self,
        declaration: DeclarationId,
        role: &str,
        position: usize,
    ) -> Result<&dsl::Predicate> {
        match self
            .expression_occurrence(declaration, role, position)
            .map(|value| value.syntax.as_ref())
        {
            Some(Syntax::Predicate(predicate)) => Ok(predicate),
            _ => Err(invalid(
                declaration,
                format!("checked predicate occurrence absent: {role}[{position}]"),
            )),
        }
    }
    /// Retained static syntax for a precise field, including finite index membership.
    /// # Errors
    /// This field is absent or belongs to another syntax grammar.
    pub fn static_at(
        &self,
        declaration: DeclarationId,
        role: &str,
        position: usize,
    ) -> Result<&StaticValue> {
        match self
            .expression_occurrence(declaration, role, position)
            .map(|value| value.syntax.as_ref())
        {
            Some(Syntax::Static(value)) => Ok(value),
            _ => Err(invalid(
                declaration,
                format!("checked static occurrence absent: {role}[{position}]"),
            )),
        }
    }
    /// Exact retained logic field, parsed once while checking its declaration.
    /// # Errors
    /// The declaration does not own a checked logic proposition.
    pub fn proposition_at(&self, declaration: DeclarationId) -> Result<&dsl::Proposition> {
        match self
            .expression_occurrence(declaration, "logic.proposition", 0)
            .map(|value| value.syntax.as_ref())
        {
            Some(Syntax::Logic(value)) => Ok(value),
            _ => Err(invalid(declaration, "checked logic occurrence absent")),
        }
    }
}

struct Collector<'a> {
    row: &'a Declaration,
    values: &'a mut Occurrences,
    indices: Vec<(String, String)>,
    scopes: Vec<usize>,
    fields: Option<&'a BTreeMap<String, SourceSpan>>,
}
impl Collector<'_> {
    fn source_span(&self, role: &str, position: usize) -> Result<SourceSpan> {
        let Some(fields) = self.fields else {
            return Ok(SourceSpan::new(
                self.row.document_id,
                self.row.source_start as u32,
                self.row.source_end as u32,
            ));
        };
        let span = {
            let normalized = |path: &str| {
                path.split('.')
                    .filter(|part| part.parse::<usize>().is_err())
                    .collect::<Vec<_>>()
                    .join(".")
            };
            let role_indices = role
                .split('.')
                .filter_map(|part| part.parse::<usize>().ok())
                .collect::<Vec<_>>();
            let mut exact = fields
                .iter()
                .filter(|(path, _)| normalized(path) == normalized(role))
                .collect::<Vec<_>>();
            let indices = |path: &str| {
                path.split('.')
                    .filter_map(|part| part.parse::<usize>().ok())
                    .collect::<Vec<_>>()
            };
            exact.sort_by_key(|(path, _)| indices(path));
            let mut expected = role_indices.clone();
            expected.push(position);
            exact
                .iter()
                .find(|(path, _)| {
                    (!role_indices.is_empty() && indices(path) == role_indices)
                        || indices(path) == expected
                })
                .map(|(_, span)| **span)
                .or_else(|| {
                    if role_indices.is_empty() {
                        exact.get(position).map(|(_, span)| **span)
                    } else {
                        None
                    }
                })
        };
        span.ok_or_else(|| {
            invalid(
                self.row.declaration_id,
                format!("parser field source range absent: {role}[{position}]"),
            )
        })
    }
    fn failure(
        &self,
        role: &str,
        position: usize,
        message: impl std::fmt::Display,
    ) -> crate::ModelingError {
        let span = match self.source_span(role, position) {
            Ok(span) => span,
            Err(error) => return error,
        };
        crate::ModelingError::Located {
            span,
            name: self.row.name.clone(),
            cause: Box::new(invalid(
                self.row.declaration_id,
                format!("{role}[{position}]: {message}"),
            )),
        }
    }
    fn insert(&mut self, role: &str, position: usize, text: &str, syntax: Syntax) -> Result<()> {
        self.values.insert(
            OccurrenceKey {
                declaration: self.row.declaration_id,
                role: role.into(),
                position,
            },
            CheckedExpression {
                source: self.source_span(role, position)?,
                text: text.into(),
                context: Vec::new(),
                syntax: Arc::new(syntax),
                source_owner: None,
                declared_type: None,
                physical_admissions: BTreeMap::new(),
                dependencies: BTreeSet::new(),
                index_obligations: self.indices.clone(),
                unresolved_references: BTreeSet::new(),
            },
        );
        Ok(())
    }
    fn expression(&mut self, role: &str, position: usize, source: &str) -> Result<()> {
        let syntax = dsl::parse_expr(source).map_err(|e| self.failure(role, position, e))?;
        self.insert(role, position, source, Syntax::Expression(syntax))?;
        Ok(())
    }
    fn predicate(&mut self, role: &str, position: usize, source: &str) -> Result<()> {
        let syntax = dsl::parse_predicate(source).map_err(|e| self.failure(role, position, e))?;
        self.insert(role, position, source, Syntax::Predicate(syntax))?;
        Ok(())
    }
    fn equation_at(&mut self, role: &str, position: usize, source: &str) -> Result<()> {
        let syntax = dsl::parse_equation(source).map_err(|e| self.failure(role, position, e))?;
        self.insert(role, position, source, Syntax::Equation(syntax))?;
        Ok(())
    }
    fn static_value(&mut self, role: &str, position: usize, source: &str) -> Result<()> {
        let syntax = language::parse_static(source).map_err(|e| self.failure(role, position, e))?;
        self.insert(role, position, source, Syntax::Static(syntax))?;
        Ok(())
    }
}

/// Collect through the generated grammar/role/binder inventory; plain strings are not syntax.
pub(crate) fn collect(
    rows: &[Declaration],
    documents: &dyn crate::document::Documents,
) -> Result<Occurrences> {
    let mut values = BTreeMap::new();
    for row in rows {
        if let Some(parsed) = documents.parsed_fields(row.declaration_id)
            && parsed.matches(row, documents.field_spans(row.declaration_id))
        {
            // Only grammar and lexical field/binder attribution are shared. Every
            // checked consumer receives fresh contextual admission/dependency maps.
            values.extend(
                parsed
                    .expressions
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone())),
            );
        } else {
            values.extend(collect_fresh(std::slice::from_ref(row), documents)?);
        }
    }
    Ok(values)
}
impl pse_model::generated::authored::modeling_declarations::SourceExpressionVisitor
    for Collector<'_>
{
    type Error = crate::ModelingError;
    fn expression(
        &mut self,
        grammar: pse_model::generated::authored::modeling_declarations::SourceGrammar,
        role: &str,
        position: usize,
        source: &str,
    ) -> Result<()> {
        use pse_model::generated::authored::modeling_declarations::SourceGrammar;
        match grammar {
            SourceGrammar::Expression => self.expression(role, position, source),
            SourceGrammar::Equation => self.equation_at(role, position, source),
            SourceGrammar::Predicate => self.predicate(role, position, source),
            SourceGrammar::Static => self.static_value(role, position, source),
            SourceGrammar::Proposition => {
                let syntax = dsl::parse_proposition(source)
                    .map_err(|error| self.failure(role, position, error))?;
                self.insert(role, position, source, Syntax::Logic(syntax))
            }
        }
    }
    fn enter_scope(&mut self) {
        self.scopes.push(self.indices.len());
    }
    fn leave_scope(&mut self) {
        if let Some(length) = self.scopes.pop() {
            self.indices.truncate(length);
        }
    }
    fn bind_index(&mut self, name: &str, domain: &str) {
        self.indices.push((name.to_owned(), domain.to_owned()));
    }
}
fn collect_fresh(
    rows: &[Declaration],
    documents: &dyn crate::document::Documents,
) -> Result<Occurrences> {
    let mut values = BTreeMap::new();
    for row in rows {
        if let Some(annotation) = &row.value.annotation {
            crate::annotation::shape(annotation, row.declaration_id)?;
        }
        let mut collector = Collector {
            row,
            values: &mut values,
            indices: Vec::new(),
            scopes: Vec::new(),
            fields: documents.field_spans(row.declaration_id),
        };
        row.value.visit_source_expressions(&mut collector)?;
    }
    Ok(values)
}

/// Bind each occurrence in its own admitted declaration/import context; never claim that
/// unresolved formals or indexed members have already received concrete instantiations.
pub(crate) fn bind(p: &mut CheckedPackage) {
    let values = std::mem::take(&mut p.expressions);
    p.expressions = values
        .into_iter()
        .map(|(key, value)| {
            let value = bind_one(p, &key, value);
            (key, value)
        })
        .collect();
}

pub(crate) fn bind_one(
    p: &CheckedPackage,
    key: &OccurrenceKey,
    mut value: CheckedExpression,
) -> CheckedExpression {
    let mut owner = Some(key.declaration);
    while let Some(id) = owner {
        value.context.push(id);
        owner = p.declarations[&id].parent_id;
    }
    let formals = value
        .context
        .iter()
        .flat_map(|id| {
            p.declarations[id]
                .value
                .scope
                .iter()
                .flat_map(|scope| {
                    scope
                        .parameters
                        .iter()
                        .map(|parameter| parameter.name.clone())
                })
                .chain(
                    p.declarations[id]
                        .value
                        .coordinate_map
                        .iter()
                        .flat_map(|map| map.arguments.iter().map(|argument| argument.name.clone())),
                )
                .chain(
                    p.functions.get(id).into_iter().flat_map(|function| {
                        function.arguments.iter().map(|(name, _)| name.clone())
                    }),
                )
        })
        .chain(value.index_obligations.iter().map(|(name, _)| name.clone()))
        .collect::<BTreeSet<_>>();
    value.declared_type = p.types.get(&key.declaration).cloned();
    let mut paths = BTreeMap::new();
    let mut visit = |path: &dsl::Path| {
        let text = dsl::render_path(path);
        let mut prefixes = Vec::new();
        let mut resolved = None;
        if path
            .segments
            .first()
            .is_some_and(|segment| !formals.contains(&segment.name))
        {
            for end in (1..=path.segments.len()).rev() {
                resolved =
                    resolved.or_else(|| p.resolve_segments(key.declaration, &path.segments[..end]));
                prefixes.push(
                    path.segments[..end]
                        .iter()
                        .map(|segment| segment.name.as_str())
                        .collect::<Vec<_>>()
                        .join("."),
                );
            }
        }
        paths.insert(text, (resolved, prefixes));
    };
    fn expression_paths(expression: &dsl::Expr, visit: &mut impl FnMut(&dsl::Path)) {
        for path in super::source_paths(expression) {
            visit(path);
        }
    }
    fn predicate_paths(predicate: &dsl::Predicate, visit: &mut impl FnMut(&dsl::Path)) {
        use dsl::PredicateKind as K;
        match &predicate.kind {
            K::Compare { lhs, rhs, .. } => {
                expression_paths(lhs, visit);
                expression_paths(rhs, visit);
            }
            K::In { expr, domain } => {
                expression_paths(expr, visit);
                visit(domain);
            }
            K::Atom(e) => expression_paths(e, visit),
            K::And(a, b) | K::Or(a, b) => {
                predicate_paths(a, visit);
                predicate_paths(b, visit);
            }
            K::Not(p) => predicate_paths(p, visit),
            _ => {}
        }
    }
    fn equation(e: &dsl::Equation, visit: &mut impl FnMut(&dsl::Path)) {
        match &e.kind {
            dsl::EquationKind::Relation { lhs, rhs, .. } => {
                expression_paths(lhs, visit);
                expression_paths(rhs, visit);
            }
            dsl::EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                predicate_paths(guard, visit);
                equation(then, visit);
                equation(otherwise, visit);
            }
        }
    }
    fn static_paths(
        value: &StaticValue,
        bound: &BTreeSet<String>,
        visit: &mut impl FnMut(&dsl::Path),
    ) {
        match value {
            StaticValue::Expression(expression) => expression_paths(expression, &mut |path| {
                if path
                    .segments
                    .first()
                    .is_some_and(|first| !bound.contains(&first.name))
                {
                    visit(path);
                }
            }),
            StaticValue::Set(values) | StaticValue::Tuple(values) => {
                for value in values {
                    static_paths(value, bound, visit);
                }
            }
            StaticValue::Apply { arguments, .. } => {
                for (_, value) in arguments {
                    static_paths(value, bound, visit);
                }
            }
            StaticValue::Comprehension {
                body,
                bindings,
                filter,
            } => {
                let mut local = bound.clone();
                for (name, domain) in bindings {
                    static_paths(domain, &local, visit);
                    local.insert(name.clone());
                }
                if let Some(filter) = filter {
                    predicate_paths(filter, &mut |path| {
                        if path
                            .segments
                            .first()
                            .is_some_and(|first| !local.contains(&first.name))
                        {
                            visit(path);
                        }
                    });
                }
                static_paths(body, &local, visit);
            }
            StaticValue::Text(_) => {}
        }
    }
    match value.syntax.as_ref() {
        Syntax::Expression(e) => expression_paths(e, &mut visit),
        Syntax::Predicate(e) => predicate_paths(e, &mut visit),
        Syntax::Logic(p) => {
            p.expressions(&mut |expression| expression_paths(expression, &mut visit))
        }
        Syntax::Equation(e) => equation(e, &mut visit),
        Syntax::Static(syntax) => static_paths(syntax, &BTreeSet::new(), &mut visit),
    }
    for (path, (resolved, prefixes)) in paths {
        if let Some(id) = resolved {
            value.dependencies.insert(id);
        } else if !matches!(path.as_str(), "true" | "false" | "missing")
            && !prefixes
                .iter()
                .any(|name| p.physical_name(key.declaration, name).is_some())
        {
            value.unresolved_references.insert(path);
        }
    }
    let mut names = BTreeSet::new();
    if let Syntax::Static(syntax) = value.syntax.as_ref() {
        static_parts(syntax, &mut |part| {
            if let StaticPart::Name(name) = part {
                names.insert(name.to_owned());
            }
        });
    }
    value.dependencies.extend(
        names
            .iter()
            .filter(|name| !formals.contains(*name))
            .filter_map(|name| p.resolve(key.declaration, name)),
    );
    value
}

pub(crate) fn retained_bytes(values: &Occurrences) -> usize {
    retained_bytes_mode(values, false)
}
fn retained_bytes_mode(values: &Occurrences, include_shared_syntax: bool) -> usize {
    values
        .iter()
        .map(|(key, value)| field_retained_bytes(key, value, include_shared_syntax))
        .sum()
}
/// Conservative known storage of one immutable attributed field, including its syntax.
pub fn field_retained_bytes(
    key: &OccurrenceKey,
    value: &CheckedExpression,
    include_shared_syntax: bool,
) -> usize {
    let syntax = if !include_shared_syntax && value.source_owner.is_some() {
        0
    } else {
        match value.syntax.as_ref() {
            Syntax::Expression(e) | Syntax::Static(StaticValue::Expression(e)) => {
                super::retained_bytes(e)
            }
            Syntax::Predicate(e) => crate::extent::predicate(e),
            Syntax::Equation(e) => crate::extent::equation(e),
            Syntax::Static(e) => static_bytes(e),
            Syntax::Logic(p) => {
                let mut bytes = 0;
                p.expressions(&mut |expression| {
                    bytes += super::retained_bytes(expression) + size_of::<dsl::Proposition>()
                });
                bytes
            }
        }
    };
    size_of::<(OccurrenceKey, CheckedExpression)>()
        + 64
        + value.declared_type.as_ref().map_or(0, crate::extent::ty)
        + value
            .physical_admissions
            .values()
            .map(|admission| {
                size_of::<(
                    super::admission::ExpressionOccurrence,
                    super::admission::PhysicalAdmission,
                )>() + 64
                    + admission.retained_bytes()
            })
            .sum::<usize>()
        + key.role.capacity()
        + value.text.capacity()
        + syntax
        + value.context.capacity() * size_of::<DeclarationId>()
        + value.dependencies.len() * (size_of::<DeclarationId>() + 64)
        + value
            .index_obligations
            .iter()
            .map(|(n, d)| size_of::<(String, String)>() + n.capacity() + d.capacity())
            .sum::<usize>()
        + value
            .unresolved_references
            .iter()
            .map(|n| size_of::<String>() + 64 + n.capacity())
            .sum::<usize>()
}

fn static_bytes(value: &StaticValue) -> usize {
    size_of::<StaticValue>()
        + match value {
            StaticValue::Text(text) => text.capacity(),
            StaticValue::Expression(e) => super::retained_bytes(e),
            StaticValue::Set(values) | StaticValue::Tuple(values) => {
                values.capacity() * size_of::<StaticValue>()
                    + values.iter().map(static_bytes).sum::<usize>()
            }
            StaticValue::Apply { name, arguments } => {
                name.capacity()
                    + arguments.capacity() * size_of::<(String, StaticValue)>()
                    + arguments
                        .iter()
                        .map(|(n, v)| n.capacity() + static_bytes(v))
                        .sum::<usize>()
            }
            StaticValue::Comprehension {
                body,
                bindings,
                filter,
            } => {
                static_bytes(body)
                    + bindings.capacity() * size_of::<(String, StaticValue)>()
                    + bindings
                        .iter()
                        .map(|(n, v)| n.capacity() + static_bytes(v))
                        .sum::<usize>()
                    + filter.as_ref().map_or(0, crate::extent::predicate)
            }
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct SourceFields(BTreeMap<DeclarationId, BTreeMap<String, SourceSpan>>);
    impl crate::document::Documents for SourceFields {
        fn field_spans(&self, declaration: DeclarationId) -> Option<&BTreeMap<String, SourceSpan>> {
            self.0.get(&declaration)
        }
        fn resolve(&self, _: pse_ids::SemanticId, _: &str) -> Option<pse_ids::SemanticId> {
            None
        }
        fn admit(
            &self,
            plan: &Arc<crate::document::DocumentPlan>,
            document: pse_ids::SemanticId,
        ) -> Result<Arc<crate::document::DocumentTable>> {
            crate::document::Documents::admit(&crate::document::NoDocuments, plan, document)
        }
    }
    fn try_checked(text: &str) -> Result<CheckedPackage> {
        let (registry, _) = crate::kernel_types::physical();
        let preconditions = pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap();
        crate::check(
            &crate::kernel_types::source(text),
            &crate::TypeContext {
                admissions: None,
                formula_authority: None,
                quantities: &registry,
                preconditions: &preconditions,
                scope: &crate::PhysicalScope::default(),
            },
        )
    }
    fn checked(text: &str) -> CheckedPackage {
        try_checked(text).unwrap()
    }
    #[test]
    fn intrinsic_callees_do_not_hide_equally_named_value_references() {
        let expression = dsl::parse_expr("size(items)+size").unwrap();
        assert_eq!(
            super::super::references(&expression),
            BTreeSet::from(["items".into(), "size".into()])
        );
        let package = checked(
            "package p {entity kind item {} entity item a {} set items:Set<item>={a}; fn count()->Integer=size(items);}",
        );
        let function = package.entry("p.count").unwrap();
        let occurrence = package
            .expression_occurrence(function, "function.body", 0)
            .unwrap();
        assert!(occurrence.unresolved_references.is_empty());
    }

    #[test]
    fn quoted_function_callees_bind_their_actual_declaration() {
        let package = checked(
            "package p {fn 'law table'<Q>(x:Q)->Q=x; def D {var x:Flow; eq e:'law table'(x)==x;} }",
        );
        let equation = package.entry("p.D.e").unwrap();
        let function = package.entry("p.law table").unwrap();
        let occurrence = package
            .expression_occurrence(equation, "equation.expression", 0)
            .unwrap();
        assert!(occurrence.dependencies.contains(&function));
        let dsl::EquationKind::Relation { lhs, .. } = &package
            .equation_at(equation, "equation.expression", 0)
            .unwrap()
            .kind
        else {
            panic!("retained relation")
        };
        let dsl::ExprKind::NamedCall { name, .. } = &lhs.kind else {
            panic!("retained typed callee")
        };
        assert_eq!(name.ident(), Some("law table"));
        crate::specialize(
            &package,
            package.entry("p.D").unwrap(),
            crate::InstanceId::from_id(pse_ids::SemanticId::NIL),
            &crate::Bindings::default(),
            crate::Limits::default(),
        )
        .unwrap();
        let dotted =
            "package p {fn 'law.table'<Q>(x:Q)->Q=x; def D {var x:Flow; eq e:'law.table'(x)==x;} }";
        let package = checked(dotted);
        crate::specialize(
            &package,
            package.entry("p.D").unwrap(),
            crate::InstanceId::from_id(pse_ids::SemanticId::NIL),
            &crate::Bindings::default(),
            crate::Limits::default(),
        )
        .unwrap();
        assert!(try_checked(&dotted.replace("'law.table'(x)", "law.table(x)")).is_err());
        let recursive = try_checked("package p {fn 'law.table'(x:Scalar)->Scalar='law.table'(x);}")
            .unwrap_err();
        assert!(
            recursive
                .to_string()
                .contains("recursive package function expansion")
        );
        checked(
            "package p {fn 'law.table'(x:Scalar)->Scalar=x; fn consumer('law.table':Fn(x:Scalar)->Scalar,x:Scalar)->Scalar='law.table'(x);}",
        );
    }

    #[test]
    fn source_mask_refuses_plain_formal_names_and_uses_declared_expression_fields() {
        let mut rows = crate::kernel_types::source("package p {fn law(x:Scalar)->Scalar=x;}");
        let function = rows
            .iter_mut()
            .find(|row| row.value.function.is_some())
            .unwrap();
        assert_eq!(
            function
                .value
                .mask_source_expressions(|_, role, position, text| role
                    == "function.arguments.name"
                    && position == 0
                    && text == "x"),
            0
        );
        assert_eq!(
            function.value.function.as_ref().unwrap().arguments[0].name,
            "x"
        );
        assert_eq!(
            function
                .value
                .mask_source_expressions(|_, role, position, text| role == "function.body"
                    && position == 0
                    && text == "x"),
            1
        );
        assert_eq!(
            function.value.function.as_ref().unwrap().body.as_deref(),
            Some("")
        );
    }

    #[test]
    fn generated_source_inventory_visits_domains_before_binding_and_restores_nested_siblings() {
        let rows = crate::kernel_types::source(
            r#"package p {def D {
            state s[i in Outer] supplied(true) {
                coordinate first[j in Members[i],k in Members[j]]=x[i,j,k];
                coordinate second[m in Members[i]]=y[i,m];
                reconstruct r[n in Members[i]]:z[i,n]==y[i,n] tolerance 1e-8;
                transport t[p in Members[i]]=z[i,p] tolerance 1e-8;
            }
        }}"#,
        );
        let row = rows
            .iter()
            .find(|row| row.value.state_specification.is_some())
            .unwrap();
        let parsed = ParsedFields::parse(row, None).unwrap();
        let at = |role: &str, position| {
            &parsed.expressions()[&OccurrenceKey {
                declaration: row.declaration_id,
                role: role.into(),
                position,
            }]
        };
        assert!(
            at("state_specification.indices.domain", 0)
                .index_obligations
                .is_empty()
        );
        assert_eq!(
            at("state_specification.coordinates.0.indices.domain", 0)
                .index_obligations
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["i"]
        );
        assert_eq!(
            at("state_specification.coordinates.0.indices.domain", 1)
                .index_obligations
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["i", "j"]
        );
        assert_eq!(
            at("state_specification.coordinates.target", 0)
                .index_obligations
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["i", "j", "k"]
        );
        for (role, position, names) in [
            ("state_specification.coordinates.target", 1, vec!["i", "m"]),
            (
                "state_specification.reconstructions.equation",
                0,
                vec!["i", "n"],
            ),
            (
                "state_specification.transports.expression",
                0,
                vec!["i", "p"],
            ),
        ] {
            assert_eq!(
                at(role, position)
                    .index_obligations
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>(),
                names
            );
        }
    }

    #[test]
    fn generated_source_inventory_preserves_flat_events_and_explicit_nested_schedule_roles() {
        let rows = crate::kernel_types::source(
            r#"package p {test T fixture {
            dof 0; route integrated; procedure integrate;
            mode first; event x-1 direction(rising) tolerance(1e-6) reset(x=2) terminal;
            event x-2 direction(falling) tolerance(2e-6) terminal;
            mode second; event y-3 direction(either) tolerance(3e-6) reset(y=4) terminal;
            integrate samples(0,1) relative(1e-6) normalized_absolute(1e-8) step(1e-4);
            schedule u at(1,2) values(3,4,5);
            schedule v at(6) values(7,8);
        } {}}"#,
        );
        let mut row = rows
            .into_iter()
            .find(|row| {
                row.value
                    .scope
                    .as_ref()
                    .is_some_and(|scope| scope.fixture.is_some())
            })
            .unwrap();
        let parsed = ParsedFields::parse(&row, None).unwrap();
        let at = |role: &str, position| {
            &parsed.expressions()[&OccurrenceKey {
                declaration: row.declaration_id,
                role: role.into(),
                position,
            }]
        };
        assert_eq!(at("scope.fixture.modes.events.guard", 2).text, "y-3");
        assert_eq!(at("scope.fixture.modes.events.tolerance", 2).text, "3e-6");
        assert_eq!(
            at("scope.fixture.modes.1.events.0.reset.0.target", 0).text,
            "y"
        );
        assert_eq!(
            at("scope.fixture.integration.schedules.1.times", 0).text,
            "6"
        );
        assert_eq!(
            at("scope.fixture.integration.schedules.0.values", 2).text,
            "5"
        );
        assert_eq!(
            row.value
                .mask_source_expressions(|_, role, position, text| role
                    == "scope.fixture.modes.events.guard"
                    && position == 2
                    && text == "y-3"),
            1
        );
        let fixture = row.value.scope.as_ref().unwrap().fixture.as_ref().unwrap();
        assert_eq!(fixture.modes[1].events[0].guard, "");
        assert_eq!(fixture.modes[0].events[0].guard, "x-1");
        assert_eq!(fixture.modes[0].name, "first");
    }

    #[test]
    fn production_source_fields_all_keep_parser_attribution() {
        let sources = [
            include_str!("../../../../packages/reference/physical/models/math.pse"),
            include_str!("../../../../packages/reference/thermodynamics/models/peng-robinson.pse"),
            include_str!("../../../../packages/reference/campaign/models/cstr-dynamics.pse"),
            include_str!("../../../../packages/reference/campaign/models/flash-diagnostics.pse"),
        ];
        for (index, text) in sources.into_iter().enumerate() {
            let document = pse_ids::SemanticId::from_bytes([index as u8 + 1; 16]);
            let (rows, spans) = language::parse_with_spans(
                text,
                document,
                language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let occurrences = collect(&rows, &SourceFields(spans)).unwrap();
            assert!(!occurrences.is_empty());
            for occurrence in occurrences.values() {
                assert_eq!(occurrence.source.document_id, document);
                assert_eq!(
                    &text[occurrence.source.start as usize..occurrence.source.end as usize],
                    occurrence.text
                );
            }
        }
    }

    #[test]
    fn parser_field_spans_survive_checking_repeated_inherited_fields() {
        let text = "package p {interface Base {var x:Scalar;} def D extends Base {eq first:x==x; eq second:x==x;} }";
        let document = pse_ids::SemanticId::from_bytes([19; 16]);
        let (rows, field_spans) = language::parse_with_spans(
            text,
            document,
            language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let quantities = pse_quantity::standard::standard_registry().unwrap();
        let preconditions = pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap();
        let documents = SourceFields(field_spans);
        let package = crate::check_with(
            &rows,
            &crate::TypeContext {
                admissions: None,
                formula_authority: None,
                quantities: &quantities,
                preconditions: &preconditions,
                scope: &crate::PhysicalScope::default(),
            },
            &documents,
        )
        .unwrap();
        let first = package.entry("p.D.first").unwrap();
        let second = package.entry("p.D.second").unwrap();
        let inherited = package.entry("p.Base.x").unwrap();
        let occurrences = [first, second].map(|id| {
            package
                .expression_occurrence(id, "equation.expression", 0)
                .unwrap()
        });
        for occurrence in occurrences {
            assert_eq!(occurrence.source.document_id, document);
            assert_eq!(
                &text[occurrence.source.start as usize..occurrence.source.end as usize],
                "x==x"
            );
            assert!(occurrence.dependencies.contains(&inherited));
        }
        assert_ne!(occurrences[0].source, occurrences[1].source);
        assert_eq!(occurrences[0].context[0], first);
        assert_eq!(occurrences[1].context[0], second);
    }

    #[test]
    fn checked_occurrences_retain_roles_source_and_unresolved_index_bindings() {
        let p = checked(
            "package p { entity kind item {} entity item a {} entity item b {} set Item:Set<item>={a,b}; def D {var x[j in Item]:Scalar; eq first[j in Item]:x[j]==1; eq second[j in Item]:x[j]==1;} }",
        );
        let first = p.entry("p.D.first").unwrap();
        let second = p.entry("p.D.second").unwrap();
        let a = p
            .expression_occurrence(first, "equation.expression", 0)
            .unwrap();
        let b = p
            .expression_occurrence(second, "equation.expression", 0)
            .unwrap();
        assert_eq!(a.text, b.text);
        assert_ne!(a.source, b.source);
        assert_eq!(a.context[0], first);
        assert_eq!(b.context[0], second);
        assert_eq!(a.index_obligations, [("j".into(), "Item".into())]);
        assert!(a.unresolved_references.contains("j"));
        assert!(std::ptr::eq(
            p.equation_at(first, "equation.expression", 0).unwrap(),
            match a.syntax.as_ref() {
                Syntax::Equation(e) => e,
                _ => panic!("equation"),
            }
        ));
    }
    #[test]
    fn checked_occurrences_keep_inherited_default_lexical_context() {
        let p = checked("package p {interface Base {param amount:Scalar=1;} def Derived:Base {} }");
        let id = p.entry("p.Base.amount").unwrap();
        let inherited = p.members[&p.entry("p.Derived").unwrap()]["amount"];
        assert_eq!(inherited, id);
        let occurrence = p
            .expression_occurrence(id, "binding.expression", 0)
            .unwrap();
        assert_eq!(
            occurrence.context,
            [id, p.entry("p.Base").unwrap(), p.entry("p").unwrap()]
        );
        assert!(p.expression_at(inherited, "binding.expression", 0).is_ok());
    }
    #[test]
    fn checked_occurrences_state_slot_indices_and_material_connectivity_have_no_scalar_port_type() {
        let p = checked(
            "package p {entity kind item {} entity item a {} entity item b {} set Item:Set<item>={a,b}; def D {var x[j in Item]:Scalar; state s supplied(true) {coordinate amount[k in Item]=x[k]; transport total=sum(k in Item | x[k]) tolerance 1e-8;} material port inlet=s; annotation connectivity inlet(0,1);} }",
        );
        let state = p.entry("p.D.s").unwrap();
        let occurrence = p
            .expression_occurrence(state, "state_specification.coordinates.target", 0)
            .unwrap();
        assert_eq!(occurrence.index_obligations, [("k".into(), "Item".into())]);
        assert!(occurrence.dependencies.contains(&p.entry("p.D.x").unwrap()));
        let transport = p
            .expression_occurrence(state, "state_specification.transports.expression", 0)
            .unwrap();
        assert!(!transport.unresolved_references.contains("k"));
        assert!(transport.dependencies.contains(&p.entry("p.D.x").unwrap()));
        assert!(!p.types.contains_key(&p.entry("p.D.inlet").unwrap()));
        assert!(!p.types.contains_key(&state));
    }
    #[test]
    fn malformed_repeated_fields_keep_the_exact_parser_range() {
        for owner in ["first", "second"] {
            let first = if owner == "first" { "1 +" } else { "1" };
            let second = if owner == "second" { "1 +" } else { "1" };
            let text = format!(
                "package p {{def D {{param first:Scalar={first}; param second:Scalar={second};}} }}"
            );
            let document = pse_ids::SemanticId::from_bytes([20; 16]);
            let (rows, field_spans) = language::parse_with_spans(
                &text,
                document,
                language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let row = rows.iter().find(|row| row.name == owner).unwrap();
            let expected = field_spans[&row.declaration_id]["binding.expression"];
            let documents = SourceFields(field_spans);
            let error = collect(&rows, &documents).unwrap_err();
            let crate::ModelingError::Located { span, name, .. } = error else {
                panic!("missing exact field location")
            };
            assert_eq!(name, owner);
            assert_eq!(span, expected);
            assert_eq!(&text[span.start as usize..span.end as usize], "1 +");
        }
    }

    #[test]
    fn checked_occurrences_malformed_repeated_fields_name_each_owning_declaration() {
        let source = crate::kernel_types::source(
            "package p {def D {param first:Scalar=1; param second:Scalar=1;} }",
        );
        for name in ["first", "second"] {
            let mut rows = source.clone();
            let row = rows.iter_mut().find(|row| row.name == name).unwrap();
            let id = row.declaration_id;
            row.value.binding.as_mut().unwrap().expression = Some("1 +".into());
            let error = collect(&rows, &crate::document::NoDocuments).unwrap_err();
            assert!(error.to_string().contains(&id.to_string()));
            assert!(error.to_string().contains("binding.expression[0]"));
        }
    }
    #[test]
    fn shared_parsed_fields_reuse_grammar_but_rebind_context_and_exact_spans() {
        struct Shared(
            BTreeMap<DeclarationId, Arc<ParsedFields>>,
            BTreeMap<DeclarationId, BTreeMap<String, SourceSpan>>,
        );
        impl crate::document::Documents for Shared {
            fn parsed_fields(&self, id: DeclarationId) -> Option<&Arc<ParsedFields>> {
                self.0.get(&id)
            }
            fn field_spans(&self, id: DeclarationId) -> Option<&BTreeMap<String, SourceSpan>> {
                self.1.get(&id)
            }
            fn resolve(&self, _: pse_ids::SemanticId, _: &str) -> Option<pse_ids::SemanticId> {
                None
            }
            fn admit(
                &self,
                plan: &Arc<crate::document::DocumentPlan>,
                id: pse_ids::SemanticId,
            ) -> Result<Arc<crate::document::DocumentTable>> {
                crate::document::Documents::admit(&crate::document::NoDocuments, plan, id)
            }
        }
        let text = "package p {param a:Scalar=1; def D {var x:Scalar; eq e:x==a;}}";
        let (rows, spans) = language::parse_with_spans(
            text,
            pse_ids::SemanticId::NIL,
            language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let fields = rows
            .iter()
            .map(|row| {
                (
                    row.declaration_id,
                    Arc::new(
                        ParsedFields::parse(row, spans.get(&row.declaration_id))
                            .unwrap()
                            .with_owner(Arc::new(())),
                    ),
                )
            })
            .collect();
        let mut documents = Shared(fields, spans);
        let equation = rows.iter().find(|row| row.name == "e").unwrap();
        let key = OccurrenceKey {
            declaration: equation.declaration_id,
            role: "equation.expression".into(),
            position: 0,
        };
        let key = documents.0[&equation.declaration_id]
            .expressions
            .keys()
            .next()
            .unwrap_or(&key)
            .clone();
        let syntax = documents.0[&equation.declaration_id].expressions[&key]
            .syntax
            .clone();
        let mut first = collect(&rows, &documents).unwrap();
        assert!(Arc::ptr_eq(&syntax, &first[&key].syntax));
        first
            .get_mut(&key)
            .unwrap()
            .dependencies
            .insert(rows[0].declaration_id);
        let second = collect(&rows, &documents).unwrap();
        assert!(
            second[&key].dependencies.is_empty(),
            "resolved maps are per contextual check"
        );
        assert!(
            second[&key].source_owner.is_some(),
            "detached grammar retains its allocation owner"
        );
        let (quantities, _) = crate::kernel_types::physical();
        let preconditions = pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap();
        let context = crate::TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &quantities,
            preconditions: &preconditions,
            scope: &crate::PhysicalScope::default(),
        };
        let checked = crate::check_with(&rows, &context, &documents).unwrap();
        assert!(Arc::ptr_eq(
            &syntax,
            &checked
                .expression_occurrence(key.declaration, &key.role, key.position)
                .unwrap()
                .syntax
        ));
        let mut without_dependency = rows.clone();
        without_dependency.retain(|row| row.name != "a");
        assert!(
            crate::check_with(&without_dependency, &context, &documents).is_err(),
            "unchanged AST cannot retain a deleted contextual binding"
        );
        let location = documents
            .1
            .get_mut(&equation.declaration_id)
            .unwrap()
            .values_mut()
            .next()
            .unwrap();
        location.start += 1;
        let different_span = collect(&rows, &documents).unwrap();
        assert!(
            !Arc::ptr_eq(&syntax, &different_span[&key].syntax),
            "equal mathematical syntax cannot retain another source span owner"
        );
        let mut changed = rows.clone();
        changed
            .iter_mut()
            .find(|row| row.name == "e")
            .unwrap()
            .value
            .equation
            .as_mut()
            .unwrap()
            .expression = "x==2".into();
        let changed = collect(&changed, &documents).unwrap();
        assert!(!Arc::ptr_eq(&syntax, &changed[&key].syntax));
    }
}
