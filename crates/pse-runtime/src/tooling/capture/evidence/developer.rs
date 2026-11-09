// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed derived views over host-verified original observations, never native verdicts.
use super::*;
use pse_host::{developer_evidence::InvocationFacts, envelope::ErrorRecord};
use pse_host::test_workflow::execute::GroupFailureAccount;
use std::borrow::Cow;
mod prerequisites;
pub use prerequisites::project as prerequisites;

fn name(value: &impl serde::Serialize) -> Result<String, CaptureError> {
    match serde_json::to_value(value).map_err(invalid)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(invalid("native enum does not have a textual projection")),
    }
}

/// Project the exact source-owned original facts through their declared registry relation.
pub fn invocations(
    facts: &[InvocationFacts],
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<RecordBatch, CaptureError> {
    cancel.checkpoint()?;
    let owner = reserve_document(&facts, pool)?;
    let actions = facts
        .iter()
        .map(|f| name(&f.report.action))
        .collect::<Result<Vec<_>, _>>()?;
    let outcomes = facts
        .iter()
        .map(|f| name(&f.report.outcome))
        .collect::<Result<Vec<_>, _>>()?;
    let settlement = facts
        .iter()
        .map(|f| {
            f.readback
                .recorded_manager_settlement
                .as_ref()
                .map(name)
                .transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let selected = facts
        .iter()
        .map(|f| {
            f.report
                .selected_tests
                .map(u64::try_from)
                .transpose()
                .map_err(invalid)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let ids = facts
        .iter()
        .map(|f| f.report.invocation.to_string())
        .collect::<Vec<_>>();
    publish(
        "runtime.developer_invocation_facts",
        vec![
            Arc::new(UInt64Array::from_iter_values(
                (0..facts.len()).map(|n| n as u64),
            )),
            strings(ids.iter().map(|s| Some(s.as_str()))),
            strings(
                facts
                    .iter()
                    .map(|f| Some(f.readback.report_sha256.as_str())),
            ),
            Arc::new(UInt32Array::from_iter_values(
                facts.iter().map(|f| f.report.version),
            )),
            strings(actions.iter().map(|s| Some(s.as_str()))),
            strings(outcomes.iter().map(|s| Some(s.as_str()))),
            Arc::new(UInt64Array::from(selected)),
            Arc::new(BooleanArray::from(
                facts
                    .iter()
                    .map(|f| f.report.required_capture_complete)
                    .collect::<Vec<_>>(),
            )),
            strings(settlement.iter().map(|s| s.as_deref())),
            Arc::new(BooleanArray::from(
                facts
                    .iter()
                    .map(|f| f.readback.recorded_native_lifetime_pending)
                    .collect::<Vec<_>>(),
            )),
            // These portable roots do not record generation/clock coordinates. Reader time
            // and the currently compiled generation cannot restore those original facts.
            strings(facts.iter().map(|_| None)),
            strings(facts.iter().map(|_| None)),
        ],
        owner,
        cancel,
    )
}

struct Cause<'a> {
    member: u64,
    facts: &'a InvocationFacts,
    pointer: String,
    kind: &'static str,
    code: Option<&'a str>,
    class: Option<Cow<'a, str>>,
}

#[derive(Clone, Copy)]
enum CauseAccount {
    Operation,
    Dependency,
}

fn add<'a>(rows: &mut Vec<Cause<'a>>, row: Cause<'a>) -> Result<(), CaptureError> {
    if rows.len() == 4096 {
        return Err(invalid(
            "selected original diagnostic population exceeds 4096 rows",
        ));
    }
    rows.push(row);
    Ok(())
}
fn diagnostic<'a>(
    rows: &mut Vec<Cause<'a>>,
    member: u64,
    facts: &'a InvocationFacts,
    error: &'a ErrorRecord,
    pointer: String,
    account: CauseAccount,
    cancel: &CancellationToken,
) -> Result<(), CaptureError> {
    cancel.checkpoint()?;
    let (kind, source_kind) = match account {
        CauseAccount::Operation => ("diagnostic", "ordinary_source"),
        CauseAccount::Dependency => ("evidence_diagnostic", "evidence_ordinary_source"),
    };
    add(
        rows,
        Cause {
            member,
            facts,
            pointer: pointer.clone(),
            kind,
            code: error.code.as_deref(),
            class: error.class.as_deref().map(Cow::Borrowed),
        },
    )?;
    for (index, _) in error.causes.iter().enumerate() {
        add(
            rows,
            Cause {
                member,
                facts,
                pointer: format!("{pointer}/causes/{index}"),
                kind: source_kind,
                code: None,
                class: None,
            },
        )?;
    }
    for (index, child) in error.children.iter().enumerate() {
        diagnostic(
            rows,
            member,
            facts,
            child,
            format!("{pointer}/children/{index}"),
            account,
            cancel,
        )?;
    }
    if let Some(child) = &error.diagnostic_source {
        diagnostic(
            rows,
            member,
            facts,
            child,
            format!("{pointer}/diagnostic_source"),
            account,
            cancel,
        )?;
    }
    for (index, child) in error.related.iter().enumerate() {
        diagnostic(
            rows,
            member,
            facts,
            child,
            format!("{pointer}/related/{index}"),
            account,
            cancel,
        )?;
    }
    Ok(())
}

/// Preserve original cause multiplicity and exact committed-report pointers.
pub fn causes(
    facts: &[InvocationFacts],
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<RecordBatch, CaptureError> {
    cancel.checkpoint()?;
    let owner = reserve_document(&facts, pool)?;
    let mut rows = Vec::new();
    for (member, facts) in facts.iter().enumerate() {
        for (index, error) in facts.report.failures.iter().enumerate() {
            diagnostic(
                &mut rows,
                member as u64,
                facts,
                error,
                format!("/failures/{index}"),
                CauseAccount::Operation,
                cancel,
            )?;
        }
        for (index, evidence) in facts.report.evidence().map_err(invalid)?.iter().enumerate() {
            cancel.checkpoint()?;
            if let Some(error) = &evidence.failure {
                diagnostic(
                    &mut rows,
                    member as u64,
                    facts,
                    error,
                    format!("/evidence/{index}/failure"),
                    CauseAccount::Dependency,
                    cancel,
                )?;
            }
        }
        for (index, group) in facts.group_failures.iter().flatten().enumerate() {
            cancel.checkpoint()?;
            let mut record = |failure, kind, code, setup: Option<(usize, &pse_host::extraction::TargetSetupRefusal)>| {
                cancel.checkpoint()?;
                let mut pointer = format!("/observations/{index}/observation/failures/{failure}");
                let class = if let Some((marker, refusal)) = setup {
                    use std::fmt::Write as _;
                    write!(pointer, "/target_setup_refusals/{marker}").map_err(invalid)?;
                    Some(Cow::Owned(name(refusal)?))
                } else {
                    None
                };
                add(&mut rows, Cause {
                    member: member as u64,
                    facts,
                    pointer,
                    kind,
                    code,
                    class,
                })
            };
            match group {
                GroupFailureAccount::Check { observation, .. } => {
                    for (failure, diagnostic) in observation.failures.iter().enumerate() {
                        record(failure, "compiler_group_failure", diagnostic.code.as_deref(), None)?;
                    }
                }
                GroupFailureAccount::Test { observation, .. } => {
                    for (failure, test) in observation.failures.iter().enumerate() {
                        if test.target_setup_refusals.is_empty() {
                            record(failure, "test_group_failure", None, None)?;
                        }
                        for (marker, refusal) in test.target_setup_refusals.iter().enumerate() {
                            record(failure, "test_group_setup_refusal", None, Some((marker, refusal)))?;
                        }
                    }
                }
            }
        }
    }
    let ids = rows
        .iter()
        .map(|r| r.facts.report.invocation.to_string())
        .collect::<Vec<_>>();
    publish(
        "runtime.developer_diagnostic_causes",
        vec![
            Arc::new(UInt64Array::from_iter_values(rows.iter().map(|r| r.member))),
            strings(ids.iter().map(|s| Some(s.as_str()))),
            strings(
                rows.iter()
                    .map(|r| Some(r.facts.readback.report_sha256.as_str())),
            ),
            Arc::new(UInt64Array::from_iter_values(
                (0..rows.len()).map(|n| n as u64),
            )),
            strings(rows.iter().map(|r| Some(r.pointer.as_str()))),
            strings(rows.iter().map(|r| Some(r.kind))),
            strings(rows.iter().map(|r| r.code)),
            strings(rows.iter().map(|r| r.class.as_deref())),
        ],
        owner,
        cancel,
    )
}
