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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Prefix {
    nodes: usize, predicates: usize, binders: usize, functions: usize,
    evidence: usize, carriers: usize, groups: usize,
}
impl Prefix {
    fn of(arena: &Arena) -> Self { Self { nodes:arena.nodes.len(),predicates:arena.predicates.len(),binders:arena.binders.len(),functions:arena.functions.len(),evidence:arena.evidence.len(),carriers:arena.carriers.len(),groups:arena.groups.len() } }
}
/// Coordinates of already-owned foreign premises in one receiving root. They
/// never participate in local query equality or retained product ownership.
#[derive(Debug)]
pub(crate) struct ForeignBindings { relocation: Relocation, prefix: Prefix }
fn reverse<T:Ord+Copy>(map:&BTreeMap<T,T>)->BTreeMap<T,T> {map.iter().map(|(from,to)|(*to,*from)).collect()}
fn inverse(relocation:&Relocation)->Relocation { Relocation {
    symbols:reverse(&relocation.symbols),nodes:reverse(&relocation.nodes),predicates:reverse(&relocation.predicates),
    binders:reverse(&relocation.binders),functions:reverse(&relocation.functions),evidence:reverse(&relocation.evidence),carriers:reverse(&relocation.carriers),groups:reverse(&relocation.groups),
} }
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
    prefix: Prefix,
}
impl PartialEq for Fragment {
    fn eq(&self, other: &Self) -> bool {
        self.arena == other.arena && self.literals == other.literals
            && self.scalars == other.scalars && self.outputs == other.outputs
            && self.guards == other.guards && self.used == other.used && self.prefix==other.prefix
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

/// The reached foreign premises of a local producer. Actual identities and source
/// inspections stay intact; only typed coordinates are relocated. Unrelated root
/// arena state is never retained by this boundary.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CapturePremises {
    pub(crate) fragment: Fragment,
    pub(crate) symbols: crate::specialize::Symbols,
}
impl CapturePremises {
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(self.fragment.retained_bytes())
            .saturating_add(self.symbols.retained_bytes())
    }
}
/// Receiver-only correspondence is deliberately not an equality-bearing input.
#[derive(Debug)]
pub(crate) struct CapturedFragment {
    pub(crate) premises: CapturePremises,
    pub(crate) actual: Vec<SymbolIx>,
    pub(crate) bindings: ForeignBindings,
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

impl Construction {
    /// Capture only the closure reached from explicitly selected foreign roots.
    /// This uses the same checked traversal as mathematical selection, retaining
    /// original meaning and attribution instead of making a neutral math region.
    pub(crate) fn capture_fragment(
        &self,
        outputs: &[NodeIx],
        guards: &[PredicateIx],
        symbols: &crate::specialize::Symbols,
    ) -> Result<CapturedFragment> {
        let layout = self.layout(symbols, outputs, guards)?;
        let relocation = Relocation::new(&layout);
        let mut local_symbols = crate::specialize::Symbols::default();
        let actual = layout.symbols.iter().map(|value| value.symbol).collect::<Vec<_>>();
        let mut contracts = Vec::with_capacity(actual.len());
        for slot in &actual {
            let mut symbol = symbols.at(*slot).ok_or_else(|| invalid(
                SemanticId::NIL, "captured scalar is outside its source owner",
            ))?.clone();
            contracts.push(ScalarContract::of(&symbol));
            symbol.definition = symbol.definition.map(|root| relocation.node(root)).transpose()?;
            local_symbols.push(symbol);
        }
        let mut arena = Arena::default();
        let mut admissions = BTreeMap::new();
        let mut literals = BTreeMap::new();
        for old in &layout.nodes {
            let local = relocation.node(*old)?;
            arena.nodes.push(relocation.operation(&self.arena.nodes[*old])?);
            if let Some(source) = self.arena.node_sources.get(old) {
                arena.node_sources.insert(local, source.clone());
            }
            if let Some(value) = self.admissions.get(old) { admissions.insert(local, value.clone()); }
            if let Some(value) = self.literals.get(old) { literals.insert(local, value.clone()); }
            if let Some(value) = self.arena.finite_folds.get(old) {
                arena.finite_folds.insert(local, FiniteFold {
                    carrier: relocation.carrier(value.carrier)?, selected: value.selected.clone(),
                    prototype: value.prototype.clone(),
                });
            }
        }
        for old in &layout.predicates {
            let local = relocation.predicate(*old)?;
            arena.predicates.push(relocation.guard(&self.arena.predicates[*old])?);
            if let Some(source) = self.arena.predicate_sources.get(old) {
                arena.predicate_sources.insert(local, source.clone());
            }
        }
        for old in &layout.binders {
            arena.binders.push(match self.arena.binders[*old] {
                Binding::Value(root) => Binding::Value(relocation.node(root)?),
                Binding::Coordinate(carrier) => Binding::Coordinate(relocation.carrier(carrier)?),
                Binding::Accumulator => Binding::Accumulator, Binding::Item => Binding::Item,
            });
        }
        for old in &layout.functions { arena.functions.push(self.arena.functions[*old].clone()); }
        for old in &layout.evidence { arena.evidence.push(relocation.usage(&self.arena.evidence[*old])?); }
        for old in &layout.carriers { arena.carriers.push(self.arena.carriers[*old].clone()); }
        for old in &layout.groups {
            let value = &self.arena.groups[*old];
            arena.groups.push(IndexedGroup {
                axes: value.axes.iter().map(|slot| relocation.carrier(*slot)).collect::<Result<_>>()?,
                members: value.members.iter().map(|(tuple, slot)| Ok((tuple.clone(),
                    slot.map(|slot| relocation.symbol(slot)).transpose()?))).collect::<Result<_>>()?,
            });
        }
        for slot in &actual {
            if let Some(value) = self.arena.member_ranges.get(slot) {
                arena.member_ranges.insert(relocation.symbol(*slot)?, MemberRange {
                    lower: relocation.node(value.lower)?, upper: relocation.node(value.upper)?, layer: value.layer,
                });
            }
        }
        let used = arena.nodes.len().saturating_add(arena.predicates.len())
            .saturating_add(arena.binders.len()).saturating_add(arena.functions.len())
            .saturating_add(arena.evidence.len()).saturating_add(arena.carriers.len())
            .saturating_add(arena.groups.len()).saturating_add(arena.member_ranges.len())
            .saturating_add(arena.finite_folds.values().map(|fold| fold.selected.len().saturating_add(1)).sum::<usize>())
            .saturating_add(admissions.len()).saturating_add(literals.len());
        Ok(CapturedFragment { premises: CapturePremises { fragment: Fragment {
            arena: Arc::new(arena), admissions: Arc::new(admissions), literals: Arc::new(literals),
            scalars: contracts.into(), outputs: relocation.nodes(outputs)?,
            guards: guards.iter().map(|slot| relocation.predicate(*slot)).collect::<Result<_>>()?, used, prefix:Prefix::default(),
        }, symbols: local_symbols }, actual, bindings:ForeignBindings {prefix:Prefix {nodes:layout.nodes.len(),predicates:layout.predicates.len(),binders:layout.binders.len(),functions:layout.functions.len(),evidence:layout.evidence.len(),carriers:layout.carriers.len(),groups:layout.groups.len()},relocation:inverse(&relocation)} })
    }

    /// Capture selected existing scalar premises without realizing any lazy member.
    pub(crate) fn capture_scalar_premises(&self, symbols: &crate::specialize::Symbols, selected: &[SymbolIx]) -> Result<CapturedFragment> {
        let roots = selected.iter().filter_map(|slot| symbols.at(*slot).and_then(|symbol| symbol.definition)).collect::<Vec<_>>();
        let mut captured = self.capture_fragment(&roots, &[], symbols)?;
        let capture=&mut captured.premises;
        let mut contracts = capture.fragment.scalars.to_vec();
        for slot in selected {
            let symbol = symbols.at(*slot).ok_or_else(|| invalid(SemanticId::NIL, "foreign scalar premise has no owner"))?;
            if !capture.symbols.contains_key(&symbol.id) {
                let mut symbol=symbol.clone();
                symbol.definition=symbol.definition.map(|root| roots.iter().position(|candidate|*candidate==root)
                    .map(|ordinal|capture.fragment.outputs[ordinal]).ok_or_else(||invalid(symbol.id,"foreign scalar definition escaped reached capture"))).transpose()?;
                contracts.push(ScalarContract::of(&symbol));
                capture.symbols.push(symbol);
                captured.actual.push(*slot);
            }
        }
        capture.fragment.scalars = contracts.into();
        Ok(captured)
    }

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
            self.used, &outputs, &guards, bindings,Prefix::default(),None)?;
        Ok(Fragment {
            arena: Arc::new(local.arena), admissions: Arc::new(local.admissions),
            literals: Arc::new(local.literals), scalars: scalars.into(),
            outputs: imported.outputs, guards: imported.guards, used: self.used, prefix:Prefix::default(),
        })
    }

    /// Freeze only newly constructed spaces. Foreign typed premises remain
    /// borrowed coordinates to be rebound at import; no captured prefix is owned twice.
    pub(crate) fn into_fragment_delta(self,outputs:Vec<NodeIx>,guards:Vec<PredicateIx>,symbols:&crate::specialize::Symbols,foreign:&CapturePremises)->Result<Fragment> {
        let slots=(0..symbols.len()).map(SymbolIx::from).collect::<Vec<_>>();
        let mut fragment=self.into_fragment(outputs,guards,symbols,&slots)?;
        let prefix=Prefix::of(&foreign.fragment.arena);
        let arena=Arc::make_mut(&mut fragment.arena);
        arena.nodes=std::mem::take(&mut arena.nodes).into_iter().skip(prefix.nodes).collect();
        arena.predicates=std::mem::take(&mut arena.predicates).into_iter().skip(prefix.predicates).collect();
        arena.binders=std::mem::take(&mut arena.binders).into_iter().skip(prefix.binders).collect();
        arena.functions=std::mem::take(&mut arena.functions).into_iter().skip(prefix.functions).collect();
        arena.evidence=std::mem::take(&mut arena.evidence).into_iter().skip(prefix.evidence).collect();
        arena.carriers=std::mem::take(&mut arena.carriers).into_iter().skip(prefix.carriers).collect();
        arena.groups=std::mem::take(&mut arena.groups).into_iter().skip(prefix.groups).collect();
        arena.node_sources.retain(|slot,_|slot.position()>=prefix.nodes);
        arena.predicate_sources.retain(|slot,_|slot.position()>=prefix.predicates);
        arena.finite_folds.retain(|slot,_|slot.position()>=prefix.nodes);
        arena.member_ranges.retain(|slot,_|!foreign.fragment.arena.member_ranges.contains_key(slot));
        Arc::make_mut(&mut fragment.admissions).retain(|slot,value|slot.position()>=prefix.nodes || foreign.fragment.admissions.get(slot).is_none_or(|prior|PhysicalAdmissionRecord::capture(prior)!=PhysicalAdmissionRecord::capture(value)));
        Arc::make_mut(&mut fragment.literals).retain(|slot,value|slot.position()>=prefix.nodes || foreign.fragment.literals.get(slot)!=Some(value));
        fragment.used=fragment.used.checked_sub(foreign.fragment.used).ok_or_else(||invalid(SemanticId::NIL,"fragment foreign construction charge differs"))?;
        fragment.prefix=prefix;
        Ok(fragment)
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
        self.import_fragment_with_foreign(fragment,symbols,actual,None)
    }
    pub(crate) fn import_fragment_with_foreign(&mut self, fragment:&Fragment, symbols:&crate::specialize::Symbols, actual:&[SymbolIx], foreign:Option<&ForeignBindings>)->Result<ImportedFragment> {
        if actual.len()!=fragment.scalars.len() {return Err(invalid(SemanticId::NIL,"fragment scalar binding arity differs"));}
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
            fragment.used, &fragment.outputs, &fragment.guards, bindings,fragment.prefix,foreign)
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
        prefix:Prefix,
        foreign:Option<&ForeignBindings>,
    ) -> Result<ImportedFragment> {
        if foreign.map_or(Prefix::default(),|foreign|foreign.prefix)!=prefix {return Err(invalid(SemanticId::NIL,"fragment foreign coordinate contract differs"));}
        fn coordinates<T:Ord+Copy+From<usize>>(count:usize,start:usize,prefix:usize,foreign:Option<&BTreeMap<T,T>>)->Result<BTreeMap<T,T>> {
            let mut mapping=foreign.cloned().unwrap_or_default();
            if mapping.len()!=prefix {return Err(invalid(SemanticId::NIL,"fragment foreign space arity differs"));}
            for ordinal in 0..count {let target=start.checked_add(ordinal).ok_or_else(||crate::ModelingError::Budget("fragment coordinate overflow".into()))?;mapping.insert(T::from(prefix+ordinal),T::from(target));}
            Ok(mapping)
        }
        let relocation = Relocation {
            symbols,
            nodes: coordinates(source.nodes.len(), self.arena.nodes.len(),prefix.nodes,foreign.map(|value|&value.relocation.nodes))?,
            predicates: coordinates(source.predicates.len(), self.arena.predicates.len(),prefix.predicates,foreign.map(|value|&value.relocation.predicates))?,
            binders: coordinates(source.binders.len(), self.arena.binders.len(),prefix.binders,foreign.map(|value|&value.relocation.binders))?,
            functions: coordinates(source.functions.len(), self.arena.functions.len(),prefix.functions,foreign.map(|value|&value.relocation.functions))?,
            evidence: coordinates(source.evidence.len(), self.arena.evidence.len(),prefix.evidence,foreign.map(|value|&value.relocation.evidence))?,
            carriers: coordinates(source.carriers.len(), self.arena.carriers.len(),prefix.carriers,foreign.map(|value|&value.relocation.carriers))?,
            groups: coordinates(source.groups.len(), self.arena.groups.len(),prefix.groups,foreign.map(|value|&value.relocation.groups))?,
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
        for (slot,value) in &admissions {if self.admissions.get(slot).is_some_and(|prior|PhysicalAdmissionRecord::capture(prior)!=PhysicalAdmissionRecord::capture(value)){return Err(invalid(SemanticId::NIL,"fragment changes an existing source physical obligation"));}}
        for (slot,value) in &literals {if self.literals.get(slot).is_some_and(|prior|prior!=value){return Err(invalid(SemanticId::NIL,"fragment changes an existing contextual literal contract"));}}
        self.reserve(used)?;
        // All validation precedes publication. Source/function provenance stays
        // original, unlike the attribution-neutral mathematical-region contract.
        for (ordinal, function) in arena.functions.iter().enumerate() {
            let source_slot = FunctionIx::from(prefix.functions+ordinal);
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
    fn fragment_capture_selects_foreign_definition_closure_without_root_offsets() {
        fn capture(prefix: bool) -> CapturedFragment {
            let original = scalar_symbols(10, Type::Integer);
            let mut symbols = crate::specialize::Symbols::default();
            if prefix { symbols.push(scalar_symbols(90, Type::Integer).at(SymbolIx::from(0)).unwrap().clone()); }
            for symbol in original.slots() { symbols.push(symbol.clone()); }
            let selected = symbols.slot(SemanticId::from_bytes([11;16])).unwrap();
            let mut root = Construction::new(128);
            if prefix { literal(&mut root, "999", 99); }
            let one = literal(&mut root, "1", 1);
            let definition = root.node(Node::Neg(one), source(2)).unwrap();
            symbols.define(selected, definition).unwrap();
            let reference = root.node(Node::Reference(Reference::Symbol(selected)), source(3)).unwrap();
            let zero = literal(&mut root, "0", 4);
            root.member_range(selected, MemberRange { lower: zero, upper: one,
                layer: pse_model::generated::enums::ModelingValidityLayer::Form }).unwrap();
            let guard = root.predicate(ResolvedPredicate::Compare {
                op: CompareOp::Ge, lhs: reference, rhs: zero,
            }, source(5)).unwrap();
            literal(&mut root, "888", 88);
            root.capture_fragment(&[reference], &[guard], &symbols).unwrap()
        }
        let shifted = capture(true);
        let capture = capture(false);
        assert_ne!(capture.actual, shifted.actual);
        assert_eq!(capture.premises, shifted.premises);
        let capture = capture.premises;
        let shifted = shifted.premises;
        assert_eq!(capture.fragment, shifted.fragment);
        assert_eq!(capture.symbols, shifted.symbols);
        assert_eq!(capture.symbols.len(), 1);
        assert_eq!(capture.fragment.arena.nodes.len(), 4);
        assert!(capture.retained_bytes() > capture.fragment.retained_bytes());
        let mut local = Construction::new(128);
        let imported = local.import_fragment(&capture.fragment, &capture.symbols, &[SymbolIx::from(0)]).unwrap();
        let definition = capture.symbols.at(SymbolIx::from(0)).unwrap().definition.unwrap();
        assert!(matches!(local.arena.nodes[imported.node(definition).unwrap()], Node::Neg(_)));
        assert_eq!(local.node_source(imported.outputs[0]).unwrap(), &source(3));
        assert_eq!(local.predicate_source(imported.guards[0]).unwrap(), &source(5));
        assert_eq!(local.layout(&capture.symbols, &imported.outputs, &imported.guards).unwrap().symbols.len(), 1);
    }
    #[test]
    fn fragment_delta_binds_foreign_nodes_without_republishing_their_owner() {
        let mut symbols=scalar_symbols(10,Type::Integer);
        let mut root=Construction::new(128);
        literal(&mut root,"99",99);
        let one=literal(&mut root,"1",1);
        let definition=root.node(Node::Neg(one),source(2)).unwrap();
        symbols.define(SymbolIx::from(1),definition).unwrap();
        let captured=root.capture_scalar_premises(&symbols,&[SymbolIx::from(1)]).unwrap();
        let premises=&captured.premises;
        let mut local=Construction::new(128);
        let slots=(0..premises.symbols.len()).map(SymbolIx::from).collect::<Vec<_>>();
        let imported=local.import_fragment(&premises.fragment,&premises.symbols,&slots).unwrap();
        let output=local.node(Node::Neg(imported.outputs[0]),source(3)).unwrap();
        let delta=local.into_fragment_delta(vec![output,imported.outputs[0]],vec![],&premises.symbols,premises).unwrap();
        assert_eq!(delta.arena.nodes.len(),1);
        assert_eq!(delta.used,2,"the new operation and its one edge are charged; foreign operations are not");
        let before=root.arena.nodes.len();
        assert!(root.import_fragment(&delta,&symbols,&captured.actual).is_err());
        assert_eq!(root.arena.nodes.len(),before);
        let imported=root.import_fragment_with_foreign(&delta,&symbols,&captured.actual,Some(&captured.bindings)).unwrap();
        assert_eq!(root.arena.nodes.len(),before+1);
        assert_eq!(imported.outputs[1],definition);
        assert_eq!(root.arena.nodes[imported.outputs[0]],Node::Neg(definition));
        assert_eq!(root.node_source(imported.outputs[0]).unwrap(),&source(3));
    }
    #[test]
    fn fragment_delta_publishes_new_literal_obligation_on_a_bound_foreign_node() {
        let mut symbols=scalar_symbols(10,Type::Integer);
        let mut root=Construction::new(32);
        let ExprKind::Number(number)=dsl::parse_expr("1").unwrap().kind else {panic!()};
        let original=root.node(Node::Number(number),source(1)).unwrap();
        symbols.define(SymbolIx::from(1),original).unwrap();
        let captured=root.capture_scalar_premises(&symbols,&[SymbolIx::from(1)]).unwrap();
        let mut local=Construction::new(32);
        let actual=(0..captured.premises.symbols.len()).map(SymbolIx::from).collect::<Vec<_>>();
        let imported=local.import_fragment(&captured.premises.fragment,&captured.premises.symbols,&actual).unwrap();
        local.literal(imported.outputs[0],Type::Integer).unwrap();
        let delta=local.into_fragment_delta(imported.outputs,vec![],&captured.premises.symbols,&captured.premises).unwrap();
        assert!(delta.arena.nodes.is_empty());
        assert_eq!(delta.used,1);
        let before=root.arena.nodes.len();
        root.import_fragment_with_foreign(&delta,&symbols,&captured.actual,Some(&captured.bindings)).unwrap();
        assert_eq!(root.arena.nodes.len(),before);
        assert_eq!(root.literal_at(original),Some(&Type::Integer));
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
    #[test]
    fn fragment_import_preserves_finite_fold_membership_and_carrier_order() {
        use crate::specialize::Value;
        let symbols = crate::specialize::Symbols::default();
        let mut local = Construction::new(32);
        let carrier = local.carrier(Carrier {
            kind: CarrierKind::OrderedSet, element: Type::Integer,
            members: vec![Value::Integer(4), Value::Integer(9)].into(),
        }).unwrap();
        let value = literal(&mut local, "2", 0);
        let binder = local.binding(Binding::Value(value)).unwrap();
        let root = local.node(Node::Let { bindings: vec![binder], body: value }, source(1)).unwrap();
        local.finite_fold(root, carrier, vec![Value::Integer(9)], Type::Integer).unwrap();
        let fragment = local.into_fragment(vec![root], vec![], &symbols, &[]).unwrap();
        let mut target = Construction::new(64);
        target.carrier(Carrier {
            kind: CarrierKind::OrderedSet, element: Type::Boolean,
            members: vec![Value::Boolean(false)].into(),
        }).unwrap();
        let imported = target.import_fragment(&fragment, &symbols, &[]).unwrap();
        let fold = target.fold_at(imported.outputs[0]).unwrap();
        assert_eq!(fold.carrier, CarrierIx::from(1));
        assert_eq!(fold.selected.as_ref(), &[Value::Integer(9)]);
        assert_eq!(target.carrier_at(fold.carrier).unwrap().members.as_ref(),
            &[Value::Integer(4), Value::Integer(9)]);
        assert_eq!(target.node_source(imported.outputs[0]).unwrap(), &source(1));
    }
}
