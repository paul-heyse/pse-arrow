// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Stage keys: the artifact-hash memo key (blueprint §14.3, ADR-0041, ADR-0042).
//!
//! Every declared input is keyed, including an explicit absence. The plan fingerprint is
//! **not** part of the key: the plan is a pure function of the rule plan, the catalog
//! snapshot and the engine profile, all of which are already in it (§14.2 rule 5).
//!
use super::{InputBundle, PassContext, StageKey, dag::invalid};
use crate::CompilerError;
use pse_ids::{FramedHasher, derive::context};
use pse_schema::model::PassSpec;

/// Construct a lookup key only after complete typed input admission.
///
/// This key selects a memo bucket. A hit still requires direct dependency comparison.
/// # Errors
/// Invalid input or policy membership, or missing declared engine semantics.
pub fn stage_key(
    spec: &PassSpec,
    inputs: &InputBundle,
    ctx: &PassContext<'_>,
) -> Result<StageKey, CompilerError> {
    inputs.validate(spec, ctx.registry)?;
    if ctx.registry.pass(&spec.qualified_name()) != Some(spec) {
        return Err(invalid("pass descriptor is not the registered declaration"));
    }
    let mut hash = FramedHasher::new(context::STAGE_KEY);
    hash.id(&spec.id)
        .str(spec.version)
        .hash(&ctx.registry.fingerprint());
    hash.u64(inputs.ports.len() as u64);
    for (port, binding) in &inputs.ports {
        hash.str(port).bool(binding.is_some());
        if let Some(input) = binding {
            hash.id(&input.relation_id())
                .u32(input.schema_version().0)
                .hash(&input.logical_hash().content_hash());
        }
    }
    hash.u64(ctx.policies.0.len() as u64);
    for (name, policy) in &ctx.policies.0 {
        let relation = ctx
            .registry
            .relation_by_id(policy.input.relation_id())
            .ok_or_else(|| invalid("policy relation undeclared"))?;
        policy
            .input
            .relation
            .contract()
            .validate_against_registry(ctx.registry, relation)?;
        let [key] = relation.primary_key.as_slice() else {
            return Err(invalid(
                "a selected policy must have one semantic identity key",
            ));
        };
        let ordinal = relation
            .columns
            .iter()
            .position(|column| {
                column.name == *key && column.logical_type == pse_schema::model::LogicalType::id()
            })
            .ok_or_else(|| invalid("policy key is not a declared semantic identity"))?;
        let batch = policy.input.relation.batch();
        let mut reservation = ctx.reserver.open("compiler:policy-key-admission");
        let extent = batch
            .num_rows()
            .checked_mul(
                size_of::<pse_schema::model::Cell>()
                    + size_of::<Vec<pse_schema::model::Cell>>()
                    + 16,
            )
            .and_then(|bytes| bytes.checked_add(4096))
            .ok_or_else(|| invalid("policy key extent overflow"))?;
        reservation
            .try_grow(extent)
            .map_err(|error| CompilerError::Catalog(error.into()))?;
        let keys = batch
            .project(&[ordinal])
            .map_err(|error| invalid(error.to_string()))?;
        let cells = pse_relations::cells::decode_columns(ctx.registry, &keys)?;
        if cells
            .iter()
            .filter(|row| row.first() == Some(&pse_schema::model::Cell::Id(policy.policy_id)))
            .count()
            != 1
        {
            return Err(invalid(
                "selected policy identity absent from actual declared key",
            ));
        }
        hash.str(name)
            .id(&policy.policy_id)
            .id(&relation.id)
            .hash(&policy.input.logical_hash().content_hash());
    }
    hash.bool(spec.executes_plans);
    if spec.executes_plans {
        let session = ctx
            .session
            .ok_or_else(|| invalid("rule-executing pass lacks sealed session"))?;
        hash.hash(&session.profile_hash())
            .hash(&session.function_registry_hash());
    }
    Ok(StageKey(hash.finish_hash()))
}
