// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded immutable derived graphs with exact source and result lineage.

use crate::{
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
use std::{collections::BTreeSet, time::Duration};
use surrealdb::types::Object;

/// Maximum complete graph admitted by this initial method boundary.
pub const ANALYSIS_NODES: usize = 4096;
/// Maximum complete directed graph edge membership.
pub const ANALYSIS_EDGES: usize = 8192;

/// Versioned framed identity shared by analysis producers and native receipts.
pub fn analysis_key(kind: &str, parts: &[&[u8]]) -> String {
    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str(kind);
    for part in parts {
        hash.part(part);
    }
    hash.finish_hash().to_hex()
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_analyses_server_unit {
    use super::*;
    use crate::canonical::{CanonicalOptions, checked};
    #[tokio::test]
    async fn canonical_analysis_graph_staging_requires_complete_immutable_membership() {
        let state = std::env::var("PSE_SURREAL_STATE").expect("explicit native fixture required");
        let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!("analysis_{}", uuid::Uuid::new_v4().simple());
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let revision = store
            .edit("analysis-source", None, "source", &[])
            .await
            .unwrap();
        let key = "graph";
        let header = Analysis {
            key: key.into(),
            revision: revision.key.clone(),
            method: "mechanism-fixture:v1".into(),
            configuration: vec![1].into(),
            input_digest: "exact-fixture".into(),
            interpretation: wire::INTERPRETATION.into(),
            node_count: 2,
            edge_count: 1,
            active: false,
        };
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
                .query("RETURN fn::pse_analysis_v1::begin($row,$sources,$inputs);")
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
                    .query("RETURN fn::pse_analysis_v1::activate($key);")
                    .bind(("key", key))
            )
            .await
            .is_err()
        );
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
                .query("RETURN fn::pse_analysis_v1::activate($key);")
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
        store.forget_analysis_results(key).await.unwrap();
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
        store
            .db
            .query(format!("REMOVE DATABASE {};", options.database))
            .await
            .and_then(checked)
            .unwrap();
    }
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
    /// Retrieve only the actual primary and physical source receipts of a run.
    pub async fn analysis_run_sources(&self, run: &str) -> Result<Vec<Revision>, CanonicalError> {
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
        self.ensure_writes()?;
        identity(&header.key)?;
        identity(&header.method)?;
        identity(&header.input_digest)?;
        if header.active
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
        let mut node_keys = BTreeSet::new();
        for node in nodes {
            identity(&node.key)?;
            identity(&node.semantic)?;
            identity(&node.kind)?;
            if node.analysis != header.key || !node_keys.insert(&node.key) {
                return Err(invalid("analysis node closure"));
            }
        }
        let mut edge_keys = BTreeSet::new();
        for edge in edges {
            identity(&edge.key)?;
            identity(&edge.kind)?;
            if edge
                .evidence
                .as_ref()
                .is_some_and(|value| value.len() > 4096)
                || edge.analysis != header.key
                || !node_keys.contains(&edge.source)
                || !node_keys.contains(&edge.target)
                || !edge_keys.insert(&edge.key)
            {
                return Err(invalid("analysis edge closure"));
            }
        }
        if inputs.iter().any(|input| !input.belongs_to(self)) {
            return Err(invalid("analysis input belongs to another store"));
        }
        // Pins close the interval between immutable receipt selection and the
        // transaction that creates all retained analysis roots.
        let mut pins = Vec::with_capacity(sources.len());
        for source in sources {
            match self.protect(source.clone(), Duration::from_secs(600)).await {
                Ok(pin) => pins.push(pin),
                Err(error) => {
                    for pin in &pins {
                        let _ = self.release(pin).await;
                    }
                    return Err(error);
                }
            }
        }
        let admission = async {
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
            protected_query(|| {
                Ok(self
                    .db
                    .query("RETURN fn::pse_analysis_v1::begin($row,$sources,$inputs);")
                    .bind(("row", row.clone()))
                    .bind(("sources", roots.clone()))
                    .bind(("inputs", selected.clone())))
            })
            .await?;
            Ok::<(), CanonicalError>(())
        }
        .await;
        for pin in &pins {
            let _ = self.release(pin).await;
        }
        admission?;
        for page in nodes.chunks(64) {
            let encoded = page
                .iter()
                .map(wire::encode_canonical_analysis_nodes)
                .collect::<Result<Vec<_>, _>>()?;
            self.append_analysis(&header.key, encoded, vec![]).await?;
        }
        for page in edges.chunks(64) {
            let encoded = page
                .iter()
                .map(wire::encode_canonical_analysis_edges)
                .collect::<Result<Vec<_>, _>>()?;
            self.append_analysis(&header.key, vec![], encoded).await?;
        }
        let mut response = protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_analysis_v1::activate($key);")
                .bind(("key", header.key.clone())))
        })
        .await?;
        Ok(wire::decode_canonical_analyses(
            response
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
    }
    async fn append_analysis(
        &self,
        key: &str,
        nodes: Vec<Object>,
        edges: Vec<Object>,
    ) -> Result<(), CanonicalError> {
        protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_analysis_v1::append($key,$nodes,$edges);")
                .bind(("key", key.to_owned()))
                .bind(("nodes", nodes.clone()))
                .bind(("edges", edges.clone())))
        })
        .await?;
        Ok(())
    }
    /// Exact active header; incomplete or explicitly withdrawn graphs refuse reads.
    pub async fn analysis(&self, key: &str) -> Result<Analysis, CanonicalError> {
        identity(key)?;
        let mut result=protected_query(||Ok(self.db.query("LET $row=fn::pse_analysis_v1::available($key); IF !$row.active { THROW 'analysis graph incomplete'; }; RETURN $row;").bind(("key",key.to_owned())))).await?;
        let index = result.num_statements().saturating_sub(1);
        Ok(wire::decode_canonical_analyses(
            result
                .take::<Option<Object>>(index)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
    }
    /// Ordered bounded graph nodes from one exact active analysis.
    pub async fn analysis_node_page(
        &self,
        key: &str,
        after: Option<&str>,
    ) -> Result<Vec<AnalysisNode>, CanonicalError> {
        self.analysis_page(key, after, "nodes")
            .await?
            .into_iter()
            .map(|row| wire::decode_canonical_analysis_nodes(row).map_err(Into::into))
            .collect()
    }
    /// Ordered bounded native graph edges from one exact active analysis.
    pub async fn analysis_edge_page(
        &self,
        key: &str,
        after: Option<&str>,
    ) -> Result<Vec<AnalysisEdge>, CanonicalError> {
        self.analysis_page(key, after, "edges")
            .await?
            .into_iter()
            .map(|row| wire::decode_canonical_analysis_edges(row).map_err(Into::into))
            .collect()
    }
    async fn analysis_page(
        &self,
        key: &str,
        after: Option<&str>,
        kind: &str,
    ) -> Result<Vec<Object>, CanonicalError> {
        identity(key)?;
        let mut response = protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_analysis_v1::read($key,$after,$kind);")
                .bind(("key", key.to_owned()))
                .bind(("after", after.unwrap_or("").to_owned()))
                .bind(("kind", kind.to_owned())))
        })
        .await?;
        let rows = response.take::<Vec<Object>>(0)?;
        if rows.len() > 64 {
            return Err(CanonicalError::PayloadLimit);
        }
        Ok(rows)
    }
}
