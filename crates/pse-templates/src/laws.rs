// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Mathematical construction over already classified, explicitly ordered law descriptors.
//! Scope, participation and ordering are supplied by admitted relational rules.
use crate::{GroupProjection, InstantiationEnvironment, TemplateError, instantiate};
use pse_ids::{CancellationToken, SemanticId};
use pse_mathir::{
    DomainRef, ExprGraph, NodeId, Opcode, Payload,
    payload::AffineTerm,
    relations::{LoadedMath, vec_sink::KernelBinding},
};
use pse_quantity::{BoundIndexRef, ConversionId, IndexSet, ReductionKind, UnitConvertSpec};
use std::collections::BTreeMap;

/// One actual included contribution occurrence after relational classification.
#[derive(Clone, Debug)]
pub struct ConservationTerm {
    /// The actual realized contribution identity, used for diagnostics and kernel scope.
    pub contribution: SemanticId,
    /// Root in the supplied source graph.
    pub root: NodeId,
    /// Exact source-to-law binder, symbol, group and kernel bindings.
    pub environment: InstantiationEnvironment,
    /// Axes reduced from this term, in explicit inner-to-outer order.
    pub reductions: Vec<BoundIndexRef>,
    /// Explicit law axes introduced only after source projection and reduction.
    pub broadcasts: Vec<BoundIndexRef>,
    /// Signed orientation: exactly +1 or -1 from the declared participation row.
    pub sign: i32,
    /// Named physical conversion, after reduction, with actual admitted coefficients.
    /// P10 independently checks the rule, both complete quantities and this edge.
    pub conversion: Option<(ConversionId, UnitConvertSpec)>,
}
/// One indexed balance body and its complete remapped kernel bindings.
#[derive(Debug)]
pub struct ConservationBody {
    /// Ordered signed contribution combination; physical inference remains P10's task.
    pub body: NodeId,
    /// Actual kernel occurrences introduced while rebinding contribution expressions.
    pub kernel_bindings: BTreeMap<SemanticId, KernelBinding>,
    /// Explicit named physical edges, retained as claims for P10 revalidation.
    pub selections: Vec<pse_mathir::relations::vec_sink::QuantitySelection>,
    /// Complete actual subgroup correspondence introduced by fixed source coordinates.
    pub projected_groups: BTreeMap<SemanticId, GroupProjection>,
}
/// Construct one conservation body without expanding its remaining free indices.
/// The caller reserves complete source, environment and output workspace before this call.
/// # Errors
/// Empty descriptors, duplicate identities, invalid signs, foreign axes, incompatible
/// final binder sets, unresolved bindings, cancellation or malformed mathematical graphs.
pub fn conservation(
    source: &LoadedMath,
    owner: SemanticId,
    free_indices: &IndexSet,
    terms: &[ConservationTerm],
    destination: &mut ExprGraph,
    cancel: &CancellationToken,
) -> Result<ConservationBody, TemplateError> {
    construct(
        source,
        owner,
        free_indices,
        terms,
        destination,
        cancel,
        false,
    )
}

/// Construct an intensive equality as subtraction, preserving point/difference
/// quantity algebra regardless of contribution identity order.
/// # Errors
/// Requires two distinct opposing terms, no reduction, and the exact free axes.
pub fn equality(
    source: &LoadedMath,
    owner: SemanticId,
    free_indices: &IndexSet,
    terms: &[ConservationTerm],
    destination: &mut ExprGraph,
    cancel: &CancellationToken,
) -> Result<ConservationBody, TemplateError> {
    if terms.len() != 2
        || terms[0].sign == terms[1].sign
        || terms.iter().any(|term| !term.reductions.is_empty())
    {
        return Err(TemplateError::Binding {
            instance: owner,
            detail: "intensive equality requires two opposing complete members without summation"
                .to_owned(),
        });
    }
    construct(
        source,
        owner,
        free_indices,
        terms,
        destination,
        cancel,
        true,
    )
}

fn construct(
    source: &LoadedMath,
    owner: SemanticId,
    free_indices: &IndexSet,
    terms: &[ConservationTerm],
    destination: &mut ExprGraph,
    cancel: &CancellationToken,
    equality: bool,
) -> Result<ConservationBody, TemplateError> {
    let failure = |detail: &str| TemplateError::Binding {
        instance: owner,
        detail: detail.to_owned(),
    };
    if terms.is_empty() {
        return Err(failure(
            "conservation requires an explicit nonempty included descriptor list",
        ));
    }
    let mut identities = std::collections::BTreeSet::new();
    let mut affine = Vec::new();
    let mut bindings = BTreeMap::new();
    let mut selections = Vec::new();
    let mut projections = BTreeMap::new();
    for term in terms {
        cancel.checkpoint()?;
        if !identities.insert(term.contribution) || !matches!(term.sign, -1 | 1) {
            return Err(failure(
                "contribution identity repeats or its orientation is not signed unity",
            ));
        }
        validate_axes(term, free_indices, owner)?;
        let instantiated =
            instantiate(source, &[term.root], &term.environment, destination, cancel)?;
        for (id, projection) in instantiated.projected_groups {
            if projections
                .insert(id, projection.clone())
                .is_some_and(|old| old != projection)
            {
                return Err(failure(
                    "projected group has conflicting actual source correspondence",
                ));
            }
        }
        let &[mut root] = instantiated.roots.as_slice() else {
            return Err(failure("law occurrence root inventory changed"));
        };
        for bound in &term.reductions {
            root = destination.insert(
                Opcode::SumOver,
                Payload::Reduction {
                    kind: ReductionKind::Sum,
                    domain: DomainRef::Actual(bound.domain),
                    bound_index: bound.bound_index,
                    filter: None,
                },
                &[root],
                Some(owner),
            )?;
        }
        for bound in &term.broadcasts {
            root = destination.insert(
                Opcode::Broadcast,
                Payload::Broadcast {
                    domain: DomainRef::Actual(bound.domain),
                    bound_index: bound.bound_index,
                },
                &[root],
                Some(owner),
            )?;
        }
        if let Some((conversion, spec)) = term.conversion {
            root = destination.unit_convert(root, spec)?;
            selections.push(pse_mathir::relations::vec_sink::QuantitySelection {
                node: root,
                operation: None,
                builtin: Some(pse_quantity::infer::BuiltInRule::UnitConvert),
                permutation: Vec::new(),
                conversions: vec![(0, conversion)],
                deferred_static_check: false,
            });
        }
        for (id, binding) in instantiated.kernel_bindings {
            if bindings
                .insert(id, binding.clone())
                .is_some_and(|old| old != binding)
            {
                return Err(failure(
                    "one law kernel occurrence has conflicting actual bindings",
                ));
            }
        }
        affine.push(AffineTerm {
            coefficient: f64::from(term.sign),
            child: root,
        });
    }
    let body = assemble_body(destination, affine, owner, equality)?;
    Ok(ConservationBody {
        body,
        kernel_bindings: bindings,
        selections,
        projected_groups: projections,
    })
}

fn validate_axes(
    term: &ConservationTerm,
    free_indices: &IndexSet,
    owner: SemanticId,
) -> Result<(), TemplateError> {
    let failure = |detail: &str| TemplateError::Binding {
        instance: owner,
        detail: detail.to_owned(),
    };
    let reduction_set = IndexSet::try_from_iter(term.reductions.iter().copied())
        .map_err(|_| failure("reduction identity has conflicting actual domains"))?;
    if reduction_set.len() != term.reductions.len()
        || !term
            .reductions
            .iter()
            .all(|bound| term.environment.free_indices.contains(bound))
    {
        return Err(failure(
            "term reductions do not leave exactly the declared law index environment",
        ));
    }
    let mut remaining = term.environment.free_indices.difference(&reduction_set);
    for bound in &term.broadcasts {
        if remaining.contains_index(bound.bound_index)
            || term
                .environment
                .domain_facts
                .get(&bound.domain)
                .is_none_or(|domain| domain.kind != bound.kind)
            || !remaining
                .insert(*bound)
                .map_err(|_| failure("broadcast binder domain conflict"))?
        {
            return Err(failure(
                "law broadcast duplicates a source axis or lacks an actual domain",
            ));
        }
    }
    if remaining != *free_indices {
        return Err(failure(
            "explicit projections, reductions and broadcasts do not leave the law axes",
        ));
    }
    Ok(())
}

fn assemble_body(
    destination: &mut ExprGraph,
    affine: Vec<AffineTerm>,
    owner: SemanticId,
    equality: bool,
) -> Result<NodeId, TemplateError> {
    let children = affine.iter().map(|term| term.child).collect::<Vec<_>>();
    let body = if equality {
        let (positive, negative) = if affine[0].coefficient.to_bits() == 1.0_f64.to_bits() {
            (affine[0].child, affine[1].child)
        } else {
            (affine[1].child, affine[0].child)
        };
        destination.sub(positive, negative)?
    } else {
        destination.insert(
            Opcode::Affine,
            Payload::Affine {
                constant: 0.0,
                constant_quantity_type: None,
                constant_unit: None,
                terms: affine,
            },
            &children,
            Some(owner),
        )?
    };
    Ok(body)
}
