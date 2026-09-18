// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Union complete native branches without discarding their independent supports.

use super::{
    Arc, BTreeMap, CancellationToken, NativeInput, ReservationLease, RuleError, SnapshotSession,
    internal, materialize_with_constraint, provenance, relational, witness,
};

impl NativeInput {
    /// Union compatible actual native branches. Equal complete rows coalesce; equal
    /// keys with different payloads fail. Every independent support remains present.
    /// # Errors
    /// Empty/mixed branches, incompatible declarations, conflicts or resources.
    pub async fn union(
        inputs: &[Arc<Self>],
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<Arc<Self>, RuleError> {
        let first = inputs
            .first()
            .ok_or_else(|| internal("native union needs an explicit branch"))?;
        let registry = session.registry();
        let target = registry
            .relation(&first.relation.qualified_name())
            .ok_or_else(|| internal("native union contract absent"))?;
        let support_spec = registry
            .relation("provenance.constructed_supports")
            .ok_or_else(|| internal("native support contract absent"))?;
        let mut receipt = session.reserver().open("rules:native-union-receipt");
        receipt
            .try_grow(
                inputs
                    .len()
                    .checked_mul(size_of::<Arc<Self>>() + 512)
                    .and_then(|size| size.checked_add(size_of::<Self>()))
                    .ok_or_else(|| internal("native union receipt overflow"))?,
            )
            .map_err(pse_catalog::CatalogError::from)?;
        let mut roles = BTreeMap::new();
        for (position, input) in inputs.iter().enumerate() {
            if input.relation != first.relation || input.pass_id != first.pass_id {
                return Err(internal("native union mixes relations or producers"));
            }
            input.validate_owner(first.relation, session, cancel)?;
            roles.insert(
                format!("__native_union_values:{position}"),
                input.checked.clone(),
            );
            roles.insert(
                format!("__native_union_support:{position}"),
                input.support.clone(),
            );
        }
        let owner = session.with_checked_role_inputs(roles, cancel)?;
        let values = inputs
            .iter()
            .enumerate()
            .map(|(position, _)| {
                owner
                    .scan_role(&format!("__native_union_values:{position}"))
                    .map_err(RuleError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let support = inputs
            .iter()
            .enumerate()
            .map(|(position, _)| {
                owner
                    .scan_role(&format!("__native_union_support:{position}"))
                    .map_err(RuleError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let plan = relational::union(values)?;
        let checked = materialize_with_constraint(
            plan,
            target,
            &owner,
            cancel,
            |values| witness::duplicate_keys(values, target),
            "native branches conflict on a declared complete key",
        )
        .await?;
        let support = relational::union(support)?;
        let support = materialize_with_constraint(
            support,
            support_spec,
            &owner,
            cancel,
            |values| relational::identity_collisions(values, "mapping_id"),
            "native branch support identity collision",
        )
        .await?;
        let derivations = provenance::materialize(
            &checked,
            &support,
            first.relation,
            first.pass_id,
            &owner,
            cancel,
        )
        .await?;
        receipt.shrink(receipt.size().saturating_sub(size_of::<Self>()));
        Ok(Arc::new(Self {
            relation: first.relation,
            pass_id: first.pass_id,
            checked,
            support,
            derivations,
            registry: Arc::clone(registry),
            _receipt: ReservationLease::new(receipt),
        }))
    }
}
