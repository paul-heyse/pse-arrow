// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Composition of existing typed arenas and admissions; no source syntax or inference.
use super::*;
use pse_model::diagnostic::SourceLocation;

fn clean_symbol(source: &crate::specialize::Symbol, id: SemanticId) -> crate::specialize::Symbol {
    crate::specialize::Symbol {
        id,
        ty: source.ty.clone(),
        role: source.role,
        domain: source.domain,
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
    }
}
fn same_declaration(left: &crate::specialize::Symbol, right: &crate::specialize::Symbol) -> bool {
    left.ty == right.ty
        && left.role == right.role
        && left.domain == right.domain
        && left.local_role() == right.local_role()
}
fn empty_map() -> Relocation {
    Relocation {
        symbols: BTreeMap::new(),
        nodes: BTreeMap::new(),
        predicates: BTreeMap::new(),
        binders: BTreeMap::new(),
        functions: BTreeMap::new(),
        evidence: BTreeMap::new(),
        carriers: BTreeMap::new(),
        groups: BTreeMap::new(),
    }
}
fn offset<T: Ord + Copy + From<usize>>(count: usize, start: usize) -> Result<BTreeMap<T, T>> {
    (0..count)
        .map(|index| {
            Ok((
                T::from(index),
                T::from(start.checked_add(index).ok_or_else(|| {
                    crate::ModelingError::Budget("region composition extent overflow".into())
                })?),
            ))
        })
        .collect()
}
fn bound_actual(region: &SelectedRegion, slot: SymbolIx) -> Result<SemanticId> {
    let mut values = region
        .bindings
        .iter()
        .filter(|binding| binding.local == slot);
    let binding = values.next().ok_or_else(|| {
        invalid(
            SemanticId::NIL,
            "source scalar lacks an issued instance binding",
        )
    })?;
    if values.next().is_some() {
        return Err(invalid(
            binding.actual,
            "source scalar has repeated binding declarations",
        ));
    }
    let symbol = region
        .syntax
        .symbols
        .at(slot)
        .ok_or_else(|| invalid(binding.actual, "source binding is outside its typed owner"))?;
    if symbol.local_role() != binding.role {
        return Err(invalid(
            binding.actual,
            "source binding changes its declared scalar role",
        ));
    }
    Ok(binding.actual)
}
fn final_slot<T: Ord + Copy>(
    slot: T,
    finals: &BTreeMap<T, T>,
    aliases: &BTreeMap<T, T>,
) -> Result<T> {
    let mut current = slot;
    for _ in 0..=aliases.len() {
        if let Some(value) = finals.get(&current) {
            return Ok(*value);
        }
        current = *aliases.get(&current).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "composed coordinate has neither a reached owner nor a typed alias",
            )
        })?;
    }
    Err(invalid(
        SemanticId::NIL,
        "composed coordinate alias is cyclic",
    ))
}
fn finish_map(raw: &Relocation, finals: &Relocation, aliases: &Relocation) -> Relocation {
    fn select<T: Ord + Copy>(
        raw: &BTreeMap<T, T>,
        finals: &BTreeMap<T, T>,
        aliases: &BTreeMap<T, T>,
    ) -> BTreeMap<T, T> {
        raw.iter()
            .filter_map(|(old, new)| {
                final_slot(*new, finals, aliases)
                    .ok()
                    .map(|local| (*old, local))
            })
            .collect()
    }
    Relocation {
        symbols: select(&raw.symbols, &finals.symbols, &aliases.symbols),
        nodes: select(&raw.nodes, &finals.nodes, &aliases.nodes),
        predicates: select(&raw.predicates, &finals.predicates, &aliases.predicates),
        binders: select(&raw.binders, &finals.binders, &aliases.binders),
        functions: select(&raw.functions, &finals.functions, &aliases.functions),
        evidence: select(&raw.evidence, &finals.evidence, &aliases.evidence),
        carriers: select(&raw.carriers, &finals.carriers, &aliases.carriers),
        groups: select(&raw.groups, &finals.groups, &aliases.groups),
    }
}
fn bind_alias<T: Ord + Copy>(left: T, right: T, aliases: &mut BTreeMap<T, T>) -> Result<()> {
    if left == right {
        return Ok(());
    }
    if aliases
        .insert(left, right)
        .is_some_and(|previous| previous != right)
    {
        return Err(invalid(
            SemanticId::NIL,
            "one typed duplicate has contradictory coordinate correspondences",
        ));
    }
    Ok(())
}
// Equal actual input IDs alone never alias a range. Both complete rooted typed
// layouts, actual scalar role bindings, child owners and physical admissions must agree.
fn range_correspondence(
    left: &SelectedRegion,
    left_symbol: SymbolIx,
    left_map: &Relocation,
    right: &SelectedRegion,
    right_symbol: SymbolIx,
    right_map: &Relocation,
    aliases: &mut Relocation,
) -> Result<()> {
    let a = left
        .syntax
        .member_ranges
        .get(&left_symbol)
        .ok_or_else(|| invalid(SemanticId::NIL, "duplicate input lost original hard range"))?;
    let b = right
        .syntax
        .member_ranges
        .get(&right_symbol)
        .ok_or_else(|| invalid(SemanticId::NIL, "shared input lost original hard range"))?;
    if a.layer != b.layer {
        return Err(invalid(
            SemanticId::NIL,
            "same input has conflicting original hard-range layers",
        ));
    }
    let a = left
        .syntax
        .select(vec![a.lower, a.upper], vec![])
        .canonical_layout()?;
    let b = right
        .syntax
        .select(vec![b.lower, b.upper], vec![])
        .canonical_layout()?;
    if a.identity != b.identity {
        return Err(invalid(
            SemanticId::NIL,
            "same input has different rooted hard-range syntax",
        ));
    }
    if [
        a.symbols.len(),
        a.nodes.len(),
        a.predicates.len(),
        a.binders.len(),
        a.functions.len(),
        a.evidence.len(),
        a.carriers.len(),
        a.groups.len(),
    ] != [
        b.symbols.len(),
        b.nodes.len(),
        b.predicates.len(),
        b.binders.len(),
        b.functions.len(),
        b.evidence.len(),
        b.carriers.len(),
        b.groups.len(),
    ] {
        return Err(invalid(
            SemanticId::NIL,
            "same input hard range has different typed correspondence extents",
        ));
    }
    for (a, b) in a.symbols.iter().zip(&b.symbols) {
        let x = left
            .syntax
            .symbols
            .at(a.symbol)
            .ok_or_else(|| invalid(SemanticId::NIL, "range scalar left source owner"))?;
        let y = right
            .syntax
            .symbols
            .at(b.symbol)
            .ok_or_else(|| invalid(SemanticId::NIL, "range scalar left destination owner"))?;
        if a.role != b.role
            || a.ordinal != b.ordinal
            || !same_declaration(x, y)
            || bound_actual(left, a.symbol)? != bound_actual(right, b.symbol)?
        {
            return Err(invalid(
                SemanticId::NIL,
                "same input range has different actual typed scalar bindings",
            ));
        }
        bind_alias(
            left_map.symbols[&a.symbol],
            right_map.symbols[&b.symbol],
            &mut aliases.symbols,
        )?;
    }
    for (a, b) in a.functions.iter().zip(&b.functions) {
        if !Arc::ptr_eq(
            &left.syntax.functions[*a].product,
            &right.syntax.functions[*b].product,
        ) {
            return Err(invalid(
                SemanticId::NIL,
                "same input range changed its actual immutable function owner",
            ));
        }
    }
    for (a, b) in a.nodes.iter().zip(&b.nodes) {
        if !left.support.same_node(*a, &right.support, *b)? {
            return Err(invalid(
                SemanticId::NIL,
                "same input range has different issued physical facts",
            ));
        }
    }
    macro_rules! pairs {
        ($field:ident,$a:expr,$b:expr) => {
            for (a, b) in $a.iter().zip($b) {
                bind_alias(left_map.$field[a], right_map.$field[b], &mut aliases.$field)?;
            }
        };
    }
    pairs!(nodes, a.nodes, &b.nodes);
    pairs!(predicates, a.predicates, &b.predicates);
    pairs!(binders, a.binders, &b.binders);
    pairs!(functions, a.functions, &b.functions);
    pairs!(evidence, a.evidence, &b.evidence);
    pairs!(carriers, a.carriers, &b.carriers);
    pairs!(groups, a.groups, &b.groups);
    Ok(())
}
impl SelectedRegion {
    /// Combine admitted output regions in exact supplied order. Declarations are
    /// borrowed from the actual source owner; unused formals are never inferred from IDs.
    pub fn combine(
        regions: &[Self],
        ordered_actual_inputs: &[&crate::specialize::Symbol],
        context: &crate::TypeContext<'_>,
    ) -> Result<Self> {
        let mut raw = ResolvedSyntax::default();
        let mut actuals = BTreeMap::new();
        let mut declarations = BTreeMap::<SemanticId, crate::specialize::Symbol>::new();
        let mut input_slots = BTreeMap::new();
        for source in ordered_actual_inputs {
            if source.definition.is_some()
                || source.local_role() == LocalRole::Member
                || input_slots.contains_key(&source.id)
            {
                return Err(invalid(
                    source.id,
                    "composition requires distinct actual independent input declarations",
                ));
            }
            let slot = SymbolIx::from(raw.symbols.len());
            raw.symbols.push(clean_symbol(source, source.id));
            input_slots.insert(source.id, slot);
            actuals.insert(source.id, source.id);
            declarations.insert(source.id, clean_symbol(source, source.id));
        }
        let mut maps = Vec::with_capacity(regions.len());
        let mut range_owner = BTreeMap::<SymbolIx, (usize, SymbolIx)>::new();
        let mut duplicates = Vec::new();
        for (part, region) in regions.iter().enumerate() {
            if region.support.syntax() != region.syntax.identity()?
                || region.support_identity != support_identity(&region.support)
            {
                return Err(invalid(
                    SemanticId::NIL,
                    "composition received stale syntax/support correspondence",
                ));
            }
            // Replay evidence is intentionally not accepted as live preparation authority.
            region.support.premises()?;
            let mut map = empty_map();
            for (ordinal, source) in region.syntax.symbols.slots().iter().enumerate() {
                let old = SymbolIx::from(ordinal);
                let actual = bound_actual(region, old)?;
                if let Some(previous) = declarations.get(&actual) {
                    if !same_declaration(previous, source) {
                        return Err(invalid(
                            actual,
                            "same actual scalar changes its complete type/role/domain",
                        ));
                    }
                } else {
                    declarations.insert(actual, clean_symbol(source, actual));
                }
                let local = if source.local_role() != LocalRole::Member {
                    *input_slots.get(&actual).ok_or_else(|| {
                        invalid(
                            actual,
                            "guard or numerical input is absent from the complete declared layout",
                        )
                    })?
                } else {
                    let mut key = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
                    key.str("composition-member-slot")
                        .u64(part as u64)
                        .u64(ordinal as u64);
                    let id = key.finish_id();
                    if raw.symbols.contains_key(&id) {
                        return Err(invalid(id, "internal member slot identity collision"));
                    }
                    let slot = SymbolIx::from(raw.symbols.len());
                    raw.symbols.push(clean_symbol(source, id));
                    actuals.insert(id, actual);
                    slot
                };
                map.symbols.insert(old, local);
            }
            map.nodes = offset(region.syntax.nodes.len(), raw.nodes.len())?;
            map.predicates = offset(region.syntax.predicates.len(), raw.predicates.len())?;
            map.binders = offset(region.syntax.binders.len(), raw.binders.len())?;
            map.carriers = offset(region.syntax.carriers.len(), raw.carriers.len())?;
            map.evidence = offset(region.syntax.evidence.len(), raw.evidence.len())?;
            map.groups = offset(region.syntax.groups.len(), raw.groups.len())?;
            for (ordinal, function) in region.syntax.functions.iter().enumerate() {
                let existing = raw
                    .functions
                    .iter()
                    .enumerate()
                    .find(|(_, value)| Arc::ptr_eq(&value.product, &function.product))
                    .map(|(index, _)| FunctionIx::from(index));
                let local = if let Some(local) = existing {
                    if raw.functions[local] != *function {
                        return Err(invalid(
                            function.declaration,
                            "shared immutable function selection metadata differs",
                        ));
                    }
                    local
                } else {
                    let local = FunctionIx::from(raw.functions.len());
                    raw.functions.push(function.clone());
                    local
                };
                map.functions.insert(FunctionIx::from(ordinal), local);
            }
            for source in &region.syntax.nodes {
                raw.nodes.push(map.operation(source)?);
            }
            for source in &region.syntax.predicates {
                raw.predicates.push(map.guard(source)?);
            }
            for source in &region.syntax.binders {
                raw.binders.push(match source {
                    Binding::Value(root) => Binding::Value(map.node(*root)?),
                    Binding::Coordinate(carrier) => Binding::Coordinate(map.carrier(*carrier)?),
                    Binding::Accumulator => Binding::Accumulator,
                    Binding::Item => Binding::Item,
                });
            }
            for source in &region.syntax.carriers {
                raw.carriers.push(source.clone());
            }
            for source in &region.syntax.evidence {
                raw.evidence.push(map.usage(source)?);
            }
            for source in &region.syntax.groups {
                raw.groups.push(IndexedGroup {
                    axes: source
                        .axes
                        .iter()
                        .map(|slot| map.carrier(*slot))
                        .collect::<Result<_>>()?,
                    members: source
                        .members
                        .iter()
                        .map(|(tuple, slot)| {
                            Ok((
                                tuple.clone(),
                                slot.map(|slot| map.symbol(slot)).transpose()?,
                            ))
                        })
                        .collect::<Result<_>>()?,
                });
            }
            for (old, source) in &region.attribution.nodes {
                raw.node_sources.insert(map.node(*old)?, source.clone());
            }
            for (old, source) in &region.attribution.predicates {
                raw.predicate_sources
                    .insert(map.predicate(*old)?, source.clone());
            }
            for (ordinal, source) in region.syntax.symbols.slots().iter().enumerate() {
                if let Some(root) = source.definition {
                    raw.symbols
                        .define(map.symbol(SymbolIx::from(ordinal))?, map.node(root)?)?;
                }
            }
            for (old, fold) in &region.syntax.finite_folds {
                raw.finite_folds.insert(
                    map.node(*old)?,
                    FiniteFold {
                        carrier: map.carrier(fold.carrier)?,
                        selected: fold.selected.clone(),
                        prototype: fold.prototype.clone(),
                    },
                );
            }
            for (old, range) in &region.syntax.member_ranges {
                let local = map.symbol(*old)?;
                if let Some((owner, symbol)) = range_owner.get(&local) {
                    duplicates.push((part, *old, *owner, *symbol));
                } else {
                    range_owner.insert(local, (part, *old));
                    raw.member_ranges.insert(
                        local,
                        MemberRange {
                            lower: map.node(range.lower)?,
                            upper: map.node(range.upper)?,
                            layer: range.layer,
                        },
                    );
                }
            }
            raw.outputs.extend(map.nodes(&region.syntax.outputs)?);
            raw.validity.extend(
                region
                    .syntax
                    .validity
                    .iter()
                    .map(|slot| map.predicate(*slot))
                    .collect::<Result<Vec<_>>>()?,
            );
            maps.push(map);
        }
        // Presence/absence of a range on the same independent source must agree.
        for (part, region) in regions.iter().enumerate() {
            let reached = region
                .syntax
                .canonical_layout()?
                .symbols
                .into_iter()
                .map(|value| value.symbol)
                .collect::<BTreeSet<_>>();
            for binding in &region.bindings {
                if binding.role != LocalRole::Member && reached.contains(&binding.local) {
                    let local = maps[part].symbol(binding.local)?;
                    if raw.member_ranges.contains_key(&local)
                        != region.syntax.member_ranges.contains_key(&binding.local)
                    {
                        return Err(invalid(
                            binding.actual,
                            "shared input has contradictory original range presence",
                        ));
                    }
                }
            }
        }
        let mut aliases = empty_map();
        for (left, symbol, right, other) in duplicates {
            range_correspondence(
                &regions[left],
                symbol,
                &maps[left],
                &regions[right],
                other,
                &maps[right],
                &mut aliases,
            )?;
        }
        let layout = raw.canonical_layout()?;
        let (mut syntax, _, mut bindings, attribution, mut final_map) = compact(&raw, &layout)?;
        for binding in &mut bindings {
            binding.actual = *actuals.get(&binding.actual).ok_or_else(|| {
                invalid(
                    binding.actual,
                    "composed scalar lost its exact actual binding",
                )
            })?;
        }
        let mut inputs = Vec::with_capacity(ordered_actual_inputs.len());
        let mut counts = [0usize; 3];
        for value in &syntax.canonical_layout()?.symbols {
            counts[match value.role {
                LocalRole::Coordinate => 0,
                LocalRole::Parameter => 1,
                LocalRole::Member => 2,
            }] += 1;
        }
        for source in ordered_actual_inputs {
            let old = input_slots[&source.id];
            let local = if let Some(local) = final_map.symbols.get(&old) {
                *local
            } else {
                let role = source.local_role();
                let index = match role {
                    LocalRole::Coordinate => 0,
                    LocalRole::Parameter => 1,
                    LocalRole::Member => {
                        return Err(invalid(
                            source.id,
                            "unused member cannot become an independent port",
                        ));
                    }
                };
                let mut key = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
                key.str("local-scalar")
                    .str(if role == LocalRole::Coordinate {
                        "coordinate"
                    } else {
                        "parameter"
                    })
                    .u64(counts[index] as u64);
                counts[index] += 1;
                let local = SymbolIx::from(syntax.symbols.len());
                syntax.symbols.push(clean_symbol(source, key.finish_id()));
                final_map.symbols.insert(old, local);
                bindings.push(RegionBinding {
                    local,
                    actual: source.id,
                    role,
                });
                local
            };
            inputs.push(local);
        }
        let relocated = maps
            .iter()
            .map(|map| finish_map(map, &final_map, &aliases))
            .collect::<Vec<_>>();
        let parts = regions
            .iter()
            .zip(&relocated)
            .map(|(region, map)| (&region.support, map))
            .collect::<Vec<_>>();
        let support = PhysicalSupport::combine_relocated(&parts, &syntax, &inputs, context)?;
        let mut checked_members = BTreeMap::new();
        let mut checked_sources = BTreeMap::new();
        let new_symbols = syntax
            .canonical_layout()?
            .symbols
            .into_iter()
            .map(|value| (value.symbol, value))
            .collect::<BTreeMap<_, _>>();
        for (region, map) in regions.iter().zip(&relocated) {
            let old_symbols = region
                .syntax
                .canonical_layout()?
                .symbols
                .into_iter()
                .map(|value| (value.symbol, value))
                .collect::<BTreeMap<_, _>>();
            let ranges = region.support.checked_ranges(&region.syntax)?;
            if ranges.len() != region.checked_members.len()
                || ranges.len() != region.checked_sources.len()
            {
                return Err(invalid(
                    SemanticId::NIL,
                    "source checked inventories do not match their issued typed range roles",
                ));
            }
            for range in ranges {
                let old = old_symbols[&range.symbol];
                let local = map.symbol(range.symbol)?;
                let new = *new_symbols.get(&local).ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "checked scalar left composed canonical roles",
                    )
                })?;
                if old.role != new.role {
                    return Err(invalid(
                        SemanticId::NIL,
                        "checked scalar changed canonical role",
                    ));
                }
                merge_value(
                    &mut checked_members,
                    crate::derived::Relationship::CheckedMember {
                        role: new.role,
                        ordinal: new.ordinal as u64,
                    }
                    .identity(),
                    *region.checked_members.get(&range.member).ok_or_else(|| {
                        invalid(range.member, "source region lost its checked member")
                    })?,
                )?;
                merge_value(
                    &mut checked_sources,
                    crate::derived::Relationship::CheckedValiditySource {
                        role: new.role,
                        ordinal: new.ordinal as u64,
                    }
                    .identity(),
                    *region.checked_sources.get(&range.source).ok_or_else(|| {
                        invalid(range.source, "source region lost its checked annotation")
                    })?,
                )?;
            }
        }
        let mut derivations = crate::derived::Facts::default();
        let mut reads = RegionReads::default();
        let mut occurrences = BTreeMap::new();
        for (region, map) in regions.iter().zip(&relocated) {
            for (relationship, identity) in region.derivations.iter() {
                if derivations.publish(*relationship)? != *identity {
                    return Err(invalid(
                        *identity,
                        "source derivation is not its actual published relationship",
                    ));
                }
            }
            reads.numeric.extend(region.reads.numeric.iter().copied());
            reads.guards.extend(region.reads.guards.iter().copied());
            let mut remaining = region.occurrences.len();
            relocate_occurrences(
                &region.syntax,
                &syntax,
                Some(map),
                &region.occurrences,
                (&mut Vec::new(), &mut Vec::new()),
                &mut remaining,
                &mut occurrences,
            )?;
            if remaining != 0 {
                return Err(invalid(
                    SemanticId::NIL,
                    "source occurrence inventory has unconsumed correspondence",
                ));
            }
        }
        if syntax.occurrence_tokens(occurrences.len())? != occurrences.keys().copied().collect() {
            return Err(invalid(
                SemanticId::NIL,
                "composition lost an original occurrence or invented a new source token",
            ));
        }
        let support_identity = support_identity(&support);
        Ok(Self {
            syntax,
            inputs,
            support,
            support_identity,
            bindings,
            attribution,
            checked_members,
            checked_sources,
            derivations,
            occurrences,
            reads,
        })
    }
}
fn merge_value<T: PartialEq>(
    map: &mut BTreeMap<SemanticId, T>,
    token: SemanticId,
    value: T,
) -> Result<()> {
    if let Some(previous) = map.get(&token) {
        if previous != &value {
            return Err(invalid(
                token,
                "typed relocated token has contradictory original owners",
            ));
        }
    } else {
        map.insert(token, value);
    }
    Ok(())
}
fn relocate_occurrences(
    source: &ResolvedSyntax,
    target: &ResolvedSyntax,
    map: Option<&Relocation>,
    original: &BTreeMap<SemanticId, SourceLocation>,
    paths: (&mut Vec<usize>, &mut Vec<usize>),
    remaining: &mut usize,
    output: &mut BTreeMap<SemanticId, SourceLocation>,
) -> Result<()> {
    let (old_path, new_path) = paths;
    if old_path.len() >= 128 {
        return Err(crate::ModelingError::Budget(
            "composed source call-path depth".into(),
        ));
    }
    let old = source.canonical_layout()?;
    let new = target.canonical_layout()?;
    let new_nodes = new
        .nodes
        .iter()
        .copied()
        .enumerate()
        .map(|(ordinal, slot)| (slot, ordinal))
        .collect::<BTreeMap<_, _>>();
    let new_predicates = new
        .predicates
        .iter()
        .copied()
        .enumerate()
        .map(|(ordinal, slot)| (slot, ordinal))
        .collect::<BTreeMap<_, _>>();
    for (predicate, count) in [(false, old.nodes.len()), (true, old.predicates.len())] {
        for ordinal in 0..count {
            *remaining = remaining.checked_sub(1).ok_or_else(|| {
                crate::ModelingError::Budget(
                    "composed source correspondence exceeds issued inventory".into(),
                )
            })?;
            let actual = if predicate {
                let slot = old.predicates[ordinal];
                let slot = map.map_or(Ok(slot), |map| map.predicate(slot))?;
                *new_predicates.get(&slot).ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "original guard has no reached typed correspondence",
                    )
                })?
            } else {
                let slot = old.nodes[ordinal];
                let slot = map.map_or(Ok(slot), |map| map.node(slot))?;
                *new_nodes.get(&slot).ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "original operation has no reached typed correspondence",
                    )
                })?
            };
            let previous = occurrence_token(old_path, predicate, ordinal);
            let value = original
                .get(&previous)
                .ok_or_else(|| invalid(previous, "source occurrence DTO is absent"))?;
            merge_value(
                output,
                occurrence_token(new_path, predicate, actual),
                value.clone(),
            )?;
        }
    }
    for (ordinal, slot) in old.nodes.iter().enumerate() {
        if let Node::Apply { function, .. } | Node::Partial { function, .. } = &source.nodes[*slot]
        {
            let target_slot = map.map_or(Ok(*slot), |map| map.node(*slot))?;
            let target_function = match &target.nodes[target_slot] {
                Node::Apply { function, .. } | Node::Partial { function, .. } => *function,
                _ => {
                    return Err(invalid(
                        SemanticId::NIL,
                        "typed original call changed its actual operation role",
                    ));
                }
            };
            let left = &source.functions[*function].product;
            let right = &target.functions[target_function].product;
            if !Arc::ptr_eq(left, right) {
                return Err(invalid(
                    left.declaration,
                    "composed call changed its actual immutable child owner",
                ));
            }
            old_path.push(ordinal);
            new_path.push(*new_nodes.get(&target_slot).ok_or_else(|| {
                invalid(
                    left.declaration,
                    "composed call absent from canonical correspondence",
                )
            })?);
            relocate_occurrences(
                &left.syntax,
                &right.syntax,
                None,
                original,
                (old_path, new_path),
                remaining,
                output,
            )?;
            old_path.pop();
            new_path.pop();
        }
    }
    Ok(())
}
impl ResolvedSyntax {
    /// Complete supplied numerical layout, including unused declared ports. The reached
    /// role correspondence binds graph operands to numeric port positions without IDs.
    pub fn frame_input_layout(&self, inputs: &[SymbolIx], hash: &mut FramedHasher) -> Result<()> {
        let roles = self
            .canonical_layout()?
            .symbols
            .into_iter()
            .map(|value| (value.symbol, value))
            .collect::<BTreeMap<_, _>>();
        let mut seen = BTreeSet::new();
        hash.str("resolved-input-layout-v1")
            .u64(inputs.len() as u64);
        for slot in inputs {
            if !seen.insert(*slot) {
                return Err(invalid(
                    SemanticId::NIL,
                    "input layout repeats a scalar slot",
                ));
            }
            let value = self.symbols.at(*slot).ok_or_else(|| {
                invalid(SemanticId::NIL, "input layout left its actual typed owner")
            })?;
            if value.definition.is_some() || value.local_role() == LocalRole::Member {
                return Err(invalid(value.id, "input layout contains a defined member"));
            }
            hash.str(value.role.as_str()).str(value.domain.as_str());
            frame_type(&value.ty, hash);
            if let Some(role) = roles.get(slot) {
                hash.str("reached")
                    .str(if role.role == LocalRole::Coordinate {
                        "coordinate"
                    } else {
                        "parameter"
                    })
                    .u64(role.ordinal as u64);
            } else {
                hash.str("unused")
                    .str(if value.local_role() == LocalRole::Coordinate {
                        "coordinate"
                    } else {
                        "parameter"
                    });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{context, model};
    use super::*;
    fn setup() -> (
        pse_quantity::QuantityRegistry,
        pse_quantity::PhysicalPreconditions,
        crate::PhysicalScope,
        Type,
    ) {
        let (registry, _) = crate::kernel_types::physical();
        let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            registry.neutral_dimensionless().unwrap(),
        ));
        (
            registry,
            pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
            crate::PhysicalScope::default(),
            ty,
        )
    }
    fn actual_inputs(model: &crate::SpecializedModel) -> Vec<&crate::specialize::Symbol> {
        model.symbols.slots().iter().collect()
    }
    fn layout_key(region: &SelectedRegion, inputs: &[SymbolIx]) -> ContentHash {
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
        region.syntax.frame_input_layout(inputs, &mut hash).unwrap();
        hash.finish_hash()
    }
    #[test]
    fn region_composition_keeps_order_repeated_parts_unused_ports_and_guard_only_reads() {
        let (registry, facts, scope, ty) = setup();
        let context = context(&registry, &facts, &scope);
        let (model, relation, predicate) = model(11, &ty);
        let row = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::RelationResidual(relation)],
                    validity: &[predicate],
                },
                &context,
            )
            .unwrap();
        let indicator = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::PredicateIndicator(predicate)],
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        let ports = actual_inputs(&model);
        let parts = [row.clone(), indicator, row.clone()];
        let result = SelectedRegion::combine(&parts, &ports, &context).unwrap();
        assert_eq!(result.syntax.outputs.len(), 3);
        assert_eq!(result.syntax.validity.len(), 2);
        assert_eq!(result.inputs.len(), 3);
        assert_eq!(result.support.inputs().len(), 3);
        for (source, slot) in ports.iter().zip(&result.inputs) {
            assert_eq!(
                result
                    .bindings
                    .iter()
                    .find(|binding| binding.local == *slot)
                    .unwrap()
                    .actual,
                source.id
            );
            assert_eq!(result.syntax.symbols.at(*slot).unwrap().ty, source.ty);
            assert_eq!(
                result
                    .support
                    .input(*slot)
                    .unwrap()
                    .require_named()
                    .unwrap(),
                registry.neutral_dimensionless().unwrap()
            );
        }
        assert!(result.support.observe(&registry, &facts).unwrap());
        assert!(!result.reads.numeric.contains(&ports[2].id));
        assert!(!result.reads.guards.contains(&ports[2].id));
        assert_eq!(
            result.reads.guards,
            parts
                .iter()
                .flat_map(|part| part.reads.guards.iter().copied())
                .collect()
        );
        assert_eq!(
            result
                .syntax
                .occurrence_tokens(result.occurrences.len())
                .unwrap(),
            result.occurrences.keys().copied().collect()
        );
        let mut swapped = result.inputs.clone();
        swapped.swap(0, 1);
        assert_ne!(
            layout_key(&result, &result.inputs),
            layout_key(&result, &swapped)
        );
        assert_eq!(
            result.support.output(0).unwrap(),
            row.support.output(0).unwrap()
        );
        assert_eq!(
            result.support.output(2).unwrap(),
            row.support.output(0).unwrap()
        );
    }
    #[test]
    fn region_composition_unifies_hard_ranges_by_typed_correspondence_and_keeps_original_sources() {
        let (registry, facts, scope, ty) = setup();
        let context = context(&registry, &facts, &scope);
        let (mut model, relation, _) = model(40, &ty);
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
            .insert(SymbolIx::from(0), SemanticId::from_bytes([90; 16]));
        let a = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::Value(NodeIx::from(0))],
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        let b = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::RelationResidual(relation)],
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        let result =
            SelectedRegion::combine(&[a.clone(), b.clone()], &actual_inputs(&model), &context)
                .unwrap();
        assert_eq!(result.checked_members.len(), 1);
        assert_eq!(
            result.checked_sources.values().copied().collect::<Vec<_>>(),
            vec![SemanticId::from_bytes([90; 16])]
        );
        assert!(
            result
                .reads
                .guards
                .contains(&model.symbols.at(SymbolIx::from(1)).unwrap().id)
        );
        for value in a.occurrences.values().chain(b.occurrences.values()) {
            assert!(result.occurrences.values().any(|actual| actual == value));
        }
        let mut conflicting = b.clone();
        *conflicting.checked_sources.values_mut().next().unwrap() =
            SemanticId::from_bytes([91; 16]);
        assert!(
            SelectedRegion::combine(&[a, conflicting], &actual_inputs(&model), &context).is_err()
        );
    }
    #[test]
    fn region_composition_preserves_separate_defined_members_and_their_nominal_boundaries() {
        let (registry, facts, scope, ty) = setup();
        let context = context(&registry, &facts, &scope);
        let (mut model, _, _) = model(50, &ty);
        let actual = model.symbols.at(SymbolIx::from(2)).unwrap().id;
        let member = model.symbols.get_mut(&actual).unwrap();
        member.role = ModelingDeclarationKind::Definition;
        member.definition = Some(NodeIx::from(3));
        model.resolved.symbols = model.symbols.clone();
        let root = NodeIx::from(model.resolved.nodes.len());
        model
            .resolved
            .nodes
            .push(Node::Reference(Reference::Symbol(SymbolIx::from(2))));
        let source = model.resolved.node_sources[&NodeIx::from(3)].clone();
        model.resolved.node_sources.insert(root, source);
        let first = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::Value(root)],
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        let second = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::Value(root)],
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        let ports = model.symbols.slots().iter().take(2).collect::<Vec<_>>();
        let combined = SelectedRegion::combine(&[first, second], &ports, &context).unwrap();
        assert_eq!(
            combined
                .bindings
                .iter()
                .filter(|binding| binding.role == LocalRole::Member && binding.actual == actual)
                .count(),
            2
        );
        for root in &combined.syntax.outputs {
            assert!(combined.support.node(*root).unwrap().boundary.is_some());
        }
        assert_eq!(combined.inputs.len(), 2);
    }
    #[test]
    fn region_composition_relocates_nested_call_paths_without_changing_child_owners() {
        let (registry, facts, scope, _) = setup();
        let context = context(&registry, &facts, &scope);
        let source = "package p {fn pick(a:Scalar,b:Scalar)->Scalar valid(a>0)=a+b; def Root {var x:Scalar; var y:Scalar; eq e:pick(x,y)==1;}}";
        let package =
            crate::check(&crate::kernel_types::try_source(source).unwrap(), &context).unwrap();
        let model = crate::specialize(
            &package,
            package.names["p.Root"],
            SemanticId::from_bytes([9; 16]).into(),
            &crate::Bindings::default(),
            crate::Limits::default(),
        )
        .unwrap();
        let relation = *model.equation_roots.get(&model.equations[0].id).unwrap();
        let row = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::RelationResidual(relation)],
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        let original = row.syntax.functions[FunctionIx::from(0)].product.clone();
        let result =
            SelectedRegion::combine(&[row.clone(), row], &actual_inputs(&model), &context).unwrap();
        assert!(
            result
                .syntax
                .functions
                .iter()
                .all(|function| Arc::ptr_eq(&function.product, &original))
        );
        assert_eq!(
            result
                .syntax
                .occurrence_tokens(result.occurrences.len())
                .unwrap(),
            result.occurrences.keys().copied().collect()
        );
        assert_eq!(result.syntax.outputs.len(), 2);
    }
    #[test]
    fn region_composition_refuses_wrong_actual_layout_and_stale_live_context() {
        let (registry, facts, scope, ty) = setup();
        let context = context(&registry, &facts, &scope);
        let (model, relation, predicate) = model(60, &ty);
        let row = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::RelationResidual(relation)],
                    validity: &[predicate],
                },
                &context,
            )
            .unwrap();
        let ports = actual_inputs(&model);
        assert!(
            SelectedRegion::combine(std::slice::from_ref(&row), &ports[..1], &context).is_err()
        );
        let mut wrong = (*ports[0]).clone();
        wrong.domain = ModelingVariableDomain::Integer;
        let mut wrong_ports = ports.clone();
        wrong_ports[0] = &wrong;
        assert!(
            SelectedRegion::combine(std::slice::from_ref(&row), &wrong_ports, &context).is_err()
        );
        let empty = pse_quantity::QuantityRegistryBuilder::new()
            .build()
            .unwrap();
        let stale = crate::TypeContext {
            quantities: &empty,
            ..context
        };
        assert!(SelectedRegion::combine(std::slice::from_ref(&row), &ports, &stale).is_err());
    }
    #[test]
    #[allow(
        unsafe_code,
        reason = "controlled test recipe transfers only actual admitted support; restoration must remain non-live"
    )]
    fn region_composition_refuses_qualified_restored_support_as_live_context() {
        let (registry, facts, scope, ty) = setup();
        let context = context(&registry, &facts, &scope);
        let (model, relation, _) = model(80, &ty);
        let mut row = model
            .prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::RelationResidual(relation)],
                    validity: &[],
                },
                &context,
            )
            .unwrap();
        let record = SupportRecord::capture(&row.support).unwrap();
        let (_, stream) =
            pse_quantity::resolved::receipts::capture(|| Ok::<_, pse_quantity::QuantityError>(()))
                .unwrap();
        let payload = serde_json::to_vec(&(&stream, &record)).unwrap();
        let digest = pse_ids::scientific_replay::admitted_recipe_payload_hash(&payload);
        // SAFETY: Isolated immutable fixture of actual successful model admission,
        // using the private process authority boundary solely to exercise strict replay.
        let recipe = unsafe {
            pse_ids::scientific_replay::QualifiedScientificRecipe::from_admitted_process_message(
                payload,
                digest,
                ContentHash::from_bytes([1; 32]),
                ContentHash::from_bytes([2; 32]),
            )
        }
        .unwrap();
        row.support = pse_quantity::resolved::receipts::replay(&stream, recipe.records(), || {
            Ok::<_, pse_quantity::QuantityError>(record.restore(&row.syntax))
        })
        .unwrap()
        .unwrap();
        row.support_identity = support_identity(&row.support);
        assert!(SelectedRegion::combine(&[row], &actual_inputs(&model), &context).is_err());
    }
}
