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
    #[arg(required=true,num_args=1..)]
    files: Vec<PathBuf>,
    #[arg(long)]
    cursor: Option<String>,
    #[command(flatten)]
    parsing: crate::cli::SourceParsing,
}
#[derive(Debug, Args)]
pub(in crate::cli::investigation) struct Declaration {
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
}
pub(super) fn entity(
    options: Entity,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    use crate::semantic::Kind;
    let index = || {
        options.select.ok_or_else(||HostError::Ownership("lazy candidate/hint inspection requires --select and the original source/generation/fingerprint guards".into()))
    };
    let kind = match options.view {
        EntityView::Explain => Kind::Explain,
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
            let mut task=crate::source_command::batch::start_page(files,options.parsing.language,options.parsing.edition,tokio_util::sync::CancellationToken::new(),output.presentation.budget);
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
