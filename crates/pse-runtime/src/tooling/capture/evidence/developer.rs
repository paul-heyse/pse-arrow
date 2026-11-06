// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed derived views over host-verified original observations, never native verdicts.
use super::*;
use pse_host::{developer_evidence::InvocationFacts, envelope::ErrorRecord};

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
    cancel.checkpoint().map_err(invalid)?;
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
    class: Option<&'a str>,
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
    cancel: &CancellationToken,
) -> Result<(), CaptureError> {
    cancel.checkpoint().map_err(invalid)?;
    add(
        rows,
        Cause {
            member,
            facts,
            pointer: pointer.clone(),
            kind: "diagnostic",
            code: error.code.as_deref(),
            class: error.class.as_deref(),
        },
    )?;
    for (index, _) in error.causes.iter().enumerate() {
        add(
            rows,
            Cause {
                member,
                facts,
                pointer: format!("{pointer}/causes/{index}"),
                kind: "ordinary_source",
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
    cancel.checkpoint().map_err(invalid)?;
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
                cancel,
            )?;
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
            strings(rows.iter().map(|r| r.class)),
        ],
        owner,
        cancel,
    )
}
