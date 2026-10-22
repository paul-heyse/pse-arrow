// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact tagged framing of the reached resolved graph, with canonical first-use ordinals.
use super::*;

pub(super) fn identity(syntax: &ResolvedSyntax) -> Result<ContentHash> {
    Ok(canonical(syntax)?.hash.finish_hash())
}

pub(super) fn layout(syntax: &ResolvedSyntax) -> Result<CanonicalLayout> {
    layout_of(canonical(syntax)?)
}
pub(super) fn layout_parts(
    symbols: &crate::specialize::Symbols,
    arena: &Arena,
    outputs: &[NodeIx],
    validity: &[PredicateIx],
) -> Result<CanonicalLayout> {
    layout_of(canonical_parts(symbols, arena, outputs, validity)?)
}
fn layout_of(canonical: Canonical<'_>) -> Result<CanonicalLayout> {
    let syntax = canonical.syntax;
    let mut symbols = canonical
        .symbols
        .iter()
        .map(|(slot, ordinal)| (*slot, *ordinal))
        .collect::<Vec<_>>();
    symbols.sort_by_key(|(_, ordinal)| *ordinal);
    let mut counts = [0usize; 3];
    let symbols = symbols
        .into_iter()
        .map(|(symbol, _)| {
            let role = syntax.symbols[symbol].local_role();
            let role_slot = match role {
                LocalRole::Coordinate => 0,
                LocalRole::Parameter => 1,
                LocalRole::Member => 2,
            };
            let ordinal = counts[role_slot];
            counts[role_slot] += 1;
            CanonicalSymbol {
                symbol,
                role,
                ordinal,
            }
        })
        .collect();
    Ok(CanonicalLayout {
        identity: canonical.hash.finish_hash(),
        symbols,
        nodes: ordered(canonical.nodes),
        predicates: ordered(canonical.predicates),
        binders: ordered(canonical.binders),
        functions: ordered(canonical.functions),
        evidence: ordered(canonical.evidence),
        carriers: ordered(canonical.carriers),
        groups: ordered(canonical.groups),
    })
}

fn ordered<T: Ord>(values: BTreeMap<T, usize>) -> Vec<T> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort_by_key(|(_, ordinal)| *ordinal);
    values.into_iter().map(|(slot, _)| slot).collect()
}

fn canonical(syntax: &ResolvedSyntax) -> Result<Canonical<'_>> {
    canonical_parts(
        &syntax.symbols,
        &syntax.arena,
        &syntax.outputs,
        &syntax.validity,
    )
}
fn canonical_parts<'a>(
    symbols: &'a crate::specialize::Symbols,
    arena: &'a Arena,
    outputs: &'a [NodeIx],
    validity: &'a [PredicateIx],
) -> Result<Canonical<'a>> {
    let syntax = SyntaxView {
        symbols,
        arena,
        outputs,
        validity,
    };
    let mut frame = Canonical {
        syntax,
        hash: FramedHasher::new(pse_ids::Frame::ModelingResolvedSyntaxV1),
        symbols: BTreeMap::new(),
        nodes: BTreeMap::new(),
        predicates: BTreeMap::new(),
        binders: BTreeMap::new(),
        functions: BTreeMap::new(),
        evidence: BTreeMap::new(),
        carriers: BTreeMap::new(),
        groups: BTreeMap::new(),
    };
    frame.hash.str("outputs").u64(syntax.outputs.len() as u64);
    for root in syntax.outputs {
        frame.node(*root)?;
    }
    frame
        .hash
        .str("original_validity")
        .u64(syntax.validity.len() as u64);
    for root in syntax.validity {
        frame.predicate(*root)?;
    }
    Ok(frame)
}

type SyntaxView<'a> = View<'a>;
struct Canonical<'a> {
    syntax: SyntaxView<'a>,
    hash: FramedHasher,
    symbols: BTreeMap<SymbolIx, usize>,
    nodes: BTreeMap<NodeIx, usize>,
    predicates: BTreeMap<PredicateIx, usize>,
    binders: BTreeMap<BinderIx, usize>,
    functions: BTreeMap<FunctionIx, usize>,
    evidence: BTreeMap<EvidenceIx, usize>,
    carriers: BTreeMap<CarrierIx, usize>,
    groups: BTreeMap<GroupIx, usize>,
}
macro_rules! first {
    ($self:ident, $map:ident, $slot:ident, $tag:literal) => {
        $self.hash.str($tag);
        if let Some(ordinal) = $self.$map.get(&$slot) {
            $self.hash.str("reference").u64(*ordinal as u64);
            return Ok(());
        }
        let ordinal = $self.$map.len();
        $self.$map.insert($slot, ordinal);
        $self.hash.str("declaration").u64(ordinal as u64);
    };
}
fn absent(kind: &'static str) -> crate::ModelingError {
    invalid(
        SemanticId::NIL,
        format!("resolved {kind} edge is outside its owner"),
    )
}
impl Canonical<'_> {
    fn symbol(&mut self, slot: SymbolIx) -> Result<()> {
        let syntax = self.syntax;
        let symbol = syntax.symbols.at(slot).ok_or_else(|| absent("symbol"))?;
        first!(self, symbols, slot, "symbol");
        self.hash
            .str(symbol.role.as_str())
            .str(symbol.domain.as_str())
            .str(match symbol.local_role() {
                LocalRole::Coordinate => "coordinate",
                LocalRole::Parameter => "parameter",
                LocalRole::Member => "member",
            });
        frame_type(&symbol.ty, &mut self.hash);
        if let Some(root) = symbol.definition {
            self.hash.str("defined");
            self.node(root)?;
        } else {
            self.hash.str("input");
        }
        if let Some(range) = syntax.member_ranges.get(&slot) {
            self.hash
                .str("original-member-range")
                .str(range.layer.as_str());
            self.node(range.lower)?;
            self.node(range.upper)?;
        } else {
            self.hash.str("no-original-member-range");
        }
        Ok(())
    }
    fn binder(&mut self, slot: BinderIx) -> Result<()> {
        let syntax = self.syntax;
        let binder = syntax.binders.get(slot).ok_or_else(|| absent("binder"))?;
        first!(self, binders, slot, "binder");
        match binder {
            Binding::Value(root) => {
                self.hash.str("local_value");
                self.node(*root)?;
            }
            Binding::Coordinate(carrier) => {
                self.hash.str("coordinate");
                self.carrier(*carrier)?;
            }
            Binding::Accumulator => {
                self.hash.str("accumulator");
            }
            Binding::Item => {
                self.hash.str("item");
            }
        }
        Ok(())
    }
    fn carrier(&mut self, slot: CarrierIx) -> Result<()> {
        let syntax = self.syntax;
        let carrier = syntax.carriers.get(slot).ok_or_else(|| absent("carrier"))?;
        first!(self, carriers, slot, "carrier");
        match carrier.kind {
            CarrierKind::OrderedSet => {
                self.hash.str("ordered_set");
            }
            CarrierKind::Continuous { domain } => {
                self.hash.str("continuous").id(&domain.as_id());
            }
        }
        frame_type(&carrier.element, &mut self.hash);
        self.hash.u64(carrier.members.len() as u64);
        for value in carrier.members.iter() {
            value.frame(&mut self.hash);
        }
        Ok(())
    }
    fn group(&mut self, slot: GroupIx) -> Result<()> {
        let syntax = self.syntax;
        let group = syntax.groups.get(slot).ok_or_else(|| absent("group"))?;
        first!(self, groups, slot, "indexed_group");
        self.hash.u64(group.axes.len() as u64);
        for axis in &group.axes {
            self.carrier(*axis)?;
        }
        self.hash.u64(group.members.len() as u64);
        let mut seen = std::collections::BTreeSet::new();
        for (tuple, member) in &group.members {
            if tuple.len() != group.axes.len() || !seen.insert(tuple) {
                return Err(invalid(
                    SemanticId::NIL,
                    "resolved indexed layout has a repeated tuple or wrong rank",
                ));
            }
            self.hash.u64(tuple.len() as u64);
            for value in tuple {
                value.frame(&mut self.hash);
            }
            match member {
                Some(symbol) => {
                    self.hash.str("present");
                    self.symbol(*symbol)?;
                }
                None => {
                    self.hash.str("absent");
                }
            }
        }
        Ok(())
    }
    fn function(&mut self, slot: FunctionIx) -> Result<()> {
        let syntax = self.syntax;
        let function = syntax
            .functions
            .get(slot)
            .ok_or_else(|| absent("function"))?;
        first!(self, functions, slot, "function");
        self.hash
            .hash(&function.identity)
            .u64(function.formals.len() as u64);
        Ok(())
    }
    fn evidence(&mut self, slot: EvidenceIx) -> Result<()> {
        let syntax = self.syntax;
        let usage = syntax
            .evidence
            .get(slot)
            .ok_or_else(|| absent("evidence"))?;
        first!(self, evidence, slot, "scientific-evidence");
        usage.node.frame(&mut self.hash);
        self.hash.u64(usage.predicates.len() as u64);
        for predicate in &usage.predicates {
            self.predicate(*predicate)?;
        }
        self.nodes(&usage.inputs)
    }
    fn nodes(&mut self, nodes: &[NodeIx]) -> Result<()> {
        self.hash.u64(nodes.len() as u64);
        for node in nodes {
            self.node(*node)?;
        }
        Ok(())
    }
    fn filter(&mut self, filter: Option<PredicateIx>) -> Result<()> {
        match filter {
            Some(filter) => {
                self.hash.str("filtered");
                self.predicate(filter)?;
            }
            None => {
                self.hash.str("unfiltered");
            }
        }
        Ok(())
    }
    fn node(&mut self, slot: NodeIx) -> Result<()> {
        let syntax = self.syntax;
        let node = syntax.nodes.get(slot).ok_or_else(|| absent("operation"))?;
        first!(self, nodes, slot, "operation");
        if let Some(fold) = syntax.finite_folds.get(&slot) {
            self.hash.str("actual_finite_fold");
            self.carrier(fold.carrier)?;
            frame_type(&fold.prototype, &mut self.hash);
            self.hash.u64(fold.selected.len() as u64);
            for value in fold.selected.iter() {
                value.frame(&mut self.hash);
            }
        } else {
            self.hash.str("ordinary_operation");
        }
        match node {
            Node::Number(number) => {
                if !number.value.is_finite() {
                    return Err(invalid(SemanticId::NIL, "nonfinite resolved literal"));
                }
                self.hash.str("literal").u64(number.value.to_bits());
                match number.integer() {
                    Some(integer) => {
                        self.hash.str("integer").part(&integer.to_le_bytes());
                    }
                    None => {
                        self.hash.str("real");
                    }
                }
                match &number.unit {
                    Some(unit) => {
                        self.hash.str("unit").u64(unit.factors().len() as u64);
                        for (symbol, power) in unit.factors() {
                            self.hash
                                .str(symbol)
                                .part(&power.num().to_le_bytes())
                                .part(&power.den().to_le_bytes());
                        }
                    }
                    None => {
                        self.hash.str("untyped_literal");
                    }
                }
            }
            Node::Reference(Reference::Symbol(symbol)) => {
                self.hash.str("scalar");
                self.symbol(*symbol)?;
            }
            Node::Reference(Reference::Lexical(binder)) => {
                self.hash.str("lexical");
                self.binder(*binder)?;
            }
            Node::Reference(Reference::Indexed { group, indices }) => {
                self.hash.str("indexed");
                self.group(*group)?;
                self.nodes(indices)?;
            }
            Node::Neg(value) => {
                self.hash.str("neg");
                self.node(*value)?;
            }
            Node::Binary { op, lhs, rhs } => {
                self.hash.str("binary").str(op.as_str());
                self.node(*lhs)?;
                self.node(*rhs)?;
            }
            Node::Call { function, args } => {
                self.hash.str("builtin").str(function.as_str());
                self.nodes(args)?;
            }
            Node::Apply { function, args } => {
                self.hash.str("apply");
                self.function(*function)?;
                self.nodes(args)?;
            }
            Node::Partial {
                function,
                wrt,
                args,
            } => {
                self.hash.str("partial");
                self.function(*function)?;
                self.hash.u64(wrt.len() as u64);
                for ordinal in wrt {
                    self.hash.u64(*ordinal as u64);
                }
                self.nodes(args)?;
            }
            Node::Provider { selection, args } => {
                self.hash
                    .str("provider")
                    .hash(&selection.identity)
                    .u64(selection.output as u64);
                self.nodes(args)?;
            }
            Node::Reduce {
                kind,
                binder,
                filter,
                body,
            } => {
                self.hash.str("reduce").str(kind.as_str());
                self.binder(*binder)?;
                self.filter(*filter)?;
                self.node(*body)?;
            }
            Node::Fold {
                binder,
                filter,
                value,
                accumulator,
                item,
                step,
            } => {
                self.hash.str("fold");
                self.binder(*binder)?;
                self.filter(*filter)?;
                self.node(*value)?;
                self.binder(*accumulator)?;
                self.binder(*item)?;
                self.node(*step)?;
            }
            Node::Derivative { body, domain } => {
                self.hash.str("derivative");
                self.node(*body)?;
                self.carrier(*domain)?;
            }
            Node::Conditional {
                guard,
                then,
                otherwise,
            } => {
                self.hash.str("conditional");
                self.predicate(*guard)?;
                self.node(*then)?;
                self.node(*otherwise)?;
            }
            Node::Evidence { value, evidence } => {
                self.hash.str("evidence-read");
                self.evidence(*evidence)?;
                self.node(*value)?;
            }
            Node::Indicator(value) => {
                self.hash.str("predicate-indicator");
                self.predicate(*value)?;
            }
            Node::Sequence {
                value,
                prerequisites,
            } => {
                self.hash.str("source-observations");
                self.nodes(prerequisites)?;
                self.node(*value)?;
            }
            Node::Let { bindings, body } => {
                self.hash.str("let").u64(bindings.len() as u64);
                for binding in bindings {
                    self.binder(*binding)?;
                }
                self.node(*body)?;
            }
        }
        Ok(())
    }
    fn predicate(&mut self, slot: PredicateIx) -> Result<()> {
        let syntax = self.syntax;
        let predicate = syntax
            .predicates
            .get(slot)
            .ok_or_else(|| absent("predicate"))?;
        first!(self, predicates, slot, "predicate");
        match predicate {
            ResolvedPredicate::Compare { op, lhs, rhs } => {
                self.hash.str("compare").str(op.as_str());
                self.node(*lhs)?;
                self.node(*rhs)?;
            }
            ResolvedPredicate::In { value, domain } => {
                self.hash.str("in");
                self.node(*value)?;
                self.carrier(*domain)?;
            }
            ResolvedPredicate::And(lhs, rhs) => {
                self.hash.str("and");
                self.predicate(*lhs)?;
                self.predicate(*rhs)?;
            }
            ResolvedPredicate::Or(lhs, rhs) => {
                self.hash.str("or");
                self.predicate(*lhs)?;
                self.predicate(*rhs)?;
            }
            ResolvedPredicate::Not(value) => {
                self.hash.str("not");
                self.predicate(*value)?;
            }
            ResolvedPredicate::Atom(value) => {
                self.hash.str("atom");
                self.node(*value)?;
            }
            ResolvedPredicate::Bool(value) => {
                self.hash.str("bool").bool(*value);
            }
            ResolvedPredicate::Null => {
                self.hash.str("null");
            }
        }
        Ok(())
    }
}
