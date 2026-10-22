// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Stateless, read-only database adapter for the native study policy owner.
//! Scope selection and every effect remain in the enclosing database transaction.
#![cfg(any(target_family = "wasm", test))]
use std::collections::{BTreeMap, BTreeSet};
use pse_model::study::{ActionKind, PointFacts, PointPolicy, WaitReason};
use surrealdb_types::{Bytes, Decimal, SurrealValue};

#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
struct Page {
    generation: Decimal,
    cancelled: bool,
    rows: Vec<Row>,
    items: Vec<Item>,
}
#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
struct Row {
    key: String,
    occurrence: Decimal,
    revision: Decimal,
    policy: Bytes,
    facts: Bytes,
    assigned: bool,
    settled: bool,
}
#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
struct Item {
    point: String,
    predecessors: Vec<String>,
}
#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
struct Expected {
    key: String,
    revision: Decimal,
}
#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
struct Decision {
    point: String,
    expected: Vec<Expected>,
    pending: Decimal,
    eligible: bool,
    premises: Bytes,
}
#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
struct Reply {
    accepted: bool,
    decisions: Vec<Decision>,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("invalid or oversized readiness policy scope")]
#[diagnostic(code(validation::invariant))]
struct InvalidScope;

fn uint(value: Decimal) -> Result<u64, InvalidScope> {
    if !value.fract().is_zero() || value < Decimal::ZERO || value > Decimal::from(u64::MAX) {
        return Err(InvalidScope);
    }
    value.normalize().to_string().parse().map_err(|_| InvalidScope)
}

struct DecodedRow {
    policy: PointPolicy,
    facts: PointFacts,
    revision: u64,
    assigned: bool,
    settled: bool,
}

fn decisions(page: Page) -> Result<Vec<Decision>, InvalidScope> {
    let generation = uint(page.generation)?;
    if page.rows.len() > 128 || page.items.len() > 65 { return Err(InvalidScope); }
    // Preflight aggregate input before decoding JSON or constructing policy products.
    // Native scope acquisition also bounds its encoded value before invoking WASM.
    let mut bytes = 0usize;
    for row in &page.rows {
        if row.key.len() > 4096 || row.policy.len() > 16 * 1024 || row.facts.len() > 128 * 1024 {
            return Err(InvalidScope);
        }
        bytes = bytes.checked_add(row.key.len()).and_then(|n| n.checked_add(row.policy.len()))
            .and_then(|n| n.checked_add(row.facts.len())).ok_or(InvalidScope)?;
    }
    for item in &page.items {
        if item.predecessors.len() > 64 || item.point.len() > 4096 { return Err(InvalidScope); }
        bytes = bytes.checked_add(item.point.len()).ok_or(InvalidScope)?;
        for key in &item.predecessors {
            if key.len() > 4096 { return Err(InvalidScope); }
            bytes = bytes.checked_add(key.len()).ok_or(InvalidScope)?;
        }
    }
    if bytes > 1024 * 1024 { return Err(InvalidScope); }
    let mut rows = BTreeMap::new();
    let mut occurrences = BTreeSet::new();
    for row in page.rows {
        let occurrence = uint(row.occurrence)?;
        let revision = uint(row.revision)?;
        let policy: PointPolicy = serde_json::from_slice(row.policy.as_ref()).map_err(|_| InvalidScope)?;
        let mut facts: PointFacts = serde_json::from_slice(row.facts.as_ref()).map_err(|_| InvalidScope)?;
        if u64::from(policy.key.0) != occurrence || u64::from(facts.key.0) != occurrence
            || !occurrences.insert(occurrence) { return Err(InvalidScope); }
        facts.revision = revision;
        if rows.insert(row.key, DecodedRow { policy, facts, revision, assigned: row.assigned, settled: row.settled }).is_some() {
            return Err(InvalidScope);
        }
    }
    let mut required = BTreeSet::new();
    let mut candidates = BTreeSet::new();
    let mut output = Vec::with_capacity(page.items.len());
    let mut output_bytes = 0usize;
    for item in page.items {
        if !candidates.insert(item.point.clone()) { return Err(InvalidScope); }
        let point = rows.get(&item.point).ok_or(InvalidScope)?;
        let predecessors = item.predecessors.iter().collect::<BTreeSet<_>>();
        if predecessors.len() != item.predecessors.len() || predecessors.contains(&item.point) { return Err(InvalidScope); }
        let keys = std::iter::once(&item.point).chain(item.predecessors.iter()).collect::<Vec<_>>();
        let mut facts = Vec::with_capacity(keys.len());
        let mut policies = Vec::with_capacity(item.predecessors.len());
        let mut expected = Vec::with_capacity(keys.len());
        let mut premises = Vec::with_capacity(keys.len());
        let mut pending = 0u64;
        for (position, key) in keys.into_iter().enumerate() {
            let row = rows.get(key).ok_or(InvalidScope)?;
            required.insert(key.clone());
            facts.push(row.facts.clone());
            if position != 0 {
                policies.push(row.policy.clone());
                pending += u64::from(!row.settled);
            }
            expected.push(Expected { key: key.clone(), revision: Decimal::from(row.revision) });
            premises.push((key.clone(), row.revision));
        }
        // The stored candidate seed is already among these exact facts. Unknown-effect
        // precedence and every scientific disposition belong solely to candidate_action.
        let action = pse_model::study_policy::candidate_action(&point.policy, &policies, &facts, page.cancelled)
            .map_err(|_| InvalidScope)?;
        let eligible = !point.assigned && !point.settled && !matches!(action.kind,
            ActionKind::Wait(WaitReason::Dependency { .. } | WaitReason::Assigned | WaitReason::Terminal));
        let premises = serde_json::to_vec(&(generation, premises)).map_err(|_| InvalidScope)?;
        if premises.len() > 16 * 1024 { return Err(InvalidScope); }
        // Conservative native-value envelope, including both repeated read-set keys and
        // JSON premises. Count the complete reply rather than only its blob payloads.
        let expected_bytes = expected.iter().try_fold(0usize, |sum, row| {
            sum.checked_add(128)?.checked_add(row.key.len())
        }).ok_or(InvalidScope)?;
        output_bytes = output_bytes.checked_add(512).and_then(|n| n.checked_add(item.point.len()))
            .and_then(|n| n.checked_add(expected_bytes)).and_then(|n| n.checked_add(premises.len()))
            .ok_or(InvalidScope)?;
        if output_bytes > 1024 * 1024 { return Err(InvalidScope); }
        output.push(Decision { point: item.point, expected, pending: Decimal::from(pending), eligible, premises: Bytes::from(premises) });
    }
    if required != rows.keys().cloned().collect() { return Err(InvalidScope); }
    Ok(output)
}

fn evaluate(page: Page) -> Reply {
    match decisions(page) {
        Ok(decisions) => Reply { accepted: true, decisions },
        Err(_) => Reply { accepted: false, decisions: Vec::new() },
    }
}

#[cfg(target_family = "wasm")]
#[surrealism::surrealism]
mod study {
    use super::{Page, Reply, evaluate};
    /// Compute readiness from one complete bounded native scope. Grants no permission.
    #[surrealism::surrealism]
    fn readiness(page: Page) -> Reply { evaluate(page) }
}

// These exports occur only in a separately hashed qualification artifact. The
// production packager verifies the exact single-export manifest before activation.
#[cfg(all(target_family = "wasm", feature = "module-qualification"))]
#[surrealism::surrealism]
mod qualification {
    /// Trap after a sibling native write so rollback can be observed independently.
    #[surrealism::surrealism]
    fn trap() -> bool { core::arch::wasm32::unreachable() }

    /// Deliberately consume an invocation until strict runtime interruption.
    #[surrealism::surrealism]
    fn busy() -> bool { loop { std::hint::spin_loop(); } }

    /// An allocation larger than the declared linear-memory ceiling must refuse.
    #[surrealism::surrealism]
    fn memory_refused() -> bool {
        let mut bytes = Vec::<u8>::new();
        bytes.try_reserve_exact(96 * 1024 * 1024).is_err()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::generated::enums::StudyPointState;
    use pse_model::study::{Dependency, EffectState, OccurrenceKey, ScientificFacts, SeedNeed, StartPolicy};

    fn row(key: u32, dependencies: Vec<Dependency>, settled: bool) -> Row {
        let policy = PointPolicy { key: OccurrenceKey(key), dependencies, seed_need: SeedNeed::NotNeeded,
            start: StartPolicy::Fresh, attempt_limit: 1 };
        let facts = PointFacts { key: policy.key, revision: 0, lifecycle: if settled { StudyPointState::Completed } else { StudyPointState::Pending },
            native_started: false, scientific: ScientificFacts { usable: settled, seed_permission: settled, candidate_use: None },
            attempt_count: 0, retry_failure: None, effect: EffectState::Absent, seed: None, admitted_seed_need: None };
        Row { key: format!("p{key}"), occurrence: Decimal::from(key), revision: Decimal::from(u64::MAX),
            policy: Bytes::from(serde_json::to_vec(&policy).unwrap()), facts: Bytes::from(serde_json::to_vec(&facts).unwrap()),
            assigned: false, settled }
    }
    fn shared(settled: bool) -> Page {
        Page { generation: Decimal::from(u64::MAX), cancelled: false,
            rows: vec![row(0, vec![], settled), row(1, vec![Dependency::Ordering(OccurrenceKey(0))], false),
                row(2, vec![Dependency::UsableResult(OccurrenceKey(0))], false)],
            items: vec![Item { point: "p1".into(), predecessors: vec!["p0".into()] },
                Item { point: "p2".into(), predecessors: vec!["p0".into()] }] }
    }
    #[test]
    fn shared_scope_preserves_native_policy_and_full_unsigned_premises() {
        for settled in [false, true] {
            let page = shared(settled);
            let reply = evaluate(page.clone());
            assert!(reply.accepted);
            assert_eq!(reply.decisions.len(), 2);
            for (item, decision) in page.items.iter().zip(reply.decisions) {
                let native = page.rows.iter().find(|row| row.key == item.point).unwrap();
                let predecessor = &page.rows[0];
                let policy: PointPolicy = serde_json::from_slice(native.policy.as_ref()).unwrap();
                let predecessor_policy: PointPolicy = serde_json::from_slice(predecessor.policy.as_ref()).unwrap();
                let mut facts: PointFacts = serde_json::from_slice(native.facts.as_ref()).unwrap();
                let mut predecessor_facts: PointFacts = serde_json::from_slice(predecessor.facts.as_ref()).unwrap();
                facts.revision = u64::MAX; predecessor_facts.revision = u64::MAX;
                let action = pse_model::study_policy::candidate_action(&policy, &[predecessor_policy], &[facts, predecessor_facts], false).unwrap();
                assert_eq!(decision.eligible, !matches!(action.kind, ActionKind::Wait(WaitReason::Dependency { .. } | WaitReason::Assigned | WaitReason::Terminal)));
                assert_eq!(uint(decision.pending).unwrap(), u64::from(!settled));
                let premises: (u64, Vec<(String, u64)>) = serde_json::from_slice(decision.premises.as_ref()).unwrap();
                assert_eq!(premises, (u64::MAX, vec![(item.point.clone(), u64::MAX), ("p0".into(), u64::MAX)]));
            }
        }
    }
    #[test]
    fn malformed_or_incomplete_scopes_return_no_partial_decisions() {
        let mut bad = Vec::new();
        let mut page = shared(true); page.rows.push(page.rows[0].clone()); bad.push(page);
        let mut page = shared(true); page.items[0].predecessors.clear(); bad.push(page);
        let mut page = shared(true); page.rows[0].occurrence = Decimal::from(19); bad.push(page);
        let mut page = shared(true); page.rows[0].revision = Decimal::new(15, 1); bad.push(page);
        let mut page = shared(true); page.rows[1].policy = Bytes::from(vec![b' '; 16 * 1024 + 1]); bad.push(page);
        let mut page = shared(true); page.items.push(page.items[0].clone()); bad.push(page);
        let mut page = shared(true); page.rows.push(row(7, vec![], false)); bad.push(page);
        for page in bad { let reply = evaluate(page); assert!(!reply.accepted); assert!(reply.decisions.is_empty()); }
    }
    #[test]
    fn native_assigned_and_settled_gates_remain_authoritative() {
        for assigned in [false, true] {
            for settled in [false, true] {
                let mut page = shared(true);
                page.rows[1].assigned = assigned; page.rows[1].settled = settled;
                let reply = evaluate(page);
                assert!(reply.accepted);
                assert_eq!(reply.decisions[0].eligible, !assigned && !settled);
            }
        }
    }
}
