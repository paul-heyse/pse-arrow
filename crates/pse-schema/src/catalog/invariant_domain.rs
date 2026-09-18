// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native continuous-domain and ordinal uniqueness queries.
use super::inv::declare as invariant;
use crate::{RegistryBuilder, model::InvariantKind};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    invariant(
        builder,
        "authored.domains",
        "cardinality:continuous_detail",
        InvariantKind::Cardinality,
        &["domain_id"],
        "SELECT d.domain_id FROM authored.domains d WHERE d.continuous IS TRUE AND NOT EXISTS (SELECT 1 FROM authored.continuous_domains c WHERE c.domain_id = d.domain_id)",
        &["authored.domains", "authored.continuous_domains"],
        "Every continuous domain has one detail; its declared primary key enforces at most one.",
    );
    invariant(
        builder,
        "authored.continuous_domains",
        "domain:continuous_context",
        InvariantKind::Domain,
        &["domain_id"],
        "SELECT c.domain_id FROM authored.continuous_domains c JOIN authored.domains d ON c.domain_id = d.domain_id WHERE d.continuous IS NOT TRUE OR d.unit_id IS NULL OR c.unit_id IS DISTINCT FROM d.unit_id",
        &["authored.continuous_domains", "authored.domains"],
        "Only continuous domains carry a detail, with an explicit parent unit equal to the detail unit.",
    );
    invariant(
        builder,
        "authored.domain_members",
        "cardinality:member_ordinal",
        InvariantKind::Cardinality,
        &["member_id"],
        "SELECT member_id FROM (SELECT member_id, COUNT(*) OVER (PARTITION BY domain_id, ordinal) AS occurrences FROM authored.domain_members) s WHERE occurrences > 1",
        &["authored.domain_members"],
        "Member ordinals identify exactly one member within each actual domain; every offending member is reported.",
    );
}
