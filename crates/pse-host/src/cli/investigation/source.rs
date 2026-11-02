// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact written-source investigation delegates acquisition, parsing and coordinates.
use crate::{
    HostError,
    envelope::{DetailView, OutputOptions},
};
use clap::{Args, ValueEnum};
use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Args)]
pub(in crate::cli::investigation) struct Outline {
    /// Construct navigation, interpreted facts, or facts with decoded character mappings.
    #[arg(long, value_enum, default_value = "navigation")]
    detail: crate::source_read::SourceDetail,
    #[arg(required=true,num_args=1..)]
    files: Vec<PathBuf>,
    #[arg(long)]
    cursor: Option<String>,
    #[command(flatten)]
    parsing: crate::cli::SourceParsing,
}
#[derive(Debug, Args)]
pub(in crate::cli::investigation) struct Declaration {
    /// Interpretation demand; declarations default to facts, exact ranges to navigation.
    #[arg(long, value_enum)]
    detail: Option<crate::source_read::SourceDetail>,
    source: OsString,
    #[arg(long)]
    item: Vec<String>,
    #[arg(long)]
    at_byte: Option<usize>,
    #[arg(long, requires="at_column", conflicts_with_all=["at_byte","range_start"])]
    at_line: Option<usize>,
    #[arg(long, requires = "at_line")]
    at_column: Option<usize>,
    #[arg(long, requires="range_end", conflicts_with_all=["at_byte","at_line","item"])]
    range_start: Option<usize>,
    #[arg(long, requires = "range_start")]
    range_end: Option<usize>,
    #[command(flatten)]
    parsing: crate::cli::SourceParsing,
}
#[derive(Debug, Args)]
pub(in crate::cli::investigation) struct Origin {
    source: PathBuf,
    #[arg(long)]
    range_start: usize,
    #[arg(long)]
    range_end: usize,
    #[arg(long)]
    if_source: Option<String>,
    #[command(flatten)]
    parsing: crate::cli::SourceParsing,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum EntityView {
    Lifetime,
    Compatibility,
    Visibility,
    FormatOrigin,
    DeriveOrigin,
    Explain,
    /// Inspect fallible layout with native type/target context.
    Layout,
    Symbols,
    Refs,
    Calls,
    Implementations,
    Macro,
    Tests,
    Candidates,
    Candidate,
    Hints,
    Hint,
    Predicate,
    Assists,
}
#[derive(Debug, Args)]
pub(in crate::cli::investigation) struct Entity {
    #[command(flatten)]
    subject: crate::cli::code::semantic::Subject,
    #[arg(long, value_enum, default_value = "explain")]
    view: EntityView,
    #[arg(long)]
    select: Option<usize>,
    #[arg(long)]
    text: Option<String>,
    /// Passively associate an original type-sizes acquisition; requires --view layout.
    #[arg(long)]
    compiler_layout: Option<String>,
    /// Original compiler artifact reference accompanying a lifetime inquiry.
    #[arg(long)]
    compiler_evidence: Option<String>,
    /// Original work invocation accompanying the inquiry; no execution association is inferred.
    #[arg(long)]
    work_result: Option<String>,
    /// Original guarded position for a native compatibility or visibility comparison.
    #[arg(long)]
    against_result: Option<String>,
    #[arg(long, value_enum)]
    operation: Option<crate::semantic::CompatibilityOperation>,
}
pub(super) fn entity(
    options: Entity,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    use crate::semantic::Kind;
    let root = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
    if (options.against_result.is_some()
        && !matches!(
            options.view,
            EntityView::Compatibility | EntityView::Visibility
        ))
        || (options.operation.is_some() && !matches!(options.view, EntityView::Compatibility))
    {
        return Err(HostError::Ownership(
            "against-result requires compatibility/visibility; operation requires compatibility"
                .into(),
        ));
    }
    if matches!(options.view, EntityView::Lifetime) {
        if options.compiler_layout.is_some() {
            return Err(HostError::Ownership(
                "compiler-layout requires layout view".into(),
            ));
        }
        let locator = options.subject.lifetime_locator(&root)?;
        let report = crate::cli::investigation::lifetime_inspection(
            &root,
            &locator,
            options.compiler_evidence.as_deref(),
            options.work_result.as_deref(),
        )?;
        let partial = matches!(
            report.correspondence,
            crate::investigation::lifetime::Correspondence::Unjoined { .. }
        );
        let mut response = crate::envelope::Envelope::response(
            "inspect entity",
            "qualified lifetime observations from original evidence",
            report,
        );
        if partial {
            response.observation.completeness = crate::envelope::Completeness::Partial;
        }
        return Ok(output.emit_page(response, DetailView::default())?);
    }
    if options.compiler_evidence.is_some() || options.work_result.is_some() {
        return Err(HostError::Ownership(
            "compiler-evidence/work-result require lifetime view".into(),
        ));
    }
    let index = || {
        options.select.ok_or_else(||HostError::Ownership("lazy candidate/hint inspection requires --select and the original source/generation/fingerprint guards".into()))
    };
    let kind = match options.view {
        EntityView::Explain => Kind::Explain,
        EntityView::Lifetime => Kind::Body,
        EntityView::FormatOrigin => Kind::Origins {
            kind: crate::semantic::OriginKind::Format,
        },
        EntityView::DeriveOrigin => Kind::Origins {
            kind: crate::semantic::OriginKind::Derive,
        },
        EntityView::Compatibility | EntityView::Visibility => {
            let primary = options.subject.lifetime_locator(&root)?;
            let reference = crate::investigation::ResultReference::decode(
                options.against_result.as_deref().ok_or_else(|| {
                    HostError::Ownership("native comparison requires --against-result".into())
                })?,
            )?;
            let (against, generation) = crate::cli::runtime()?.block_on(
                crate::investigation::rust::guarded_operand(&root, &primary, &reference),
            )?;
            if matches!(options.view, EntityView::Visibility) {
                Kind::Visibility {
                    destination: against,
                    destination_generation: generation,
                }
            } else {
                Kind::Compatibility {
                    against,
                    against_generation: generation,
                    operation: options.operation.ok_or_else(|| {
                        HostError::Ownership("native compatibility requires --operation".into())
                    })?,
                }
            }
        }
        EntityView::Layout => Kind::Layout,
        EntityView::Symbols => Kind::Symbols,
        EntityView::Refs => Kind::Refs,
        EntityView::Calls => Kind::Calls,
        EntityView::Implementations => Kind::Implementations,
        EntityView::Macro => Kind::Macro,
        EntityView::Tests => Kind::Tests,
        EntityView::Candidates => Kind::Candidates,
        EntityView::Candidate => Kind::Candidate { index: index()? },
        EntityView::Hints => Kind::Hints,
        EntityView::Hint => Kind::Hint { index: index()? },
        EntityView::Predicate => Kind::Predicate {
            text: options.text.ok_or_else(|| {
                HostError::Ownership("native predicate inspection requires --text".into())
            })?,
        },
        EntityView::Assists => Kind::Assists,
    };
    crate::cli::code::semantic::query(
        kind,
        options.subject,
        selected,
        output,
        options.compiler_layout.as_deref(),
    )
}

pub(super) fn outline(
    options: Outline,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    if options.cursor.is_some() && options.files.len() != 1 {
        return Err(HostError::Ownership(
            "an outline continuation selects exactly one source".into(),
        ));
    }
    let mut files = options
        .files
        .into_iter()
        .map(|file| {
            selected.map_or_else(
                || Ok(file.clone()),
                |root| std::path::absolute(root.join(&file)),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    if files.len() > 1 {
        if options.parsing.index || options.parsing.revision.is_some() {
            return Err(HostError::Ownership(
                "select one index/revision source per addressed outline".into(),
            ));
        }
        let mut response=crate::cli::runtime()?.block_on(async {
            let mut task=crate::source_command::batch::start_page(files,options.parsing.language,options.parsing.edition,tokio_util::sync::CancellationToken::new(),output.presentation.budget,options.detail);
            tokio::select! {
                response=task.settle()=>response.map_err(HostError::from),
                signal=tokio::signal::ctrl_c()=>{task.cancel();let response=task.settle().await?;signal?;Ok(response)},
            }
        })?;
        response.command = "inspect outline batch".into();
        let omitted_records = response
            .payload
            .records
            .iter()
            .map(|record| match record {
                crate::source_command::batch::Record::Read { reading } => {
                    reading.page.omitted_records
                }
                _ => 0,
            })
            .sum();
        return Ok(output.emit_page(
            response,
            DetailView {
                budget: output.presentation.budget,
                omitted_records,
                truncated: omitted_records > 0,
                ..Default::default()
            },
        )?);
    }
    let file = files.pop().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "select at least one source file",
        )
    })?;
    let request = crate::cli::source_request(selected, file, &options.parsing)?;
    let mut response = crate::source_command::outline_page(
        &request,
        options.parsing.language,
        options.parsing.edition,
        crate::source_read::PageRequest {
            detail: options.detail,
            max_records: output.presentation.budget,
            cursor: options.cursor,
        },
    )?;
    response.command = "inspect outline".into();
    let detail = DetailView {
        budget: output.presentation.budget,
        omitted_records: response.payload.page.omitted_records,
        truncated: response.payload.page.omitted_records > 0,
        ..Default::default()
    };
    Ok(output.emit_page(response, detail)?)
}

pub(super) fn declaration(
    options: Declaration,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let positional =
        options.at_byte.is_some() || options.at_line.is_some() || options.range_start.is_some();
    let (file, items) = if positional
        && options.item.is_empty()
        && !options
            .source
            .as_encoded_bytes()
            .windows(2)
            .any(|part| part == b"::")
    {
        (PathBuf::from(options.source), Vec::new())
    } else {
        crate::source_command::selector(&options.source, options.item)?
    };
    let file = selected.map_or_else(
        || Ok(file.clone()),
        |root| std::path::absolute(root.join(&file)),
    )?;
    let position = if let Some(start) = options.range_start {
        crate::source_read::PositionSelector::Range {
            start,
            end: options
                .range_end
                .ok_or_else(|| HostError::Ownership("range end required".into()))?,
        }
    } else if let Some(line) = options.at_line {
        crate::source_read::PositionSelector::LineColumn {
            line,
            column: options
                .at_column
                .ok_or_else(|| HostError::Ownership("column required".into()))?,
        }
    } else if let Some(offset) = options.at_byte {
        crate::source_read::PositionSelector::Byte { offset }
    } else {
        crate::source_read::PositionSelector::Path
    };
    let mut response = crate::source_command::show_addressed(
        &crate::cli::source_request(selected, file, &options.parsing)?,
        &items,
        &position,
        options.parsing.language,
        options.parsing.edition,
        options.detail.unwrap_or(if options.range_start.is_some() {
            crate::source_read::SourceDetail::Navigation
        } else {
            crate::source_read::SourceDetail::Facts
        }),
    )?;
    response.command = "inspect declaration".into();
    crate::cli::source_emit(output, response, crate::source_command::selection_text)
}

pub(super) fn origin(
    options: Origin,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
    let file = if options.source.is_absolute() {
        options.source
    } else {
        root.join(options.source)
    };
    let mut response = crate::source_command::generated_origin(
        &root,
        &crate::cli::source_request(Some(&root), file, &options.parsing)?,
        options.range_start..options.range_end,
        options.if_source.as_deref(),
    )?;
    response.command = "inspect origin".into();
    Ok(output.emit_page(response, DetailView::default())?)
}

#[cfg(test)]
mod detail_tests {
    use super::*;
    use clap::FromArgMatches as _;

    #[test]
    fn parser_owns_detail_defaults_and_refuses_unknown_or_conflicting_selection() {
        let command = Outline::augment_args(clap::Command::new("outline"));
        let matches = command
            .clone()
            .try_get_matches_from(["outline", "file.rs"])
            .unwrap();
        assert_eq!(
            Outline::from_arg_matches(&matches).unwrap().detail,
            crate::source_read::SourceDetail::Navigation
        );
        let matches = command
            .clone()
            .try_get_matches_from(["outline", "file.rs", "--detail", "mapped"])
            .unwrap();
        assert_eq!(
            Outline::from_arg_matches(&matches).unwrap().detail,
            crate::source_read::SourceDetail::Mapped
        );
        assert!(
            command
                .try_get_matches_from(["outline", "file.rs", "--detail", "unknown"])
                .is_err()
        );
        let command = Declaration::augment_args(clap::Command::new("declaration"));
        let matches = command
            .clone()
            .try_get_matches_from(["declaration", "file.rs::selected"])
            .unwrap();
        assert!(
            Declaration::from_arg_matches(&matches)
                .unwrap()
                .detail
                .is_none()
        );
        assert!(
            command
                .try_get_matches_from([
                    "declaration",
                    "file.rs",
                    "--range-start",
                    "0",
                    "--range-end",
                    "1",
                    "--at-byte",
                    "0",
                    "--detail",
                    "facts"
                ])
                .is_err()
        );
    }
}
