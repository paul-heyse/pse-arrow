// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Schema discovery shares investigation demands and records, never source-tree acquisition.
use super::{Bundle, SchemaError};
use crate::investigation::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};
use tokio_util::sync::CancellationToken;

/// One explicitly selected offline schema world.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Query {
    /// Complete descriptor artifact; relative paths are resolved at the command root.
    pub descriptor: NativePath,
    /// Case-insensitive name/comment substring.
    #[serde(default)]
    pub query: String,
    /// Selected schema declaration kinds.
    #[serde(default)]
    pub kinds: Vec<String>,
    /// Exact containing declaration or logical file.
    #[serde(default)]
    pub owner: Option<String>,
}
impl Query {
    /// Reject unsupported filters before acquisition rather than claiming empty matches.
    pub fn validate(&self) -> Result<(), Error> {
        if self.descriptor.to_path_buf().as_os_str().is_empty() {
            return Err(Error::Invalid(
                "schema query requires an explicit descriptor artifact".into(),
            ));
        }
        for kind in &self.kinds {
            if ![
                "file",
                "message",
                "field",
                "extension",
                "oneof",
                "enum",
                "enum_value",
                "service",
                "method",
            ]
            .contains(&kind.as_str())
            {
                return Err(Error::Invalid(format!("unsupported schema kind: {kind}")));
            }
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
struct Cursor {
    version: u32,
    digest: String,
    query: String,
    offset: usize,
}
/// Deliver schema results using the same stream/completion owner as other global providers.
pub fn run(
    root: &Path,
    request: &Request,
    query: &Query,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
) -> Result<Summary, Error> {
    if request.sources != SourceSelector::default() || request.scope.is_some() {
        return Err(Error::Unavailable("schema discovery has an explicit descriptor universe; repository source selectors do not select schema declarations".into()));
    }
    if request.answer.count_unit != CountUnit::Matches {
        return Err(Error::Unavailable("schema search counts schema declaration matches, not source files/lines or resolved Rust entities".into()));
    }
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    query.validate()?;
    if request.answer.kind == AnswerKind::Files {
        return Err(Error::Unavailable("schema search returns declaration matches; source-file witness demand is not a schema query".into()));
    }
    let bundle = Bundle::load(&root.join(query.descriptor.to_path_buf())).map_err(convert)?;
    let query_digest = Sha256::digest(serde_json::to_vec(query)?)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let offset = if let Some(cursor) = &request.answer.cursor {
        let cursor: Cursor = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(cursor)?)?;
        if cursor.version != 1 || cursor.digest != bundle.digest || cursor.query != query_digest {
            return Err(Error::Stale(
                "schema continuation artifact or query changed".into(),
            ));
        }
        cursor.offset
    } else {
        0
    };
    let needle = query.query.to_lowercase();
    let nodes = bundle
        .declarations()
        .iter()
        .filter(|node| {
            (node.name.to_lowercase().contains(&needle)
                || node.comments.to_lowercase().contains(&needle))
                && (query.kinds.is_empty() || query.kinds.contains(&node.kind))
                && query
                    .owner
                    .as_ref()
                    .is_none_or(|owner| node.parent.as_ref() == Some(owner) || &node.file == owner)
        })
        .collect::<Vec<_>>();
    if offset > nodes.len() {
        return Err(Error::Stale(
            "schema continuation exceeds selected declaration population".into(),
        ));
    }
    let source = format!("schema:{}", bundle.digest);
    emit(Event::Source(Box::new(SourceContext {
        id: source.clone(),
        path: bundle.path.clone().into(),
        origin: None,
        provider: "buffa-descriptor/0.9.2".into(),
        generation: Some(bundle.digest.clone()),
        searched_encoding: None,
        native: None,
    })))?;
    let limit = match request.answer.kind {
        AnswerKind::Count => usize::MAX,
        AnswerKind::Exists => 1,
        AnswerKind::Matches | AnswerKind::Files => request.answer.limit.unwrap_or(usize::MAX),
    };
    let total = nodes.len();
    let mut summary = Summary {
        examined_files: bundle.set.file.len() as u64,
        examined_bytes: bundle.original.len() as u64,
        matches: total as u64,
        count: Count::Exact(total as u64),
        ..Summary::default()
    };
    for node in nodes.iter().skip(offset).take(limit) {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if request.answer.kind == AnswerKind::Count {
            continue;
        }
        emit(Event::Result(Box::new(SearchResult {
            source: source.clone(),
            kind: node.kind.clone(),
            name: Some(node.name.clone()),
            range: None,
            context: node.comments.clone(),
            bytes: None,
            reference: Some(ResultReference::new(Locator::Schema {
                descriptor: bundle.path.clone().into(),
                descriptor_sha256: bundle.digest.clone(),
                declaration: node.address.clone(),
            })),
            captures: BTreeMap::new(),
            entity: None,
            candidate_reference: None,
            refinement: None,
        })))?;
        summary.emitted += 1;
    }
    if request.answer.kind != AnswerKind::Count
        && offset.saturating_add(summary.emitted as usize) < total
    {
        summary.complete = false;
        summary.stop = StopReason::Limit;
        if request.answer.order == ResultOrder::Stable {
            summary.next_cursor = Some(URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Cursor {
                version: 1,
                digest: bundle.digest,
                query: query_digest,
                offset: offset + summary.emitted as usize,
            })?));
        }
    }
    Ok(summary)
}
fn convert(error: SchemaError) -> Error {
    match error {
        SchemaError::Io(error) => Error::Io(error),
        SchemaError::Json(error) => Error::Json(error),
        other => Error::Invalid(other.to_string()),
    }
}
/// Guard the acquired descriptor before expanding an addressed schema declaration.
pub fn inspect(root: &Path, locator: &Locator) -> Result<super::Report, Error> {
    let Locator::Schema {
        descriptor,
        descriptor_sha256,
        declaration,
    } = locator
    else {
        return Err(Error::Invalid(
            "schema inspection requires a schema address".into(),
        ));
    };
    let bundle = Bundle::load(&descriptor.to_path_buf()).map_err(convert)?;
    if &bundle.digest != descriptor_sha256 {
        return Err(Error::Stale(
            "descriptor artifact changed since the schema result was acquired".into(),
        ));
    }
    bundle
        .inspect(Some(declaration))
        .and_then(|report| report.with_associations(root, &bundle))
        .map_err(convert)
}

#[cfg(test)]
mod tests {
    use super::*;
    const CORPUS: &[u8] = include_bytes!("../generated/plan61/descriptor.bin");
    fn selected(path: &Path) -> (Request, Query) {
        let query = Query {
            descriptor: path.to_path_buf().into(),
            query: "Payload".into(),
            kinds: vec![],
            owner: None,
        };
        let request = Request {
            predicate: Predicate::Schema(query.clone()),
            answer: AnswerDemand {
                limit: Some(1),
                order: ResultOrder::Stable,
                ..Default::default()
            },
            ..Default::default()
        };
        (request, query)
    }
    #[test]
    fn schema_cursor_binds_query_and_original_descriptor_and_expansion() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("schema.bin");
        std::fs::write(&path, CORPUS).unwrap();
        let (mut request, query) = selected(&path);
        let mut events = Vec::new();
        let first = run(
            directory.path(),
            &request,
            &query,
            &CancellationToken::new(),
            &mut |event| {
                events.push(event);
                Ok(())
            },
        )
        .unwrap();
        let reference = events
            .iter()
            .find_map(|event| {
                if let Event::Result(row) = event {
                    row.reference.as_ref()
                } else {
                    None
                }
            })
            .unwrap();
        reference.validate().unwrap();
        assert!(inspect(directory.path(), &reference.locator).is_ok());
        request.answer.cursor = first.next_cursor;
        assert!(request.answer.cursor.is_some());
        let second = run(
            directory.path(),
            &request,
            &query,
            &CancellationToken::new(),
            &mut |_| Ok(()),
        )
        .unwrap();
        assert_eq!(second.emitted, 1);
        let mut changed = query.clone();
        changed.query = "different".into();
        assert!(matches!(
            run(
                directory.path(),
                &request,
                &changed,
                &CancellationToken::new(),
                &mut |_| Ok(())
            ),
            Err(Error::Stale(_))
        ));
        let mut changed_bytes = CORPUS.to_vec();
        changed_bytes.extend([0xa0, 0x06, 1]);
        std::fs::write(&path, changed_bytes).unwrap();
        assert!(matches!(
            run(
                directory.path(),
                &request,
                &query,
                &CancellationToken::new(),
                &mut |_| Ok(())
            ),
            Err(Error::Stale(_))
        ));
        assert!(matches!(
            inspect(directory.path(), &reference.locator),
            Err(Error::Stale(_))
        ));
    }
    #[test]
    fn schema_count_filters_and_source_scope_refusal() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("schema.bin");
        std::fs::write(&path, CORPUS).unwrap();
        let (mut request, mut query) = selected(&path);
        request.answer.kind = AnswerKind::Count;
        request.answer.limit = None;
        query.kinds = vec!["message".into()];
        let count = run(
            directory.path(),
            &request,
            &query,
            &CancellationToken::new(),
            &mut |_| Ok(()),
        )
        .unwrap();
        assert_eq!(count.matches, 1);
        assert_eq!(count.emitted, 0);
        assert!(count.complete);
        request.sources.paths.push("not-a-schema".into());
        assert!(matches!(
            run(
                directory.path(),
                &request,
                &query,
                &CancellationToken::new(),
                &mut |_| Ok(())
            ),
            Err(Error::Unavailable(_))
        ));
        query.kinds = vec!["invented_kind".into()];
        assert!(query.validate().is_err());
    }
}
