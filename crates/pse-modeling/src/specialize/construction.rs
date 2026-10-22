// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact source scopes for the single construction-time specialization rewrite.
use super::*;
use crate::expression::{admission::ExpressionOccurrence, occurrences::OccurrenceKey};
use crate::resolved::{
    InventoryCoordinate, InventoryGrammar, InventoryStage, Node, NodeIx, Reference,
    ResolvedPredicate, Source, construction::RewrittenPredicate,
};
use std::sync::Arc;

/// Attribution indexes only. This owner never interprets or reconstructs executable syntax.
#[derive(Debug)]
pub(super) struct OriginalScope {
    expressions: BTreeMap<usize, Source>,
    predicates: BTreeMap<usize, Source>,
    predicate_positions: BTreeMap<OccurrenceKey, usize>,
    admissions: BTreeMap<usize, crate::expression::admission::PhysicalAdmission>,
    allowance: usize,
}
impl OriginalScope {
    pub(super) fn new(allowance: usize) -> Self {
        Self {
            expressions: BTreeMap::new(),
            predicates: BTreeMap::new(),
            predicate_positions: BTreeMap::new(),
            admissions: BTreeMap::new(),
            allowance,
        }
    }
    fn charge(&self) -> Result<()> {
        if self.expressions.len().saturating_add(self.predicates.len()) >= self.allowance {
            Err(ModelingError::Budget(
                "original source inventory allowance exceeded".into(),
            ))
        } else {
            Ok(())
        }
    }
    fn source(
        field: &OccurrenceKey,
        span: Span,
        grammar: InventoryGrammar,
        body: usize,
        position: usize,
    ) -> Source {
        Source {
            field: field.clone(),
            span,
            coordinate: Some(InventoryCoordinate {
                stage: InventoryStage::Original,
                grammar,
                occurrence: ExpressionOccurrence::Node { body, position },
            }),
        }
    }
    fn predicate(&mut self, field: &OccurrenceKey, value: &dsl::Predicate) -> Result<()> {
        self.charge()?;
        let position = self.predicate_positions.entry(field.clone()).or_default();
        let source = Self::source(field, value.span, InventoryGrammar::Predicate, 0, *position);
        *position += 1;
        if self
            .predicates
            .insert(std::ptr::from_ref(value) as usize, source)
            .is_some()
        {
            return Err(invalid(
                field.declaration,
                "original predicate roots overlap",
            ));
        }
        match &value.kind {
            dsl::PredicateKind::And(lhs, rhs) | dsl::PredicateKind::Or(lhs, rhs) => {
                self.predicate(field, lhs)?;
                self.predicate(field, rhs)?;
            }
            dsl::PredicateKind::Not(value) => self.predicate(field, value)?,
            dsl::PredicateKind::Compare { .. }
            | dsl::PredicateKind::In { .. }
            | dsl::PredicateKind::Atom(_)
            | dsl::PredicateKind::Bool(_)
            | dsl::PredicateKind::Null => {}
        }
        Ok(())
    }
    fn expression(&mut self, field: &OccurrenceKey, part: usize, value: &Expr) -> Result<()> {
        let mut position = 0;
        let mut failure = None;
        value.walk(|expression| {
            if failure.is_some() {
                return;
            }
            let result = (|| {
                self.charge()?;
                let source = Self::source(
                    field,
                    expression.span,
                    InventoryGrammar::Expression,
                    part,
                    position,
                );
                if self
                    .expressions
                    .insert(std::ptr::from_ref(expression) as usize, source)
                    .is_some()
                {
                    return Err(invalid(
                        field.declaration,
                        "original expression roots overlap",
                    ));
                }
                match &expression.kind {
                    ExprKind::Conditional { guard, .. } => self.predicate(field, guard)?,
                    ExprKind::Reduce { binder, .. } | ExprKind::Fold { binder, .. } => {
                        if let Some(filter) = &binder.filter {
                            self.predicate(field, filter)?;
                        }
                    }
                    _ => {}
                }
                Ok(())
            })();
            if let Err(error) = result {
                failure = Some(error);
            }
            position += 1;
        });
        failure.map_or(Ok(()), Err)
    }
    fn attach_admissions(
        &mut self,
        at: DeclarationId,
        roots: &[&Expr],
        admissions: &crate::expression::admission::ExpressionAdmissions,
    ) -> Result<()> {
        let positions = crate::expression::inventory::positions(roots.iter().copied());
        let mut occupied = BTreeSet::new();
        for (address, position) in positions {
            if let Some(admission) = admissions.get(&position) {
                if self.admissions.insert(address, admission.clone()).is_some() {
                    return Err(invalid(
                        at,
                        "one original operation has duplicate physical obligations",
                    ));
                }
                occupied.insert(position);
            }
        }
        if admissions.keys().any(|position| {
            !matches!(position, ExpressionOccurrence::FiniteReduction)
                && !occupied.contains(position)
        }) {
            return Err(invalid(
                at,
                "checked physical obligation has no actual original operation",
            ));
        }
        Ok(())
    }
    fn checked_field(
        &mut self,
        package: &CheckedPackage,
        field: &OccurrenceKey,
        roots: &[&Expr],
    ) -> Result<()> {
        let checked = package.expressions.get(field).ok_or_else(|| {
            invalid(
                field.declaration,
                format!(
                    "required original checked source field {}[{}] is absent",
                    field.role, field.position
                ),
            )
        })?;
        self.attach_admissions(field.declaration, roots, &checked.physical_admissions)
    }
    /// Own the checked function's actual inventory before its source is rewritten.
    pub(super) fn function(
        value: &crate::Function,
        package: &CheckedPackage,
        allowance: usize,
    ) -> Result<Self> {
        let mut scope = Self::new(allowance);
        let roots = crate::expression::inventory::function(value);
        let expressions = roots.iter().map(|root| root.expression).collect::<Vec<_>>();
        scope.attach_admissions(value.id, &expressions, &value.physical_admissions)?;
        for root in roots {
            let role = if root.role == "function.body" {
                match value.physical_operation {
                    Some(crate::PhysicalOperation::Coordinate { .. }) => {
                        "coordinate_slot.expression"
                    }
                    Some(crate::PhysicalOperation::Response { .. }) => "response.body",
                    Some(crate::PhysicalOperation::Reconstruction { .. }) => {
                        "derived.reconstruction"
                    }
                    _ => root.role,
                }
            } else {
                root.role
            };
            let field = OccurrenceKey {
                declaration: value.id,
                role: role.into(),
                position: root.repeated,
            };
            scope.expression(&field, root.part, root.expression)?;
            if role == "derived.reconstruction" {
                for source in scope
                    .expressions
                    .values_mut()
                    .chain(scope.predicates.values_mut())
                {
                    if source.field == field {
                        source.coordinate = None;
                    }
                }
                let ExprKind::Binary {
                    op: BinaryOp::Mul,
                    lhs,
                    ..
                } = &root.expression.kind
                else {
                    return Err(invalid(
                        value.id,
                        "checked reconstruction lost its declared normalization operand",
                    ));
                };
                let original =
                    package.expression_at(value.id, "reconstruction.normalization", 0)?;
                if !lhs.structural_eq(original) {
                    return Err(invalid(
                        value.id,
                        "reconstruction normalization differs from its original source",
                    ));
                }
                let mut normalization = Self::new(allowance);
                normalization.expression(
                    &OccurrenceKey {
                        declaration: value.id,
                        role: "reconstruction.normalization".into(),
                        position: 0,
                    },
                    0,
                    lhs,
                )?;
                scope.expressions.extend(normalization.expressions);
                scope.predicates.extend(normalization.predicates);
            }
        }
        if let Some(predicate) = &value.validity {
            let (declaration, role) = match value.physical_operation {
                Some(crate::PhysicalOperation::Coordinate { map, .. }) => {
                    (map, "coordinate_map.validity")
                }
                _ => (value.id, "function.validity"),
            };
            scope.predicate(
                &OccurrenceKey {
                    declaration,
                    role: role.into(),
                    position: 0,
                },
                predicate,
            )?;
        }
        for (position, guard) in value.envelopes.iter().enumerate() {
            scope.predicate(
                &OccurrenceKey {
                    declaration: value.id,
                    role: "function.envelopes".into(),
                    position,
                },
                &guard.predicate,
            )?;
        }
        for (position, usage) in value.applicability_uses.iter().enumerate() {
            for predicate in &usage.predicates {
                scope.predicate(
                    &OccurrenceKey {
                        declaration: value.id,
                        role: "function.applicability_uses".into(),
                        position,
                    },
                    predicate,
                )?;
            }
        }
        Ok(scope)
    }
    fn equation_predicates(&mut self, field: &OccurrenceKey, value: &Equation) -> Result<()> {
        if let EquationKind::Conditional {
            guard,
            then,
            otherwise,
        } = &value.kind
        {
            self.predicate(field, guard)?;
            self.equation_predicates(field, then)?;
            self.equation_predicates(field, otherwise)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OriginalGuard {
    pub(crate) source: Source,
    pub(crate) selected: bool,
    pub(crate) declarations: BTreeSet<DeclarationId>,
}

impl Engine<'_, '_> {
    pub(super) fn in_original_scope<T>(
        &mut self,
        scope: OriginalScope,
        apply: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        self.originals.push(Arc::new(scope));
        let result = apply(self);
        self.originals.pop();
        result
    }
    pub(super) fn with_original_expression<T>(
        &mut self,
        at: DeclarationId,
        role: &str,
        position: usize,
        value: &Expr,
        apply: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let mut scope = OriginalScope::new(self.limits.items);
        let field = OccurrenceKey {
            declaration: at,
            role: role.into(),
            position,
        };
        scope.expression(&field, 0, value)?;
        scope.checked_field(self.p, &field, &[value])?;
        self.in_original_scope(scope, apply)
    }
    /// Revisit one actual grammar root retained before a constructor/default was inlined.
    pub(super) fn with_original_part<T>(
        &mut self,
        at: DeclarationId,
        role: &str,
        position: usize,
        part: usize,
        value: &Expr,
        apply: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let field = OccurrenceKey {
            declaration: at,
            role: role.into(),
            position,
        };
        let checked = self.p.expressions.get(&field).ok_or_else(|| {
            invalid(
                at,
                "retained numerical source lacks its checked original field",
            )
        })?;
        let roots = crate::expression::inventory::field(&checked.syntax);
        if roots
            .get(part)
            .is_none_or(|original| !original.structural_eq(value))
        {
            return Err(invalid(
                at,
                "retained numerical source differs from its original grammar root",
            ));
        }
        let mut scope = OriginalScope::new(self.limits.items);
        scope.expression(&field, part, value)?;
        let mut occupied = BTreeSet::new();
        for (address, local) in crate::expression::inventory::positions([value]) {
            let ExpressionOccurrence::Node { position, .. } = local else {
                continue;
            };
            let original = ExpressionOccurrence::Node {
                body: part,
                position,
            };
            if let Some(admission) = checked.physical_admissions.get(&original) {
                scope.admissions.insert(address, admission.clone());
                occupied.insert(original);
            }
        }
        if checked.physical_admissions.keys().any(|occurrence| {
            matches!(occurrence,ExpressionOccurrence::Node {body,..} if *body==part)
                && !occupied.contains(occurrence)
        }) {
            return Err(invalid(
                at,
                "retained original grammar root lost a physical obligation",
            ));
        }
        self.in_original_scope(scope, apply)
    }
    pub(super) fn with_original_predicate<T>(
        &mut self,
        at: DeclarationId,
        role: &str,
        position: usize,
        value: &dsl::Predicate,
        apply: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let field = OccurrenceKey {
            declaration: at,
            role: role.into(),
            position,
        };
        let mut scope = OriginalScope::new(self.limits.items);
        scope.predicate(&field, value)?;
        let mut roots = Vec::new();
        crate::expression::inventory::predicate(value, &mut roots);
        for (part, root) in roots.iter().enumerate() {
            scope.expression(&field, part, root)?;
        }
        scope.checked_field(self.p, &field, &roots)?;
        self.in_original_scope(scope, apply)
    }
    pub(super) fn with_original_logic<T>(
        &mut self,
        at: DeclarationId,
        role: &str,
        position: usize,
        value: &dsl::Proposition,
        apply: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let field = OccurrenceKey {
            declaration: at,
            role: role.into(),
            position,
        };
        let mut scope = OriginalScope::new(self.limits.items);
        let mut roots = Vec::new();
        value.expressions(&mut |expression| roots.push(expression));
        for (part, root) in roots.iter().enumerate() {
            scope.expression(&field, part, root)?;
        }
        scope.checked_field(self.p, &field, &roots)?;
        self.in_original_scope(scope, apply)
    }
    pub(super) fn with_original_equation<T>(
        &mut self,
        at: DeclarationId,
        role: &str,
        position: usize,
        value: &Equation,
        apply: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let field = OccurrenceKey {
            declaration: at,
            role: role.into(),
            position,
        };
        let mut scope = OriginalScope::new(self.limits.items);
        scope.equation_predicates(&field, value)?;
        let mut actual = Vec::new();
        // Borrow the one grammar owner's actual roots, including unselected arms.
        crate::expression::inventory::equation(value, &mut actual);
        for (part, root) in actual.iter().enumerate() {
            scope.expression(&field, part, root)?;
        }
        scope.checked_field(self.p, &field, &actual)?;
        self.in_original_scope(scope, apply)
    }
    /// Source-owned physical field admission happens before this one rewrite. Consumers
    /// receive its actual typed root; they cannot retag an arbitrary constructed value.
    pub(crate) fn rewrite_physical_field(
        &mut self,
        rewrite: RewriteContext<'_>,
        at: DeclarationId,
        role: &str,
        position: usize,
        expected: Option<&Type>,
    ) -> Result<crate::resolved::construction::Rewritten> {
        let RewriteContext {
            instance,
            env,
            chain,
        } = rewrite;
        let original = self.p.expression_at(at, role, position)?.clone();
        let actual = crate::expression::infer(
            &original,
            &self.source_types(at, env)?,
            self.p,
            self.c,
            at,
            expected,
        )?;
        if expected.is_some_and(|expected| actual != *expected)
            || actual.quantity_scheme().is_none()
        {
            return Err(invalid(
                at,
                "original field has an incompatible physical type",
            ));
        }
        let value = self.with_original_expression(at, role, position, &original, |engine| {
            engine.rewrite(instance, &original, env, chain)
        })?;
        let support = self.constructed_support_with_context(value.root, Some(&actual))?;
        if support
            .outputs
            .first()
            .is_none_or(|output| output.ty != actual)
        {
            return Err(invalid(
                at,
                "constructed field differs from its admitted original physical type",
            ));
        }
        // Retain the source-owned context at every actual emitted literal, including
        // literals under negation/conditionals. Later region selection consumes it.
        for (root, ty) in support.literals {
            self.construction.literal(root, ty)?;
        }
        Ok(value)
    }
    pub(super) fn rewrite_field(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        role: &str,
        position: usize,
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<crate::resolved::construction::Rewritten> {
        let expression = self.p.expression_at(at, role, position)?.clone();
        self.with_original_expression(at, role, position, &expression, |engine| {
            engine.rewrite(instance, &expression, env, chain)
        })
    }
    pub(crate) fn rewrite_predicate_field(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        role: &str,
        position: usize,
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<RewrittenPredicate> {
        let predicate = self.p.predicate_at(at, role, position)?.clone();
        self.with_original_predicate(at, role, position, &predicate, |engine| {
            engine.rewrite_predicate(instance, &predicate, env, chain)
        })
    }
    pub(super) fn rewrite_equation_field(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        role: &str,
        position: usize,
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<crate::resolved::construction::RewrittenEquation> {
        let equation = self.p.equation_at(at, role, position)?.clone();
        self.with_original_equation(at, role, position, &equation, |engine| {
            engine.rewrite_equation(instance, &equation, env, chain)
        })
    }
    pub(super) fn original_expression_source(
        &self,
        value: &Expr,
        at: DeclarationId,
    ) -> Result<Source> {
        let address = std::ptr::from_ref(value) as usize;
        self.originals.iter().rev().find_map(|scope| scope.expressions.get(&address)).cloned()
            .ok_or_else(|| invalid(at,format!("expression at {}..{} lacks its actual original occurrence before rewriting: {}",value.span.start,value.span.end,dsl::render_expr(value))))
    }
    pub(super) fn original_predicate_source(
        &self,
        value: &dsl::Predicate,
        at: DeclarationId,
    ) -> Result<Source> {
        let address = std::ptr::from_ref(value) as usize;
        self.originals
            .iter()
            .rev()
            .find_map(|scope| scope.predicates.get(&address))
            .cloned()
            .ok_or_else(|| {
                invalid(
                    at,
                    "predicate lacks its actual original occurrence before rewriting",
                )
            })
    }
    pub(super) fn record_static_guard(
        &mut self,
        value: &dsl::Predicate,
        selected: bool,
        at: DeclarationId,
    ) -> Result<()> {
        let source = self.original_predicate_source(value, at)?;
        let declarations=self.p.expressions.get(&source.field)
            .ok_or_else(|| invalid(at,format!("selected guard lacks its original checked field dependencies {}[{}] owned by {} ({:?}); available: {:?}",source.field.role,source.field.position,source.field.declaration,self.p.declarations.get(&source.field.declaration).map(|row|(&row.name,row.value.kind)),self.p.expressions.keys().filter(|field|field.declaration==source.field.declaration).map(|field|(&field.role,field.position)).collect::<Vec<_>>())))?.dependencies.clone();
        self.original_guards.push(OriginalGuard {
            source,
            selected,
            declarations,
        });
        Ok(())
    }
    pub(super) fn pair_expression(
        &mut self,
        node: Node,
        inspection: ExprKind,
        source: Source,
        original: Option<&Expr>,
    ) -> Result<crate::resolved::construction::Rewritten> {
        let admission = original.and_then(|expression| {
            let address = std::ptr::from_ref(expression) as usize;
            self.originals
                .iter()
                .rev()
                .find_map(|scope| scope.admissions.get(&address))
                .cloned()
        });
        let rewritten = self.construction.pair(node, inspection, source.clone())?;
        if let Some(admission) = admission {
            // A source reduction has become a selected function, whose own complete
            // admission is retained by that product. Never attach it to an Apply as
            // though it were an original scalar arithmetic operation.
            if matches!(
                self.construction.get(rewritten.root),
                Some(
                    Node::Neg(_)
                        | Node::Binary { .. }
                        | Node::Call { .. }
                        | Node::Conditional { .. }
                )
            ) {
                let formula = self
                    .function_stack
                    .last()
                    .and_then(|at| self.p.functions.get(at))
                    .filter(|function| {
                        matches!(
                            function.physical_operation,
                            Some(crate::PhysicalOperation::Response { .. })
                        )
                    })
                    .map(|function| {
                        pse_quantity::PhysicalFormulaAuthority::response(function.id.as_id())
                    });
                let concrete = admission.instantiated(
                    self.c.quantities,
                    &self.function_substitutions,
                    self.c.preconditions,
                    formula.as_ref(),
                    source.field.declaration,
                )?;
                self.construction.admit(rewritten.root, concrete)?;
            }
        }
        Ok(rewritten)
    }
    pub(super) fn pair_predicate(
        &mut self,
        node: ResolvedPredicate,
        inspection: dsl::PredicateKind,
        source: Source,
    ) -> Result<RewrittenPredicate> {
        let span = source.span;
        Ok(RewrittenPredicate {
            root: self.construction.predicate(node, source)?,
            inspection: dsl::Predicate {
                kind: inspection,
                span,
            },
        })
    }
    pub(super) fn constructed_type(&self, root: NodeIx) -> Result<Type> {
        self.constructed_type_with_context(root, None)
    }
    pub(super) fn constructed_type_with_context(
        &self,
        root: NodeIx,
        expected: Option<&Type>,
    ) -> Result<Type> {
        self.constructed_support_with_context(root, expected)?
            .outputs
            .into_iter()
            .next()
            .map(|value| value.ty)
            .ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "constructed type inquiry lacks its actual requested output",
                )
            })
    }
    fn constructed_support_with_context(
        &self,
        root: NodeIx,
        expected: Option<&Type>,
    ) -> Result<crate::resolved::InferredView> {
        let roots = [root];
        let view = self.construction.view(self.current_symbols(), &roots, &[]);
        let owner = self
            .function_stack
            .last()
            .copied()
            .unwrap_or_else(|| SemanticId::NIL.into());
        let authority = self
            .p
            .functions
            .get(&owner)
            .and_then(|function| function.physical_operation.as_ref());
        crate::resolved::PhysicalSupport::infer_view(
            view,
            self.c,
            self.construction.literal_contracts(),
            authority,
            owner,
            expected,
        )
    }
    pub(super) fn current_symbols(&self) -> &Symbols {
        self.function_symbols
            .as_ref()
            .unwrap_or(&self.model.symbols)
    }
    pub(super) fn current_layout(
        &self,
        outputs: &[NodeIx],
        validity: &[crate::resolved::PredicateIx],
    ) -> Result<crate::resolved::CanonicalLayout> {
        self.construction
            .layout(self.current_symbols(), outputs, validity)
    }
    pub(super) fn symbol_of_root(&self, root: NodeIx) -> Option<SemanticId> {
        match self.construction.get(root)? {
            Node::Reference(Reference::Symbol(symbol)) => {
                self.model.symbols.at(*symbol).map(|symbol| symbol.id)
            }
            _ => None,
        }
    }
}
