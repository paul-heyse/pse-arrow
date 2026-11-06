// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Schema discovery shares investigation demands and records, never source-tree acquisition.
use super::{Bundle, Declaration, SchemaError};
use crate::investigation::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
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
    let limit = match request.answer.kind {
        AnswerKind::Count => 0,
        AnswerKind::Exists => 1,
        AnswerKind::Matches | AnswerKind::Files => request.answer.limit.unwrap_or(usize::MAX),
    };
    let (total, nodes) = selected_page(&bundle, query, offset, limit, token)?;
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
    let mut summary = Summary {
        examined_files: bundle.set.file.len() as u64,
        examined_bytes: bundle.original.len() as u64,
        matches: total as u64,
        count: Count::Exact(total as u64),
        ..Summary::default()
    };
    for node in &nodes {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        emit(Event::Result(Box::new(SearchResult {
            source_relationship: None,
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
/// Scan the complete match universe without rendering its option/details JSON. Only a
/// stable requested prefix of address strings is retained, then only the page is rendered.
fn selected_page(
    bundle: &Bundle,
    query: &Query,
    offset: usize,
    limit: usize,
    token: &CancellationToken,
) -> Result<(usize, Vec<Declaration>), Error> {
    let needle = query.query.to_lowercase();
    let mut total = 0;
    let mut addresses = BTreeSet::new();
    let retained = if limit == 0 {
        0
    } else {
        offset.saturating_add(limit)
    };
    bundle
        .select(None, &mut |file, path, node| {
            if token.is_cancelled() {
                return Err(SchemaError::Cancelled);
            }
            if (!query.kinds.is_empty() && !query.kinds.iter().any(|kind| kind == node.kind))
                || query.owner.as_ref().is_some_and(|owner| {
                    node.parent.as_ref() != Some(owner) && file.name.as_ref() != Some(owner)
                })
            {
                return Ok(false);
            }
            let matches = node.name.to_lowercase().contains(&needle)
                || Bundle::location(file, path).is_some_and(|location| {
                    [
                        location.leading_comments.as_deref().unwrap_or(""),
                        location.trailing_comments.as_deref().unwrap_or(""),
                    ]
                    .join("\n")
                    .to_lowercase()
                    .contains(&needle)
                });
            if matches {
                total += 1;
                if retained != 0 {
                    addresses.insert(node.address.clone());
                    if addresses.len() > retained {
                        addresses.pop_last();
                    }
                }
            }
            // Optional presentation and linked-option conversion are never needed to count.
            Ok(false)
        })
        .map_err(convert)?;
    if offset > total {
        return Err(Error::Stale(
            "schema continuation exceeds selected declaration population".into(),
        ));
    }
    if limit == 0 {
        return Ok((total, Vec::new()));
    }
    let addresses = addresses.into_iter().skip(offset).collect::<BTreeSet<_>>();
    let nodes = bundle
        .select(None, &mut |_, _, node| {
            if token.is_cancelled() {
                return Err(SchemaError::Cancelled);
            }
            Ok(addresses.contains(&node.address))
        })
        .map_err(convert)?;
    Ok((total, nodes))
}
fn convert(error: SchemaError) -> Error {
    match error {
        SchemaError::Cancelled => Error::Cancelled,
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
    fn schema_search_counts_and_pages_without_unselected_presentation() {
        use std::sync::atomic::Ordering;
        let bundle = Bundle::from_bytes("fixture".into(), CORPUS.to_vec()).unwrap();
        let (_, mut query) = selected(Path::new("fixture"));
        query.query.clear();
        let token = CancellationToken::new();
        let (total, count) = selected_page(&bundle, &query, 0, 0, &token).unwrap();
        assert!(total > 20);
        assert!(count.is_empty());
        assert_eq!(bundle.presented.load(Ordering::Relaxed), 0);
        let (page_total, first) = selected_page(&bundle, &query, 0, 2, &token).unwrap();
        assert_eq!(page_total, total);
        assert_eq!(first.len(), 2);
        assert_eq!(bundle.presented.load(Ordering::Relaxed), 2);
        let (_, second) = selected_page(&bundle, &query, 2, 3, &token).unwrap();
        assert_eq!(second.len(), 3);
        assert_eq!(bundle.presented.load(Ordering::Relaxed), 5);
        assert!(first[1].address < second[0].address);
        let (_, exists) = selected_page(&bundle, &query, 0, 1, &token).unwrap();
        assert_eq!(bundle.presented.load(Ordering::Relaxed), 6);
        assert_eq!(exists[0].address, first[0].address);
        assert!(matches!(
            selected_page(&bundle, &query, total + 1, 1, &token),
            Err(Error::Stale(_))
        ));
        token.cancel();
        assert!(matches!(
            selected_page(&bundle, &query, 0, 1, &token),
            Err(Error::Cancelled)
        ));
        assert_eq!(bundle.presented.load(Ordering::Relaxed), 6);

        // Compare complete row shape against the explicitly exhaustive walker.
        let expected = bundle.index(None).unwrap();
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&expected[..2]).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&second).unwrap(),
            serde_json::to_value(&expected[2..5]).unwrap()
        );
    }
    #[test]
    fn schema_search_matches_all_declared_kinds_comments_and_exact_owners() {
        let bundle = Bundle::from_bytes("fixture".into(), CORPUS.to_vec()).unwrap();
        let (_, mut query) = selected(Path::new("fixture"));
        let token = CancellationToken::new();
        for (needle, kinds, owner) in [
            ("", vec![], None),
            ("PAYLOAD", vec![], None),
            (
                "",
                vec!["field".into()],
                Some("plan61.qualification.v1.Payload".into()),
            ),
            ("", vec!["enum_value".into(), "oneof".into()], None),
            ("", vec!["extension".into()], None),
            ("", vec!["method".into(), "service".into()], None),
        ] {
            query.query = needle.into();
            query.kinds = kinds;
            query.owner = owner;
            let (_, actual) = selected_page(&bundle, &query, 0, usize::MAX, &token).unwrap();
            let needle = query.query.to_lowercase();
            let expected = bundle
                .index(None)
                .unwrap()
                .into_iter()
                .filter(|node| {
                    (node.name.to_lowercase().contains(&needle)
                        || node.comments.to_lowercase().contains(&needle))
                        && (query.kinds.is_empty() || query.kinds.contains(&node.kind))
                        && query.owner.as_ref().is_none_or(|owner| {
                            node.parent.as_ref() == Some(owner) || &node.file == owner
                        })
                })
                .collect::<Vec<_>>();
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::to_value(expected).unwrap()
            );
        }
        // Independently authored fixture expectations include comment-only matches and
        // lexical field-tag addresses rather than numeric declaration order.
        query.query = "π".into();
        query.kinds.clear();
        query.owner = None;
        let (count, rows) = selected_page(&bundle, &query, 0, 2, &token).unwrap();
        assert_eq!(count, 1);
        assert_eq!(rows[0].address, "plan61.qualification.v1.Payload#6");
        query.query.clear();
        query.kinds = vec!["field".into()];
        query.owner = Some("plan61.qualification.v1.Payload".into());
        let (count, rows) = selected_page(&bundle, &query, 0, usize::MAX, &token).unwrap();
        assert_eq!(count, 10);
        assert_eq!(
            rows.iter()
                .map(|row| row.field_number.unwrap())
                .collect::<Vec<_>>(),
            [1, 10, 2, 3, 4, 5, 6, 7, 8, 9]
        );
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
