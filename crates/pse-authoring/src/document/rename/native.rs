// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native comparisons at external reader admission and rename-preservation boundaries.
use crate::{
    AuthoringError,
    document::{Batches, load::contract},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::generated::authored;

pub(super) async fn correspondence(
    expected: &Batches,
    actual: &Batches,
    session: &SnapshotSession,
    cancel: &CancellationToken,
    completed: &mut crate::native_relations::plans::Completions,
) -> Result<(), AuthoringError> {
    let execution = crate::native_relations::plans::session(session)?;
    for (id, expected) in expected {
        let Some(actual) = actual.get(id) else {
            // AuthoredReader explicitly represents an absent relation as empty.
            // A projected nonempty source relation still requires actual rows.
            if expected.batch().num_rows() == 0 {
                continue;
            }
            return Err(contract(
                None,
                &format!("reader omits nonempty source relation {id}"),
            ));
        };
        if expected.batch().num_rows() == 0 && actual.batch().num_rows() == 0 {
            continue;
        }
        let spec = session
            .registry()
            .relation_by_id(*id)
            .ok_or_else(|| contract(None, "source relation absent"))?;
        let bound = crate::native_relations::plans::roles(
            &execution,
            actual.clone(),
            expected.clone(),
            cancel,
        )?;
        for role in ["change_before", "change_after"] {
            completed.push(crate::native_relations::unique(&bound, role, spec, cancel).await?);
        }
        let plan = crate::native_relations::plans::difference(&bound, spec)?;
        if crate::native_relations::plans::execute_recorded(&bound, plan, cancel, completed)
            .await?
            .iter()
            .any(|batch| batch.num_rows() != 0)
        {
            return Err(contract(
                None,
                "reader rows differ from their exact original sources",
            ));
        }
    }
    Ok(())
}
pub(super) async fn preserved(
    before: &Batches,
    after: &Batches,
    session: &SnapshotSession,
    cancel: &CancellationToken,
    completed: &mut crate::native_relations::plans::Completions,
) -> Result<(), AuthoringError> {
    let targets = [
        authored::case_spec_targets::RELATION_ID,
        authored::case_activation_targets::RELATION_ID,
        authored::observation_targets::RELATION_ID,
    ];
    let expected = before
        .iter()
        .filter(|(id, _)| targets.contains(id))
        .map(|(id, batch)| (*id, batch.clone()))
        .collect();
    correspondence(&expected, after, session, cancel, completed).await?;
    Ok(())
}
