// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure native readiness policy inside the invoking transaction.
use crate::{
    generated as native,
    native::{self as boundary, Unsigned},
};
use pse_model::study::{ActionKind, PointFacts, PointPolicy, WaitReason};
use std::collections::{BTreeMap, BTreeSet};
use surrealdb_types::{SurrealValue, Value};

struct AdmittedPage(Page);

/// Compute the bounded native product without acquiring data or granting authority.
/// # Errors
/// Oversized or malformed native input refuses before semantic products are decoded.
pub fn evaluate_readiness(value: Value) -> Result<Value, boundary::NativeError> {
    let input = AdmittedPage::from_value(value).map_err(|_| boundary::NativeError)?;
    Ok(evaluate(input.0).into_value())
}
impl SurrealValue for AdmittedPage {
    fn kind_of() -> surrealdb_types::Kind {
        Page::kind_of()
    }
    fn into_value(self) -> Value {
        self.0.into_value()
    }
    fn from_value(value: Value) -> Result<Self, surrealdb_types::Error> {
        boundary::admit(&value, 1024 * 1024, 128)
            .map_err(|_| surrealdb_types::conversion_error(Page::kind_of(), Value::None))?;
        preflight(&value)
            .map_err(|_| surrealdb_types::conversion_error(Page::kind_of(), Value::None))?;
        Page::from_value(value).map(Self)
    }
}

fn preflight(value: &Value) -> Result<(), InvalidScope> {
    let Value::Object(page) = value else {
        return Err(InvalidScope);
    };
    if matches!(page.get("mode"),Some(Value::String(mode)) if mode=="claim")
        && !page.contains_key("seed")
    {
        return Err(InvalidScope);
    }
    let Some(Value::Array(rows)) = page.get("rows") else {
        return Err(InvalidScope);
    };
    let Some(Value::Array(items)) = page.get("items") else {
        return Err(InvalidScope);
    };
    if rows.len() > 128 || items.len() > 65 {
        return Err(InvalidScope);
    }
    if let Some(seed) = page.get("seed") {
        boundary::admit(seed, 16 * 1024, 128).map_err(|_| InvalidScope)?;
    }
    let key = |value: Option<&Value>| matches!(value,Some(Value::String(text)) if text.len()<=4096);
    for row in rows.iter() {
        let Value::Object(row) = row else {
            return Err(InvalidScope);
        };
        if !key(row.get("key")) {
            return Err(InvalidScope);
        }
        for (field, maximum) in [
            ("policy", 16 * 1024),
            ("facts", 128 * 1024),
            ("outcome", 128 * 1024),
        ] {
            if let Some(value) = row.get(field) {
                boundary::admit(value, maximum, 128).map_err(|_| InvalidScope)?;
            }
        }
    }
    for item in items.iter() {
        let Value::Object(item) = item else {
            return Err(InvalidScope);
        };
        let Some(Value::Array(predecessors)) = item.get("predecessors") else {
            return Err(InvalidScope);
        };
        if !key(item.get("point"))
            || predecessors.len() > 64
            || predecessors.iter().any(|value| !key(Some(value)))
        {
            return Err(InvalidScope);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
struct Page {
    generation: Unsigned,
    cancelled: bool,
    mode: Option<String>,
    seed: Option<Value>,
    rows: Vec<Row>,
    items: Vec<Item>,
}
#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
struct Row {
    key: String,
    occurrence: Unsigned,
    revision: Unsigned,
    policy: Value,
    facts: Value,
    assigned: bool,
    settled: bool,
    outcome: Option<Value>,
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
    revision: Unsigned,
}
#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
struct Decision {
    point: String,
    expected: Vec<Expected>,
    pending: Unsigned,
    eligible: bool,
    premises: Value,
    action: Option<Value>,
    disposition: Option<String>,
    start: Option<Value>,
    facts: Option<Value>,
    outcome: Option<Value>,
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

struct DecodedRow {
    policy: PointPolicy,
    facts: PointFacts,
    revision: u64,
    assigned: bool,
    settled: bool,
    outcome: Option<pse_model::study::PointOutcome>,
}

fn decisions(page: Page) -> Result<Vec<Decision>, InvalidScope> {
    let generation = page.generation.0;
    if page.rows.len() > 128 || page.items.len() > 65 {
        return Err(InvalidScope);
    }
    let progression = match page.mode.as_deref() {
        None => false,
        Some("prepare" | "claim") if page.items.len() == 1 => true,
        _ => return Err(InvalidScope),
    };
    let resolved_seed = if page.mode.as_deref() == Some("claim") {
        Some(match page.seed {
            None | Some(Value::None) => None,
            Some(value) => Some(native::decode_study_seed_fact(value).map_err(|_| InvalidScope)?),
        })
    } else {
        None
    };
    for row in &page.rows {
        if row.key.len() > 4096 {
            return Err(InvalidScope);
        }
        boundary::admit(&row.policy, 16 * 1024, 128).map_err(|_| InvalidScope)?;
        boundary::admit(&row.facts, 128 * 1024, 128).map_err(|_| InvalidScope)?;
    }
    for item in &page.items {
        if item.predecessors.len() > 64
            || item.point.len() > 4096
            || item.predecessors.iter().any(|key| key.len() > 4096)
        {
            return Err(InvalidScope);
        }
    }
    let mut rows = BTreeMap::new();
    let mut occurrences = BTreeSet::new();
    for row in page.rows {
        let occurrence = row.occurrence.0;
        let revision = row.revision.0;
        let policy: PointPolicy =
            native::decode_admitted_study_point_policy(row.policy).map_err(|_| InvalidScope)?;
        let mut facts: PointFacts =
            native::decode_study_point_facts(row.facts).map_err(|_| InvalidScope)?;
        let outcome: Option<pse_model::study::PointOutcome> = row
            .outcome
            .map(|value| native::decode_study_point_outcome(value).map_err(|_| InvalidScope))
            .transpose()?;
        if u64::from(policy.key.0) != occurrence
            || u64::from(facts.key.0) != occurrence
            || !occurrences.insert(occurrence)
            || outcome
                .as_ref()
                .is_some_and(|value| u64::from(value.key.0) != occurrence)
        {
            return Err(InvalidScope);
        }
        facts.revision = revision;
        if rows
            .insert(
                row.key,
                DecodedRow {
                    policy,
                    facts,
                    revision,
                    assigned: row.assigned,
                    settled: row.settled,
                    outcome,
                },
            )
            .is_some()
        {
            return Err(InvalidScope);
        }
    }
    let mut required = BTreeSet::new();
    let mut candidates = BTreeSet::new();
    let mut output = Vec::with_capacity(page.items.len());
    let mut output_bytes = 0usize;
    for item in page.items {
        if !candidates.insert(item.point.clone()) {
            return Err(InvalidScope);
        }
        let point = rows.get(&item.point).ok_or(InvalidScope)?;
        let predecessors = item.predecessors.iter().collect::<BTreeSet<_>>();
        if predecessors.len() != item.predecessors.len() || predecessors.contains(&item.point) {
            return Err(InvalidScope);
        }
        let keys = std::iter::once(&item.point)
            .chain(item.predecessors.iter())
            .collect::<Vec<_>>();
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
            expected.push(Expected {
                key: key.clone(),
                revision: Unsigned(row.revision),
            });
            premises.push((key.clone(), row.revision));
        }
        if progression {
            facts[0].seed = resolved_seed
                .clone()
                .unwrap_or_else(|| pse_model::study_policy::preparation_seed(&point.policy));
        }
        // The stored candidate seed is already among these exact facts. Unknown-effect
        // precedence and every scientific disposition belong solely to candidate_action.
        let action = pse_model::study_policy::candidate_action(
            &point.policy,
            &policies,
            &facts,
            page.cancelled,
        )
        .map_err(|_| InvalidScope)?;
        let eligible = !point.assigned
            && !point.settled
            && !matches!(
                action.kind,
                ActionKind::Wait(
                    WaitReason::Dependency { .. } | WaitReason::Assigned | WaitReason::Terminal
                )
            );
        let mut disposition = None;
        let mut start = None;
        let mut next_facts = None;
        let mut outcome = None;
        let action_bytes = if progression {
            disposition = Some(
                match &action.kind {
                    ActionKind::Wait(WaitReason::ScientificAdmission) => "admit",
                    ActionKind::Wait(WaitReason::SeedResolution { .. }) => "prepare",
                    ActionKind::Start(provenance) => {
                        start = Some(
                            native::encode_study_start_provenance(provenance.clone())
                                .map_err(|_| InvalidScope)?,
                        );
                        let mut assigned = facts[0].clone();
                        assigned.revision = 0;
                        assigned.attempt_count =
                            assigned.attempt_count.checked_add(1).ok_or(InvalidScope)?;
                        assigned.lifecycle = pse_model::generated::enums::StudyPointState::Assigned;
                        assigned.native_started = false;
                        next_facts = Some(
                            native::encode_study_point_facts(assigned).map_err(|_| InvalidScope)?,
                        );
                        if page.mode.as_deref() == Some("claim") {
                            "start"
                        } else {
                            "prepare"
                        }
                    }
                    ActionKind::Refuse(_) | ActionKind::Cancel
                        if !point.assigned && !point.settled =>
                    {
                        if let Some((settled, observation)) =
                            pse_model::study_policy::policy_settlement(
                                &facts[0],
                                point.outcome.clone(),
                                &action,
                            )
                        {
                            next_facts = Some(
                                native::encode_study_point_facts(settled)
                                    .map_err(|_| InvalidScope)?,
                            );
                            outcome = Some(
                                native::encode_study_point_outcome(observation)
                                    .map_err(|_| InvalidScope)?,
                            );
                            "settle"
                        } else {
                            "wait"
                        }
                    }
                    _ => "wait",
                }
                .to_owned(),
            );
            Some(native::encode_study_point_action(action).map_err(|_| InvalidScope)?)
        } else {
            None
        };
        let premises =
            native::encode_study_readiness_premises(pse_model::study::ReadinessPremises {
                generation,
                expected: premises,
            })
            .map_err(|_| InvalidScope)?;
        boundary::admit(&premises, 16 * 1024, 128).map_err(|_| InvalidScope)?;
        let encoded = Value::Array(
            vec![
                premises.clone(),
                action_bytes.clone().unwrap_or(Value::None),
                start.clone().unwrap_or(Value::None),
                next_facts.clone().unwrap_or(Value::None),
                outcome.clone().unwrap_or(Value::None),
            ]
            .into(),
        );
        boundary::admit(&encoded, 1024 * 1024, 128).map_err(|_| InvalidScope)?;
        output_bytes = output_bytes
            .checked_add(
                item.point.len()
                    + expected
                        .iter()
                        .map(|row| row.key.len() + 128)
                        .sum::<usize>()
                    + 512,
            )
            .ok_or(InvalidScope)?;
        if output_bytes > 1024 * 1024 {
            return Err(InvalidScope);
        }
        output.push(Decision {
            point: item.point,
            expected,
            pending: Unsigned(pending),
            eligible,
            premises,
            action: action_bytes,
            disposition,
            start,
            facts: next_facts,
            outcome,
        });
    }
    if required != rows.keys().cloned().collect() {
        return Err(InvalidScope);
    }
    Ok(output)
}

fn evaluate(page: Page) -> Reply {
    match decisions(page) {
        Ok(decisions) => {
            let reply = Reply {
                accepted: true,
                decisions,
            };
            if boundary::admit(&reply.clone().into_value(), 1024 * 1024, 128).is_err() {
                Reply {
                    accepted: false,
                    decisions: Vec::new(),
                }
            } else {
                reply
            }
        }
        Err(_) => Reply {
            accepted: false,
            decisions: Vec::new(),
        },
    }
}

#[cfg(target_family = "wasm")]
#[surrealism::surrealism]
// The SDK module macro recognizes unqualified nested attributes when composing namespaces.
mod study {
    use super::{AdmittedPage, Reply, evaluate};
    /// Compute readiness from one complete bounded native scope. Grants no permission.
    #[surrealism]
    fn readiness(page: AdmittedPage) -> Reply {
        evaluate(page.0)
    }
}

// These exports occur only in a separately hashed qualification artifact. The
// production packager verifies the exact readonly policy export.
#[cfg(all(target_family = "wasm", feature = "module-qualification"))]
#[surrealism::surrealism]
mod qualification {
    fn utc_micros() -> Result<surrealdb_types::Decimal, String> {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?;
        u64::try_from(time.as_micros())
            .map(surrealdb_types::Decimal::from)
            .map_err(|error| error.to_string())
    }

    /// Hold an actual controller across cooperative WASI clock polling.
    /// Returned times certify overlap; the guest retains no state or effects.
    #[surrealism]
    fn hold() -> Result<Vec<surrealdb_types::Decimal>, String> {
        let started = utc_micros()?;
        // The separately admitted qualification artifact can hold through a cold
        // runtime compilation. Production exports contain neither variant.
        let milliseconds = if cfg!(feature = "module-qualification-long-hold") {
            5_000
        } else {
            500
        };
        std::thread::sleep(std::time::Duration::from_millis(milliseconds));
        Ok(vec![started, utc_micros()?])
    }

    /// Trap after a sibling native write so rollback can be observed independently.
    #[surrealism]
    fn trap() -> bool {
        core::arch::wasm32::unreachable()
    }

    /// Deliberately consume an invocation until strict runtime interruption.
    #[surrealism]
    fn busy() -> bool {
        loop {
            std::hint::spin_loop();
        }
    }

    /// An allocation larger than the declared linear-memory ceiling must refuse.
    #[surrealism]
    fn memory_refused() -> bool {
        let mut bytes = Vec::<u8>::new();
        bytes.try_reserve_exact(96 * 1024 * 1024).is_err()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::generated::enums::StudyPointState;
    use pse_model::study::{
        Dependency, EffectState, OccurrenceKey, ScientificFacts, SeedNeed, StartPolicy,
    };
    fn page_object(value: &mut Value) -> &mut surrealdb_types::Object {
        match value {
            Value::Object(object) => Some(object),
            _ => None,
        }
        .expect("a typed page encodes as one native object")
    }
    fn row(key: u32, dependencies: Vec<Dependency>, settled: bool) -> Row {
        let policy = PointPolicy {
            key: OccurrenceKey(key),
            dependencies,
            seed_need: SeedNeed::NotNeeded,
            start: StartPolicy::Fresh,
            attempt_limit: 1,
        };
        let facts = PointFacts {
            key: policy.key,
            revision: 0,
            lifecycle: if settled {
                StudyPointState::Completed
            } else {
                StudyPointState::Pending
            },
            native_started: false,
            scientific: ScientificFacts {
                usable: settled,
                seed_permission: settled,
                candidate_use: None,
            },
            attempt_count: 0,
            retry_failure: None,
            effect: EffectState::Absent,
            seed: None,
            admitted_seed_need: None,
        };
        Row {
            key: format!("p{key}"),
            occurrence: Unsigned(u64::from(key)),
            revision: Unsigned(u64::MAX),
            policy: native::encode_admitted_study_point_policy(policy).unwrap(),
            facts: native::encode_study_point_facts(facts).unwrap(),
            assigned: false,
            settled,
            outcome: None,
        }
    }
    fn page(settled: bool) -> Page {
        Page {
            generation: Unsigned(u64::MAX),
            cancelled: false,
            mode: None,
            seed: None,
            rows: vec![
                row(0, vec![], settled),
                row(1, vec![Dependency::UsableResult(OccurrenceKey(0))], false),
            ],
            items: vec![Item {
                point: "p1".into(),
                predecessors: vec!["p0".into()],
            }],
        }
    }
    #[test]
    fn typed_readiness_uses_shared_policy_and_exact_original_revisions() {
        for settled in [false, true] {
            let input = AdmittedPage::from_value(page(settled).into_value()).unwrap();
            let reply = evaluate(input.0);
            assert!(reply.accepted);
            assert_eq!(reply.decisions[0].eligible, settled);
            assert_eq!(reply.decisions[0].pending.0, u64::from(!settled));
            let premises =
                native::decode_study_readiness_premises(reply.decisions[0].premises.clone())
                    .unwrap();
            assert_eq!(premises.generation, u64::MAX);
            assert_eq!(
                premises.expected,
                vec![("p1".into(), u64::MAX), ("p0".into(), u64::MAX)]
            );
        }
    }
    #[test]
    fn typed_progression_cancellation_preserves_semantic_outcome() {
        let mut input = page(true);
        input.mode = Some("prepare".into());
        input.cancelled = true;
        let reply = evaluate(input);
        assert!(reply.accepted);
        assert_eq!(reply.decisions[0].disposition.as_deref(), Some("settle"));
        let facts =
            native::decode_study_point_facts(reply.decisions[0].facts.clone().unwrap()).unwrap();
        let outcome =
            native::decode_study_point_outcome(reply.decisions[0].outcome.clone().unwrap())
                .unwrap();
        assert_eq!(facts.lifecycle, StudyPointState::Cancelled);
        assert_eq!(facts.key, outcome.key);
        assert!(outcome.attempts.is_empty());
    }
    #[test]
    fn typed_ingress_refuses_oversized_and_fractional_values_before_policy() {
        let mut value = page(true).into_value();
        let object = page_object(&mut value);
        object.insert(
            "generation",
            Value::Number(surrealdb_types::Number::Float(1.5)),
        );
        assert!(AdmittedPage::from_value(value).is_err());
        let mut value = page(true).into_value();
        let object = page_object(&mut value);
        object.insert(
            "generation",
            Value::Array(vec![Value::String(String::new()); 65537].into()),
        );
        assert!(AdmittedPage::from_value(value).is_err());
        let mut value = page(true).into_value();
        let object = page_object(&mut value);
        object.insert(
            "rows",
            Value::Array(
                (0..129)
                    .map(|_| Value::Object(Default::default()))
                    .collect::<Vec<_>>()
                    .into(),
            ),
        );
        assert!(AdmittedPage::from_value(value).is_err());
        let mut value = page(true).into_value();
        let object = page_object(&mut value);
        object.insert("mode", Value::String("x".repeat(1024 * 1024)));
        assert!(AdmittedPage::from_value(value).is_err());
    }
    #[test]
    fn typed_policy_refuses_incomplete_or_duplicate_scope_without_partial_decisions() {
        let mut input = page(true);
        input.rows.push(input.rows[0].clone());
        let reply = evaluate(input);
        assert!(!reply.accepted);
        assert!(reply.decisions.is_empty());
        let mut input = page(true);
        input.rows.remove(0);
        let reply = evaluate(input);
        assert!(!reply.accepted);
        assert!(reply.decisions.is_empty());
    }
    #[test]
    fn typed_claim_requires_explicit_resolved_seed_even_when_absent() {
        let mut input = page(true);
        input.mode = Some("claim".into());
        let mut value = input.into_value();
        page_object(&mut value).remove("seed");
        assert!(AdmittedPage::from_value(value.clone()).is_err());
        page_object(&mut value).insert("seed", Value::None);
        assert!(AdmittedPage::from_value(value).is_ok());
    }
    #[test]
    fn typed_unknown_effect_precedes_cancellation_and_seed_claim_uses_exact_artifact() {
        let mut input = page(true);
        input.mode = Some("prepare".into());
        input.cancelled = true;
        let mut facts = native::decode_study_point_facts(input.rows[1].facts.clone()).unwrap();
        facts.effect = EffectState::Unknown;
        input.rows[1].facts = native::encode_study_point_facts(facts).unwrap();
        let reply = evaluate(input);
        assert!(reply.accepted);
        let action =
            native::decode_study_point_action(reply.decisions[0].action.clone().unwrap()).unwrap();
        assert!(matches!(action.kind, ActionKind::Reconcile));

        use pse_model::study::{SeedAvailability, SeedFact, SeedRole};
        let seed = pse_model::generated::identities::SolutionId::from_bytes([7; 16]);
        let role = SeedRole::PrimalSolution;
        let mut point = row(0, vec![], false);
        let mut policy = native::decode_admitted_study_point_policy(point.policy).unwrap();
        policy.seed_need = SeedNeed::Required;
        policy.start = StartPolicy::Explicit { role, seed };
        point.policy = native::encode_admitted_study_point_policy(policy).unwrap();
        let input = Page {
            generation: Unsigned(1),
            cancelled: false,
            mode: Some("claim".into()),
            seed: Some(
                native::encode_study_seed_fact(SeedFact {
                    role,
                    availability: SeedAvailability::Compatible { seed },
                })
                .unwrap(),
            ),
            rows: vec![point],
            items: vec![Item {
                point: "p0".into(),
                predecessors: vec![],
            }],
        };
        let reply = evaluate(input);
        assert!(reply.accepted);
        assert_eq!(reply.decisions[0].disposition.as_deref(), Some("start"));
        let action =
            native::decode_study_point_action(reply.decisions[0].action.clone().unwrap()).unwrap();
        assert!(
            matches!(action.kind,ActionKind::Start(pse_model::study::StartProvenance::Explicit {seed:actual,..}) if actual==seed)
        );
    }
}
