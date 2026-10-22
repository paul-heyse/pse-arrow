// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable producer-local construction, imported without resolving source names again.
use super::{construction::Construction, selection::Relocation, *};
use crate::expression::admission::{PhysicalAdmission, PhysicalAdmissionRecord};

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScalarContract {
    ty: Type,
    role: ModelingDeclarationKind,
    domain: ModelingVariableDomain,
}
impl ScalarContract {
    fn of(symbol: &ResolvedSymbol) -> Self {
        Self { ty: symbol.ty.clone(), role: symbol.role, domain: symbol.domain }
    }
}

/// A query-owned field/instance product. Every coordinate is fragment-local;
/// actual scalar bindings and complete root obligations belong to its receiver.
#[derive(Clone, Debug)]
pub(crate) struct Fragment {
    arena: Arc<Arena>,
    admissions: Arc<BTreeMap<NodeIx, PhysicalAdmission>>,
    literals: Arc<BTreeMap<NodeIx, Type>>,
    scalars: Arc<[ScalarContract]>,
    outputs: Vec<NodeIx>,
    guards: Vec<PredicateIx>,
    used: usize,
}
impl PartialEq for Fragment {
    fn eq(&self, other: &Self) -> bool {
        self.arena == other.arena && self.literals == other.literals
            && self.scalars == other.scalars && self.outputs == other.outputs
            && self.guards == other.guards && self.used == other.used
            && self.admissions.len() == other.admissions.len()
            && self.admissions.iter().all(|(slot, admission)| {
                other.admissions.get(slot).is_some_and(|other| {
                    PhysicalAdmissionRecord::capture(admission) == PhysicalAdmissionRecord::capture(other)
                })
            })
    }
}
impl Fragment {
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(extent::arena(&self.arena))
            .saturating_add(self.admissions.len() * (size_of::<(NodeIx, PhysicalAdmission)>() + 96))
            .saturating_add(self.admissions.values().map(PhysicalAdmission::retained_bytes).sum::<usize>())
            .saturating_add(self.literals.len() * (size_of::<(NodeIx, Type)>() + 96))
            .saturating_add(self.literals.values().map(extent::ty).sum::<usize>())
            .saturating_add(self.scalars.len() * size_of::<ScalarContract>())
            .saturating_add(self.scalars.iter().map(|value| extent::ty(&value.ty)).sum::<usize>())
            .saturating_add(self.outputs.capacity() * size_of::<NodeIx>())
            .saturating_add(self.guards.capacity() * size_of::<PredicateIx>())
    }
}

/// Complete relocation issued by one successful import, including non-output
/// roots used by the producer's original contribution/connection obligations.
#[derive(Debug)]
pub(crate) struct ImportedFragment {
    pub(crate) outputs: Vec<NodeIx>,
    pub(crate) guards: Vec<PredicateIx>,
    relocation: Relocation,
}
impl ImportedFragment {
    pub(crate) fn node(&self, local: NodeIx) -> Result<NodeIx> { self.relocation.node(local) }
    pub(crate) fn predicate(&self, local: PredicateIx) -> Result<PredicateIx> {
        self.relocation.predicate(local)
    }
}

fn offsets<T: Ord + Copy + From<usize>>(count: usize, start: usize) -> Result<BTreeMap<T, T>> {
    (0..count).map(|index| {
        let target = start.checked_add(index).ok_or_else(|| {
            crate::ModelingError::Budget("fragment coordinate overflow".into())
        })?;
        Ok((T::from(index), T::from(target)))
    }).collect()
}

impl Construction {
    /// Freeze one local producer. The ordered scalar contract is explicit: all
    /// referenced scalars must appear once, and no source/root index survives.
    pub(crate) fn into_fragment(
        self,
        outputs: Vec<NodeIx>,
        guards: Vec<PredicateIx>,
        symbols: &crate::specialize::Symbols,
        actual: &[SymbolIx],
    ) -> Result<Fragment> {
        let mut bindings = BTreeMap::new();
        let mut scalars = Vec::with_capacity(actual.len());
        for (ordinal, slot) in actual.iter().enumerate() {
            let symbol = symbols.at(*slot).ok_or_else(|| invalid(
                SemanticId::NIL, "fragment scalar is outside its source owner",
            ))?;
            if bindings.insert(*slot, SymbolIx::from(ordinal)).is_some() {
                return Err(invalid(symbol.id, "fragment repeats a scalar contract"));
            }
            scalars.push(ScalarContract::of(symbol));
        }
        let mut local = Construction::new(self.used);
        let imported = local.import_parts(&self.arena, &self.admissions, &self.literals,
            self.used, &outputs, &guards, bindings)?;
        Ok(Fragment {
            arena: Arc::new(local.arena), admissions: Arc::new(local.admissions),
            literals: Arc::new(local.literals), scalars: scalars.into(),
            outputs: imported.outputs, guards: imported.guards, used: self.used,
        })
    }

    /// Import one pure product into the receiver's authoritative scalar table.
    /// Validate every coordinate/fact before appending any arena state.
    pub(crate) fn import_fragment(
        &mut self,
        fragment: &Fragment,
        symbols: &crate::specialize::Symbols,
        actual: &[SymbolIx],
    ) -> Result<ImportedFragment> {
        if actual.len() != fragment.scalars.len() {
            return Err(invalid(SemanticId::NIL, "fragment scalar binding arity differs"));
        }
        let mut bindings = BTreeMap::new();
        for (ordinal, (slot, contract)) in actual.iter().zip(fragment.scalars.iter()).enumerate() {
            let symbol = symbols.at(*slot).ok_or_else(|| invalid(
                SemanticId::NIL, "fragment binding is outside the receiving scalar owner",
            ))?;
            if ScalarContract::of(symbol) != *contract {
                return Err(invalid(symbol.id, "fragment binding changes its physical or scalar contract"));
            }
            bindings.insert(SymbolIx::from(ordinal), *slot);
        }
        self.import_parts(&fragment.arena, &fragment.admissions, &fragment.literals,
            fragment.used, &fragment.outputs, &fragment.guards, bindings)
    }

    #[expect(clippy::too_many_arguments, reason = "one immutable fragment's complete typed spaces are relocated together")]
    fn import_parts(
        &mut self,
        source: &Arena,
        admissions: &BTreeMap<NodeIx, PhysicalAdmission>,
        literals: &BTreeMap<NodeIx, Type>,
        used: usize,
        outputs: &[NodeIx],
        guards: &[PredicateIx],
        symbols: BTreeMap<SymbolIx, SymbolIx>,
    ) -> Result<ImportedFragment> {
        let relocation = Relocation {
            symbols,
            nodes: offsets(source.nodes.len(), self.arena.nodes.len())?,
            predicates: offsets(source.predicates.len(), self.arena.predicates.len())?,
            binders: offsets(source.binders.len(), self.arena.binders.len())?,
            functions: offsets(source.functions.len(), self.arena.functions.len())?,
            evidence: offsets(source.evidence.len(), self.arena.evidence.len())?,
            carriers: offsets(source.carriers.len(), self.arena.carriers.len())?,
            groups: offsets(source.groups.len(), self.arena.groups.len())?,
        };
        let mut arena = Arena::default();
        for node in &source.nodes { arena.nodes.push(relocation.operation(node)?); }
        for guard in &source.predicates { arena.predicates.push(relocation.guard(guard)?); }
        for binding in &source.binders {
            arena.binders.push(match binding {
                Binding::Value(node) => Binding::Value(relocation.node(*node)?),
                Binding::Coordinate(carrier) => Binding::Coordinate(relocation.carrier(*carrier)?),
                Binding::Accumulator => Binding::Accumulator,
                Binding::Item => Binding::Item,
            });
        }
        arena.functions = source.functions.clone();
        for usage in &source.evidence { arena.evidence.push(relocation.usage(usage)?); }
        arena.carriers = source.carriers.clone();
        for group in &source.groups {
            arena.groups.push(IndexedGroup {
                axes: group.axes.iter().map(|slot| relocation.carrier(*slot)).collect::<Result<_>>()?,
                members: group.members.iter().map(|(tuple, member)| Ok((
                    tuple.clone(), member.map(|slot| relocation.symbol(slot)).transpose()?,
                ))).collect::<Result<_>>()?,
            });
        }
        for (slot, range) in &source.member_ranges {
            let target = relocation.symbol(*slot)?;
            // A fragment may declare an owned range, but never overwrite one
            // previously issued by another root/fragment producer.
            if self.arena.member_ranges.contains_key(&target) {
                return Err(invalid(SemanticId::NIL, "fragment repeats a receiving hard-range owner"));
            }
            if arena.member_ranges.insert(target, MemberRange {
                lower: relocation.node(range.lower)?, upper: relocation.node(range.upper)?, layer: range.layer,
            }).is_some() {
                return Err(invalid(SemanticId::NIL, "fragment bindings alias distinct hard-range owners"));
            }
        }
        for (slot, fold) in &source.finite_folds {
            arena.finite_folds.insert(relocation.node(*slot)?, FiniteFold {
                carrier: relocation.carrier(fold.carrier)?, selected: fold.selected.clone(), prototype: fold.prototype.clone(),
            });
        }
        for (slot, source) in &source.node_sources { arena.node_sources.insert(relocation.node(*slot)?, source.clone()); }
        for (slot, source) in &source.predicate_sources { arena.predicate_sources.insert(relocation.predicate(*slot)?, source.clone()); }
        let admissions = admissions.iter().map(|(slot, value)| Ok((relocation.node(*slot)?, value.clone()))).collect::<Result<BTreeMap<_,_>>>()?;
        let literals = literals.iter().map(|(slot, value)| Ok((relocation.node(*slot)?, value.clone()))).collect::<Result<BTreeMap<_,_>>>()?;
        let outputs = relocation.nodes(outputs)?;
        let guards = guards.iter().map(|slot| relocation.predicate(*slot)).collect::<Result<_>>()?;
        self.reserve(used)?;
        // All validation precedes publication. Source/function provenance stays
        // original, unlike the attribution-neutral mathematical-region contract.
        for (ordinal, function) in arena.functions.iter().enumerate() {
            let source_slot = FunctionIx::from(ordinal);
            self.functions.entry(Arc::as_ptr(&function.product) as usize)
                .or_insert(relocation.function(source_slot)?);
        }
        self.arena.nodes.extend(arena.nodes);
        self.arena.predicates.extend(arena.predicates);
        self.arena.binders.extend(arena.binders);
        self.arena.functions.extend(arena.functions);
        self.arena.evidence.extend(arena.evidence);
        self.arena.carriers.extend(arena.carriers);
        self.arena.groups.extend(arena.groups);
        self.arena.member_ranges.extend(arena.member_ranges);
        self.arena.finite_folds.extend(arena.finite_folds);
        self.arena.node_sources.extend(arena.node_sources);
        self.arena.predicate_sources.extend(arena.predicate_sources);
        self.admissions.extend(admissions);
        self.literals.extend(literals);
        Ok(ImportedFragment { outputs, guards, relocation })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(position: u32) -> Source {
        Source {
            field: crate::expression::occurrences::OccurrenceKey {
                declaration: SemanticId::from_bytes([9; 16]).into(),
                role: "equation.expression".into(), position: 0,
            },
            span: dsl::Span { start: position, end: position + 1 },
            coordinate: Some(InventoryCoordinate {
                stage: InventoryStage::Original, grammar: InventoryGrammar::Expression,
                occurrence: crate::expression::admission::ExpressionOccurrence::Node {
                    body: 0, position: position as usize,
                },
            }),
        }
    }
    fn scalar_symbols(seed: u8, ty: Type) -> crate::specialize::Symbols {
        let mut symbols = crate::specialize::Symbols::default();
        for index in 0..3u8 {
            let id = SemanticId::from_bytes([seed + index; 16]);
            symbols.push(ResolvedSymbol {
                id, ty: ty.clone(), role: ModelingDeclarationKind::Variable,
                domain: ModelingVariableDomain::Continuous,
                definition: None, expression: None, initial: None,
                lineage: crate::specialize::Lineage {
                    declaration: id.into(), instance: crate::InstanceId::from_id(id),
                    path: format!("instance_{seed}.member_{index}"), demand: vec![],
                    default_owner: None, is_override: false, presets: vec![],
                },
            });
        }
        symbols
    }
    fn literal(construction: &mut Construction, text: &str, position: u32) -> NodeIx {
        let ExprKind::Number(number) = dsl::parse_expr(text).unwrap().kind else { panic!() };
        let node = construction.node(Node::Number(number), source(position)).unwrap();
        construction.literal(node, Type::Integer).unwrap();
        node
    }
    #[test]
    fn fragment_import_relocates_shared_nodes_guards_ranges_and_original_support() {
        let symbols = scalar_symbols(10, Type::Integer);
        let mut local = Construction::new(128);
        let zero = literal(&mut local, "0", 0);
        let one = literal(&mut local, "1", 1);
        let input = local.node(Node::Reference(Reference::Symbol(SymbolIx::from(2))), source(2)).unwrap();
        let neg = local.node(Node::Neg(input), source(3)).unwrap();
        let admission = PhysicalAdmission::checked(&pse_quantity::infer::OpRequest::Neg, vec![], None,
            SemanticId::from_bytes([9;16]).into()).unwrap();
        local.admit(neg, admission.clone()).unwrap();
        let binder = local.binding(Binding::Value(neg)).unwrap();
        let root = local.node(Node::Let { bindings: vec![binder], body: neg }, source(4)).unwrap();
        let guard = local.predicate(ResolvedPredicate::Compare {
            op: CompareOp::Ge, lhs: input, rhs: zero,
        }, source(5)).unwrap();
        local.member_range(SymbolIx::from(2), MemberRange { lower: zero, upper: one,
            layer: pse_model::generated::enums::ModelingValidityLayer::Form }).unwrap();
        let fragment = local.into_fragment(vec![root, neg], vec![guard], &symbols, &[SymbolIx::from(2)]).unwrap();
        assert!(fragment.retained_bytes() > size_of::<Fragment>());
        assert_eq!(fragment.arena.nodes[NodeIx::from(2)], Node::Reference(Reference::Symbol(SymbolIx::from(0))));
        let target_symbols = scalar_symbols(30, Type::Integer);
        let mut target = Construction::new(256);
        literal(&mut target, "99", 99);
        let imported = target.import_fragment(&fragment, &target_symbols, &[SymbolIx::from(1)]).unwrap();
        assert_eq!(target.arena.nodes[imported.node(NodeIx::from(2)).unwrap()], Node::Reference(Reference::Symbol(SymbolIx::from(1))));
        assert_eq!(target.node_source(imported.outputs[0]).unwrap(), &source(4));
        assert_eq!(target.predicate_source(imported.guards[0]).unwrap(), &source(5));
        assert_eq!(imported.predicate(guard).unwrap(), imported.guards[0]);
        assert_eq!(PhysicalAdmissionRecord::capture(target.admission_at(imported.outputs[1]).unwrap()), PhysicalAdmissionRecord::capture(&admission));
        assert_eq!(target.literal_at(imported.node(zero).unwrap()), Some(&Type::Integer));
        assert_eq!(target.arena.member_ranges[&SymbolIx::from(1)].lower, imported.node(zero).unwrap());
        assert_eq!(target.arena.nodes[imported.outputs[0]], Node::Let {
            bindings: vec![BinderIx::from(0)], body: imported.outputs[1],
        });
        target.finish(target_symbols, imported.outputs, imported.guards).unwrap();
    }
    #[test]
    fn fragment_refusals_do_not_partially_publish_into_the_receiving_arena() {
        let symbols = scalar_symbols(10, Type::Integer);
        let mut local = Construction::new(8);
        let input = local.node(Node::Reference(Reference::Symbol(SymbolIx::from(1))), source(0)).unwrap();
        assert!(local.into_fragment(vec![input], vec![], &symbols, &[]).is_err());
        let mut local = Construction::new(8);
        let input = local.node(Node::Reference(Reference::Symbol(SymbolIx::from(1))), source(0)).unwrap();
        let fragment = local.into_fragment(vec![input], vec![], &symbols, &[SymbolIx::from(1)]).unwrap();
        let mut target = Construction::new(0);
        assert!(target.import_fragment(&fragment, &symbols, &[SymbolIx::from(0)]).is_err());
        assert!(target.arena.nodes.is_empty());
        let mut target = Construction::new(8);
        assert!(target.import_fragment(&fragment, &symbols, &[]).is_err());
        assert!(target.import_fragment(&fragment, &scalar_symbols(30, Type::Boolean), &[SymbolIx::from(0)]).is_err());
        assert!(target.import_fragment(&fragment, &symbols, &[SymbolIx::from(99)]).is_err());
        assert!(target.arena.nodes.is_empty());
    }
}
