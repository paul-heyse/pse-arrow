// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Reached region selection and pure coordinate relocation. No syntax is interpreted here.
use super::*;
use std::collections::BTreeSet;
#[path = "provider_substitution.rs"]
mod provider_substitution;
#[path = "region_composition.rs"]
mod region_composition;
pub use provider_substitution::ProviderSubstitution;

/// Model-owned numerical construction requested for an already issued typed root.
#[derive(Clone, Copy, Debug)]
pub enum RegionOutput {
    /// Existing physical scalar.
    Value(NodeIx),
    /// Original relation residual, admitted as the actual left-minus-right operation.
    RelationResidual(Relation),
    /// Numerical indicator of the original predicate, not an assertion that it holds.
    PredicateIndicator(PredicateIx),
    /// Difference from a demand-issued physical parameter with the same complete point contract.
    ConditionalBoundary(SemanticId),
}
/// Actual demand supplied by the compiler; every operand was issued by specialization.
#[derive(Clone, Copy, Debug)]
pub struct RegionRequest<'a> {
    /// Requested values and model-owned derived constructors in semantic order.
    pub outputs: &'a [RegionOutput],
    /// Original guards in requested semantic order.
    pub validity: &'a [PredicateIx],
}
/// A local role refers to exactly one current instance scalar; values remain outside bodies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionBinding {
    /// Scalar in the selected compact owner.
    pub local: SymbolIx,
    /// Current actual instantiated scalar.
    pub actual: SemanticId,
    /// Derivative coordinate, parameter or defined member.
    pub role: LocalRole,
}
/// Current receiver attribution. This map is not part of a reusable body's semantic key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RegionAttribution {
    /// Compact operation to its actual original occurrence.
    pub nodes: BTreeMap<NodeIx, Source>,
    /// Compact guard to its actual original occurrence.
    pub predicates: BTreeMap<PredicateIx, Source>,
}
/// Exact root dependencies; original guard reads remain distinct from numerical values.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RegionReads {
    /// Actual scalar identities consumed numerically, including original member links.
    pub numeric: BTreeSet<SemanticId>,
    /// Actual scalar identities consumed by original guards and scientific observations.
    pub guards: BTreeSet<SemanticId>,
}
/// One compact region. Current instance bindings and attribution have separate owners.
#[derive(Clone, Debug)]
pub struct SelectedRegion {
    /// Only reached typed syntax; this never holds the original whole model's scalar Arc.
    pub syntax: ResolvedSyntax,
    /// Independent local scalar slots in canonical first-use order.
    pub inputs: Vec<SymbolIx>,
    /// Exact complete semantic support meaning, prepared once with this region.
    support_identity: ContentHash,
    /// Exact physical products, relocated without reinference or readmission.
    pub support: PhysicalSupport,
    /// Current receiver bindings, never part of the portable arithmetic definition.
    pub bindings: Vec<RegionBinding>,
    /// Current original source attribution.
    pub attribution: RegionAttribution,
    /// Local checked-member token to actual current scalar member.
    pub checked_members: BTreeMap<SemanticId, SemanticId>,
    /// Distinct local checked-source token to actual current annotation declaration.
    pub checked_sources: BTreeMap<SemanticId, SemanticId>,
    /// Derivations issued by demand-specific model constructors, kept outside reusable math.
    pub derivations: crate::derived::Facts,
    /// Complete current original source correspondence, including selected function call paths.
    pub occurrences: BTreeMap<SemanticId, pse_model::diagnostic::SourceLocation>,
    /// Actual current numerical and guard dependency identities.
    pub reads: RegionReads,
}
impl SelectedRegion {
    /// Conservative owned-container extent, including current receiver attribution.
    /// Shared immutable owners may be counted more than once; this is not RSS.
    pub fn retained_bytes(&self) -> usize {
        fn map<K, V>(value: &BTreeMap<K, V>) -> usize {
            value.len().saturating_mul(size_of::<(K, V)>() + 96)
        }
        size_of::<Self>()
            .saturating_add(self.syntax.retained_bytes())
            .saturating_add(self.support.retained_bytes())
            .saturating_add(self.inputs.capacity().saturating_mul(size_of::<SymbolIx>()))
            .saturating_add(
                self.bindings
                    .capacity()
                    .saturating_mul(size_of::<RegionBinding>()),
            )
            .saturating_add(map(&self.attribution.nodes))
            .saturating_add(map(&self.attribution.predicates))
            .saturating_add(
                self.attribution
                    .nodes
                    .values()
                    .chain(self.attribution.predicates.values())
                    .map(|source| source.field.role.capacity())
                    .sum::<usize>(),
            )
            .saturating_add(map(&self.checked_members))
            .saturating_add(map(&self.checked_sources))
            .saturating_add(self.derivations.retained_bytes())
            .saturating_add(map(&self.occurrences))
            .saturating_add(
                self.occurrences
                    .values()
                    .map(pse_model::diagnostic::SourceLocation::heap_bytes)
                    .sum::<usize>(),
            )
            .saturating_add(
                self.reads
                    .numeric
                    .len()
                    .saturating_add(self.reads.guards.len())
                    .saturating_mul(size_of::<SemanticId>() + 96),
            )
    }
}
impl PartialEq for SelectedRegion {
    fn eq(&self, other: &Self) -> bool {
        self.syntax == other.syntax
            && self.support_identity == other.support_identity
            && self.inputs == other.inputs
            && self.bindings == other.bindings
            && self.attribution == other.attribution
            && self.reads == other.reads
            && self.checked_members == other.checked_members
            && self.checked_sources == other.checked_sources
            && self.derivations == other.derivations
            && self.occurrences == other.occurrences
    }
}
fn support_identity(support: &PhysicalSupport) -> ContentHash {
    let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
    support.frame(&mut hash);
    hash.finish_hash()
}
/// Source-role tokens issued from the same canonical scalar correspondence as body identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckedRange {
    /// Local scalar owner.
    pub symbol: SymbolIx,
    /// Checked member token, independently bound to the current actual scalar.
    pub member: SemanticId,
    /// Distinct checked source token, bound to the current actual annotation.
    pub source: SemanticId,
}
impl crate::SpecializedModel {
    /// Obtain the source producer's exact scalar reference without expanding its definition.
    pub fn symbol_root(&self, id: SemanticId) -> Result<NodeIx> {
        self.symbol_roots
            .get(&id)
            .copied()
            .ok_or_else(|| invalid(id, "scalar has no producer-issued reference root"))
    }
    /// Select one already constructed region and prepare its actual consumed physical
    /// context once, before compacting both syntax and support with the same correspondence.
    pub fn prepare_region(
        &self,
        request: RegionRequest<'_>,
        context: &crate::TypeContext<'_>,
    ) -> Result<SelectedRegion> {
        let base_outputs = request
            .outputs
            .iter()
            .map(|output| match output {
                RegionOutput::ConditionalBoundary(id) => {
                    Ok(RegionOutput::Value(self.symbol_root(*id)?))
                }
                other => Ok(*other),
            })
            .collect::<Result<Vec<_>>>()?;
        let mut roots = Vec::new();
        let mut predicates = request.validity.to_vec();
        for output in &base_outputs {
            match output {
                RegionOutput::Value(root) => roots.push(*root),
                RegionOutput::RelationResidual(relation) => {
                    roots.extend([relation.lhs, relation.rhs])
                }
                RegionOutput::PredicateIndicator(predicate) => predicates.push(*predicate),
                RegionOutput::ConditionalBoundary(id) => {
                    return Err(invalid(
                        *id,
                        "conditional boundary was not issued before selection",
                    ));
                }
            }
        }
        let selected = self.resolved.select(roots, predicates);
        let layout = selected.canonical_layout()?;
        let mut reads = reads_for(
            &selected,
            RegionRequest {
                outputs: &base_outputs,
                validity: request.validity,
            },
        )?;
        let (mut syntax, mut inputs, mut bindings, mut attribution, relocation) =
            compact(&selected, &layout)?;
        let mut derivations = crate::derived::Facts::default();
        let mut conditional_nodes = Vec::new();
        let mut conditional_parameters = BTreeMap::<SemanticId, NodeIx>::new();
        // Only the compact reached owner detaches here, never the source model's Arena.
        syntax.outputs.clear();
        syntax.validity = request
            .validity
            .iter()
            .map(|root| relocation.predicate(*root))
            .collect::<Result<_>>()?;
        for output in request.outputs {
            let root = match output {
                RegionOutput::Value(root) => relocation.node(*root)?,
                RegionOutput::RelationResidual(relation) => {
                    let lhs = relocation.node(relation.lhs)?;
                    let rhs = relocation.node(relation.rhs)?;
                    let root = NodeIx::from(syntax.nodes.len());
                    syntax.nodes.push(Node::Binary {
                        op: BinaryOp::Sub,
                        lhs,
                        rhs,
                    });
                    let mut source = attribution.nodes.get(&lhs).cloned().ok_or_else(|| {
                        invalid(
                            SemanticId::NIL,
                            "relation residual lacks original operand attribution",
                        )
                    })?;
                    source.coordinate = None;
                    attribution.nodes.insert(root, source);
                    syntax
                        .node_sources
                        .insert(root, neutral_source(root.position()));
                    root
                }
                RegionOutput::ConditionalBoundary(id) => {
                    let lhs = relocation.node(self.symbol_root(*id)?)?;
                    let rhs = if let Some(root) = conditional_parameters.get(id) {
                        *root
                    } else {
                        let actual = self.symbols.get(id).ok_or_else(|| {
                            invalid(*id, "conditional boundary has no actual scalar contract")
                        })?;
                        let parameter = derivations.publish(
                            crate::derived::Relationship::ConditionalBoundaryValue { symbol: *id },
                        )?;
                        derivations.publish(
                            crate::derived::Relationship::ConditionalBoundaryResidual {
                                symbol: *id,
                            },
                        )?;
                        let local = SymbolIx::from(syntax.symbols.len());
                        let ordinal = bindings
                            .iter()
                            .filter(|binding| binding.role == LocalRole::Parameter)
                            .count();
                        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
                        hash.str("local-scalar")
                            .str("parameter")
                            .u64(ordinal as u64);
                        syntax.symbols.push(crate::specialize::Symbol {
                            id: hash.finish_id(),
                            ty: actual.ty.clone(),
                            role: ModelingDeclarationKind::Parameter,
                            domain: ModelingVariableDomain::Continuous,
                            definition: None,
                            expression: None,
                            initial: None,
                            lineage: crate::specialize::Lineage {
                                declaration: SemanticId::NIL.into(),
                                instance: crate::InstanceId::from_id(SemanticId::NIL),
                                path: String::new(),
                                demand: vec![],
                                default_owner: None,
                                is_override: false,
                                presets: vec![],
                            },
                        });
                        inputs.push(local);
                        bindings.push(RegionBinding {
                            local,
                            actual: parameter,
                            role: LocalRole::Parameter,
                        });
                        reads.numeric.insert(parameter);
                        let root = NodeIx::from(syntax.nodes.len());
                        syntax.nodes.push(Node::Reference(Reference::Symbol(local)));
                        let mut source = attribution.nodes.get(&lhs).cloned().ok_or_else(|| {
                            invalid(
                                *id,
                                "conditional boundary lacks original source attribution",
                            )
                        })?;
                        source.coordinate = None;
                        attribution.nodes.insert(root, source);
                        syntax
                            .node_sources
                            .insert(root, neutral_source(root.position()));
                        conditional_parameters.insert(*id, root);
                        root
                    };
                    let root = NodeIx::from(syntax.nodes.len());
                    syntax.nodes.push(Node::Binary {
                        op: BinaryOp::Sub,
                        lhs,
                        rhs,
                    });
                    let mut source = attribution.nodes.get(&lhs).cloned().ok_or_else(|| {
                        invalid(*id, "conditional residual lacks original attribution")
                    })?;
                    source.coordinate = None;
                    attribution.nodes.insert(root, source);
                    syntax
                        .node_sources
                        .insert(root, neutral_source(root.position()));
                    conditional_nodes.push(root);
                    root
                }
                RegionOutput::PredicateIndicator(predicate) => {
                    let predicate = relocation.predicate(*predicate)?;
                    let root = NodeIx::from(syntax.nodes.len());
                    syntax.nodes.push(Node::Indicator(predicate));
                    let mut source =
                        attribution
                            .predicates
                            .get(&predicate)
                            .cloned()
                            .ok_or_else(|| {
                                invalid(
                                    SemanticId::NIL,
                                    "predicate indicator lacks original guard attribution",
                                )
                            })?;
                    source.coordinate = None;
                    attribution.nodes.insert(root, source);
                    syntax
                        .node_sources
                        .insert(root, neutral_source(root.position()));
                    root
                }
            };
            syntax.outputs.push(root);
        }
        let reached = layout.nodes.iter().copied().collect::<BTreeSet<_>>();
        let admissions = self
            .physical_admissions
            .iter()
            .filter(|(slot, _)| reached.contains(slot))
            .map(|(slot, value)| Ok((relocation.node(*slot)?, value.clone())))
            .collect::<Result<_>>()?;
        let literals = self
            .literal_contracts
            .iter()
            .filter(|(slot, _)| reached.contains(slot))
            .map(|(slot, value)| Ok((relocation.node(*slot)?, value.clone())))
            .collect::<Result<_>>()?;
        let support = PhysicalSupport::prepare_reached(&syntax, context, &literals, &admissions)?
            .with_inputs(&syntax, &inputs, context)?;
        for root in conditional_nodes {
            let operation = support.node(root)?.operation.as_ref().ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "conditional boundary lacks its actual subtraction admission",
                )
            })?;
            if operation.operand_scales != [1.0, 1.0] || operation.result_scale != 1.0 {
                return Err(invalid(
                    SemanticId::NIL,
                    "conditional boundary requires canonical difference coordinates",
                ));
            }
        }
        let inverse_symbols = relocation
            .symbols
            .iter()
            .map(|(old, local)| (*local, *old))
            .collect::<BTreeMap<_, _>>();
        let actual_bindings = bindings
            .iter()
            .map(|binding| (binding.local, binding.actual))
            .collect::<BTreeMap<_, _>>();
        let mut checked_members = BTreeMap::new();
        let mut checked_sources = BTreeMap::new();
        for range in support.checked_ranges(&syntax)? {
            let old = inverse_symbols.get(&range.symbol).ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "checked range has no original source scalar",
                )
            })?;
            checked_members.insert(
                range.member,
                *actual_bindings.get(&range.symbol).ok_or_else(|| {
                    invalid(SemanticId::NIL, "checked range lacks actual member binding")
                })?,
            );
            checked_sources.insert(
                range.source,
                *self.member_range_sources.get(old).ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "range source lacks its actual annotation owner",
                    )
                })?,
            );
        }
        let support_identity = support_identity(&support);
        let occurrences = occurrences::collect(self, &syntax, &attribution)?;
        Ok(SelectedRegion {
            syntax,
            inputs,
            support,
            support_identity,
            bindings,
            attribution,
            reads,
            checked_members,
            checked_sources,
            derivations,
            occurrences,
        })
    }
}
/// Coordinate maps are issued from exactly one canonical first-use walk.
#[derive(Debug)]
pub(super) struct Relocation {
    pub(super) symbols: BTreeMap<SymbolIx, SymbolIx>,
    pub(super) nodes: BTreeMap<NodeIx, NodeIx>,
    pub(super) predicates: BTreeMap<PredicateIx, PredicateIx>,
    pub(super) binders: BTreeMap<BinderIx, BinderIx>,
    pub(super) functions: BTreeMap<FunctionIx, FunctionIx>,
    pub(super) evidence: BTreeMap<EvidenceIx, EvidenceIx>,
    pub(super) carriers: BTreeMap<CarrierIx, CarrierIx>,
    pub(super) groups: BTreeMap<GroupIx, GroupIx>,
}
fn map<T: Ord + Copy, U: From<usize>>(slots: impl IntoIterator<Item = T>) -> BTreeMap<T, U> {
    slots
        .into_iter()
        .enumerate()
        .map(|(ordinal, slot)| (slot, U::from(ordinal)))
        .collect()
}
fn required<T: Ord, U: Copy>(map: &BTreeMap<T, U>, slot: &T, kind: &str) -> Result<U> {
    map.get(slot).copied().ok_or_else(|| {
        invalid(
            SemanticId::NIL,
            format!("selected {kind} is outside the canonical reached correspondence"),
        )
    })
}
impl Relocation {
    fn new(layout: &CanonicalLayout) -> Self {
        Self {
            symbols: map(layout.symbols.iter().map(|value| value.symbol)),
            nodes: map(layout.nodes.iter().copied()),
            predicates: map(layout.predicates.iter().copied()),
            binders: map(layout.binders.iter().copied()),
            functions: map(layout.functions.iter().copied()),
            evidence: map(layout.evidence.iter().copied()),
            carriers: map(layout.carriers.iter().copied()),
            groups: map(layout.groups.iter().copied()),
        }
    }
    pub(super) fn node(&self, slot: NodeIx) -> Result<NodeIx> {
        required(&self.nodes, &slot, "operation")
    }
    pub(super) fn function(&self, slot: FunctionIx) -> Result<FunctionIx> {
        required(&self.functions, &slot, "function")
    }
    pub(super) fn predicate(&self, slot: PredicateIx) -> Result<PredicateIx> {
        required(&self.predicates, &slot, "predicate")
    }
    fn binder(&self, slot: BinderIx) -> Result<BinderIx> {
        required(&self.binders, &slot, "binding")
    }
    pub(super) fn carrier(&self, slot: CarrierIx) -> Result<CarrierIx> {
        required(&self.carriers, &slot, "carrier")
    }
    fn group(&self, slot: GroupIx) -> Result<GroupIx> {
        required(&self.groups, &slot, "group")
    }
    pub(super) fn symbol(&self, slot: SymbolIx) -> Result<SymbolIx> {
        required(&self.symbols, &slot, "scalar")
    }
    fn evidence(&self, slot: EvidenceIx) -> Result<EvidenceIx> {
        required(&self.evidence, &slot, "evidence")
    }
    pub(super) fn nodes(&self, values: &[NodeIx]) -> Result<Vec<NodeIx>> {
        values.iter().map(|slot| self.node(*slot)).collect()
    }
    pub(super) fn usage(&self, usage: &EvidenceUse) -> Result<EvidenceUse> {
        Ok(EvidenceUse {
            node: usage.node.clone(),
            predicates: usage
                .predicates
                .iter()
                .map(|slot| self.predicate(*slot))
                .collect::<Result<_>>()?,
            inputs: self.nodes(&usage.inputs)?,
        })
    }
    pub(super) fn operation(&self, value: &Node) -> Result<Node> {
        Ok(match value {
            Node::Number(number) => Node::Number(number.clone()),
            Node::Reference(reference) => Node::Reference(match reference {
                Reference::Symbol(slot) => Reference::Symbol(self.symbol(*slot)?),
                Reference::Lexical(slot) => Reference::Lexical(self.binder(*slot)?),
                Reference::Indexed { group, indices } => Reference::Indexed {
                    group: self.group(*group)?,
                    indices: self.nodes(indices)?,
                },
            }),
            Node::Neg(value) => Node::Neg(self.node(*value)?),
            Node::Indicator(value) => Node::Indicator(self.predicate(*value)?),
            Node::Binary { op, lhs, rhs } => Node::Binary {
                op: *op,
                lhs: self.node(*lhs)?,
                rhs: self.node(*rhs)?,
            },
            Node::Call { function, args } => Node::Call {
                function: *function,
                args: self.nodes(args)?,
            },
            Node::Apply { function, args } => Node::Apply {
                function: self.function(*function)?,
                args: self.nodes(args)?,
            },
            Node::Partial {
                function,
                args,
                wrt,
            } => Node::Partial {
                function: self.function(*function)?,
                args: self.nodes(args)?,
                wrt: wrt.clone(),
            },
            Node::Provider { selection, args } => Node::Provider {
                selection: *selection,
                args: self.nodes(args)?,
            },
            Node::Conditional {
                guard,
                then,
                otherwise,
            } => Node::Conditional {
                guard: self.predicate(*guard)?,
                then: self.node(*then)?,
                otherwise: self.node(*otherwise)?,
            },
            Node::Let { bindings, body } => Node::Let {
                bindings: bindings
                    .iter()
                    .map(|slot| self.binder(*slot))
                    .collect::<Result<_>>()?,
                body: self.node(*body)?,
            },
            Node::Sequence {
                value,
                prerequisites,
            } => Node::Sequence {
                value: self.node(*value)?,
                prerequisites: self.nodes(prerequisites)?,
            },
            Node::Evidence { value, evidence } => Node::Evidence {
                value: self.node(*value)?,
                evidence: self.evidence(*evidence)?,
            },
            Node::Reduce {
                kind,
                binder,
                filter,
                body,
            } => Node::Reduce {
                kind: *kind,
                binder: self.binder(*binder)?,
                filter: filter.map(|slot| self.predicate(slot)).transpose()?,
                body: self.node(*body)?,
            },
            Node::Fold {
                binder,
                filter,
                value,
                accumulator,
                item,
                step,
            } => Node::Fold {
                binder: self.binder(*binder)?,
                filter: filter.map(|slot| self.predicate(slot)).transpose()?,
                value: self.node(*value)?,
                accumulator: self.binder(*accumulator)?,
                item: self.binder(*item)?,
                step: self.node(*step)?,
            },
            Node::Derivative { body, domain } => Node::Derivative {
                body: self.node(*body)?,
                domain: self.carrier(*domain)?,
            },
        })
    }
    pub(super) fn guard(&self, value: &ResolvedPredicate) -> Result<ResolvedPredicate> {
        Ok(match value {
            ResolvedPredicate::Compare { op, lhs, rhs } => ResolvedPredicate::Compare {
                op: *op,
                lhs: self.node(*lhs)?,
                rhs: self.node(*rhs)?,
            },
            ResolvedPredicate::In { value, domain } => ResolvedPredicate::In {
                value: self.node(*value)?,
                domain: self.carrier(*domain)?,
            },
            ResolvedPredicate::And(lhs, rhs) => {
                ResolvedPredicate::And(self.predicate(*lhs)?, self.predicate(*rhs)?)
            }
            ResolvedPredicate::Or(lhs, rhs) => {
                ResolvedPredicate::Or(self.predicate(*lhs)?, self.predicate(*rhs)?)
            }
            ResolvedPredicate::Not(value) => ResolvedPredicate::Not(self.predicate(*value)?),
            ResolvedPredicate::Atom(value) => ResolvedPredicate::Atom(self.node(*value)?),
            ResolvedPredicate::Bool(value) => ResolvedPredicate::Bool(*value),
            ResolvedPredicate::Null => ResolvedPredicate::Null,
        })
    }
}
fn neutral_source(position: usize) -> Source {
    Source {
        field: crate::expression::occurrences::OccurrenceKey {
            declaration: SemanticId::NIL.into(),
            role: "resolved.local".into(),
            position,
        },
        span: dsl::Span::default(),
        coordinate: None,
    }
}
type CompactRegion = (
    ResolvedSyntax,
    Vec<SymbolIx>,
    Vec<RegionBinding>,
    RegionAttribution,
    Relocation,
);
fn compact(source: &ResolvedSyntax, layout: &CanonicalLayout) -> Result<CompactRegion> {
    let relocation = Relocation::new(layout);
    let mut symbols = crate::specialize::Symbols::default();
    let mut bindings = Vec::with_capacity(layout.symbols.len());
    let mut inputs = Vec::new();
    for value in &layout.symbols {
        let actual = source.symbols.at(value.symbol).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "canonical scalar is outside its source owner",
            )
        })?;
        let local = relocation.symbol(value.symbol)?;
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
        hash.str("local-scalar")
            .str(match value.role {
                LocalRole::Coordinate => "coordinate",
                LocalRole::Parameter => "parameter",
                LocalRole::Member => "member",
            })
            .u64(value.ordinal as u64);
        let id = hash.finish_id();
        if actual.definition.is_none() {
            inputs.push(local);
        }
        symbols.push(crate::specialize::Symbol {
            id,
            ty: actual.ty.clone(),
            role: actual.role,
            domain: actual.domain,
            definition: actual
                .definition
                .map(|slot| relocation.node(slot))
                .transpose()?,
            expression: None,
            initial: None,
            lineage: crate::specialize::Lineage {
                declaration: SemanticId::NIL.into(),
                instance: crate::InstanceId::from_id(SemanticId::NIL),
                path: String::new(),
                demand: vec![],
                default_owner: None,
                is_override: false,
                presets: vec![],
            },
        });
        bindings.push(RegionBinding {
            local,
            actual: actual.id,
            role: value.role,
        });
    }
    let mut arena = Arena::default();
    let mut attribution = RegionAttribution::default();
    for value in &layout.symbols {
        if let Some(range) = source.member_ranges.get(&value.symbol) {
            arena.member_ranges.insert(
                relocation.symbol(value.symbol)?,
                MemberRange {
                    lower: relocation.node(range.lower)?,
                    upper: relocation.node(range.upper)?,
                    layer: range.layer,
                },
            );
        }
    }
    for old in &layout.nodes {
        let local = relocation.node(*old)?;
        arena.nodes.push(relocation.operation(&source.nodes[*old])?);
        let actual = source.node_sources.get(old).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "selected operation has no original attribution",
            )
        })?;
        attribution.nodes.insert(local, actual.clone());
        arena
            .node_sources
            .insert(local, neutral_source(local.position()));
        if let Some(fold) = source.finite_folds.get(old) {
            arena.finite_folds.insert(
                local,
                FiniteFold {
                    carrier: relocation.carrier(fold.carrier)?,
                    selected: fold.selected.clone(),
                    prototype: fold.prototype.clone(),
                },
            );
        }
    }
    for old in &layout.predicates {
        let local = relocation.predicate(*old)?;
        arena
            .predicates
            .push(relocation.guard(&source.predicates[*old])?);
        let actual = source.predicate_sources.get(old).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "selected guard has no original attribution",
            )
        })?;
        attribution.predicates.insert(local, actual.clone());
        arena
            .predicate_sources
            .insert(local, neutral_source(local.position()));
    }
    for old in &layout.binders {
        arena.binders.push(match source.binders[*old] {
            Binding::Value(value) => Binding::Value(relocation.node(value)?),
            Binding::Coordinate(carrier) => Binding::Coordinate(relocation.carrier(carrier)?),
            Binding::Accumulator => Binding::Accumulator,
            Binding::Item => Binding::Item,
        });
    }
    for old in &layout.functions {
        let value = &source.functions[*old];
        arena.functions.push(FunctionSelection {
            identity: value.identity,
            declaration: value.declaration,
            formals: vec![String::new(); value.formals.len()],
            product: value.product.clone(),
        });
    }
    for old in &layout.evidence {
        arena
            .evidence
            .push(relocation.usage(&source.evidence[*old])?);
    }
    for old in &layout.carriers {
        arena.carriers.push(source.carriers[*old].clone());
    }
    for old in &layout.groups {
        let value = &source.groups[*old];
        arena.groups.push(IndexedGroup {
            axes: value
                .axes
                .iter()
                .map(|slot| relocation.carrier(*slot))
                .collect::<Result<_>>()?,
            members: value
                .members
                .iter()
                .map(|(tuple, value)| {
                    Ok((
                        tuple.clone(),
                        value.map(|slot| relocation.symbol(slot)).transpose()?,
                    ))
                })
                .collect::<Result<_>>()?,
        });
    }
    let syntax = ResolvedSyntax {
        symbols,
        arena: Arc::new(arena),
        outputs: relocation.nodes(&source.outputs)?,
        validity: source
            .validity
            .iter()
            .map(|slot| relocation.predicate(*slot))
            .collect::<Result<_>>()?,
    };
    if syntax.identity()? != layout.identity {
        return Err(invalid(
            SemanticId::NIL,
            "coordinate relocation changed the exact selected semantic identity",
        ));
    }
    Ok((syntax, inputs, bindings, attribution, relocation))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum UseRole {
    Numeric,
    Guard,
}
#[derive(Clone, Debug, Default)]
struct SymbolReads {
    numeric: BTreeSet<SymbolIx>,
    guards: BTreeSet<SymbolIx>,
}
#[derive(Clone, Debug, Default)]
struct FormalReads {
    numeric: BTreeSet<usize>,
    guards: BTreeSet<usize>,
}
#[derive(Default)]
struct ReadCache {
    functions: BTreeMap<usize, FormalReads>,
    active: BTreeSet<usize>,
    depth: usize,
}
fn reads_for(syntax: &ResolvedSyntax, request: RegionRequest<'_>) -> Result<RegionReads> {
    reads_view(syntax.view(), request)
}
pub(crate) fn reads_view(syntax: View<'_>, request: RegionRequest<'_>) -> Result<RegionReads> {
    let mut nodes = Vec::new();
    let mut predicates = request.validity.to_vec();
    for output in request.outputs {
        match output {
            RegionOutput::Value(root) => nodes.push((*root, UseRole::Numeric)),
            RegionOutput::RelationResidual(relation) => nodes.extend([
                (relation.lhs, UseRole::Numeric),
                (relation.rhs, UseRole::Numeric),
            ]),
            RegionOutput::PredicateIndicator(predicate) => predicates.push(*predicate),
            RegionOutput::ConditionalBoundary(id) => {
                return Err(invalid(
                    *id,
                    "dependency projection requires the model-issued conditional boundary region",
                ));
            }
        }
    }
    let reached = read_symbols(syntax, nodes, predicates, &mut ReadCache::default())?;
    let actual = |values: BTreeSet<SymbolIx>| {
        values
            .into_iter()
            .map(|slot| {
                syntax
                    .symbols
                    .at(slot)
                    .map(|symbol| symbol.id)
                    .ok_or_else(|| {
                        invalid(
                            SemanticId::NIL,
                            "dependency scalar is outside its actual owner",
                        )
                    })
            })
            .collect::<Result<BTreeSet<_>>>()
    };
    Ok(RegionReads {
        numeric: actual(reached.numeric)?,
        guards: actual(reached.guards)?,
    })
}
impl ReadCache {
    fn function(&mut self, product: &Arc<FunctionProduct>) -> Result<FormalReads> {
        // The pointer is a construction-local memo lookup only, never semantic identity.
        let owner = Arc::as_ptr(product) as usize;
        if let Some(value) = self.functions.get(&owner) {
            return Ok(value.clone());
        }
        if self.depth >= 128 || !self.active.insert(owner) {
            return Err(crate::ModelingError::Budget(
                "function dependency cycle or depth".into(),
            ));
        }
        self.depth += 1;
        let mut nodes = product
            .body
            .into_iter()
            .map(|root| (root, UseRole::Numeric))
            .collect::<Vec<_>>();
        let mut guards = product.validity.into_iter().collect::<Vec<_>>();
        guards.extend(product.envelopes.iter().map(|value| value.predicate));
        for usage in &product.applicability {
            guards.extend(usage.predicates.iter().copied());
            nodes.extend(usage.inputs.iter().map(|root| (*root, UseRole::Guard)));
        }
        for ordinal in &product.prerequisites {
            let root = product.formal_root(*ordinal)?;
            nodes.push((root, UseRole::Guard));
        }
        if product.external.is_some() || product.reduction.is_some() {
            for ordinal in 0..product.formals.len() {
                let root = product.formal_root(ordinal)?;
                nodes.push((root, UseRole::Numeric));
            }
        }
        let selected = read_symbols(product.syntax.view(), nodes, guards, self)?;
        let mut result = FormalReads::default();
        for (ordinal, slot) in product.formals.iter().enumerate() {
            if selected.numeric.contains(slot) {
                result.numeric.insert(ordinal);
            }
            if selected.guards.contains(slot) {
                result.guards.insert(ordinal);
            }
        }
        self.depth -= 1;
        self.active.remove(&owner);
        self.functions.insert(owner, result.clone());
        Ok(result)
    }
}
fn read_symbols(
    syntax: View<'_>,
    mut nodes: Vec<(NodeIx, UseRole)>,
    mut predicates: Vec<PredicateIx>,
    cache: &mut ReadCache,
) -> Result<SymbolReads> {
    let mut result = SymbolReads::default();
    let mut seen = BTreeSet::new();
    let mut guards = BTreeSet::new();
    while !nodes.is_empty() || !predicates.is_empty() {
        while let Some(slot) = predicates.pop() {
            if !guards.insert(slot) {
                continue;
            }
            match syntax.predicates.get(slot).ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "dependency predicate is outside its actual owner",
                )
            })? {
                ResolvedPredicate::Compare { lhs, rhs, .. } => {
                    nodes.extend([(*lhs, UseRole::Guard), (*rhs, UseRole::Guard)])
                }
                ResolvedPredicate::In { value, .. } | ResolvedPredicate::Atom(value) => {
                    nodes.push((*value, UseRole::Guard))
                }
                ResolvedPredicate::And(lhs, rhs) | ResolvedPredicate::Or(lhs, rhs) => {
                    predicates.extend([*lhs, *rhs])
                }
                ResolvedPredicate::Not(value) => predicates.push(*value),
                ResolvedPredicate::Bool(_) | ResolvedPredicate::Null => {}
            }
        }
        let Some((slot, role)) = nodes.pop() else {
            continue;
        };
        if !seen.insert((slot, role)) {
            continue;
        }
        match syntax.nodes.get(slot).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "dependency operation is outside its actual owner",
            )
        })? {
            Node::Number(_) => {}
            Node::Reference(reference) => {
                let mut symbol = |slot: SymbolIx| -> Result<()> {
                    let value = syntax.symbols.at(slot).ok_or_else(|| {
                        invalid(
                            SemanticId::NIL,
                            "dependency scalar is outside its actual owner",
                        )
                    })?;
                    match role {
                        UseRole::Numeric => {
                            result.numeric.insert(slot);
                        }
                        UseRole::Guard => {
                            result.guards.insert(slot);
                        }
                    }
                    if let Some(root) = value.definition {
                        nodes.push((root, role));
                    }
                    if let Some(range) = syntax.member_ranges.get(&slot) {
                        nodes
                            .extend([(range.lower, UseRole::Guard), (range.upper, UseRole::Guard)]);
                    }
                    Ok(())
                };
                match reference {
                    Reference::Symbol(slot) => symbol(*slot)?,
                    Reference::Lexical(binder) => {
                        if let Some(Binding::Value(root)) = syntax.binders.get(*binder) {
                            nodes.push((*root, role));
                        }
                    }
                    Reference::Indexed { group, indices } => {
                        let group = syntax.groups.get(*group).ok_or_else(|| {
                            invalid(
                                SemanticId::NIL,
                                "dependency indexed family is outside its actual owner",
                            )
                        })?;
                        for (_, value) in &group.members {
                            if let Some(value) = value {
                                symbol(*value)?;
                            }
                        }
                        nodes.extend(indices.iter().map(|root| (*root, role)));
                    }
                }
            }
            Node::Neg(root) | Node::Derivative { body: root, .. } => nodes.push((*root, role)),
            Node::Binary { lhs, rhs, .. } => nodes.extend([(*lhs, role), (*rhs, role)]),
            Node::Call { args, .. } | Node::Provider { args, .. } => {
                nodes.extend(args.iter().map(|root| (*root, role)))
            }
            Node::Apply { function, args } | Node::Partial { function, args, .. } => {
                let selected = syntax.functions.get(*function).ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "dependency function is outside its actual owner",
                    )
                })?;
                let reached = cache.function(&selected.product)?;
                for ordinal in reached.numeric {
                    nodes.push((
                        *args.get(ordinal).ok_or_else(|| {
                            invalid(
                                selected.declaration,
                                "dependency actual argument is outside its signature",
                            )
                        })?,
                        role,
                    ));
                }
                for ordinal in reached.guards {
                    nodes.push((
                        *args.get(ordinal).ok_or_else(|| {
                            invalid(
                                selected.declaration,
                                "dependency guard argument is outside its signature",
                            )
                        })?,
                        UseRole::Guard,
                    ));
                }
            }
            Node::Indicator(predicate) => predicates.push(*predicate),
            Node::Conditional {
                guard,
                then,
                otherwise,
            } => {
                predicates.push(*guard);
                nodes.extend([(*then, role), (*otherwise, role)]);
            }
            Node::Let { body, .. } => nodes.push((*body, role)),
            Node::Sequence {
                value,
                prerequisites,
            } => {
                nodes.push((*value, role));
                nodes.extend(prerequisites.iter().map(|root| (*root, UseRole::Guard)));
            }
            Node::Evidence { value, evidence } => {
                nodes.push((*value, role));
                let usage = syntax.evidence.get(*evidence).ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "dependency evidence is outside its actual owner",
                    )
                })?;
                predicates.extend(usage.predicates.iter().copied());
                nodes.extend(usage.inputs.iter().map(|root| (*root, UseRole::Guard)));
            }
            Node::Reduce { body, filter, .. } => {
                nodes.push((*body, role));
                predicates.extend(filter);
            }
            Node::Fold {
                value,
                step,
                filter,
                ..
            } => {
                nodes.extend([(*value, role), (*step, role)]);
                predicates.extend(filter);
            }
        }
    }
    Ok(result)
}

/// One original conditioning term admitted in its actual parent relation context.
#[derive(Clone, Debug)]
pub struct PreparedTerm {
    /// Original left-to-right additive ordinal, before normalization.
    pub ordinal: usize,
    /// Sign prescribed by original addition, subtraction and negation.
    pub negative: bool,
    /// Compact independently selectable term with actual parent-admitted physical facts.
    pub region: SelectedRegion,
}
impl crate::SpecializedModel {
    /// Issue original conditioning terms through the same model-owned admission factory.
    /// Every term keeps its actual parent context without accepting a caller quantity hint.
    pub fn prepare_relation_terms(
        &self,
        relation: Relation,
        context: &crate::TypeContext<'_>,
    ) -> Result<Vec<PreparedTerm>> {
        let outputs = [RegionOutput::RelationResidual(relation)];
        let parent = self.prepare_region(
            RegionRequest {
                outputs: &outputs,
                validity: &[],
            },
            context,
        )?;
        let residual =
            *parent.syntax.outputs.first().ok_or_else(|| {
                invalid(SemanticId::NIL, "parent relation has no admitted residual")
            })?;
        let mut pending = vec![(residual, false)];
        let mut terms = Vec::new();
        while let Some((root, negative)) = pending.pop() {
            match parent.syntax.nodes.get(root).ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "conditioning term is outside its actual parent",
                )
            })? {
                Node::Binary {
                    op: BinaryOp::Add,
                    lhs,
                    rhs,
                } => {
                    pending.push((*rhs, negative));
                    pending.push((*lhs, negative));
                }
                Node::Binary {
                    op: BinaryOp::Sub,
                    lhs,
                    rhs,
                } => {
                    pending.push((*rhs, !negative));
                    pending.push((*lhs, negative));
                }
                Node::Neg(value) => pending.push((*value, !negative)),
                _ => terms.push((root, negative)),
            }
        }
        terms
            .into_iter()
            .enumerate()
            .map(|(ordinal, (root, negative))| {
                Ok(PreparedTerm {
                    ordinal,
                    negative,
                    region: parent.select_admitted(root, context)?,
                })
            })
            .collect()
    }
}
impl SelectedRegion {
    fn select_admitted(&self, root: NodeIx, context: &crate::TypeContext<'_>) -> Result<Self> {
        let selected = self.syntax.select(vec![root], self.syntax.validity.clone());
        let layout = selected.canonical_layout()?;
        let (syntax, inputs, mut bindings, _, relocation) = compact(&selected, &layout)?;
        let parent_actual = self
            .bindings
            .iter()
            .map(|binding| {
                Ok((
                    self.syntax
                        .symbols
                        .at(binding.local)
                        .ok_or_else(|| {
                            invalid(
                                SemanticId::NIL,
                                "parent instance binding lost its compact scalar",
                            )
                        })?
                        .id,
                    binding.actual,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        for binding in &mut bindings {
            binding.actual = *parent_actual.get(&binding.actual).ok_or_else(|| {
                invalid(
                    binding.actual,
                    "term binding is outside its actual parent region",
                )
            })?;
        }
        let mut attribution = RegionAttribution::default();
        for old in &layout.nodes {
            attribution.nodes.insert(
                relocation.node(*old)?,
                self.attribution.nodes.get(old).cloned().ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "term source is outside its actual parent correspondence",
                    )
                })?,
            );
        }
        for old in &layout.predicates {
            attribution.predicates.insert(
                relocation.predicate(*old)?,
                self.attribution
                    .predicates
                    .get(old)
                    .cloned()
                    .ok_or_else(|| {
                        invalid(
                            SemanticId::NIL,
                            "term guard is outside its actual parent correspondence",
                        )
                    })?,
            );
        }
        let outputs = [RegionOutput::Value(root)];
        let reads = reads_for(
            &self.syntax,
            RegionRequest {
                outputs: &outputs,
                validity: &self.syntax.validity,
            },
        )?;
        let actual = |values: BTreeSet<SemanticId>| {
            values
                .into_iter()
                .map(|id| {
                    parent_actual.get(&id).copied().ok_or_else(|| {
                        invalid(id, "term dependency is outside its actual parent binding")
                    })
                })
                .collect::<Result<BTreeSet<_>>>()
        };
        let reads = RegionReads {
            numeric: actual(reads.numeric)?,
            guards: actual(reads.guards)?,
        };
        let support = self
            .support
            .select_relocated(&relocation, syntax.identity()?, root, context.quantities)?
            .with_inputs(&syntax, &inputs, context)?;
        let parent_layout = self
            .syntax
            .canonical_layout()?
            .symbols
            .into_iter()
            .map(|value| (value.symbol, value))
            .collect::<BTreeMap<_, _>>();
        let inverse_symbols = relocation
            .symbols
            .iter()
            .map(|(old, new)| (*new, *old))
            .collect::<BTreeMap<_, _>>();
        let mut checked_members = BTreeMap::new();
        let mut checked_sources = BTreeMap::new();
        for range in support.checked_ranges(&syntax)? {
            let old = inverse_symbols.get(&range.symbol).ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "term range left original scalar relocation",
                )
            })?;
            let parent = parent_layout.get(old).ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "term range is absent from parent correspondence",
                )
            })?;
            let old_member = crate::derived::Relationship::CheckedMember {
                role: parent.role,
                ordinal: parent.ordinal as u64,
            }
            .identity();
            let old_source = crate::derived::Relationship::CheckedValiditySource {
                role: parent.role,
                ordinal: parent.ordinal as u64,
            }
            .identity();
            checked_members.insert(
                range.member,
                *self
                    .checked_members
                    .get(&old_member)
                    .ok_or_else(|| invalid(old_member, "term lost its actual member"))?,
            );
            checked_sources.insert(
                range.source,
                *self
                    .checked_sources
                    .get(&old_source)
                    .ok_or_else(|| invalid(old_source, "term lost its actual source"))?,
            );
        }
        let support_identity = support_identity(&support);
        let occurrences =
            occurrences::reselect(&self.syntax, &syntax, &self.occurrences, &relocation)?;
        Ok(Self {
            syntax,
            inputs,
            support,
            support_identity,
            bindings,
            attribution,
            reads,
            checked_members,
            checked_sources,
            derivations: self.derivations.clone(),
            occurrences,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::construction::Construction;
    use super::*;
    pub(super) fn context<'a>(
        registry: &'a pse_quantity::QuantityRegistry,
        facts: &'a pse_quantity::PhysicalPreconditions,
        scope: &'a crate::PhysicalScope,
    ) -> crate::TypeContext<'a> {
        crate::TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: registry,
            preconditions: facts,
            scope,
        }
    }
    fn source(seed: u8) -> Source {
        Source {
            field: crate::expression::occurrences::OccurrenceKey {
                declaration: SemanticId::from_bytes([seed; 16]).into(),
                role: "equation.expression".into(),
                position: 0,
            },
            span: dsl::Span {
                start: u32::from(seed),
                end: u32::from(seed) + 1,
            },
            coordinate: Some(InventoryCoordinate {
                stage: InventoryStage::Original,
                grammar: InventoryGrammar::Expression,
                occurrence: crate::expression::admission::ExpressionOccurrence::Node {
                    body: 0,
                    position: 0,
                },
            }),
        }
    }
    pub(super) fn model(seed: u8, ty: &Type) -> (crate::SpecializedModel, Relation, PredicateIx) {
        let mut symbols = crate::specialize::Symbols::default();
        for index in 0..3u8 {
            let id = SemanticId::from_bytes([seed + index; 16]);
            symbols.push(crate::specialize::Symbol {
                id,
                ty: ty.clone(),
                role: ModelingDeclarationKind::Variable,
                domain: ModelingVariableDomain::Continuous,
                definition: None,
                expression: None,
                initial: None,
                lineage: crate::specialize::Lineage {
                    declaration: id.into(),
                    instance: crate::InstanceId::from_id(id),
                    path: format!("instance_{seed}.variable_{index}"),
                    demand: vec![],
                    default_owner: None,
                    is_override: false,
                    presets: vec![],
                },
            });
        }
        let mut construction = Construction::new(100);
        let x = construction
            .node(
                Node::Reference(Reference::Symbol(SymbolIx::from(0))),
                source(seed),
            )
            .unwrap();
        let y = construction
            .node(
                Node::Reference(Reference::Symbol(SymbolIx::from(1))),
                source(seed),
            )
            .unwrap();
        let negative = construction.node(Node::Neg(y), source(seed)).unwrap();
        let lhs = construction
            .node(
                Node::Binary {
                    op: BinaryOp::Add,
                    lhs: x,
                    rhs: negative,
                },
                source(seed),
            )
            .unwrap();
        let one = construction
            .node(
                Node::Number(dsl::Number {
                    value: 1.0,
                    exact_integer: Some(1),
                    unit: None,
                }),
                source(seed),
            )
            .unwrap();
        let predicate = construction
            .predicate(
                ResolvedPredicate::Compare {
                    op: CompareOp::Gt,
                    lhs: y,
                    rhs: one,
                },
                source(seed),
            )
            .unwrap();
        let resolved = construction
            .finish(symbols.clone(), vec![], vec![])
            .unwrap();
        let mut model = crate::SpecializedModel::default();
        model.symbols = symbols;
        model.resolved = resolved;
        (
            model,
            Relation {
                lhs,
                sense: dsl::EquationSense::Eq,
                rhs: one,
            },
            predicate,
        )
    }
    #[test]
    fn selected_regions_share_local_meaning_and_keep_current_bindings_and_sources_outside() {
        let (registry, _) = crate::kernel_types::physical();
        let facts = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let scope = crate::PhysicalScope::default();
        let context = context(&registry, &facts, &scope);
        let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            registry.neutral_dimensionless().unwrap(),
        ));
        let (first, relation, _) = model(11, &ty);
        let (second, other_relation, _) = model(21, &ty);
        let outputs = [RegionOutput::RelationResidual(relation)];
        let other_outputs = [RegionOutput::RelationResidual(other_relation)];
        let a = first
            .prepare_region(
                RegionRequest {
                    outputs: &outputs,
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        let b = second
            .prepare_region(
                RegionRequest {
                    outputs: &other_outputs,
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        assert_eq!(a.syntax.identity().unwrap(), b.syntax.identity().unwrap());
        let mut left = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
        a.support.frame(&mut left);
        let mut right = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
        b.support.frame(&mut right);
        assert_eq!(left.finish_hash(), right.finish_hash());
        assert_ne!(a.bindings, b.bindings);
        assert_ne!(a.attribution, b.attribution);
        assert_eq!(a.syntax.symbols.len(), 2);
        assert_eq!(first.symbols.len(), 3);
        assert!(!Arc::ptr_eq(&a.syntax.arena, &first.resolved.arena));
        for symbol in a.syntax.symbols.values() {
            assert!(symbol.initial.is_none());
            assert!(symbol.expression.is_none());
            assert!(symbol.lineage.path.is_empty());
        }
        assert!(
            a.syntax
                .node_sources
                .values()
                .all(|source| source.field.declaration.as_id() == SemanticId::NIL
                    && source.coordinate.is_none())
        );
    }
    #[test]
    fn original_ranges_keep_transitive_guard_reads_and_distinct_actual_sources() {
        let (registry, _) = crate::kernel_types::physical();
        let facts = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let scope = crate::PhysicalScope::default();
        let context = context(&registry, &facts, &scope);
        let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            registry.neutral_dimensionless().unwrap(),
        ));
        let prepare = |seed: u8| {
            let (mut model, _, _) = model(seed, &ty);
            let x = NodeIx::from(0);
            let y = NodeIx::from(1);
            let one = NodeIx::from(4);
            model.resolved.member_ranges.insert(
                SymbolIx::from(0),
                MemberRange {
                    lower: one,
                    upper: y,
                    layer: pse_model::generated::enums::ModelingValidityLayer::Closure,
                },
            );
            model
                .member_range_sources
                .insert(SymbolIx::from(0), SemanticId::from_bytes([seed + 10; 16]));
            model
                .symbol_roots
                .insert(SemanticId::from_bytes([seed; 16]), x);
            let outputs = [RegionOutput::Value(x)];
            let region = model
                .prepare_region(
                    RegionRequest {
                        outputs: &outputs,
                        validity: &[],
                    },
                    &context,
                )
                .unwrap();
            (model, region)
        };
        let (mut first, a) = prepare(40);
        let (_, b) = prepare(60);
        assert_eq!(a.syntax.identity().unwrap(), b.syntax.identity().unwrap());
        assert_eq!(a.support_identity, b.support_identity);
        assert_ne!(
            a, b,
            "backdating must preserve actual instance bindings and independently authored sources"
        );
        assert_eq!(
            a.checked_members.values().copied().collect::<Vec<_>>(),
            vec![SemanticId::from_bytes([40; 16])]
        );
        assert_eq!(
            a.checked_sources.values().copied().collect::<Vec<_>>(),
            vec![SemanticId::from_bytes([50; 16])]
        );
        assert!(
            a.checked_members
                .keys()
                .all(|member| !a.checked_sources.contains_key(member))
        );
        assert_eq!(
            a.reads.numeric,
            BTreeSet::from([SemanticId::from_bytes([40; 16])])
        );
        assert_eq!(
            a.reads.guards,
            BTreeSet::from([SemanticId::from_bytes([41; 16])])
        );
        let output = a.syntax.outputs[0];
        assert!(a.support.node(output).unwrap().range.is_some());
        first.resolved.member_ranges.insert(
            SymbolIx::from(1),
            MemberRange {
                lower: NodeIx::from(0),
                upper: NodeIx::from(4),
                layer: pse_model::generated::enums::ModelingValidityLayer::Closure,
            },
        );
        first
            .member_range_sources
            .insert(SymbolIx::from(1), SemanticId::from_bytes([70; 16]));
        assert!(
            first
                .prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::Value(NodeIx::from(0))],
                        validity: &[]
                    },
                    &context
                )
                .is_err(),
            "cyclic original range closure must refuse"
        );
    }
    #[test]
    fn conditional_boundaries_issue_typed_parameters_and_preserve_original_owner() {
        let (registry, _) = crate::kernel_types::physical();
        let facts = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let scope = crate::PhysicalScope::default();
        let context = context(&registry, &facts, &scope);
        let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            registry.neutral_dimensionless().unwrap(),
        ));
        let (mut model, _, _) = model(80, &ty);
        let id = SemanticId::from_bytes([80; 16]);
        model.symbol_roots.insert(id, NodeIx::from(0));
        let original_nodes = model.resolved.nodes.len();
        let original_symbols = model.symbols.len();
        let region = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::ConditionalBoundary(id)],
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        assert_eq!(model.resolved.nodes.len(), original_nodes);
        assert_eq!(model.symbols.len(), original_symbols);
        let parameter = region
            .derivations
            .require(crate::derived::Relationship::ConditionalBoundaryValue { symbol: id })
            .unwrap();
        assert_ne!(
            parameter,
            region
                .derivations
                .require(crate::derived::Relationship::ConditionalBoundaryResidual { symbol: id })
                .unwrap()
        );
        assert!(
            region
                .bindings
                .iter()
                .any(|binding| binding.actual == parameter && binding.role == LocalRole::Parameter)
        );
        assert_eq!(region.inputs.len(), 2);
        assert!(region.reads.numeric.contains(&parameter));
        let mut rebound = region.clone();
        rebound
            .attribution
            .nodes
            .values_mut()
            .next()
            .unwrap()
            .field
            .declaration = SemanticId::from_bytes([99; 16]).into();
        assert_ne!(
            region, rebound,
            "projection equality includes actual original source correspondence"
        );
    }
    #[test]
    fn unused_lexical_ranges_do_not_become_eager_closure_obligations() {
        let (registry, _) = crate::kernel_types::physical();
        let facts = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let scope = crate::PhysicalScope::default();
        let context = context(&registry, &facts, &scope);
        let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            registry.neutral_dimensionless().unwrap(),
        ));
        let (mut model, _, _) = model(100, &ty);
        model.resolved.member_ranges.insert(
            SymbolIx::from(0),
            MemberRange {
                lower: NodeIx::from(4),
                upper: NodeIx::from(1),
                layer: pse_model::generated::enums::ModelingValidityLayer::Closure,
            },
        );
        model
            .member_range_sources
            .insert(SymbolIx::from(0), SemanticId::from_bytes([110; 16]));
        let binder = BinderIx::from(model.resolved.binders.len());
        model.resolved.binders.push(Binding::Value(NodeIx::from(0)));
        let root = NodeIx::from(model.resolved.nodes.len());
        model.resolved.nodes.push(Node::Let {
            bindings: vec![binder],
            body: NodeIx::from(4),
        });
        model.resolved.node_sources.insert(root, source(100));
        let region = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::Value(root)],
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        assert!(region.checked_members.is_empty());
        assert!(region.checked_sources.is_empty());
        assert!(region.reads.numeric.is_empty());
        assert!(region.reads.guards.is_empty());
    }
    #[test]
    fn predicate_indicators_keep_guard_dependencies_without_asserting_truth() {
        let (registry, _) = crate::kernel_types::physical();
        let facts = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let scope = crate::PhysicalScope::default();
        let context = context(&registry, &facts, &scope);
        let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            registry.neutral_dimensionless().unwrap(),
        ));
        let (model, _, predicate) = model(31, &ty);
        let outputs = [RegionOutput::PredicateIndicator(predicate)];
        let selected = model
            .prepare_region(
                RegionRequest {
                    outputs: &outputs,
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        assert!(selected.syntax.validity.is_empty());
        assert!(selected.reads.numeric.is_empty());
        assert_eq!(
            selected.reads.guards,
            BTreeSet::from([model.symbols.at(SymbolIx::from(1)).unwrap().id])
        );
        assert!(matches!(
            selected.syntax.nodes[selected.syntax.outputs[0]],
            Node::Indicator(_)
        ));
    }
    #[test]
    fn original_conditioning_terms_keep_additive_order_sign_and_parent_admission() {
        let (registry, _) = crate::kernel_types::physical();
        let facts = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let scope = crate::PhysicalScope::default();
        let context = context(&registry, &facts, &scope);
        let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            registry.neutral_dimensionless().unwrap(),
        ));
        let (model, relation, _) = model(41, &ty);
        let terms = model.prepare_relation_terms(relation, &context).unwrap();
        assert_eq!(
            terms
                .iter()
                .map(|term| (term.ordinal, term.negative))
                .collect::<Vec<_>>(),
            vec![(0, false), (1, true), (2, true)]
        );
        assert_eq!(
            terms[0].region.reads.numeric,
            BTreeSet::from([model.symbols.at(SymbolIx::from(0)).unwrap().id])
        );
        assert_eq!(
            terms[1].region.reads.numeric,
            BTreeSet::from([model.symbols.at(SymbolIx::from(1)).unwrap().id])
        );
        assert!(terms[2].region.reads.numeric.is_empty());
        for term in terms {
            let root = term.region.syntax.outputs[0];
            assert_eq!(term.region.support.node(root).unwrap().ty, ty);
            assert_eq!(
                term.region.support.syntax(),
                term.region.syntax.identity().unwrap()
            );
        }
    }
}
