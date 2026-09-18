// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::{
    BindingValue, GroupProjection, InstantiationEnvironment, PathBinding, TemplateError,
    project_group,
};
use pse_ids::{CancellationToken, SemanticId};
use pse_mathir::{
    ExprGraph, GuardRef, NodeId, Opcode, Payload, ValueRef,
    relations::{LoadedMath, vec_sink::KernelBinding},
};
use pse_quantity::{BoundIndexId, IndexSet};
use std::collections::{BTreeMap, BTreeSet};

/// Rebuilt occurrence correspondence. Every external root uses this same ordinal map.
#[derive(Debug)]
pub struct Instantiation {
    /// Roots in exact caller order and multiplicity.
    pub roots: Vec<NodeId>,
    /// Original admitted graph ordinal to destination ordinal.
    pub node_mapping: BTreeMap<NodeId, NodeId>,
    /// Reachable kernel bindings with parameters and all ordered input references remapped.
    pub kernel_bindings: BTreeMap<SemanticId, KernelBinding>,
    /// Newly demanded mixed-coordinate subgroups and exact source correspondences.
    pub projected_groups: BTreeMap<SemanticId, GroupProjection>,
}

/// Rebuild demanded expression occurrences into one destination graph.
/// The caller reserves its input/output workspace before constructing these native values.
/// Only the selected branch of a decided predicate is instantiated. This does not type,
/// fold solver values, expand indexed equations or certify any supplied quantity claim.
///
/// # Errors
/// Missing/incompatible actual bindings, undecided guards, malformed graphs or cancellation.
pub fn instantiate(
    source: &LoadedMath,
    roots: &[NodeId],
    environment: &InstantiationEnvironment,
    destination: &mut ExprGraph,
    cancel: &CancellationToken,
) -> Result<Instantiation, TemplateError> {
    environment.validate()?;
    let mut output = Instantiation {
        roots: Vec::new(),
        node_mapping: BTreeMap::new(),
        kernel_bindings: BTreeMap::new(),
        projected_groups: BTreeMap::new(),
    };
    let mut done = BTreeSet::new();
    let mut active = BTreeSet::new();
    for root in roots {
        let mut tasks = vec![(*root, environment.free_indices.clone(), false)];
        while let Some((id, indices, exiting)) = tasks.pop() {
            cancel.checkpoint()?;
            let key = (id, indices.clone());
            if done.contains(&key) {
                continue;
            }
            let node = source.graph.node(id)?;
            if exiting {
                let new = rebuild(source, id, &indices, environment, destination, &mut output)?;
                if output
                    .node_mapping
                    .insert(id, new)
                    .is_some_and(|old| old != new)
                {
                    return Err(environment.invalid(
                        "shared source node changed meaning between lexical occurrences",
                    ));
                }
                done.insert(key.clone());
                active.remove(&key);
                continue;
            }
            if !active.insert(key) {
                return Err(environment.invalid("cyclic template occurrence"));
            }
            tasks.push((id, indices.clone(), true));
            let nested = nested_indices(id, &node.payload, &indices, environment)?;
            let dependencies = dependencies(source, id, environment)?;
            tasks.extend(
                dependencies
                    .into_iter()
                    .rev()
                    .map(|child| (child, nested.clone(), false)),
            );
        }
    }
    output.roots = roots
        .iter()
        .map(|id| mapped(&output.node_mapping, *id, environment))
        .collect::<Result<_, _>>()?;
    Ok(output)
}

fn nested_indices(
    node: NodeId,
    payload: &Payload,
    current: &IndexSet,
    env: &InstantiationEnvironment,
) -> Result<IndexSet, TemplateError> {
    let (domain, bound) = match payload {
        Payload::Reduction {
            domain,
            bound_index,
            ..
        }
        | Payload::Broadcast {
            domain,
            bound_index,
        }
        | Payload::Integral {
            domain,
            bound_index,
            ..
        } => (domain, *bound_index),
        _ => return Ok(current.clone()),
    };
    let binding = env
        .binders
        .get(&bound)
        .ok_or_else(|| env.invalid("local binder mapping absent"))?;
    let actual = env.domain(domain)?;
    if actual != binding.domain {
        return Err(env.invalid("local binder differs from operator domain"));
    }
    let facts = env
        .domain_facts
        .get(&actual)
        .ok_or_else(|| env.invalid("local domain facts absent"))?;
    if matches!(payload, Payload::Broadcast { .. }) {
        // Broadcast adds a free axis to a value; its lexical index is already
        // available from the owning source or enclosing reduction.
        if current
            .get(binding.bound_index)
            .is_some_and(|bound| bound.domain != actual || bound.kind != facts.kind)
        {
            return Err(env.invalid("broadcast index conflicts with its actual lexical domain"));
        }
        return Ok(current.clone());
    }
    Ok(pse_mathir::index::bind(
        node,
        current,
        binding.bound_index,
        actual,
        facts,
    )?)
}
fn mapped(
    mapping: &BTreeMap<NodeId, NodeId>,
    id: NodeId,
    env: &InstantiationEnvironment,
) -> Result<NodeId, TemplateError> {
    mapping
        .get(&id)
        .copied()
        .ok_or_else(|| env.invalid(format!("node {id:?} was not instantiated")))
}
fn selected(
    payload: &Payload,
    children: &[NodeId],
    env: &InstantiationEnvironment,
) -> Result<Option<NodeId>, TemplateError> {
    if let Payload::Broadcast { bound_index, .. } = payload
        && env.fixed_indices.contains_key(bound_index)
    {
        // Selecting an actual member of a broadcast returns its value. The
        // environment validates the member and nested_indices checks its domain.
        return children
            .first()
            .copied()
            .map(Some)
            .ok_or_else(|| env.invalid("broadcast value absent"));
    }
    if let Payload::Conditional {
        guard: GuardRef::Predicate {
            source_id,
            predicate_id,
        },
    } = payload
    {
        if env
            .predicate_masks
            .contains_key(&(*source_id, *predicate_id))
        {
            return Ok(None);
        }
        let branch = usize::from(!env.predicate(*source_id, *predicate_id)?);
        return children
            .get(branch)
            .copied()
            .map(Some)
            .ok_or_else(|| env.invalid("conditional branch absent"));
    }
    Ok(None)
}
fn dependencies(
    source: &LoadedMath,
    id: NodeId,
    env: &InstantiationEnvironment,
) -> Result<Vec<NodeId>, TemplateError> {
    let node = source.graph.node(id)?;
    if let Some(branch) = selected(&node.payload, &node.children, env)? {
        return Ok(vec![branch]);
    }
    if matches!(
        node.payload,
        Payload::PendingGather { .. } | Payload::PendingPath { .. }
    ) {
        return Ok(Vec::new());
    }
    let mut children = node
        .children
        .iter()
        .copied()
        .chain(node.payload.referenced_nodes())
        .collect::<Vec<_>>();
    if let Payload::KernelCall { kernel_binding, .. } = node.payload {
        let binding = source
            .kernel_bindings
            .get(&kernel_binding)
            .ok_or_else(|| env.invalid("source kernel binding absent"))?;
        children.extend(binding.inputs.iter().map(|(_, input)| *input));
    }
    Ok(children)
}

/// Reachable pending coordinate uses under the same branch selection as realization.
/// The caller reserves the traversal workspace and evaluates these original source nodes.
/// # Errors
/// Missing nodes, undecided scalar predicates, malformed kernels or cancellation.
pub fn coordinate_nodes(
    source: &LoadedMath,
    roots: &[NodeId],
    env: &InstantiationEnvironment,
    cancel: &CancellationToken,
) -> Result<Vec<NodeId>, TemplateError> {
    let mut seen = BTreeSet::new();
    let mut pending = roots.to_vec();
    let mut result = Vec::new();
    while let Some(node) = pending.pop() {
        cancel.checkpoint()?;
        if !seen.insert(node) {
            continue;
        }
        if matches!(
            source.graph.node(node)?.payload,
            Payload::PendingGather { .. } | Payload::PendingPath { .. }
        ) {
            result.push(node);
        } else {
            pending.extend(dependencies(source, node, env)?);
        }
    }
    Ok(result)
}
fn rebuild(
    source: &LoadedMath,
    id: NodeId,
    indices: &IndexSet,
    env: &InstantiationEnvironment,
    graph: &mut ExprGraph,
    output: &mut Instantiation,
) -> Result<NodeId, TemplateError> {
    let Instantiation {
        node_mapping: mapping,
        kernel_bindings: kernels,
        projected_groups: projections,
        ..
    } = output;
    let node = source.graph.node(id)?;
    if let Some(branch) = selected(&node.payload, &node.children, env)? {
        return mapped(mapping, branch, env);
    }
    if let Payload::SymbolRef { symbol } = &node.payload {
        let value = env
            .values
            .get(symbol)
            .ok_or_else(|| env.invalid(format!("value {symbol:?} is unbound")))?;
        return bind_value(value, env, graph);
    }
    let mut payload = node.payload.clone();
    let children = node
        .children
        .iter()
        .map(|child| mapped(mapping, *child, env))
        .collect::<Result<Vec<_>, _>>()?;
    map_dependencies(&mut payload, mapping)?;
    if let Payload::Gather {
        group,
        coordinate_map,
    } = &node.payload
    {
        payload = resolve_bound_gather(*group, coordinate_map, indices, env, projections)?;
    } else {
        bind_payload(
            &mut payload,
            &node.payload,
            id,
            indices,
            env,
            graph,
            mapping,
        )?;
    }
    if let Some(gather) = env.evaluated_gathers.get(&id) {
        payload = evaluated_gather(gather, indices, env)?;
    } else if let Payload::PendingGather {
        group,
        indices: coordinates,
    } = &node.payload
    {
        payload = resolve_pending_gather(source, *group, coordinates, indices, env, projections)?;
    }
    if let Payload::PendingPath {
        source_id,
        path_id,
        indices: coordinates,
    } = &node.payload
        && !env.evaluated_gathers.contains_key(&id)
    {
        let binding = env
            .paths
            .get(&(*source_id, *path_id))
            .ok_or_else(|| env.invalid("exact instance path binding absent"))?;
        match binding {
            PathBinding::Value(BindingValue::Symbol(symbol))
                if coordinates.is_empty() && !env.unbound_parameters.contains(symbol) =>
            {
                payload = Payload::SymbolRef {
                    symbol: ValueRef::ActualSymbol(*symbol),
                };
            }
            PathBinding::Value(BindingValue::Literal {
                payload: literal,
                quantity,
            }) if coordinates.is_empty() => {
                return insert(
                    graph,
                    Opcode::Const,
                    literal.clone(),
                    &[],
                    *quantity,
                    env.instance,
                );
            }
            PathBinding::Group(group) => {
                payload =
                    resolve_coordinates(source, group, coordinates, indices, env, projections)?;
            }
            PathBinding::Value(_) => {
                return Err(
                    env.invalid("instance path value differs from its ordered coordinate contract")
                );
            }
        }
    }
    bind_kernel_payload(&mut payload, source, env, mapping, kernels)?;
    finish_rebuilt(node, payload, &children, env, graph)
}

fn evaluated_gather(
    gather: &crate::EvaluatedGather,
    indices: &IndexSet,
    env: &InstantiationEnvironment,
) -> Result<Payload, TemplateError> {
    env.require_group_bindings(&gather.group)?;
    if gather.coordinates.len() != gather.group.axes.len() {
        return Err(env.invalid("evaluated coordinate arity differs"));
    }
    if gather.coordinates.is_empty() {
        let symbol = gather
            .group
            .members
            .get(&Vec::new())
            .ok_or_else(|| env.invalid("evaluated scalar source tuple is absent"))?;
        return Ok(Payload::SymbolRef {
            symbol: ValueRef::ActualSymbol(*symbol),
        });
    }
    let coordinate_map = gather
        .coordinates
        .iter()
        .zip(&gather.group.axes)
        .enumerate()
        .map(|(position, (source, domain))| {
            let actual = env
                .binders
                .get(source)
                .ok_or_else(|| env.invalid("evaluated free binder absent"))?;
            if actual.domain != *domain || !indices.contains(actual) {
                return Err(env.invalid("evaluated binder is outside its actual lexical domain"));
            }
            Ok((
                actual.bound_index,
                u16::try_from(position).map_err(|_| env.invalid("evaluated axis overflow"))?,
            ))
        })
        .collect::<Result<_, TemplateError>>()?;
    Ok(Payload::Gather {
        group: gather.group.group,
        coordinate_map,
    })
}
fn insert(
    graph: &mut ExprGraph,
    opcode: Opcode,
    payload: Payload,
    children: &[NodeId],
    quantity: Option<pse_quantity::QuantityTypeId>,
    instance: SemanticId,
) -> Result<NodeId, TemplateError> {
    Ok(if let Some(quantity) = quantity {
        graph.insert_typed(opcode, payload, children, quantity, Some(instance))?
    } else {
        graph.insert(opcode, payload, children, Some(instance))?
    })
}
fn binder(
    old: BoundIndexId,
    indices: &IndexSet,
    env: &InstantiationEnvironment,
) -> Result<BoundIndexId, TemplateError> {
    let actual = env
        .binders
        .get(&old)
        .ok_or_else(|| env.invalid("lexical binder mapping absent"))?;
    if !indices.contains(actual) {
        return Err(env.invalid("index reference is outside its lexical environment"));
    }
    Ok(actual.bound_index)
}
fn bind_payload(
    payload: &mut Payload,
    source: &Payload,
    id: NodeId,
    indices: &IndexSet,
    env: &InstantiationEnvironment,
    graph: &mut ExprGraph,
    mapping: &BTreeMap<NodeId, NodeId>,
) -> Result<(), TemplateError> {
    match payload {
        Payload::Reduction {
            domain,
            bound_index,
            filter,
            ..
        }
        | Payload::Integral {
            domain,
            bound_index,
            filter,
            ..
        } => {
            let nested = nested_indices(id, source, indices, env)?;
            *domain = env.domain(domain)?.into();
            *bound_index = binder(*bound_index, &nested, env)?;
            *filter = env.guard(source.guard(), graph, mapping, &nested)?;
        }
        Payload::Broadcast {
            domain,
            bound_index,
        } => {
            let nested = nested_indices(id, source, indices, env)?;
            *domain = env.domain(domain)?.into();
            *bound_index = binder(*bound_index, &nested, env)?;
        }
        Payload::Derivative { wrt_domain, .. } => *wrt_domain = env.domain(wrt_domain)?.into(),
        Payload::Conditional { guard } => {
            *guard = env
                .guard(source.guard(), graph, mapping, indices)?
                .ok_or_else(|| env.invalid("conditional lacks a guard"))?;
        }
        Payload::Gather {
            group,
            coordinate_map,
        } => {
            let binding = env
                .groups
                .get(group)
                .ok_or_else(|| env.invalid("group declaration unbound"))?;
            if coordinate_map.len() != binding.axes.len() {
                return Err(env.invalid("gather arity differs from bound group"));
            }
            let mut positions = BTreeSet::new();
            for (bound, position) in coordinate_map {
                let axis = binding
                    .axes
                    .get(usize::from(*position))
                    .ok_or_else(|| env.invalid("gather axis absent"))?;
                let actual = env
                    .binders
                    .get(bound)
                    .ok_or_else(|| env.invalid("gather binder absent"))?;
                if actual.domain != *axis || !positions.insert(*position) {
                    return Err(env.invalid("gather domain or position differs"));
                }
                *bound = binder(*bound, indices, env)?;
            }
            *group = binding.group;
        }
        Payload::ImplicitRef {
            implicit_system, ..
        } => {
            *implicit_system = *env
                .implicit_systems
                .get(implicit_system)
                .ok_or_else(|| env.invalid("implicit system binding absent"))?;
        }
        _ => {}
    }
    Ok(())
}
fn resolve_pending_gather(
    source: &LoadedMath,
    group: SemanticId,
    coordinates: &[NodeId],
    indices: &IndexSet,
    env: &InstantiationEnvironment,
    projections: &mut BTreeMap<SemanticId, GroupProjection>,
) -> Result<Payload, TemplateError> {
    let binding = env
        .groups
        .get(&group)
        .ok_or_else(|| env.invalid("pending group binding absent"))?;
    resolve_coordinates(source, binding, coordinates, indices, env, projections)
}
fn resolve_coordinates(
    source: &LoadedMath,
    binding: &crate::GroupBinding,
    coordinates: &[NodeId],
    indices: &IndexSet,
    env: &InstantiationEnvironment,
    projections: &mut BTreeMap<SemanticId, GroupProjection>,
) -> Result<Payload, TemplateError> {
    if coordinates.len() != binding.axes.len() {
        return Err(env.invalid("pending gather arity differs"));
    }
    let mut literals = Vec::new();
    let mut bound = Vec::new();
    for (position, (coordinate, axis)) in coordinates.iter().zip(&binding.axes).enumerate() {
        let position = u16::try_from(position).map_err(|_| env.invalid("axis ordinal overflow"))?;
        match &source.graph.node(*coordinate)?.payload {
            Payload::SymbolRef {
                symbol: ValueRef::Index(index),
            } => {
                let actual = env
                    .binders
                    .get(index)
                    .ok_or_else(|| env.invalid("pending index binder absent"))?;
                if actual.domain != *axis {
                    return Err(env.invalid("pending index domain differs"));
                }
                if let Some(member) = env.fixed_indices.get(index) {
                    literals.push((position, *member));
                } else {
                    bound.push((binder(*index, indices, env)?, position));
                }
            }
            Payload::IntConst { value } => literals.push((
                position,
                *env.integer_members.get(&(*axis, *value)).ok_or_else(|| {
                    env.invalid("literal index has no exact actual domain coordinate")
                })?,
            )),
            Payload::SymbolRef { symbol } => {
                let Some(BindingValue::Member(member)) = env.values.get(symbol) else {
                    return Err(env.invalid("index expression is not an actual member"));
                };
                literals.push((position, *member));
            }
            _ => {
                return Err(env.invalid(
                    "nonconstant index expression needs an actual finite coordinate resolution",
                ));
            }
        }
    }
    finish_coordinates(binding, &literals, bound, env, projections)
}
fn finish_coordinates(
    binding: &crate::GroupBinding,
    literals: &[(u16, SemanticId)],
    bound: Vec<(BoundIndexId, u16)>,
    env: &InstantiationEnvironment,
    projections: &mut BTreeMap<SemanticId, GroupProjection>,
) -> Result<Payload, TemplateError> {
    if bound.len() == binding.axes.len() {
        env.require_group_bindings(binding)?;
        return Ok(Payload::Gather {
            group: binding.group,
            coordinate_map: bound,
        });
    }
    if literals.len() == binding.axes.len() {
        let tuple = literals
            .iter()
            .map(|(_, member)| *member)
            .collect::<Vec<_>>();
        let symbol = binding
            .members
            .get(&tuple)
            .ok_or_else(|| env.invalid("literal tuple has no actual group member"))?;
        return Ok(Payload::SymbolRef {
            symbol: ValueRef::ActualSymbol(*symbol),
        });
    }
    let fixed = literals
        .iter()
        .map(|(axis, member)| (i64::from(*axis), *member))
        .collect::<Vec<_>>();
    let projection = project_group(binding, &fixed, env)?;
    env.require_group_bindings(&projection.group)?;
    let group = projection.group.group;
    if projections
        .insert(group, projection.clone())
        .is_some_and(|old| old != projection)
    {
        return Err(env.invalid("one projected identity has conflicting actual contents"));
    }
    let coordinate_map = bound
        .into_iter()
        .enumerate()
        .map(|(axis, (binder, _))| {
            Ok((
                binder,
                u16::try_from(axis).map_err(|_| env.invalid("projected axis ordinal overflow"))?,
            ))
        })
        .collect::<Result<Vec<_>, TemplateError>>()?;
    Ok(Payload::Gather {
        group,
        coordinate_map,
    })
}
fn bind_kernel(
    source: &LoadedMath,
    old: SemanticId,
    env: &InstantiationEnvironment,
    mapping: &BTreeMap<NodeId, NodeId>,
) -> Result<KernelBinding, TemplateError> {
    let mut binding = source
        .kernel_bindings
        .get(&old)
        .cloned()
        .ok_or_else(|| env.invalid("kernel source row absent"))?;
    binding.binding = *env
        .kernel_bindings
        .get(&old)
        .ok_or_else(|| env.invalid("kernel instance binding absent"))?;
    binding.scope = env.instance;
    for (_, symbol, _, _) in &mut binding.parameters {
        if let Some(id) = symbol {
            let Some(BindingValue::Symbol(actual)) = env.values.get(&ValueRef::ActualSymbol(*id))
            else {
                return Err(env.invalid("kernel parameter does not resolve to an actual symbol"));
            };
            *id = *actual;
        }
    }
    for (_, input) in &mut binding.inputs {
        *input = mapped(mapping, *input, env)?;
    }
    Ok(binding)
}

fn resolve_bound_gather(
    group: SemanticId,
    coordinates: &[(BoundIndexId, u16)],
    indices: &IndexSet,
    env: &InstantiationEnvironment,
    projections: &mut BTreeMap<SemanticId, GroupProjection>,
) -> Result<Payload, TemplateError> {
    let binding = env
        .groups
        .get(&group)
        .ok_or_else(|| env.invalid("source gather group absent"))?;
    if coordinates.len() != binding.axes.len() {
        return Err(env.invalid("gather coordinate arity differs"));
    }
    let mut ordered = coordinates.to_vec();
    ordered.sort_by_key(|(_, position)| *position);
    let mut literals = Vec::new();
    let mut bound = Vec::new();
    for (position, (source, supplied)) in ordered.into_iter().enumerate() {
        if usize::from(supplied) != position {
            return Err(env.invalid("gather coordinates are not a complete axis partition"));
        }
        let actual = env
            .binders
            .get(&source)
            .ok_or_else(|| env.invalid("gather source binder absent"))?;
        if actual.domain != binding.axes[position] {
            return Err(env.invalid("gather binder domain differs"));
        }
        if let Some(member) = env.fixed_indices.get(&source) {
            literals.push((supplied, *member));
        } else {
            bound.push((binder(source, indices, env)?, supplied));
        }
    }
    finish_coordinates(binding, &literals, bound, env, projections)
}

fn bind_value(
    value: &BindingValue,
    env: &InstantiationEnvironment,
    graph: &mut ExprGraph,
) -> Result<NodeId, TemplateError> {
    match value {
        BindingValue::Symbol(actual) if !env.unbound_parameters.contains(actual) => Ok(graph
            .insert(
                Opcode::SymbolRef,
                Payload::SymbolRef {
                    symbol: ValueRef::ActualSymbol(*actual),
                },
                &[],
                Some(env.instance),
            )?),
        BindingValue::Literal { payload, quantity }
            if matches!(
                payload,
                Payload::FloatConst { .. } | Payload::IntConst { .. }
            ) =>
        {
            insert(
                graph,
                Opcode::Const,
                payload.clone(),
                &[],
                *quantity,
                env.instance,
            )
        }
        _ => Err(env
            .invalid("a member/unsupported literal cannot become an ordinary mathematical value")),
    }
}

fn map_dependencies(
    payload: &mut Payload,
    mapping: &BTreeMap<NodeId, NodeId>,
) -> Result<(), TemplateError> {
    if !matches!(
        payload,
        Payload::PendingGather { .. } | Payload::PendingPath { .. }
    ) {
        payload.map_node_references(|old| {
            mapping
                .get(&old)
                .copied()
                .ok_or_else(|| pse_mathir::MathIrError::Malformed {
                    node: Some(old),
                    detail: "payload dependency was not rebuilt".to_owned(),
                })
        })?;
    }
    Ok(())
}

fn finish_rebuilt(
    node: &pse_mathir::Node,
    payload: Payload,
    children: &[NodeId],
    env: &InstantiationEnvironment,
    graph: &mut ExprGraph,
) -> Result<NodeId, TemplateError> {
    if matches!(&payload, Payload::SymbolRef { symbol: ValueRef::ActualSymbol(symbol) } if env.unbound_parameters.contains(symbol))
    {
        return Err(env.invalid("demanded method parameter has no actual data"));
    }
    if let Payload::SymbolRef {
        symbol: ValueRef::ActualSymbol(symbol),
    } = payload
        && let Some(BindingValue::Literal { payload, quantity }) =
            env.values.get(&ValueRef::ActualSymbol(symbol))
    {
        return insert(
            graph,
            Opcode::Const,
            payload.clone(),
            &[],
            *quantity,
            env.instance,
        );
    }
    let opcode = if matches!(
        node.payload,
        Payload::Gather { .. } | Payload::PendingGather { .. } | Payload::PendingPath { .. }
    ) && matches!(payload, Payload::SymbolRef { .. })
    {
        Opcode::SymbolRef
    } else if matches!(node.payload, Payload::PendingPath { .. })
        && matches!(payload, Payload::Gather { .. })
    {
        Opcode::Gather
    } else {
        node.opcode
    };
    insert(
        graph,
        opcode,
        payload,
        children,
        node.quantity_type,
        env.instance,
    )
}

fn bind_kernel_payload(
    payload: &mut Payload,
    source: &LoadedMath,
    env: &InstantiationEnvironment,
    mapping: &BTreeMap<NodeId, NodeId>,
    kernels: &mut BTreeMap<SemanticId, KernelBinding>,
) -> Result<(), TemplateError> {
    if let Payload::KernelCall { kernel_binding, .. } = payload {
        let old = *kernel_binding;
        let binding = bind_kernel(source, old, env, mapping)?;
        *kernel_binding = binding.binding;
        if kernels
            .insert(binding.binding, binding.clone())
            .is_some_and(|previous| previous != binding)
        {
            return Err(env.invalid("one realized kernel identity names different actual bindings"));
        }
    }
    Ok(())
}
