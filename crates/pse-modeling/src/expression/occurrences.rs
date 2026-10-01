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
    /// Compile-time collections and named applications embed the expression grammar.
    Static(StaticValue),
}

/// Attribution and dependencies for one checked source field.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedExpression {
    /// Complete source declaration range; AST spans remain expression-relative.
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
enum StaticPart<'a> {
    Expression(&'a dsl::Expr),
    Predicate(&'a dsl::Predicate),
    Name(&'a str),
}
fn static_parts<'a>(value: &'a StaticValue, visit: &mut impl FnMut(StaticPart<'a>)) {
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
    pub(crate) fn static_source(
        &self,
        declaration: DeclarationId,
        source: &str,
    ) -> Result<&StaticValue> {
        self.expressions
            .iter()
            .find_map(|(key, value)| match &value.syntax {
                Syntax::Static(syntax)
                    if key.declaration == declaration && value.text == source =>
                {
                    Some(syntax)
                }
                _ => None,
            })
            .ok_or_else(|| {
                invalid(
                    declaration,
                    format!("checked static occurrence absent: {source}"),
                )
            })
    }
    /// Retained expression syntax for an already checked source field.
    /// # Errors
    /// The source is absent or belongs to a different grammar role.
    pub fn expression(&self, declaration: DeclarationId, source: &str) -> Result<&dsl::Expr> {
        self.expressions
            .iter()
            .find_map(|(key, value)| {
                if key.declaration != declaration || value.text != source {
                    return None;
                }
                match &value.syntax {
                    Syntax::Expression(expression)
                    | Syntax::Static(StaticValue::Expression(expression)) => Some(expression),
                    _ => None,
                }
            })
            .ok_or_else(|| {
                invalid(
                    declaration,
                    format!("checked expression occurrence absent: {source}"),
                )
            })
    }
    /// Retained predicate syntax for an already checked source field.
    /// # Errors
    /// No predicate occurrence owns this text at the declaration.
    pub fn predicate(&self, declaration: DeclarationId, source: &str) -> Result<&dsl::Predicate> {
        self.expressions
            .iter()
            .find_map(|(key, value)| match &value.syntax {
                Syntax::Predicate(predicate)
                    if key.declaration == declaration && value.text == source =>
                {
                    Some(predicate)
                }
                _ => None,
            })
            .ok_or_else(|| {
                invalid(
                    declaration,
                    format!("checked predicate occurrence absent: {source}"),
                )
            })
    }
    /// Retained equation syntax for an already checked source field.
    /// # Errors
    /// No equation occurrence owns this text at the declaration.
    pub fn equation(&self, declaration: DeclarationId, source: &str) -> Result<&dsl::Equation> {
        self.expressions
            .iter()
            .find_map(|(key, value)| match &value.syntax {
                Syntax::Equation(equation)
                    if key.declaration == declaration && value.text == source =>
                {
                    Some(equation)
                }
                _ => None,
            })
            .ok_or_else(|| {
                invalid(
                    declaration,
                    format!("checked equation occurrence absent: {source}"),
                )
            })
    }
}

struct Collector<'a> {
    row: &'a Declaration,
    values: &'a mut Occurrences,
    indices: Vec<(String, String)>,
}
impl Collector<'_> {
    fn insert(&mut self, role: &str, position: usize, text: &str, syntax: Syntax) {
        self.values.insert(
            OccurrenceKey {
                declaration: self.row.declaration_id,
                role: role.into(),
                position,
            },
            CheckedExpression {
                source: SourceSpan::new(
                    self.row.document_id,
                    self.row.source_start as u32,
                    self.row.source_end as u32,
                ),
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
    }
    fn expression(&mut self, role: &str, position: usize, source: &str) -> Result<()> {
        let syntax = dsl::parse_expr(source)
            .map_err(|e| invalid(self.row.declaration_id, format!("{role}[{position}]: {e}")))?;
        self.insert(role, position, source, Syntax::Expression(syntax));
        Ok(())
    }
    fn predicate(&mut self, role: &str, position: usize, source: &str) -> Result<()> {
        let syntax = dsl::parse_predicate(source)
            .map_err(|e| invalid(self.row.declaration_id, format!("{role}[{position}]: {e}")))?;
        self.insert(role, position, source, Syntax::Predicate(syntax));
        Ok(())
    }
    fn equation(&mut self, role: &str, source: &str) -> Result<()> {
        self.equation_at(role, 0, source)
    }
    fn equation_at(&mut self, role: &str, position: usize, source: &str) -> Result<()> {
        let syntax = dsl::parse_equation(source)
            .map_err(|e| invalid(self.row.declaration_id, format!("{role}: {e}")))?;
        self.insert(role, position, source, Syntax::Equation(syntax));
        Ok(())
    }
    fn static_value(&mut self, role: &str, position: usize, source: &str) -> Result<()> {
        let syntax = language::parse_static(source)
            .map_err(|e| invalid(self.row.declaration_id, format!("{role}[{position}]: {e}")))?;
        self.insert(role, position, source, Syntax::Static(syntax));
        Ok(())
    }
}

/// Collect field roles explicitly: interchange strings that are labels or types are not expressions.
pub(crate) fn collect(rows: &[Declaration]) -> Result<Occurrences> {
    let mut values = BTreeMap::new();
    for row in rows {
        let v = &row.value;
        let mut c = Collector {
            row,
            values: &mut values,
            indices: Vec::new(),
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
                }
            }
            for (i, parameter) in s.parameters.iter().enumerate() {
                if let Some(source) = &parameter.default_value {
                    c.static_value("scope.parameters.default_value", i, source)?;
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
            c.expression("relaxation.nominal", 0, &a.nominal)?;
        }
        if let Some(a) = &v.continuation {
            c.expression("continuation.start", 0, &a.start)?;
            c.expression("continuation.end", 0, &a.end)?;
        }
        if let Some(a) = &v.ordered_set {
            indices!(a, "ordered_set");
            c.expression("ordered_set.member", 0, &a.member)?;
        }
        if let Some(a) = &v.cardinality {
            indices!(a, "cardinality");
            c.expression("cardinality.member", 0, &a.member)?;
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
        }
        if let Some(a) = &v.annotation {
            c.expression("annotation.target", 0, &a.target)?;
            use crate::annotation::Shape;
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
                _ => {}
            }
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
        let mut bodies = BTreeSet::new();
        fn expression_bodies(expression: &dsl::Expr, bodies: &mut BTreeSet<pse_ids::ContentHash>) {
            expression.walk(|node| {
                bodies.insert(super::admission::ExpressionOccurrence::of(node).body);
            });
        }
        fn predicate_bodies(
            predicate: &dsl::Predicate,
            bodies: &mut BTreeSet<pse_ids::ContentHash>,
        ) {
            use dsl::PredicateKind as K;
            match &predicate.kind {
                K::Compare { lhs, rhs, .. } => {
                    expression_bodies(lhs, bodies);
                    expression_bodies(rhs, bodies);
                }
                K::Atom(e) | K::In { expr: e, .. } => expression_bodies(e, bodies),
                K::And(a, b) | K::Or(a, b) => {
                    predicate_bodies(a, bodies);
                    predicate_bodies(b, bodies);
                }
                K::Not(p) => predicate_bodies(p, bodies),
                _ => {}
            }
        }
        fn equation_bodies(equation: &dsl::Equation, bodies: &mut BTreeSet<pse_ids::ContentHash>) {
            match &equation.kind {
                dsl::EquationKind::Relation { lhs, rhs, .. } => {
                    expression_bodies(lhs, bodies);
                    expression_bodies(rhs, bodies);
                }
                dsl::EquationKind::Conditional {
                    guard,
                    then,
                    otherwise,
                } => {
                    predicate_bodies(guard, bodies);
                    equation_bodies(then, bodies);
                    equation_bodies(otherwise, bodies);
                }
            }
        }
        match &value.syntax {
            Syntax::Expression(e) | Syntax::Static(StaticValue::Expression(e)) => {
                expression_bodies(e, &mut bodies)
            }
            Syntax::Predicate(p) => predicate_bodies(p, &mut bodies),
            Syntax::Equation(e) => equation_bodies(e, &mut bodies),
            Syntax::Static(syntax) => static_parts(syntax, &mut |part| match part {
                StaticPart::Expression(expression) => expression_bodies(expression, &mut bodies),
                StaticPart::Predicate(predicate) => predicate_bodies(predicate, &mut bodies),
                StaticPart::Name(_) => {}
            }),
        }
        value.physical_admissions = p
            .physical_admissions
            .get(&key.declaration)
            .into_iter()
            .flat_map(|admissions| admissions.iter())
            .filter(|(occurrence, _)| bodies.contains(&occurrence.body))
            .map(|(occurrence, admission)| (occurrence.clone(), admission.clone()))
            .collect();
        let mut paths = BTreeMap::new();
        let mut visit = |path: &dsl::Path| {
            let text = dsl::render_path(path);
            let mut prefixes = Vec::new();
            if path
                .segments
                .first()
                .is_some_and(|segment| !formals.contains(&segment.name))
            {
                for end in (1..=path.segments.len()).rev() {
                    prefixes.push(
                        path.segments[..end]
                            .iter()
                            .map(|segment| segment.name.as_str())
                            .collect::<Vec<_>>()
                            .join("."),
                    );
                }
            }
            paths.insert(text, prefixes);
        };
        fn expression_paths(expression: &dsl::Expr, visit: &mut impl FnMut(&dsl::Path)) {
            for path in expression.free_paths() {
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
            Syntax::Equation(e) => equation(e, &mut visit),
            Syntax::Static(syntax) => static_paths(syntax, &BTreeSet::new(), &mut visit),
        }
        for (path, prefixes) in paths {
            if let Some(id) = prefixes
                .iter()
                .find_map(|name| p.resolve(key.declaration, name))
            {
                value.dependencies.insert(id);
            } else if !matches!(path.as_str(), "true" | "false" | "missing")
                && !prefixes
                    .iter()
                    .any(|name| p.physical_name(key.declaration, name).is_some())
            {
                value.unresolved_references.insert(path);
            }
        }
        fn calls(expression: &dsl::Expr, names: &mut BTreeSet<String>) {
            expression.walk(|node| match &node.kind {
                dsl::ExprKind::NamedCall { name, .. }
                | dsl::ExprKind::Partial { function: name, .. } => {
                    names.insert(name.clone());
                }
                _ => {}
            });
        }
        fn predicate_calls(predicate: &dsl::Predicate, names: &mut BTreeSet<String>) {
            use dsl::PredicateKind as K;
            match &predicate.kind {
                K::Compare { lhs, rhs, .. } => {
                    calls(lhs, names);
                    calls(rhs, names);
                }
                K::In { expr, .. } | K::Atom(expr) => calls(expr, names),
                K::And(a, b) | K::Or(a, b) => {
                    predicate_calls(a, names);
                    predicate_calls(b, names);
                }
                K::Not(p) => predicate_calls(p, names),
                _ => {}
            }
        }
        fn equation_calls(equation: &dsl::Equation, names: &mut BTreeSet<String>) {
            match &equation.kind {
                dsl::EquationKind::Relation { lhs, rhs, .. } => {
                    calls(lhs, names);
                    calls(rhs, names);
                }
                dsl::EquationKind::Conditional {
                    guard,
                    then,
                    otherwise,
                } => {
                    predicate_calls(guard, names);
                    equation_calls(then, names);
                    equation_calls(otherwise, names);
                }
            }
        }
        let mut names = BTreeSet::new();
        match &value.syntax {
            Syntax::Expression(e) | Syntax::Static(StaticValue::Expression(e)) => {
                calls(e, &mut names)
            }
            Syntax::Predicate(e) => predicate_calls(e, &mut names),
            Syntax::Equation(e) => equation_calls(e, &mut names),
            Syntax::Static(syntax) => static_parts(syntax, &mut |part| match part {
                StaticPart::Expression(expression) => calls(expression, &mut names),
                StaticPart::Predicate(predicate) => predicate_calls(predicate, &mut names),
                StaticPart::Name(name) => {
                    names.insert(name.into());
                }
            }),
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
            };
            size_of::<(OccurrenceKey, CheckedExpression)>()
                + 64
                + value.declared_type.as_ref().map_or(0, crate::extent::ty)
                + value
                    .physical_admissions
                    .iter()
                    .map(|(occurrence, admission)| {
                        size_of::<(
                            super::admission::ExpressionOccurrence,
                            super::admission::PhysicalAdmission,
                        )>() + 64
                            + occurrence.syntax.capacity()
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
    fn checked(text: &str) -> CheckedPackage {
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
        .unwrap()
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
            p.equation(first, &a.text).unwrap(),
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
        assert!(p.expression(inherited, "1").is_ok());
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
    fn checked_occurrences_malformed_repeated_fields_name_each_owning_declaration() {
        let source = crate::kernel_types::source(
            "package p {def D {param first:Scalar=1; param second:Scalar=1;} }",
        );
        for name in ["first", "second"] {
            let mut rows = source.clone();
            let row = rows.iter_mut().find(|row| row.name == name).unwrap();
            let id = row.declaration_id;
            row.value.binding.as_mut().unwrap().expression = Some("1 +".into());
            let error = collect(&rows).unwrap_err();
            assert!(error.to_string().contains(&id.to_string()));
            assert!(error.to_string().contains("binding.expression[0]"));
        }
    }
}
