// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded orientation from finalized contained members, with no live source lookup.
use super::*;
use crate::tooling::review_analysis::render::{ReportOrientation, ReportSection};

fn bounded_json(value: &impl Serialize) -> String {
    let mut writer = pse_ids::BoundedSink::new(Vec::new(), 4096);
    if serde_json::to_writer(&mut writer, value).is_ok() {
        String::from_utf8_lossy(writer.inner()).into_owned()
    } else {
        format!(
            "{} … [remaining retained value omitted from presentation]",
            String::from_utf8_lossy(writer.inner())
        )
    }
}
pub(super) fn orientation(
    directory: &File,
    index: &PackIndex,
) -> Result<ReportOrientation, PackError> {
    let mut out = ReportOrientation::default();
    out.sections.push(ReportSection::text("Selected offline scope",None,vec!["The local index owns membership, acknowledged byte boundaries, publication and explanatory limitations. Raw capture/profile bytes remain selected members; decoded names do not rank CPU hotspots.".into()]));
    for member in index
        .members
        .iter()
        .filter(|m| m.selection.role != PackMemberRole::Report)
        .take(256)
    {
        let mut description = format!(
            "{:?}; readable bytes {}; finalized {}",
            member.availability, member.readable_bytes, member.finalized
        );
        if let Some(result) = &member.selection.result {
            description.push_str(&format!("; original run {}, attempt {}, relation {}@{}, rows {}..{}, terminal {}, usability {:?}",result.run,result.attempt,result.relation,result.schema_version,result.row_start,result.row_end,result.terminal_class,result.usability));
        }
        if let Some(scope) = &member.selection.capture_scope {
            description.push_str(&format!(
                "; capture {}",
                scope.chars().take(2048).collect::<String>()
            ));
        }
        let link = (member.availability == PackAvailability::Available)
            .then(|| (member.selection.id.clone(), String::new()));
        out.sections.push(ReportSection::text(
            &member.selection.id,
            link,
            vec![description],
        ));
    }
    if index.members.len() > 256 {
        out.sections.push(ReportSection::text(
            "Membership presentation omitted",
            None,
            vec![format!(
                "{} additional selected members remain in the index",
                index.members.len() - 256
            )],
        ));
    }
    if !index.missing_context.is_empty() {
        out.sections.push(ReportSection::text(
            "Missing explanatory premises",
            None,
            index.missing_context.iter().take(32).cloned().collect(),
        ));
    }
    if let Some(member) = index
        .members
        .iter()
        .find(|m| m.selection.id == "frozen" && m.availability == PackAvailability::Available)
    {
        let frozen: FrozenEvidenceSet = decode(
            open_contained(directory, &member.relative_path)?.take(member.readable_bytes),
            index.selection.maximum_document_bytes,
        )?;
        for (i, context) in frozen.contexts.iter().take(20).enumerate() {
            let mut paragraphs = vec![
                format!(
                    "Original operation {:?}; run {:?}; attempt {:?}. Model sources: {}. Retained facts: {}. Explicit unavailable fields: {}.",
                    context.operation,
                    context.run_id,
                    context.attempt_id,
                    context.sources.len(),
                    context.facts.len(),
                    context.unavailable.len()
                ),
                format!(
                    "Effective settings: {}",
                    bounded_json(&context.solver_settings)
                ),
            ];
            for source in context.sources.iter().take(4) {
                paragraphs.push(format!("Model {} physical {} interpretation {}; authored objects {}; physical source streams {}",source.identity,source.physical_identity,source.interpretation,source.objects.len(),source.physical_sources.len()));
            }
            for source in context.sources.iter().take(4) {
                for object in source.objects.iter().take(20) {
                    paragraphs.push(format!(
                        "Authored {} / {} ({}, version {}, logical {}), references {:?}",
                        object.scope,
                        object.name,
                        object.kind,
                        object.version,
                        object.logical,
                        object.references.iter().take(8).collect::<Vec<_>>()
                    ));
                }
                if source.objects.len() > 20 {
                    paragraphs.push(format!(
                        "{} authored objects omitted from presentation",
                        source.objects.len() - 20
                    ));
                }
            }
            out.sections.push(ReportSection::text(
                format!("Original model and effective settings {i}"),
                Some(("frozen".into(), format!("/contexts/{i}"))),
                paragraphs,
            ));
            for (j, fact) in context.facts.iter().take(20).enumerate() {
                let interpretation = if fact.payload.len() <= index.selection.maximum_document_bytes
                {
                    serde_json::from_slice::<serde_json::Value>(&fact.payload).map(|value|bounded_json(&value)).unwrap_or_else(|_|"Retained fact is not supported JSON; original payload remains authoritative".into())
                } else {
                    "Fact payload exceeds presentation allowance; omitted".into()
                };
                out.sections.push(ReportSection::text(
                    format!(
                        "Retained input / starts / context {} ({})",
                        fact.path, fact.codec
                    ),
                    Some(("frozen".into(), format!("/contexts/{i}/facts/{j}/payload"))),
                    vec![interpretation],
                ));
            }
            if context.facts.len() > 20 {
                out.sections.push(ReportSection::text(
                    "Retained context facts omitted",
                    None,
                    vec![format!(
                        "{} additional original facts omitted from presentation",
                        context.facts.len() - 20
                    )],
                ));
            }
        }
        if frozen.contexts.len() > 20 {
            out.sections.push(ReportSection::text(
                "Contexts omitted",
                None,
                vec![format!(
                    "{} additional contexts remain in frozen",
                    frozen.contexts.len() - 20
                )],
            ));
        }
        for (i, capture) in frozen.captures.iter().take(20).enumerate() {
            let mut paragraphs = vec![format!(
                "{:?}; status {:?}; observed {:?}; open intervals {}; symbols checked {}, resolved {}, unresolved {}; loss {:?}. Raw samples are not exclusive operation CPU or complete offline symbolication. Names below are retained decoded orientation without hotspot ranking.",
                capture.kind,
                capture.status,
                capture.observed_duration,
                capture.open_intervals,
                capture.symbols.checked,
                capture.symbols.resolved_frames,
                capture.symbols.unresolved_frames,
                capture.loss
            )];
            paragraphs.extend(
                capture
                    .symbols
                    .names
                    .iter()
                    .take(20)
                    .map(|name| name.chars().take(1024).collect()),
            );
            if capture.symbols.names.len() > 20 {
                paragraphs.push(format!(
                    "{} additional decoded symbol names omitted",
                    capture.symbols.names.len() - 20
                ));
            }
            out.sections.push(ReportSection::text(
                format!("Raw profile and symbol limits {i}"),
                Some(("frozen".into(), format!("/captures/{i}"))),
                paragraphs,
            ));
        }
    }
    for member in index
        .members
        .iter()
        .filter(|m| {
            m.availability == PackAvailability::Available
                && m.selection.format == PackFormat::ArrowIpcStream
        })
        .take(20)
    {
        let mut section = ReportSection::text(
            format!("Selected table {}", member.selection.id),
            Some((member.selection.id.clone(), String::new())),
            Vec::new(),
        );
        // Decode only a bounded finalized local payload. Never open a source path or
        // perform a second query to obtain a preview of a large selected table.
        if member.readable_bytes > index.selection.maximum_document_bytes as u64 {
            section.paragraphs.push("Table preview omitted: finalized IPC exceeds the report decode allowance. Indexed raw selected table remains available.".into());
        } else {
            let file = open_contained(directory, &member.relative_path)?;
            let limits = pse_columnar::ipc_stream::Limits {
                message_bytes: index.selection.maximum_document_bytes,
                metadata_bytes: index.selection.maximum_document_bytes,
            };
            let pool = pse_columnar::ipc_stream::offline_pool(limits)
                .map_err(|e| invalid(e.to_string()))?;
            let mut stream = pse_columnar::ipc_stream::StreamReader::try_new(
                file,
                member.readable_bytes,
                limits,
                pool,
                CancellationToken::new(),
            )
            .map_err(|e| invalid(e.to_string()))?;
            section.columns = stream
                .schema()
                .fields()
                .iter()
                .take(32)
                .map(|field| field.name().clone())
                .collect();
            let mut more = false;
            while let Some(decoded) = stream.next_batch().map_err(|e| invalid(e.to_string()))? {
                let batch = &decoded.batch;
                for row in 0..batch.num_rows() {
                    if section.rows.len() == 20 {
                        more = true;
                        break;
                    }
                    let mut values = Vec::new();
                    for column in batch.columns().iter().take(32) {
                        let value = datafusion::arrow::util::display::array_value_to_string(
                            column.as_ref(),
                            row,
                        )
                        .map_err(|e| invalid(e.to_string()))?;
                        let mut value = value.chars().take(2048).collect::<String>();
                        if value.len() >= 2048 {
                            value.push_str(" [cell remainder omitted]");
                        }
                        values.push(value);
                    }
                    section.rows.push(values);
                }
                if section.rows.len() == 20 {
                    more = true;
                    break;
                }
            }
            section.paragraphs.push(format!(
                "First {} selected rows in retained order; native integer text remains exact. {}",
                section.rows.len(),
                if more {
                    "Further rows may remain and are omitted from presentation."
                } else {
                    "No further selected rows."
                }
            ));
            if stream.schema().fields().len() > 32 {
                section
                    .paragraphs
                    .push("Columns after the first 32 are omitted from presentation.".into());
            }
        }
        out.sections.push(section);
    }
    Ok(out)
}
