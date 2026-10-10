// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked source syntax with declaration-local roles and immutable lexical attribution.
use crate::{CheckedPackage, Declaration, DeclarationId, Result, invalid};
use pse_authoring::{
    SourceSpan, dsl,
    language::{self, StaticValue},
};
use std::collections::{BTreeMap, BTreeSet};

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
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedExpression {
    /// Exact parser field range when available; otherwise the programmatic declaration range.
    pub source: SourceSpan,
    /// Exact authored field text, independent of normalized mathematical syntax.
    pub text: String,
    /// Lexical declaration chain, innermost first, retaining import context.
    pub context: Vec<DeclarationId>,
    /// Parsed once before semantic checking and retained for consumers.
    pub syntax: Syntax,
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
            .map(|value| &value.syntax)
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
            .map(|value| &value.syntax)
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
            .map(|value| &value.syntax)
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
            .map(|value| &value.syntax)
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
            .map(|value| &value.syntax)
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
                syntax,
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
    fn equation(&mut self, role: &str, source: &str) -> Result<()> {
        self.equation_at(role, 0, source)
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

/// Collect field roles explicitly: interchange strings that are labels or types are not expressions.
pub(crate) fn collect(
    rows: &[Declaration],
    documents: &dyn crate::document::Documents,
) -> Result<Occurrences> {
    let mut values = BTreeMap::new();
    for row in rows {
        let v = &row.value;
        let mut c = Collector {
            row,
            values: &mut values,
            indices: Vec::new(),
            fields: documents.field_spans(row.declaration_id),
        };
        macro_rules! indices {
            ($value:expr, $role:expr) => {
                for (position, index) in $value.indices.iter().enumerate() {
                    c.static_value(
                        &format!("{}.indices.domain", $role),
                        position,
                        &index.domain,
                    )?;
                    c.indices.push((index.name.clone(), index.domain.clone()));
                }
            };
        }
        if let Some(b) = &v.binding {
            indices!(b, "binding");
            if let Some(source) = &b.expression {
                c.static_value("binding.expression", 0, source)?;
            }
            if let Some(source) = &b.defined_by {
                c.equation("binding.defined_by", source)?;
            }
        }
        if let Some(e) = &v.equation {
            indices!(e, "equation");
            c.equation("equation.expression", &e.expression)?;
            if let Some(condition) = &e.condition {
                c.expression("equation.condition.variable", 0, &condition.variable)?;
            }
        }
        if let Some(f) = &v.function {
            if let Some(source) = &f.body {
                c.expression("function.body", 0, source)?;
            }
            if let Some(source) = &f.validity {
                c.predicate("function.validity", 0, source)?;
            }
            for (i, source) in f.applicability.iter().enumerate() {
                c.expression("function.applicability", i, source)?;
            }
            if let Some(external) = &f.external {
                c.expression("function.external.output", 0, &external.output)?;
            }
        }
        if let Some(s) = &v.scope {
            if let Some(fixture) = &s.fixture {
                for (position, event) in fixture
                    .modes
                    .iter()
                    .flat_map(|mode| &mode.events)
                    .enumerate()
                {
                    c.expression("scope.fixture.modes.events.guard", position, &event.guard)?;
                    c.expression(
                        "scope.fixture.modes.events.tolerance",
                        position,
                        &event.tolerance,
                    )?;
                }
                for (mode, value) in fixture.modes.iter().enumerate() {
                    for (event, value) in value.events.iter().enumerate() {
                        for (reset, value) in value.reset.iter().enumerate() {
                            c.expression(
                                &format!(
                                    "scope.fixture.modes.{mode}.events.{event}.reset.{reset}.target"
                                ),
                                0,
                                &value.target,
                            )?;
                            c.expression(&format!("scope.fixture.modes.{mode}.events.{event}.reset.{reset}.expression"), 0, &value.expression)?;
                        }
                    }
                }
                if let Some(integration) = &fixture.integration {
                    for (position, source) in integration.samples.iter().enumerate() {
                        c.expression("scope.fixture.integration.samples", position, source)?;
                    }
                    c.expression(
                        "scope.fixture.integration.initial_step",
                        0,
                        &integration.initial_step,
                    )?;
                    for (position, value) in integration.quadratures.iter().enumerate() {
                        c.expression(
                            "scope.fixture.integration.quadratures.target",
                            position,
                            &value.target,
                        )?;
                        c.expression(
                            "scope.fixture.integration.quadratures.absolute_tolerance",
                            position,
                            &value.absolute_tolerance,
                        )?;
                    }
                    for (schedule, value) in integration.schedules.iter().enumerate() {
                        c.expression(
                            &format!("scope.fixture.integration.schedules.{schedule}.target"),
                            0,
                            &value.target,
                        )?;
                        for (position, source) in value.times.iter().enumerate() {
                            c.expression(
                                &format!("scope.fixture.integration.schedules.{schedule}.times"),
                                position,
                                source,
                            )?;
                        }
                        for (position, source) in value.values.iter().enumerate() {
                            c.expression(
                                &format!("scope.fixture.integration.schedules.{schedule}.values"),
                                position,
                                source,
                            )?;
                        }
                        if let Some(source) = &value.lower {
                            c.expression(
                                &format!("scope.fixture.integration.schedules.{schedule}.lower"),
                                0,
                                source,
                            )?;
                        }
                        if let Some(source) = &value.upper {
                            c.expression(
                                &format!("scope.fixture.integration.schedules.{schedule}.upper"),
                                0,
                                source,
                            )?;
                        }
                    }
                }
                if let Some(shooting) = &fixture.shooting {
                    for (position, source) in shooting.nodes.iter().enumerate() {
                        c.expression("scope.fixture.shooting.nodes", position, source)?;
                    }
                }
                for (position, value) in fixture.specifications.iter().enumerate() {
                    c.expression(
                        "scope.fixture.specifications.target",
                        position,
                        &value.target,
                    )?;
                    if let Some(source) = &value.expression {
                        c.expression("scope.fixture.specifications.expression", position, source)?;
                    }
                }
                for (diagnostic, value) in fixture.diagnostics.iter().enumerate() {
                    for (position, source) in value.members.iter().enumerate() {
                        c.expression(
                            &format!("scope.fixture.diagnostics.{diagnostic}.members"),
                            position,
                            source,
                        )?;
                    }
                }
                if let Some(expected) = &fixture.expected_failure {
                    for (position, source) in expected.members.iter().enumerate() {
                        c.expression("scope.fixture.expected_failure.members", position, source)?;
                    }
                    if let Some(applicability) = &expected.applicability {
                        for (position, source) in applicability.variables.iter().enumerate() {
                            c.expression(
                                "scope.fixture.expected_failure.applicability.variables",
                                position,
                                source,
                            )?;
                        }
                    }
                    if let Some(validity) = &expected.validity {
                        for (position, source) in validity.variables.iter().enumerate() {
                            c.expression(
                                "scope.fixture.expected_failure.validity.variables",
                                position,
                                source,
                            )?;
                        }
                    }
                    for (role, values) in [
                        (
                            "applicability",
                            expected.applicability.as_ref().map(|v| &v.sets),
                        ),
                        ("validity", expected.validity.as_ref().map(|v| &v.sets)),
                    ] {
                        if let Some(values) = values {
                            for (position, source) in values.iter().enumerate() {
                                c.static_value(
                                    &format!("scope.fixture.expected_failure.{role}.sets"),
                                    position,
                                    source,
                                )?;
                            }
                        }
                    }
                }
            }
            for (i, parameter) in s.parameters.iter().enumerate() {
                if let Some(source) = &parameter.default_value {
                    c.static_value("scope.parameters.default_value", i, source)?;
                }
            }
            if let Some(branch) = &s.branch {
                c.predicate("scope.branch", 0, branch)?;
            }
            if let Some(operation) = &s.operational {
                for (position, anchor) in operation.anchors.iter().enumerate() {
                    c.expression("scope.operational.anchors.target", position, &anchor.target)?;
                    c.expression(
                        "scope.operational.anchors.expression",
                        position,
                        &anchor.expression,
                    )?;
                }
                if let Some(neighborhood) = &operation.neighborhood {
                    c.predicate("scope.operational.neighborhood", 0, neighborhood)?;
                }
            }
            if let Some(selection) = &s.selection {
                c.expression("scope.selection.criterion", 0, &selection.criterion)?;
                c.expression("scope.selection.tolerance", 0, &selection.tolerance)?;
            }
            if let Some(source) = &s.eligibility {
                c.predicate("scope.eligibility", 0, source)?;
            }
        }
        if let Some(g) = &v.guard {
            c.predicate("guard.predicate", 0, &g.predicate)?;
        }
        if let Some(r) = &v.requirement {
            c.predicate("requirement.predicate", 0, &r.predicate)?;
        }
        if let Some(a) = &v.accumulator {
            indices!(a, "accumulator");
            c.expression("accumulator.tolerance", 0, &a.tolerance)?;
            if let Some(source) = &a.boundary {
                c.expression("accumulator.boundary", 0, source)?;
            }
        }
        if let Some(a) = &v.contribution {
            indices!(a, "contribution");
            c.expression("contribution.target", 0, &a.target)?;
            c.expression("contribution.expression", 0, &a.expression)?;
        }
        if let Some(a) = &v.connection {
            indices!(a, "connection");
            c.expression("connection.from", 0, &a.from)?;
            c.expression("connection.to", 0, &a.to)?;
        }
        if let Some(a) = &v.state_specification {
            indices!(a, "state_specification");
            c.expression("state_specification.supplied", 0, &a.supplied)?;
            if let Some(base) = &a.extends {
                c.expression("state_specification.extends", 0, base)?;
            }
            let outer = c.indices.clone();
            for (i, coordinate) in a.coordinates.iter().enumerate() {
                c.indices.clone_from(&outer);
                indices!(coordinate, format!("state_specification.coordinates.{i}"));
                c.expression(
                    "state_specification.coordinates.target",
                    i,
                    &coordinate.target,
                )?;
            }
            for (i, reconstruction) in a.reconstructions.iter().enumerate() {
                c.indices.clone_from(&outer);
                indices!(
                    reconstruction,
                    format!("state_specification.reconstructions.{i}")
                );
                c.equation_at(
                    "state_specification.reconstructions.equation",
                    i,
                    &reconstruction.equation,
                )?;
                c.expression(
                    "state_specification.reconstructions.tolerance",
                    i,
                    &reconstruction.tolerance,
                )?;
            }
            for (i, transport) in a.transports.iter().enumerate() {
                c.indices.clone_from(&outer);
                indices!(transport, format!("state_specification.transports.{i}"));
                c.expression(
                    "state_specification.transports.expression",
                    i,
                    &transport.expression,
                )?;
                c.expression(
                    "state_specification.transports.tolerance",
                    i,
                    &transport.tolerance,
                )?;
            }
            c.indices = outer;
        }
        if let Some(a) = &v.state_port {
            indices!(a, "state_port");
            c.expression("state_port.specification", 0, &a.specification)?;
        }
        if let Some(a) = &v.inventory_balance {
            indices!(a, "inventory_balance");
            c.expression("inventory_balance.axis", 0, &a.axis)?;
            c.expression("inventory_balance.inventory", 0, &a.inventory)?;
            c.expression("inventory_balance.flux", 0, &a.flux)?;
            c.expression("inventory_balance.tolerance", 0, &a.tolerance)?;
            for (i, transfer) in a.transfers.iter().enumerate() {
                c.expression("inventory_balance.transfers.event", i, &transfer.event)?;
                c.expression(
                    "inventory_balance.transfers.expression",
                    i,
                    &transfer.expression,
                )?;
            }
        }
        if let Some(a) = &v.boundary {
            indices!(a, "boundary");
        }
        if let Some(a) = &v.exchange {
            indices!(a, "exchange");
            c.expression("exchange.from", 0, &a.from)?;
            c.expression("exchange.to", 0, &a.to)?;
        }
        if let Some(a) = &v.coordinate_slot {
            indices!(a, "coordinate_slot");
            c.expression("coordinate_slot.expression", 0, &a.expression)?;
        }
        if let Some(a) = &v.coordinate_map
            && let Some(source) = &a.validity
        {
            c.predicate("coordinate_map.validity", 0, source)?;
        }
        if let Some(a) = &v.response {
            c.expression("response.body", 0, &a.body)?;
        }
        if let Some(a) = &v.continuous {
            c.expression("continuous.lower", 0, &a.lower)?;
            c.expression("continuous.upper", 0, &a.upper)?;
        }
        if let Some(a) = &v.discretization {
            c.expression("discretization.elements", 0, &a.elements)?;
            c.expression("discretization.order", 0, &a.order)?;
        }
        if let Some(a) = &v.relaxation {
            c.expression("relaxation.target", 0, &a.target)?;
            c.expression("relaxation.nominal", 0, &a.nominal)?;
        }
        if let Some(a) = &v.continuation {
            c.expression("continuation.target", 0, &a.target)?;
            c.expression("continuation.start", 0, &a.start)?;
            c.expression("continuation.end", 0, &a.end)?;
        }
        if let Some(a) = &v.realization {
            if let Some(function) = &a.function {
                c.expression("realization.function", 0, function)?;
            }
            if let Some(argument) = &a.argument {
                c.expression("realization.argument", 0, argument)?;
            }
        }
        if let Some(a) = &v.ordered_set {
            indices!(a, "ordered_set");
            c.expression("ordered_set.member", 0, &a.member)?;
            c.expression("ordered_set.weight", 0, &a.weight)?;
        }
        if let Some(a) = &v.cardinality {
            indices!(a, "cardinality");
            c.expression("cardinality.member", 0, &a.member)?;
            c.expression("cardinality.count", 0, &a.count)?;
        }
        if let Some(a) = &v.piecewise {
            indices!(a, "piecewise");
            for (role, source) in [
                ("piecewise.input", &a.input),
                ("piecewise.output", &a.output),
                ("piecewise.abscissa", &a.abscissa),
                ("piecewise.ordinate", &a.ordinate),
            ] {
                c.expression(role, 0, source)?;
            }
        }
        if let Some(a) = &v.complementarity {
            indices!(a, "complementarity");
            c.expression("complementarity.first", 0, &a.first)?;
            c.expression("complementarity.second", 0, &a.second)?;
        }
        if let Some(a) = &v.logic {
            indices!(a, "logic");
            let syntax = dsl::parse_proposition(&a.proposition)
                .map_err(|error| c.failure("logic.proposition", 0, error))?;
            c.insert(
                "logic.proposition",
                0,
                &a.proposition,
                Syntax::Logic(syntax),
            )?;
        }
        if let Some(a) = &v.annotation {
            use crate::annotation::Shape;
            if a.kind != pse_model::generated::enums::ModelingAnnotationKind::EngineeringRule {
                c.expression("annotation.target", 0, &a.target)?;
            }
            match crate::annotation::shape(a, row.declaration_id)? {
                Shape::Expressions(count) => {
                    for (i, source) in a.arguments[..count].iter().enumerate() {
                        c.expression("annotation.arguments", i, source)?;
                    }
                }
                Shape::Predicate => c.predicate("annotation.arguments", 0, &a.arguments[0])?,
                Shape::Objective => {
                    if let Some(members) = &a.objective {
                        for (role, source) in [
                            ("weight", &members.weight),
                            ("normalization", &members.normalization),
                            ("absolute_tolerance", &members.absolute_tolerance),
                            ("relative_tolerance", &members.relative_tolerance),
                        ] {
                            if let Some(source) = source {
                                c.expression(&format!("annotation.objective.{role}"), 0, source)?;
                            }
                        }
                    }
                }
                Shape::AccuracyGoal => {
                    if let Some(goal) = &a.accuracy_goal {
                        for (role, source) in [
                            ("time", &goal.time),
                            ("resolution", &goal.resolution),
                            ("criterion_lower", &goal.criterion_lower),
                            ("criterion_upper", &goal.criterion_upper),
                        ] {
                            if let Some(source) = source {
                                c.expression(
                                    &format!("annotation.accuracy_goal.{role}"),
                                    0,
                                    source,
                                )?;
                            }
                        }
                    }
                }
                Shape::EngineeringScale => {
                    if let Some(scale) = &a.engineering_scale {
                        c.expression("annotation.engineering_scale.value", 0, &scale.value)?;
                    }
                }
                Shape::EngineeringDefault | Shape::EngineeringRule => {}
                Shape::Label => c.static_value("annotation.arguments", 0, &a.arguments[0])?,
                _ => {}
            }
        }
        if let Some(a) = &v.attribute
            && let Some(source) = &a.derived
        {
            c.expression("attribute.derived", 0, source)?;
        }
        if let Some(a) = &v.table {
            for (position, column) in a.columns.iter().enumerate() {
                if let Some(source) = &column.derived {
                    c.expression("table.columns.derived", position, source)?;
                }
            }
            for (position, source) in a.requirements.iter().enumerate() {
                c.predicate("table.requirements", position, source)?;
            }
        }
        if let Some(a) = &v.permission {
            for (position, source) in a.targets.iter().enumerate() {
                c.expression("permission.targets", position, source)?;
            }
        }
        if let Some(a) = &v.applicability {
            c.expression("applicability.evidence", 0, &a.evidence)?;
            if let Some(source) = &a.predicate {
                c.predicate("applicability.predicate", 0, source)?;
            }
            for (role, source) in [("axis", &a.axis), ("lower", &a.lower), ("upper", &a.upper)] {
                if let Some(source) = source {
                    c.expression(&format!("applicability.{role}"), 0, source)?;
                }
            }
            for (role, sources) in [
                ("alternatives", &a.alternatives),
                ("dependencies", &a.dependencies),
            ] {
                for (position, source) in sources.iter().enumerate() {
                    c.expression(&format!("applicability.{role}"), position, source)?;
                }
            }
        }
        if let Some(a) = &v.reference_translation {
            c.expression("reference_translation.temperature", 0, &a.temperature)?;
            c.expression("reference_translation.pressure", 0, &a.pressure)?;
        }
        if let Some(a) = &v.reconstruction {
            c.expression("reconstruction.normalization", 0, &a.normalization)?;
        }
        if let Some(a) = &v.expectation {
            c.expression("expectation.actual", 0, &a.actual)?;
            c.expression("expectation.expected", 0, &a.expected)?;
            c.expression("expectation.tolerance", 0, &a.tolerance)?;
            if let Some(source) = &a.relative_tolerance {
                c.expression("expectation.relative_tolerance", 0, source)?;
            }
        }
    }
    Ok(values)
}

/// Bind each occurrence in its own admitted declaration/import context; never claim that
/// unresolved formals or indexed members have already received concrete instantiations.
pub(crate) fn bind(p: &mut CheckedPackage) {
    let mut values = std::mem::take(&mut p.expressions);
    for (key, value) in &mut values {
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
                            .flat_map(|map| {
                                map.arguments.iter().map(|argument| argument.name.clone())
                            }),
                    )
                    .chain(p.functions.get(id).into_iter().flat_map(|function| {
                        function.arguments.iter().map(|(name, _)| name.clone())
                    }))
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
                    resolved = resolved
                        .or_else(|| p.resolve_segments(key.declaration, &path.segments[..end]));
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
        match &value.syntax {
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
        if let Syntax::Static(syntax) = &value.syntax {
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
    }
    p.expressions = values;
}

pub(crate) fn retained_bytes(values: &Occurrences) -> usize {
    values
        .iter()
        .map(|(key, value)| {
            let syntax = match &value.syntax {
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
        })
        .sum()
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
            plan: &std::sync::Arc<crate::document::DocumentPlan>,
            document: pse_ids::SemanticId,
        ) -> Result<std::sync::Arc<crate::document::DocumentTable>> {
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
            match &a.syntax {
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
}
