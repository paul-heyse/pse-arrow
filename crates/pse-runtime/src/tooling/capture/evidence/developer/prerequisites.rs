// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Literal original prerequisite rows; no current readiness or preparation effects.
use super::*;
use pse_host::{
    envelope::CliInvocationId,
    fixtures::{Input, InputState, Ownership},
    test_workflow::execute::{MAX_PREREQUISITE_FACT_ROWS, Pending},
};
use serde::{
    Serialize,
    ser::{SerializeSeq, Serializer},
};

#[derive(Serialize)]
struct Row<'a> {
    member: u64,
    invocation: &'a CliInvocationId,
    report_sha256: &'a str,
    ordinal: u64,
    pointer: Option<String>,
    pending_known: bool,
    pending_entries: Option<usize>,
    pending_ordinal: Option<usize>,
    kind: Option<&'static str>,
    capability: Option<&'a str>,
    group: Option<usize>,
    fixture: Option<&'a str>,
    ownership: Option<Ownership>,
    input: Option<Input>,
    state: Option<&'static str>,
    code: Option<&'a str>,
}
fn visit<'a, E>(
    facts: &'a [InvocationFacts],
    map_error: impl Fn(CaptureError) -> E,
    mut consume: impl FnMut(Row<'a>) -> Result<(), E>,
) -> Result<(), E> {
    let mut ordinal = 0u64;
    let mut emit = |mut row: Row<'a>| {
        row.ordinal = ordinal;
        ordinal += 1;
        consume(row)
    };
    for (member, facts) in facts.iter().enumerate() {
        let base = || Row {
            member: member as u64,
            invocation: &facts.report.invocation,
            report_sha256: &facts.readback.report_sha256,
            ordinal: 0,
            pointer: facts.prerequisites.as_ref().map(|_| "/pending".into()),
            pending_known: facts.prerequisites.is_some(),
            pending_entries: facts.prerequisites.as_ref().map(Vec::len),
            pending_ordinal: None,
            kind: None,
            capability: None,
            group: None,
            fixture: None,
            ownership: None,
            input: None,
            state: None,
            code: None,
        };
        let Some(pending) = facts
            .prerequisites
            .as_ref()
            .filter(|entries| !entries.is_empty())
        else {
            emit(base())?;
            continue;
        };
        for (index, pending) in pending.iter().enumerate() {
            let pending_row = || {
                let mut row = base();
                row.pending_ordinal = Some(index);
                row.pointer = Some(format!("/pending/{index}"));
                row
            };
            match pending {
                Pending::Native { capabilities } => {
                    let native = || {
                        let mut row = pending_row();
                        row.kind = Some("native");
                        row
                    };
                    if capabilities.is_empty() {
                        emit(native())?;
                    }
                    for (capability, value) in capabilities.iter().enumerate() {
                        let mut row = native();
                        row.pointer = Some(format!("/pending/{index}/capabilities/{capability}"));
                        row.capability = Some(value);
                        emit(row)?;
                    }
                }
                Pending::Fixture(fixture) => {
                    let declared = || {
                        let mut row = pending_row();
                        row.kind = Some("fixture");
                        row.group = Some(fixture.requirement.group());
                        row.fixture = Some(&fixture.requirement.declaration().id);
                        row.ownership = Some(fixture.readiness.ownership);
                        row
                    };
                    if fixture.readiness.inputs.is_empty() {
                        emit(declared())?;
                    }
                    for (input, state) in &fixture.readiness.inputs {
                        let mut row = declared();
                        row.input = Some(*input);
                        // Unit enum serialization owns the exact original map key;
                        // an unsupported representation refuses instead of losing it.
                        let input_name = name(input).map_err(&map_error)?;
                        row.pointer =
                            Some(format!("/pending/{index}/readiness/inputs/{input_name}"));
                        (row.state, row.code) = match state {
                            InputState::Refused { code, .. } => (Some("refused"), code.as_deref()),
                            InputState::Selected { .. } => (Some("selected"), None),
                            InputState::Unprepared { code, .. } => {
                                (Some("unprepared"), Some(code.as_str()))
                            }
                            InputState::OwnerAdmissionRequired { .. } => {
                                (Some("owner-admission-required"), None)
                            }
                        };
                        emit(row)?;
                    }
                }
            }
        }
    }
    Ok(())
}

struct View<'a> {
    facts: &'a [InvocationFacts],
    rows: usize,
}
impl Serialize for View<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.rows))?;
        visit(self.facts, serde::ser::Error::custom, |row| {
            sequence.serialize_element(&row)
        })?;
        sequence.end()
    }
}

/// Project literal source states through the declared relation, retaining its lease.
pub fn project(
    facts: &[InvocationFacts],
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<RecordBatch, CaptureError> {
    cancel.checkpoint()?;
    let mut count = 0usize;
    visit(
        facts,
        |error| error,
        |_| {
            cancel.checkpoint()?;
            count = count
                .checked_add(1)
                .filter(|n| *n <= MAX_PREREQUISITE_FACT_ROWS)
                .ok_or_else(|| {
                    invalid(
                        "selected original prerequisite population exceeds its finite row limit",
                    )
                })?;
            Ok::<_, CaptureError>(())
        },
    )?;
    // Count the repeated original IDs, hashes, pointers and actual projected
    // values before retaining the row/string/Arrow populations. The existing
    // publication preflight owns them; source byte size alone is insufficient.
    let owner = reserve_document(&View { facts, rows: count }, pool)?;
    cancel.checkpoint()?;
    let mut rows = Vec::with_capacity(count);
    visit(
        facts,
        |error| error,
        |row| {
            cancel.checkpoint()?;
            rows.push(row);
            Ok::<_, CaptureError>(())
        },
    )?;
    let ids = rows
        .iter()
        .map(|row| row.invocation.to_string())
        .collect::<Vec<_>>();
    let inputs = rows
        .iter()
        .map(|row| row.input.as_ref().map(name).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    let ownership = rows
        .iter()
        .map(|row| row.ownership.as_ref().map(name).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    let counts = |values: Vec<Option<usize>>| -> Result<ArrayRef, CaptureError> {
        Ok(Arc::new(UInt64Array::from(
            values
                .into_iter()
                .map(|n| n.map(u64::try_from).transpose().map_err(invalid))
                .collect::<Result<Vec<_>, _>>()?,
        )))
    };
    publish(
        "runtime.developer_prerequisite_facts",
        vec![
            Arc::new(UInt64Array::from_iter_values(
                rows.iter().map(|row| row.member),
            )),
            strings(ids.iter().map(|value| Some(value.as_str()))),
            strings(rows.iter().map(|row| Some(row.report_sha256))),
            Arc::new(UInt64Array::from_iter_values(
                rows.iter().map(|row| row.ordinal),
            )),
            strings(rows.iter().map(|row| row.pointer.as_deref())),
            Arc::new(BooleanArray::from(
                rows.iter().map(|row| row.pending_known).collect::<Vec<_>>(),
            )),
            counts(rows.iter().map(|row| row.pending_entries).collect())?,
            counts(rows.iter().map(|row| row.pending_ordinal).collect())?,
            strings(rows.iter().map(|row| row.kind)),
            strings(rows.iter().map(|row| row.capability)),
            counts(rows.iter().map(|row| row.group).collect())?,
            strings(rows.iter().map(|row| row.fixture)),
            strings(ownership.iter().map(|value| value.as_deref())),
            strings(inputs.iter().map(|value| value.as_deref())),
            strings(rows.iter().map(|row| row.state)),
            strings(rows.iter().map(|row| row.code)),
        ],
        owner,
        cancel,
    )
}
