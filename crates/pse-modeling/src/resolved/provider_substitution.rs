// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed supplier substitution. Original arithmetic admissions and scalar contracts survive.
use super::*;

/// Final supplier selection from an actual admitted descriptor. Executable authority
/// remains with the caller's full descriptor join, never this pure metadata record.
#[derive(Clone, Debug)]
pub struct ProviderSubstitution {
    /// Actual independent scalar supplied by this selected output.
    pub member: SemanticId,
    /// Complete descriptor identity and exact selected output position.
    pub selection: ProviderSelection,
    /// Actual ordered scalar input identities and declared registered port quantities.
    pub inputs: Vec<(SemanticId, pse_quantity::QuantityTypeId)>,
    /// Actual selected descriptor output quantity.
    pub quantity: pse_quantity::QuantityTypeId,
}
impl SelectedRegion {
    /// Replace only reached independent scalar definitions by final supplier outputs.
    /// Each new input is selected from its real model owner, retaining original guards.
    pub fn substitute_providers(
        &self,
        model: &crate::SpecializedModel,
        replacements: &[ProviderSubstitution],
        context: &crate::TypeContext<'_>,
    ) -> Result<Self> {
        let mut available = BTreeMap::new();
        for replacement in replacements {
            if available.insert(replacement.member, replacement).is_some() {
                return Err(invalid(
                    replacement.member,
                    "supplier has repeated actual output ownership",
                ));
            }
        }
        let mut parts = vec![self.clone()];
        let mut selected = BTreeSet::new();
        let mut imported = BTreeMap::<SemanticId, usize>::new();
        let reached = self
            .syntax
            .canonical_layout()?
            .symbols
            .into_iter()
            .map(|symbol| symbol.symbol)
            .collect::<BTreeSet<_>>();
        let mut pending = self
            .inputs
            .iter()
            .filter(|slot| reached.contains(slot))
            .map(|slot| {
                self.bindings
                    .iter()
                    .find(|binding| binding.local == *slot)
                    .map(|binding| binding.actual)
                    .ok_or_else(|| {
                        invalid(
                            SemanticId::NIL,
                            "supplier selection lost its actual input binding",
                        )
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        let limit = if model.construction_limit == 0 {
            crate::specialize::Limits::default().items
        } else {
            model.construction_limit
        };
        while let Some(actual) = pending.pop() {
            let Some(replacement) = available.get(&actual) else {
                continue;
            };
            if !selected.insert(actual) {
                continue;
            }
            if selected.len().saturating_add(imported.len()) > limit {
                return Err(crate::ModelingError::Budget(
                    "supplier selected input extent exceeded".into(),
                ));
            }
            for (input, _) in &replacement.inputs {
                if imported.contains_key(input) {
                    continue;
                }
                let part = model.prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::Value(model.symbol_root(*input)?)],
                        validity: &[],
                    },
                    context,
                )?;
                for slot in &part.inputs {
                    pending.push(
                        part.bindings
                            .iter()
                            .find(|binding| binding.local == *slot)
                            .map(|binding| binding.actual)
                            .ok_or_else(|| {
                                invalid(*input, "supplier input has no actual original binding")
                            })?,
                    );
                }
                imported.insert(*input, parts.len());
                parts.push(part);
            }
        }
        if selected.is_empty() {
            return Ok(self.clone());
        }
        // Borrow exact producer declarations, including any already-issued synthetic
        // coordinate carried by the original region. No type is guessed from an ID.
        let mut declarations = Vec::<crate::specialize::Symbol>::new();
        let mut declared = BTreeSet::new();
        for part in &parts {
            for slot in &part.inputs {
                let binding = part
                    .bindings
                    .iter()
                    .find(|binding| binding.local == *slot)
                    .ok_or_else(|| {
                        invalid(SemanticId::NIL, "supplier composition lost an input owner")
                    })?;
                if declared.insert(binding.actual) {
                    let mut declaration =
                        part.syntax.symbols.at(*slot).cloned().ok_or_else(|| {
                            invalid(
                                binding.actual,
                                "supplier input left its typed declaration owner",
                            )
                        })?;
                    if declaration.definition.is_some() {
                        return Err(invalid(
                            binding.actual,
                            "supplier independent layout contains a defined original scalar",
                        ));
                    }
                    declaration.id = binding.actual;
                    declarations.push(declaration);
                }
            }
        }
        let mut region = Self::combine(&parts, &declarations.iter().collect::<Vec<_>>(), context)?;
        let original_outputs = self.syntax.outputs.len();
        let mut input_roots = BTreeMap::new();
        for (offset, (part, source)) in (original_outputs..).zip(parts.iter().enumerate().skip(1)) {
            let actual = imported
                .iter()
                .find_map(|(actual, index)| (*index == part).then_some(*actual))
                .ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "supplier imported root lost its actual owner",
                    )
                })?;
            if source.syntax.outputs.len() != 1 {
                return Err(invalid(
                    actual,
                    "supplier source reference has non-scalar output extent",
                ));
            }
            input_roots.insert(actual, region.syntax.outputs[offset]);
        }
        let actual_ranges = region
            .support
            .checked_ranges(&region.syntax)?
            .into_iter()
            .map(|range| {
                Ok((
                    range.symbol,
                    (
                        *region.checked_members.get(&range.member).ok_or_else(|| {
                            invalid(
                                range.member,
                                "supplier original range has no member binding",
                            )
                        })?,
                        *region.checked_sources.get(&range.source).ok_or_else(|| {
                            invalid(
                                range.source,
                                "supplier original range has no source binding",
                            )
                        })?,
                    ),
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let mut issued = Vec::new();
        for actual in &selected {
            let replacement = available[actual];
            let local = region
                .bindings
                .iter()
                .find(|binding| binding.actual == *actual && region.inputs.contains(&binding.local))
                .map(|binding| binding.local)
                .ok_or_else(|| {
                    invalid(
                        *actual,
                        "selected supplier output is not an actual independent scalar",
                    )
                })?;
            let args = replacement
                .inputs
                .iter()
                .map(|(input, _)| {
                    input_roots
                        .get(input)
                        .copied()
                        .ok_or_else(|| invalid(*input, "supplier actual input root was not issued"))
                })
                .collect::<Result<Vec<_>>>()?;
            let root = NodeIx::from(region.syntax.nodes.len());
            region.syntax.nodes.push(Node::Provider {
                selection: replacement.selection,
                args,
            });
            let source=region.attribution.nodes.iter().find_map(|(node,source)|matches!(region.syntax.nodes.get(*node),Some(Node::Reference(Reference::Symbol(slot))) if *slot==local).then_some(source.clone()))
                .ok_or_else(||invalid(*actual,"supplier output lacks its original scalar occurrence"))?;
            let mut source = source;
            source.coordinate = None;
            region.attribution.nodes.insert(root, source);
            region
                .syntax
                .node_sources
                .insert(root, neutral_source(root.position()));
            region.syntax.symbols.define(local, root)?;
            issued.push((
                root,
                local,
                replacement.quantity,
                replacement
                    .inputs
                    .iter()
                    .map(|(_, quantity)| *quantity)
                    .collect::<Vec<_>>(),
            ));
        }
        region.syntax.outputs.truncate(original_outputs);
        region.inputs.retain(|slot| {
            region
                .syntax
                .symbols
                .at(*slot)
                .is_some_and(|symbol| symbol.definition.is_none())
        });
        region.support = region.support.substitute_providers(
            &region.syntax,
            &region.inputs,
            &issued,
            original_outputs,
            context,
        )?;
        // Canonical compaction below relocates existing admissions without interpreting
        // arithmetic. Unused actual input declarations remain explicit ordered ports.
        let old = region.syntax.clone();
        let layout = old.canonical_layout()?;
        let (mut syntax, _, mut bindings, attribution, mut relocation) = compact(&old, &layout)?;
        let actual = region
            .bindings
            .iter()
            .map(|binding| (binding.local, binding.actual))
            .collect::<BTreeMap<_, _>>();
        let inverse = relocation
            .symbols
            .iter()
            .map(|(old, new)| (*new, *old))
            .collect::<BTreeMap<_, _>>();
        for binding in &mut bindings {
            binding.actual = actual[&inverse[&binding.local]];
        }
        let mut inputs = Vec::new();
        let mut role_counts = [0usize; 3];
        for symbol in &syntax.canonical_layout()?.symbols {
            role_counts[match symbol.role {
                LocalRole::Coordinate => 0,
                LocalRole::Parameter => 1,
                LocalRole::Member => 2,
            }] += 1;
        }
        for old_slot in &region.inputs {
            let local = if let Some(local) = relocation.symbols.get(old_slot) {
                *local
            } else {
                let mut source = old.symbols.at(*old_slot).cloned().ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "unused supplier input lost its actual declaration",
                    )
                })?;
                let role = source.local_role();
                let ordinal = match role {
                    LocalRole::Coordinate => 0,
                    LocalRole::Parameter => 1,
                    LocalRole::Member => {
                        return Err(invalid(
                            source.id,
                            "defined member cannot be an unused supplier input",
                        ));
                    }
                };
                let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
                hash.str("local-scalar")
                    .str(if role == LocalRole::Coordinate {
                        "coordinate"
                    } else {
                        "parameter"
                    })
                    .u64(role_counts[ordinal] as u64);
                role_counts[ordinal] += 1;
                source.id = hash.finish_id();
                let local = SymbolIx::from(syntax.symbols.len());
                syntax.symbols.push(source);
                relocation.symbols.insert(*old_slot, local);
                bindings.push(RegionBinding {
                    local,
                    actual: actual[old_slot],
                    role,
                });
                local
            };
            inputs.push(local);
        }
        let support = PhysicalSupport::combine_relocated(
            &[(&region.support, &relocation)],
            &syntax,
            &inputs,
            context,
        )?;
        let mut checked_members = BTreeMap::new();
        let mut checked_sources = BTreeMap::new();
        let old_slots = relocation
            .symbols
            .iter()
            .map(|(old, new)| (*new, *old))
            .collect::<BTreeMap<_, _>>();
        for range in support.checked_ranges(&syntax)? {
            let previous = actual_ranges
                .get(&old_slots[&range.symbol])
                .ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "supplier range lost its original scalar correspondence",
                    )
                })?;
            checked_members.insert(range.member, previous.0);
            checked_sources.insert(range.source, previous.1);
        }
        let mut actual_attribution = attribution;
        // compact reads the reusable neutral graph sources; relocate actual attribution
        // using its exact typed correspondence instead of comparing source text/spans.
        actual_attribution.nodes = region
            .attribution
            .nodes
            .iter()
            .filter_map(|(old, source)| relocation.nodes.get(old).map(|new| (*new, source.clone())))
            .collect();
        actual_attribution.predicates = region
            .attribution
            .predicates
            .iter()
            .filter_map(|(old, source)| {
                relocation
                    .predicates
                    .get(old)
                    .map(|new| (*new, source.clone()))
            })
            .collect();
        let roots = syntax
            .outputs
            .iter()
            .copied()
            .map(RegionOutput::Value)
            .collect::<Vec<_>>();
        let mut reads = reads_for(
            &syntax,
            RegionRequest {
                outputs: &roots,
                validity: &syntax.validity,
            },
        )?;
        let by_id = bindings
            .iter()
            .map(|binding| {
                (
                    syntax.symbols.at(binding.local).map(|symbol| symbol.id),
                    binding.actual,
                )
            })
            .collect::<BTreeMap<_, _>>();
        reads.numeric = reads
            .numeric
            .iter()
            .map(|id| {
                by_id
                    .get(&Some(*id))
                    .copied()
                    .ok_or_else(|| invalid(*id, "supplier numeric read lost actual binding"))
            })
            .collect::<Result<_>>()?;
        reads.guards = reads
            .guards
            .iter()
            .map(|id| {
                by_id
                    .get(&Some(*id))
                    .copied()
                    .ok_or_else(|| invalid(*id, "supplier guard read lost actual binding"))
            })
            .collect::<Result<_>>()?;
        let occurrences = occurrences::collect(model, &syntax, &actual_attribution)?;
        Ok(Self {
            support_identity: support_identity(&support),
            syntax,
            inputs,
            support,
            bindings,
            attribution: actual_attribution,
            checked_members,
            checked_sources,
            derivations: region.derivations,
            occurrences,
            reads,
        })
    }
}
