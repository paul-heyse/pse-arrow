// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Lazy structural relationships inside the existing occurrence point-result owner.
use super::*;

#[derive(Debug)]
pub(super) struct ProjectionRelationship {
    target: usize,
    donor: usize,
    mapping: Option<ProjectionMap>,
}
#[derive(Debug)]
struct ProjectionMap {
    rows: Vec<usize>,
    axes: Vec<usize>,
    reads: Vec<usize>,
    witnesses: Vec<Arc<pse_model::applicability::Node>>,
    second: Arc<Vec<(usize, usize)>>,
    first: Arc<crate::execution::FirstLayout>,
    first_sources: Vec<Option<usize>>,
}
pub(super) fn allocation_bound(
    plan: &CasePlan,
    programs: &[Arc<CompiledBody>],
) -> Result<usize, MathError> {
    plan.instances
        .iter()
        .try_fold(0usize, |bytes, instance| {
            let donors = instance.groups.values().flatten().count();
            instance
                .groups
                .values()
                .flatten()
                .try_fold(bytes, |bytes, group| {
                    let request = &plan.requests[group.request];
                    if request.product != ProductKind::SelectedPartials {
                        return Some(bytes);
                    }
                    let program = &programs[group.request];
                    let one = (request.outputs.len()
                        + request.coordinates.len()
                        + program.input_formals().len())
                    .checked_mul(2 * size_of::<usize>())?
                    .checked_add(2 * size_of::<ProjectionRelationship>())?
                    .checked_add(
                        program
                            .first_result_entries()
                            .checked_mul(size_of::<Option<usize>>())?,
                    )?
                    .checked_add(program.evaluation_cache_bytes())?;
                    bytes.checked_add(one.checked_mul(donors)?)
                })
        })
        .ok_or(MathError::Limit("lazy projection relationships"))
}
fn prepare(target: &CompiledBody, donor: &CompiledBody) -> Option<ProjectionMap> {
    let target_support = target.prepared_support();
    let donor_support = donor.prepared_support();
    if target.product_kind() != ProductKind::SelectedPartials
        || !target_support.same_source(donor_support)
        || target
            .census()
            .chain(donor.census())
            .any(|order| order.provider_stages > 0)
    {
        return None;
    }
    let rows = target_support
        .outputs()
        .iter()
        .map(|output| {
            donor_support
                .outputs()
                .iter()
                .position(|candidate| candidate == output)
        })
        .collect::<Option<Vec<_>>>()?;
    let axes = target
        .coordinates()
        .iter()
        .map(|axis| {
            donor
                .coordinates()
                .iter()
                .position(|candidate| candidate == axis)
        })
        .collect::<Option<Vec<_>>>()?;
    let reads = target
        .input_formals()
        .iter()
        .map(|formal| {
            donor
                .input_formals()
                .iter()
                .position(|candidate| candidate == formal)
        })
        .collect::<Option<Vec<_>>>()?;
    let witnesses = target.projection_witnesses(target.compiled_order());
    let first = target.projection_first_layout();
    let donor_first = donor.projection_first_layout();
    let first_sources = rows
        .iter()
        .enumerate()
        .flat_map(|(row, &donor_row)| {
            first
                .entries(row)
                .map(|(axis, _)| donor_first.offset(donor_row, axes[axis]))
                .collect::<Vec<_>>()
        })
        .collect();
    Some(ProjectionMap {
        rows,
        axes,
        reads,
        witnesses,
        second: target.projection_second_pairs(),
        first,
        first_sources,
    })
}
fn cache_pair(
    groups: &mut EnumMap<EvaluationDemand, Option<GroupCache>>,
    target: EvaluationDemand,
    donor: EvaluationDemand,
) -> Option<(&mut GroupCache, &GroupCache)> {
    let target = target as usize;
    let donor = donor as usize;
    if target == donor {
        return None;
    }
    let (a, b) = groups.as_mut_slice().split_at_mut(target.max(donor));
    if target < donor {
        Some((a[target].as_mut()?, b[0].as_ref()?))
    } else {
        Some((b[0].as_mut()?, a[donor].as_ref()?))
    }
}
fn same_input_bits(inputs: &[f64], donor_bits: &[u64], reads: &[usize]) -> bool {
    inputs.len() == reads.len()
        && inputs
            .iter()
            .zip(reads)
            .all(|(value, &read)| donor_bits.get(read) == Some(&value.to_bits()))
}
pub(super) fn project_compatible(
    groups: &mut EnumMap<EvaluationDemand, Option<GroupCache>>,
    target: EvaluationDemand,
    requested: DerivativeOrder,
    assembly: &CaseAssembly,
    instance: usize,
    relationships: &mut Vec<ProjectionRelationship>,
) -> Result<Option<DerivativeOrder>, MathError> {
    let Some(target_group) = &assembly.instances[instance].groups[target] else {
        return Ok(None);
    };
    if assembly.requests[target_group.request].product != ProductKind::SelectedPartials {
        return Ok(None);
    }
    let target_request = target_group.request;
    for (donor, group) in assembly.instances[instance].groups.iter() {
        let Some(group) = group else {
            continue;
        };
        let Some((target_cache, donor_cache)) = cache_pair(groups, target, donor) else {
            continue;
        };
        let Some(completed) = donor_cache.cache else {
            continue;
        };
        let available = if assembly.requests[group.request].product == ProductKind::SelectedPartials
        {
            completed
        } else {
            DerivativeOrder::Value
        };
        if available < requested
            || assembly.requests[target_request].body != assembly.requests[group.request].body
        {
            continue;
        }
        let index = if let Some(index) = relationships
            .iter()
            .position(|map| map.target == target_request && map.donor == group.request)
        {
            index
        } else {
            relationships
                .try_reserve(1)
                .map_err(|_| MathError::Limit("lazy projection index"))?;
            let index = relationships.len();
            relationships.push(ProjectionRelationship {
                target: target_request,
                donor: group.request,
                mapping: prepare(
                    &assembly.programs[target_request],
                    &assembly.programs[group.request],
                ),
            });
            index
        };
        let Some(mapping) = &relationships[index].mapping else {
            continue;
        };
        if !same_input_bits(&target_cache.inputs, &donor_cache.bits, &mapping.reads) {
            continue;
        }
        let achieved = available.min(assembly.requests[target_request].order);
        donor_cache.result.project_into(
            &mapping.rows,
            &mapping.axes,
            achieved,
            &mapping.witnesses,
            &mapping.second,
            &mapping.first,
            &mapping.first_sources,
            &mut target_cache.spare,
        )?;
        std::mem::swap(&mut target_cache.result, &mut target_cache.spare);
        target_cache.spare.clear();
        for (bits, value) in target_cache.bits.iter_mut().zip(&target_cache.inputs) {
            *bits = value.to_bits();
        }
        target_cache.cache = Some(achieved);
        target_cache.visible = true;
        return Ok(Some(achieved));
    }
    Ok(None)
}
#[cfg(test)]
mod tests {
    use super::same_input_bits;
    #[test]
    fn projection_signature_preserves_signed_zero_fixed_inputs_and_original_gather_order() {
        let donor = [2f64.to_bits(), 0f64.to_bits(), 3f64.to_bits()];
        assert!(same_input_bits(&[3., 0.], &donor, &[2, 1]));
        assert!(!same_input_bits(&[3., -0.], &donor, &[2, 1]));
        assert!(!same_input_bits(&[4., 0.], &donor, &[2, 1]));
        assert!(!same_input_bits(&[3., 0.], &donor, &[1, 2]));
        assert!(!same_input_bits(&[3.], &donor, &[2, 1]));
    }
}
