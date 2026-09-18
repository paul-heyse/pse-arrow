// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical subjects select actual domain members in the target law arguments.
use super::{CompilerError, DomainId, Inputs, Output, compiled, invalid, unique};
use crate::passes::physical_subject::{law_phase, member, phase};
use pse_ids::SemanticId;

pub(super) fn member(
    inputs: &Inputs,
    law: &compiled::law_applications::Row,
    contribution: &compiled::contributions::Row,
    position: usize,
    domain: DomainId,
    output: &mut Output<'_>,
) -> Result<Option<SemanticId>, CompilerError> {
    let mut selected = None;
    for (axis, subject, projected) in [
        (
            member!(&contribution.subject, axis, position),
            member!(&law.subject, fixed, entity_id),
            law.subject_projection.as_str() != "identity",
        ),
        (
            phase!(&contribution.subject, axis, position),
            law_phase!(&law.subject, fixed, entity_id),
            false,
        ),
    ] {
        if projected || axis.and_then(|value| usize::try_from(value).ok()) != Some(position) {
            continue;
        }
        let Some(target) = subject else { continue };
        let member = unique(
            &inputs.domain_members,
            |row| row.domain_id == domain.as_id() && row.ref_entity_id == Some(target),
            "fixed law subject member",
        )?;
        let eligible = unique(
            &inputs.domain_eligible_members,
            |row| row.domain_id == domain.as_id() && row.member_id == member.member_id,
            "eligible fixed subject",
        )?;
        output.use_row(member);
        output.use_row(eligible);
        if selected
            .replace(member.member_id)
            .is_some_and(|old| old != member.member_id)
        {
            return Err(invalid(
                "fixed subject and phase disagree on the actual source member",
            ));
        }
    }
    Ok(selected)
}
pub(super) fn broadcast_allowed(
    law: &compiled::law_applications::Row,
    contribution: &compiled::contributions::Row,
    position: usize,
) -> Result<(), CompilerError> {
    for (axis, subject) in [
        (
            member!(&law.subject, axis, position),
            member!(&contribution.subject, fixed, entity_id),
        ),
        (
            law_phase!(&law.subject, axis, position),
            phase!(&contribution.subject, fixed, entity_id),
        ),
    ] {
        if axis.and_then(|value| usize::try_from(value).ok()) == Some(position) && subject.is_some()
        {
            return Err(invalid(
                "fixed physical contribution cannot broadcast to every law subject without an explicit member mask",
            ));
        }
    }
    Ok(())
}
