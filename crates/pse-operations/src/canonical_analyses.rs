// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded immutable derived graphs with exact source and result lineage.

use crate::{
    canonical::transport::{original_deadline, within_clock},
    canonical::{CanonicalError, CanonicalStore, Revision, bounded_query, protected_query},
    canonical_codec as codec,
    canonical_results::ResultRead,
    generated::surreal as wire,
};
use pse_ids::{Frame, FramedHasher};
pub use pse_model::generated::runtime::{
    canonical_analyses::Row as Analysis, canonical_analysis_edges::Row as AnalysisEdge,
    canonical_analysis_inputs::Row as AnalysisInput, canonical_analysis_nodes::Row as AnalysisNode,
};
use std::collections::{BTreeMap, BTreeSet};
use surrealdb::types::Object;

#[cfg(all(test, feature = "canonical-tests"))]
#[path = "canonical_analyses/race_tests.rs"]
mod race_tests;

/// Maximum complete graph admitted by this initial method boundary.
pub const ANALYSIS_NODES: usize = 4096;
/// Maximum complete directed graph edge membership.
pub const ANALYSIS_EDGES: usize = 8192;

/// One classification per admitted edge, followed by original-order draining.
struct EdgePublication<'a> {
    endpoint_pages: Vec<Vec<(usize, &'a AnalysisEdge)>>,
    ready: BTreeMap<usize, &'a AnalysisEdge>,
}

impl<'a> EdgePublication<'a> {
    fn new(node_count: usize) -> Self {
        Self {
            endpoint_pages: vec![Vec::new(); node_count.div_ceil(64)],
            ready: BTreeMap::new(),
        }
    }

    fn classify(
        &mut self,
        ordinal: usize,
        edge: &'a AnalysisEdge,
        node_pages: &BTreeMap<&str, usize>,
    ) -> Option<()> {
        let page =
            (*node_pages.get(edge.source.as_str())?).max(*node_pages.get(edge.target.as_str())?);
        self.endpoint_pages.get_mut(page)?.push((ordinal, edge));
        Some(())
    }

    fn node_page(&mut self, page: usize) -> Vec<&'a AnalysisEdge> {
        self.ready.extend(self.endpoint_pages[page].drain(..));
        self.take_ready()
    }

    fn take_ready(&mut self) -> Vec<&'a AnalysisEdge> {
        let mut page = Vec::with_capacity(self.ready.len().min(64));
        while page.len() < 64 {
            let Some((_, edge)) = self.ready.pop_first() else {
                break;
            };
            page.push(edge);
        }
        page
    }
}

/// Versioned framed identity shared by analysis producers and native receipts.
pub fn analysis_key(kind: &str, parts: &[&[u8]]) -> String {
    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str(kind);
    for part in parts {
        hash.part(part);
    }
    hash.finish_hash().to_hex()
}

/// Seal the complete ordered publication request before any creation effect.
/// Retries reuse this header and these exact payloads; persistence checks the seal.
pub fn seal_analysis_request(
    header: &mut Analysis,
    sources: &[Revision],
    inputs: &[ResultRead],
    nodes: &[AnalysisNode],
    edges: &[AnalysisEdge],
) {
    header.creation_request_digest.clear();
    header.creation_request_digest = request_digest(header, sources, inputs, nodes, edges);
}

fn request_digest(
    header: &Analysis,
    sources: &[Revision],
    inputs: &[ResultRead],
    nodes: &[AnalysisNode],
    edges: &[AnalysisEdge],
) -> String {
    use pse_model::SemanticFrame;
    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("pse.analysis.complete-request.v2");
    let mut row = header.clone();
    row.creation_request_digest.clear();
    row.active = false;
    row.retiring = false;
    row.frame(&mut hash);
    hash.str("sources").u64(sources.len() as u64);
    for source in sources {
        source.frame(&mut hash);
    }
    hash.str("inputs").u64(inputs.len() as u64);
    for input in inputs {
        input.run().frame(&mut hash);
        input.attempt().frame(&mut hash);
        input.manifest().frame(&mut hash);
        hash.str(input.protected_selection().key());
    }
    hash.str("nodes").u64(nodes.len() as u64);
    for node in nodes {
        node.frame(&mut hash);
    }
    hash.str("edges").u64(edges.len() as u64);
    for edge in edges {
        edge.frame(&mut hash);
    }
    hash.finish_hash().to_hex()
}

fn occurrence_key(authority: &str, nonce: &str, expiry: u64) -> String {
    format!("pse.analysis.v2:{authority}:{nonce}:{expiry}")
}

fn invalid(message: &str) -> CanonicalError {
    CanonicalError::Configuration(message.into())
}
fn identity(value: &str) -> Result<(), CanonicalError> {
    if value.is_empty() || value.len() > 4096 {
        return Err(CanonicalError::PayloadLimit);
    }
    Ok(())
}

impl CanonicalStore {
    /// Prepare one immutable creation intent after read-only source-authority discovery.
    /// Seal the draft with `seal_analysis_request` after graph keys are assigned;
    /// then reuse the complete sealed intent unchanged for every creation retry.
    pub async fn new_analysis(
        &self,
        revision: &Revision,
        method: &str,
        configuration: Vec<u8>,
        input_digest: String,
        node_count: u64,
        edge_count: u64,
    ) -> Result<Analysis, CanonicalError> {
        let mut response = bounded_query(self.db.query("LET $guard=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$problem); IF $guard=NONE OR $guard.incarnation=NONE { THROW 'analysis source authority unavailable'; }; RETURN {authority:$guard.incarnation,now:time::micros()};").bind(("problem",revision.problem.clone()))).await?;
        let index = response.num_statements().saturating_sub(1);
        let mut row = response
            .take::<Option<Object>>(index)?
            .ok_or(CanonicalError::IncompleteResponse)?;
        let primary_authority = codec::decode_string(codec::required(&mut row, "authority")?)?;
        let now = codec::decode_int(codec::required(&mut row, "now")?)?;
        let creation_expires_at = u64::try_from(now)
            .map_err(|_| invalid("negative analysis creation clock"))?
            .checked_add(60_000_000)
            .ok_or_else(|| invalid("analysis creation clock overflow"))?;
        // No await occurs between creating this nonce/expiry-bound intent and returning it.
        let creation_nonce = uuid::Uuid::new_v4().to_string();
        Ok(Analysis {
            key: occurrence_key(&primary_authority, &creation_nonce, creation_expires_at),
            revision: revision.key.clone(),
            method: method.into(),
            configuration: configuration.into(),
            input_digest,
            primary_problem: revision.problem.clone(),
            primary_authority,
            creation_nonce,
            creation_request_digest: String::new(),
            creation_expires_at,
            interpretation: wire::INTERPRETATION.into(),
            node_count,
            edge_count,
            active: false,
            retiring: false,
        })
    }

    /// Retrieve only the actual primary and physical source receipts of a run.
    pub async fn analysis_run_sources(&self, run: &str) -> Result<Vec<Revision>, CanonicalError> {
        within_clock(original_deadline(crate::canonical::REQUEST_TIMEOUT), async {
            identity(run)?;
            let mut response=bounded_query(self.db.query("SELECT VALUE revision FROM canonical_run_sources WHERE run=$run ORDER BY revision LIMIT 66;").bind(("run",run.to_owned()))).await?;
            let keys = response.take::<Vec<String>>(0)?;
            if keys.is_empty() || keys.len() > 65 {
                return Err(CanonicalError::PayloadLimit);
            }
            let mut sources = Vec::with_capacity(keys.len());
            for key in keys {
                sources.push(
                    self.revision(&key)
                        .await?
                        .ok_or_else(|| invalid("analysis result source missing"))?,
                );
            }
            Ok(sources)
        }).await
    }

    /// Persist admitted graph pages, then activate exact complete membership.
    /// A failed staged write remains incomplete and retains its source inputs;
    /// explicit analysis retirement releases those obligations.
    pub async fn persist_analysis(
        &self,
        header: &Analysis,
        sources: &[Revision],
        inputs: &[ResultRead],
        nodes: &[AnalysisNode],
        edges: &[AnalysisEdge],
    ) -> Result<Analysis, CanonicalError> {
        if header.creation_request_digest != request_digest(header, sources, inputs, nodes, edges) {
            return Err(CanonicalError::OperationReused);
        }
        let result = within_clock(original_deadline(crate::canonical::ACTIVATION_REQUEST_TIMEOUT), async {
            self.ensure_writes()?;
            identity(&header.key)?;
            identity(&header.method)?;
            identity(&header.input_digest)?;
            if header.active
                || header.retiring
                || header.key != occurrence_key(&header.primary_authority, &header.creation_nonce, header.creation_expires_at)
                || header.interpretation != wire::INTERPRETATION
                || header.configuration.len() > 128 * 1024
                || sources.is_empty()
                || sources.len() > 65
                || inputs.len() > 64
                || nodes.len() > ANALYSIS_NODES
                || edges.len() > ANALYSIS_EDGES
                || header.node_count != nodes.len() as u64
                || header.edge_count != edges.len() as u64
                || !sources.iter().any(|source| source.key == header.revision)
            {
                return Err(invalid("analysis closure bounds or source mismatch"));
            }
            let mut node_pages = BTreeMap::new();
            for (ordinal, node) in nodes.iter().enumerate() {
                identity(&node.key)?;
                identity(&node.semantic)?;
                identity(&node.kind)?;
                if node.analysis != header.key
                    || node_pages.insert(node.key.as_str(), ordinal / 64).is_some()
                {
                    return Err(invalid("analysis node closure"));
                }
            }
            let mut edge_keys = BTreeSet::new();
            let mut publication = EdgePublication::new(nodes.len());
            for (ordinal, edge) in edges.iter().enumerate() {
                identity(&edge.key)?;
                identity(&edge.kind)?;
                if edge
                    .evidence
                    .as_ref()
                    .is_some_and(|value| value.len() > 4096)
                    || edge.analysis != header.key
                    || !edge_keys.insert(&edge.key)
                    || publication.classify(ordinal, edge, &node_pages).is_none()
                {
                    return Err(invalid("analysis edge closure"));
                }
            }
            if inputs.iter().any(|input| !input.belongs_to(self)) {
                return Err(invalid("analysis input belongs to another store"));
            }
            {
                let mut roots = Vec::with_capacity(sources.len());
                for source in sources {
                    let mut row = Object::new();
                    row.insert(
                        "key",
                        codec::encode_string(analysis_key(
                            "pse.analysis.root.v1",
                            &[header.key.as_bytes(), source.key.as_bytes()],
                        ))?,
                    );
                    row.insert("problem", codec::encode_string(source.problem.clone())?);
                    row.insert("revision", codec::encode_string(source.key.clone())?);
                    row.insert("sequence", codec::encode_uint(source.sequence)?);
                    roots.push(row);
                }
                let mut selected = Vec::with_capacity(inputs.len());
                for input in inputs {
                    let row = AnalysisInput {
                        key: analysis_key(
                            "pse.analysis.input.v1",
                            &[
                                header.key.as_bytes(),
                                input.attempt().key.as_bytes(),
                                input.manifest().key.as_bytes(),
                            ],
                        ),
                        analysis: header.key.clone(),
                        run: input.run().key.clone(),
                        attempt: input.attempt().key.clone(),
                        manifest: input.manifest().key.clone(),
                    };
                    let mut row = wire::encode_canonical_analysis_inputs(&row)?;
                    row.insert(
                        "protection",
                        codec::encode_string(input.protected_selection().key().to_owned())?,
                    );
                    selected.push(row);
                }
                let row = wire::encode_canonical_analyses(header)?;
                protected_query("canonical_analyses::persist_analysis", || {
                    Ok(self
                        .db
                        .query("BEGIN; RETURN fn::pse_analysis_v1::begin($pse_rpc_expires_at, $row,$sources,$inputs); COMMIT;")
                        .bind(("row", row.clone()))
                        .bind(("sources", roots.clone()))
                        .bind(("inputs", selected.clone())))
                })
                .await?;
            }
            for (ordinal, page) in nodes.chunks(64).enumerate() {
                let encoded = page
                    .iter()
                    .map(wire::encode_canonical_analysis_nodes)
                    .collect::<Result<Vec<_>, _>>()?;
                let ready = publication
                    .node_page(ordinal)
                    .into_iter()
                    .map(wire::encode_canonical_analysis_edges)
                    .collect::<Result<Vec<_>, _>>()?;
                // The native append inserts all page nodes before its edges, so
                // endpoint dependencies hold within this same guarded effect.
                self.append_analysis(&header.key, encoded, ready).await?;
            }
            loop {
                let page = publication.take_ready();
                if page.is_empty() {
                    break;
                }
                let encoded = page
                    .iter()
                    .map(|edge| wire::encode_canonical_analysis_edges(edge))
                    .collect::<Result<Vec<_>, _>>()?;
                self.append_analysis(&header.key, vec![], encoded).await?;
            }
            let mut response = protected_query("canonical_analyses::persist_analysis", || {
                Ok(self
                    .db
                    .query("BEGIN; RETURN fn::pse_analysis_v1::activate($pse_rpc_expires_at, $key); COMMIT;")
                    .bind(("key", header.key.clone())))
            })
            .await?;
            Ok(wire::decode_canonical_analyses(
                response
                    .take::<Option<Object>>(response.num_statements().saturating_sub(2))?
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?)
        }).await;
        result.map_err(|source| CanonicalError::AnalysisUnsettled {
            intent: Box::new(header.clone()),
            source: Box::new(source),
        })
    }
    async fn append_analysis(
        &self,
        key: &str,
        nodes: Vec<Object>,
        edges: Vec<Object>,
    ) -> Result<(), CanonicalError> {
        within_clock(original_deadline(crate::canonical::REQUEST_TIMEOUT), async {
            protected_query("canonical_analyses::append_analysis", || {
                Ok(self
                    .db
                    .query("BEGIN; RETURN fn::pse_analysis_v1::append($pse_rpc_expires_at, $key,$nodes,$edges); COMMIT;")
                    .bind(("key", key.to_owned()))
                    .bind(("nodes", nodes.clone()))
                    .bind(("edges", edges.clone())))
            })
            .await?;
            Ok(())
        }).await
    }
    /// Exact active header; incomplete or explicitly withdrawn graphs refuse reads.
    pub async fn analysis(&self, key: &str) -> Result<Analysis, CanonicalError> {
        within_clock(original_deadline(crate::canonical::REQUEST_TIMEOUT), async {
            identity(key)?;
            let mut result=protected_query("canonical_analyses::analysis", ||Ok(self.db.query("BEGIN; LET $row=fn::pse_analysis_v1::available($pse_rpc_expires_at, $key); IF !$row.active { THROW 'analysis graph incomplete'; }; RETURN $row; COMMIT;").bind(("key",key.to_owned())))).await?;
            let index = result.num_statements().saturating_sub(2);
            Ok(wire::decode_canonical_analyses(
                result
                    .take::<Option<Object>>(index)?
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?)
        }).await
    }
    /// Ordered bounded graph nodes from one exact active analysis.
    pub async fn analysis_node_page(
        &self,
        key: &str,
        after: Option<&str>,
    ) -> Result<Vec<AnalysisNode>, CanonicalError> {
        within_clock(
            original_deadline(crate::canonical::REQUEST_TIMEOUT),
            async {
                self.analysis_page(key, after, "nodes")
                    .await?
                    .into_iter()
                    .map(|row| wire::decode_canonical_analysis_nodes(row).map_err(Into::into))
                    .collect()
            },
        )
        .await
    }
    /// Ordered bounded native graph edges from one exact active analysis.
    pub async fn analysis_edge_page(
        &self,
        key: &str,
        after: Option<&str>,
    ) -> Result<Vec<AnalysisEdge>, CanonicalError> {
        within_clock(
            original_deadline(crate::canonical::REQUEST_TIMEOUT),
            async {
                self.analysis_page(key, after, "edges")
                    .await?
                    .into_iter()
                    .map(|row| wire::decode_canonical_analysis_edges(row).map_err(Into::into))
                    .collect()
            },
        )
        .await
    }
    async fn analysis_page(
        &self,
        key: &str,
        after: Option<&str>,
        kind: &str,
    ) -> Result<Vec<Object>, CanonicalError> {
        within_clock(original_deadline(crate::canonical::REQUEST_TIMEOUT), async {
            identity(key)?;
            let mut response = protected_query("canonical_analyses::analysis_page", || {
                Ok(self
                    .db
                    .query("BEGIN; RETURN fn::pse_analysis_v1::read($pse_rpc_expires_at, $key,$after,$kind); COMMIT;")
                    .bind(("key", key.to_owned()))
                    .bind(("after", after.unwrap_or("").to_owned()))
                    .bind(("kind", kind.to_owned())))
            })
            .await?;
            let rows = response.take::<Vec<Object>>(response.num_statements().saturating_sub(2))?;
            if rows.len() > 64 {
                return Err(CanonicalError::PayloadLimit);
            }
            Ok(rows)
        }).await
    }
}

#[cfg(test)]
mod canonical_analysis_publication_unit {
    use super::*;

    fn nodes(count: usize) -> Vec<AnalysisNode> {
        (0..count)
            .map(|ordinal| AnalysisNode {
                key: format!("node-{ordinal}"),
                analysis: "analysis".into(),
                semantic: format!("semantic-{ordinal}"),
                kind: "variable".into(),
            })
            .collect()
    }

    fn edge(ordinal: usize, source: usize, target: usize) -> AnalysisEdge {
        AnalysisEdge {
            key: format!("edge-{ordinal}"),
            analysis: "analysis".into(),
            source: format!("node-{source}"),
            target: format!("node-{target}"),
            kind: "incidence".into(),
            evidence: None,
        }
    }

    fn classify<'a>(nodes: &[AnalysisNode], edges: &'a [AnalysisEdge]) -> EdgePublication<'a> {
        let node_pages = nodes
            .iter()
            .enumerate()
            .map(|(ordinal, node)| (node.key.as_str(), ordinal / 64))
            .collect::<BTreeMap<_, _>>();
        let mut publication = EdgePublication::new(nodes.len());
        for (ordinal, edge) in edges.iter().enumerate() {
            assert!(publication.classify(ordinal, edge, &node_pages).is_some());
        }
        publication
    }

    #[test]
    fn reordered_nodes_publish_edges_only_after_their_last_endpoint_page() {
        let mut nodes = nodes(129);
        nodes.reverse();
        let edges = vec![
            edge(0, 0, 128),
            edge(1, 128, 128),
            edge(2, 64, 128),
            edge(3, 0, 64),
        ];
        let mut publication = classify(&nodes, &edges);
        assert_eq!(publication.node_page(0), vec![&edges[1]]);
        assert_eq!(publication.node_page(1), vec![&edges[2]]);
        assert_eq!(publication.node_page(2), vec![&edges[0], &edges[3]]);
        assert!(publication.take_ready().is_empty());
    }

    #[test]
    fn ready_backlog_and_late_edges_drain_in_exact_original_selection_order() {
        let nodes = nodes(65);
        let edges = (0..131)
            .map(|ordinal| {
                // The first selected edge becomes ready later than its successors.
                edge(ordinal, 0, if ordinal == 0 { 64 } else { 1 })
            })
            .collect::<Vec<_>>();
        let mut publication = classify(&nodes, &edges);
        assert_eq!(
            publication.node_page(0),
            edges[1..65].iter().collect::<Vec<_>>()
        );
        let mut second = vec![&edges[0]];
        second.extend(edges[65..128].iter());
        assert_eq!(publication.node_page(1), second);
        assert_eq!(
            publication.take_ready(),
            edges[128..].iter().collect::<Vec<_>>()
        );
        assert!(publication.take_ready().is_empty());
    }

    #[test]
    fn missing_endpoints_cannot_enter_a_publication_page() {
        let nodes = nodes(1);
        let node_pages = BTreeMap::from([(nodes[0].key.as_str(), 0)]);
        let missing_source = edge(0, 1, 0);
        let missing_target = edge(1, 0, 1);
        let mut publication = EdgePublication::new(nodes.len());
        assert!(
            publication
                .classify(0, &missing_source, &node_pages)
                .is_none()
        );
        assert!(
            publication
                .classify(1, &missing_target, &node_pages)
                .is_none()
        );
        assert!(publication.node_page(0).is_empty());
        assert!(publication.take_ready().is_empty());
        assert!(EdgePublication::new(0).take_ready().is_empty());
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_analyses_server_unit {
    use super::*;
    use crate::canonical::CanonicalOptions;
    #[tokio::test]
    async fn canonical_analysis_graph_staging_requires_complete_immutable_membership() {
        let state = std::env::var("PSE_SURREAL_STATE").expect("explicit native fixture required");
        let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!("canonical_test_analysis_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        let revision = store
            .edit("analysis-source", None, "source", &[])
            .await
            .unwrap();
        let mut header = store
            .new_analysis(
                &revision,
                "mechanism-fixture:v2",
                vec![1],
                "exact-fixture".into(),
                2,
                1,
            )
            .await
            .unwrap();
        let key_owned = header.key.clone();
        let key = key_owned.as_str();
        let first = AnalysisNode {
            key: "node-a".into(),
            analysis: key.into(),
            semantic: "a".into(),
            kind: "variable".into(),
        };
        let second = AnalysisNode {
            key: "node-b".into(),
            analysis: key.into(),
            semantic: "b".into(),
            kind: "row".into(),
        };
        let edge = AnalysisEdge {
            key: "edge".into(),
            analysis: key.into(),
            source: first.key.clone(),
            target: second.key.clone(),
            kind: "incidence".into(),
            evidence: None,
        };
        seal_analysis_request(
            &mut header,
            std::slice::from_ref(&revision),
            &[],
            &[first.clone(), second.clone()],
            std::slice::from_ref(&edge),
        );
        let mut source = Object::new();
        source.insert("key", codec::encode_string("graph-root".into()).unwrap());
        source.insert(
            "problem",
            codec::encode_string(revision.problem.clone()).unwrap(),
        );
        source.insert(
            "revision",
            codec::encode_string(revision.key.clone()).unwrap(),
        );
        source.insert("sequence", codec::encode_uint(revision.sequence).unwrap());
        bounded_query(
            store
                .db
                .query("BEGIN; RETURN fn::pse_analysis_v1::begin($pse_rpc_expires_at, $row,$sources,$inputs); COMMIT;")
                .bind(("row", wire::encode_canonical_analyses(&header).unwrap()))
                .bind(("sources", vec![source]))
                .bind(("inputs", Vec::<Object>::new())),
        )
        .await
        .unwrap();
        assert!(store.analysis(key).await.is_err());
        assert!(
            bounded_query(
                store
                    .db
                    .query("BEGIN; RETURN fn::pse_analysis_v1::activate($pse_rpc_expires_at, $key); COMMIT;")
                    .bind(("key", key))
            )
            .await
            .is_err()
        );
        store
            .append_analysis(
                key,
                vec![wire::encode_canonical_analysis_nodes(&first).unwrap()],
                vec![],
            )
            .await
            .unwrap();
        assert!(
            store
                .append_analysis(
                    key,
                    vec![],
                    vec![wire::encode_canonical_analysis_edges(&edge).unwrap()]
                )
                .await
                .is_err()
        );
        store
            .append_analysis(
                key,
                vec![wire::encode_canonical_analysis_nodes(&second).unwrap()],
                vec![wire::encode_canonical_analysis_edges(&edge).unwrap()],
            )
            .await
            .unwrap();
        bounded_query(
            store
                .db
                .query("BEGIN; RETURN fn::pse_analysis_v1::activate($pse_rpc_expires_at, $key); COMMIT;")
                .bind(("key", key)),
        )
        .await
        .unwrap();
        assert!(store.analysis(key).await.unwrap().active);
        assert_eq!(store.analysis_node_page(key, None).await.unwrap().len(), 2);
        assert_eq!(
            store.analysis_edge_page(key, None).await.unwrap(),
            vec![edge.clone()]
        );
        store
            .append_analysis(
                key,
                vec![wire::encode_canonical_analysis_nodes(&first).unwrap()],
                vec![wire::encode_canonical_analysis_edges(&edge).unwrap()],
            )
            .await
            .unwrap();
        let mut grouped = store
            .new_analysis(
                &revision,
                "grouped-fixture:v2",
                vec![1],
                "grouped".into(),
                65,
                1,
            )
            .await
            .unwrap();
        let nodes = (0..65)
            .map(|ordinal| AnalysisNode {
                key: format!("grouped:{ordinal:03}"),
                analysis: grouped.key.clone(),
                ..first.clone()
            })
            .collect::<Vec<_>>();
        let dependent = AnalysisEdge {
            key: "grouped-edge".into(),
            analysis: grouped.key.clone(),
            source: nodes[0].key.clone(),
            target: nodes[64].key.clone(),
            ..edge.clone()
        };
        seal_analysis_request(
            &mut grouped,
            std::slice::from_ref(&revision),
            &[],
            &nodes,
            std::slice::from_ref(&dependent),
        );
        let saved = store
            .persist_analysis(
                &grouped,
                std::slice::from_ref(&revision),
                &[],
                &nodes,
                std::slice::from_ref(&dependent),
            )
            .await
            .unwrap();
        assert!(
            saved.active,
            "an edge may share the page that establishes its last endpoint"
        );
        assert_eq!(
            store
                .analysis_node_page(&grouped.key, None)
                .await
                .unwrap()
                .len(),
            64
        );
        assert_eq!(
            store.analysis_edge_page(&grouped.key, None).await.unwrap(),
            vec![dependent]
        );
        let mut changed = first.clone();
        changed.kind = "parameter".into();
        assert!(
            store
                .append_analysis(
                    key,
                    vec![wire::encode_canonical_analysis_nodes(&changed).unwrap()],
                    vec![]
                )
                .await
                .is_err()
        );
        for _ in 0..5 {
            assert!(!store.forget_analysis_results(key).await.unwrap());
        }

        assert!(store.analysis_node_page(key, None).await.is_err());
        assert!(store.analysis(key).await.is_err());
        let mut response = bounded_query(
            store
                .db
                .query(
                    "SELECT key FROM canonical_roots WHERE owner_kind='analysis' AND owner=$key;",
                )
                .bind(("key", key)),
        )
        .await
        .unwrap();
        assert!(response.take::<Vec<Object>>(0).unwrap().is_empty());
        assert_eq!(store.database(), options.database);
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_analysis_retirement_drains_edges_first_and_leaves_no_occurrence_state() {
        let state = std::env::var("PSE_SURREAL_STATE").expect("explicit native fixture required");
        let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_analysis_retire_{}",
            uuid::Uuid::new_v4().simple()
        );
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        let revision = store
            .edit("retire-source", None, "retire-source-initial", &[])
            .await
            .unwrap();
        let mut intent = store
            .new_analysis(
                &revision,
                "retirement-fixture:v2",
                vec![],
                "complete".into(),
                128,
                1024,
            )
            .await
            .unwrap();
        // Construct a fresh short-window fixture before any creation is issued.
        intent.creation_expires_at -= 58_000_000;
        intent.key = occurrence_key(
            &intent.primary_authority,
            &intent.creation_nonce,
            intent.creation_expires_at,
        );
        let nodes = (0..128)
            .map(|n| AnalysisNode {
                key: format!("{}:n:{n:03}", intent.key),
                analysis: intent.key.clone(),
                semantic: format!("v:{n}"),
                kind: "variable".into(),
            })
            .collect::<Vec<_>>();
        let edges = (0..1024)
            .map(|n| AnalysisEdge {
                key: format!("{}:e:{n:04}", intent.key),
                analysis: intent.key.clone(),
                source: nodes[0].key.clone(),
                target: nodes[1 + n % 127].key.clone(),
                kind: "incidence".into(),
                evidence: None,
            })
            .collect::<Vec<_>>();
        seal_analysis_request(
            &mut intent,
            std::slice::from_ref(&revision),
            &[],
            &nodes,
            &edges,
        );
        store
            .persist_analysis(
                &intent,
                std::slice::from_ref(&revision),
                &[],
                &nodes,
                &edges,
            )
            .await
            .unwrap();
        assert!(!store.forget_analysis_results(&intent.key).await.unwrap());
        assert!(store.analysis(&intent.key).await.is_err());
        assert!(store.analysis_node_page(&intent.key, None).await.is_err());
        assert!(
            store
                .append_analysis(&intent.key, vec![], vec![])
                .await
                .is_err()
        );
        let mut counts = bounded_query(store.db.query("RETURN {nodes:count(SELECT key FROM canonical_analysis_nodes WHERE analysis=$key),edges:count(SELECT key FROM canonical_analysis_edges WHERE analysis=$key),roots:count(SELECT key FROM canonical_roots WHERE owner_kind='analysis' AND owner=$key)};").bind(("key",intent.key.clone()))).await.unwrap();
        let mut counts = counts.take::<Option<Object>>(0).unwrap().unwrap();
        assert_eq!(
            codec::decode_int(codec::required(&mut counts, "nodes").unwrap()).unwrap(),
            128
        );
        assert_eq!(
            codec::decode_int(codec::required(&mut counts, "edges").unwrap()).unwrap(),
            960
        );
        assert_eq!(
            codec::decode_int(codec::required(&mut counts, "roots").unwrap()).unwrap(),
            1
        );
        let mut complete = false;
        for _ in 0..128 {
            complete = store.forget_analysis_results(&intent.key).await.unwrap();
            if complete {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        assert!(complete);
        // The original intent resolves a lost final acknowledgement, including absent header.
        assert!(store.settle_analysis(&intent).await.unwrap());
        assert!(
            store
                .persist_analysis(
                    &intent,
                    std::slice::from_ref(&revision),
                    &[],
                    &nodes,
                    &edges
                )
                .await
                .is_err()
        );
        let mut renewed = intent.clone();
        renewed.creation_expires_at += 60_000_000;
        assert!(
            store
                .persist_analysis(
                    &renewed,
                    std::slice::from_ref(&revision),
                    &[],
                    &nodes,
                    &edges
                )
                .await
                .is_err()
        );
        let mut fresh = store
            .new_analysis(
                &revision,
                "retirement-fixture:v2",
                vec![],
                "complete".into(),
                0,
                0,
            )
            .await
            .unwrap();
        assert_ne!(fresh.key, intent.key);
        seal_analysis_request(&mut fresh, std::slice::from_ref(&revision), &[], &[], &[]);
        store
            .persist_analysis(&fresh, std::slice::from_ref(&revision), &[], &[], &[])
            .await
            .unwrap();
        let mut residue = bounded_query(store.db.query("RETURN array::concat((SELECT key FROM canonical_analyses WHERE key=$key),(SELECT key FROM canonical_analysis_nodes WHERE analysis=$key),(SELECT key FROM canonical_analysis_edges WHERE analysis=$key),(SELECT key FROM canonical_analysis_inputs WHERE analysis=$key),(SELECT key FROM canonical_roots WHERE owner_kind='analysis' AND owner=$key),(SELECT key FROM canonical_guards WHERE key='analysis:'+$key),(SELECT key FROM canonical_protections WHERE key=$key));").bind(("key",intent.key.clone()))).await.unwrap();
        assert!(residue.take::<Vec<Object>>(0).unwrap().is_empty());
        assert!(store.forget_analysis_results("unknown-key").await.is_err());
        store.remove_isolated_fixture().await.unwrap();
    }
}
