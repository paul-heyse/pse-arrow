// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native bounded path expansion; path order and work are explicit request facts.
use super::*;

/// A finite path question over one selected scientific graph owner.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalGraphPathsRequest {
    /// Standalone path request interpretation.
    pub version: pse_model::document::Version<1>,
    /// Exact selected study occurrence or immutable analysis node.
    pub anchor: GraphAnchor,
    /// Recorded prerequisites or retained consumers; edge orientation is preserved.
    pub scope: GraphScope,
    /// Maximum number of edges in a path, one through eight.
    pub depth: u32,
    /// Maximum admitted path-edge expansions, one through 4096.
    pub work: u32,
    /// Requested paths per frozen page, one through 64.
    pub limit: u32,
}
/// One occurrence-preserving path, including a closing cycle edge when observed.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
pub struct GraphPath {
    /// Nodes in traversal order; canonical edge orientation remains unchanged.
    pub nodes: Vec<GraphNode>,
    /// Original attributed edge occurrences, never deduplicated by endpoints.
    pub edges: Vec<GraphEdge>,
    /// Traversal stopped after revisiting a node already in this path.
    pub cycle: bool,
}
/// Paths from one native readonly snapshot; incompleteness is never closure.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
pub struct CanonicalGraphPathsDocument {
    /// Standalone path response interpretation.
    pub version: pse_model::document::Version<1>,
    /// Original finite question.
    pub request: CanonicalGraphPathsRequest,
    /// Selected root, preserving isolates.
    pub root: GraphNode,
    /// Ordered paths within the selected depth and expansion budget.
    pub paths: Vec<GraphPath>,
    /// Actual native path-edge expansions, not storage rows scanned.
    pub expanded: u32,
    /// Number of paths returned by earlier pages from this exact frozen owner.
    pub page_offset: u32,
    /// Paths still retained by this owner after this page; never a serializable cursor.
    pub remaining_paths: u32,
    /// This final page completes the owned sequence, including every prior page, within
    /// declared work/depth. The page alone does not establish dependency closure.
    pub complete: bool,
    /// Work or result bound that prevented completeness.
    pub incomplete_reason: Option<String>,
    /// Explicit result authority and temporal interpretation.
    pub evidence_scope: String,
    /// Existing prepaid response custody survives frozen pages and their reader.
    #[serde(skip)]
    #[schemars(skip)]
    _allocation: Option<crate::transport::client::ResponseAllocation>,
}
impl CanonicalGraphPathsRequest {
    fn validate(&self) -> Result<(), CanonicalError> {
        if !(1..=8).contains(&self.depth)
            || !(1..=4096).contains(&self.work)
            || !(1..=64).contains(&self.limit)
        {
            return Err(CanonicalError::PayloadLimit);
        }
        if !matches!(
            self.anchor,
            GraphAnchor::StudyPoint { .. } | GraphAnchor::AnalysisNode { .. }
        ) || self.scope == GraphScope::ImmediateBlockers
        {
            return Err(invalid(
                "path scope requires a study/analysis graph and explicit direction",
            ));
        }
        Ok(())
    }
    fn neighborhood(&self) -> CanonicalGraphRequest {
        CanonicalGraphRequest {
            version: pse_model::document::Version,
            anchor: self.anchor.clone(),
            scope: self.scope,
            limit: self.limit,
            continuation: None,
        }
    }
    fn body(&self) -> Result<String, CanonicalError> {
        self.validate()?;
        let (table, fields, owner, endpoint, next, root) = match (&self.anchor, self.scope) {
            (GraphAnchor::StudyPoint { .. }, scope) => (
                "canonical_study_dependencies",
                "id,in,out,key,study,dependent,predecessor,kind",
                "study=$graph_owner",
                if scope == GraphScope::RecordedSources {
                    "dependent"
                } else {
                    "predecessor"
                },
                if scope == GraphScope::RecordedSources {
                    "predecessor"
                } else {
                    "dependent"
                },
                "$point",
            ),
            (GraphAnchor::AnalysisNode { .. }, scope) => (
                "canonical_analysis_edges",
                "id,in,out,key,analysis,source,target,kind,evidence_artifact,evidence_block,evidence_row,evidence_field",
                "analysis=$analysis",
                if scope == GraphScope::RecordedSources {
                    "target"
                } else {
                    "source"
                },
                if scope == GraphScope::RecordedSources {
                    "source"
                } else {
                    "target"
                },
                "$node",
            ),
            _ => return Err(invalid("unsupported path owner")),
        };
        let mut sql = String::new();
        match self.anchor {
            GraphAnchor::StudyPoint{..}=>sql.push_str("LET $root=SELECT key,study,revision FROM ONLY type::record('canonical_study_points',$point); LET $owner=IF $root=NONE {NONE} ELSE {SELECT key,generation,active FROM ONLY type::record('canonical_studies',$root.study)}; IF $root=NONE OR $owner=NONE OR !$owner.active OR $root.revision!=$selected_revision OR $owner.generation!=$selected_generation {THROW 'selected graph point revision changed';}; fn::pse_history_v1::observe_owner($pse_rpc_expires_at,'study',$owner.key); LET $locator=SELECT study,run,issuing_store FROM ONLY type::record('canonical_history_locators','point:'+$point); IF $locator!=NONE {IF $locator.study!=$owner.key OR $locator.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store {THROW 'graph point history issuer differs';}; fn::pse_history_v1::observe_owner($pse_rpc_expires_at,'run',$locator.run);}; LET $graph_owner=$root.study; LET $frozen_root={key:$root.key,role:'study_point',revision:$root.revision};"),
            GraphAnchor::AnalysisNode{..}=>sql.push_str("LET $owner=SELECT key,revision,active,retiring FROM ONLY type::record('canonical_analyses',$analysis); LET $root=SELECT key,analysis,kind FROM ONLY type::record('canonical_analysis_nodes',$node); IF $owner=NONE OR !$owner.active OR $owner.retiring OR $owner.revision!=$selected_revision OR $root=NONE OR $root.analysis!=$analysis {THROW 'selected graph analysis node unavailable';}; LET $frozen_root={key:$root.key,role:$root.kind,revision:$owner.revision};"),
            _=>return Err(invalid("unsupported path owner")),
        }
        sql.push_str(&format!("LET $carry0={{paths:[],frontier:[{{leaf:{root},nodes:[{root}],edges:[],edge_keys:[]}}],used:0,truncated:false}};"));
        for depth in 1..=self.depth {
            let previous = depth - 1;
            sql.push_str(&format!(r#"
LET $edges{depth}=IF $carry{previous}.truncated OR array::len($carry{previous}.frontier)=0 {{[]}} ELSE {{SELECT {fields} FROM {table} WHERE {owner} AND {endpoint} IN $carry{previous}.frontier.leaf ORDER BY key LIMIT {sentinel}}};
LET $carry{depth}=$carry{previous}.frontier.fold({{paths:$carry{previous}.paths,frontier:[],used:$carry{previous}.used,truncated:$carry{previous}.truncated}},|$acc,$path| {{
    IF $acc.truncated {{RETURN $acc;}};
    LET $matches=$edges{depth}.filter(|$edge|$edge.{endpoint}=$path.leaf);
    LET $remaining=$work-$acc.used;
    LET $accepted=array::slice($matches,0,$remaining);
    LET $extended=$accepted.map(|$edge|{{leaf:$edge.{next},nodes:array::concat($path.nodes,[$edge.{next}]),edges:array::concat($path.edges,[$edge]),edge_keys:array::concat($path.edge_keys,[$edge.key]),cycle:$edge.{next} IN $path.nodes,depth:{depth}}});
    RETURN {{paths:array::concat($acc.paths,$extended),frontier:array::concat($acc.frontier,$extended.filter(|$item|!$item.cycle)),used:$acc.used+array::len($accepted),truncated:array::len($matches)>$remaining}};
}});
"#,sentinel=self.work+1));
        }
        sql.push_str(&format!(
            "LET $selected=SELECT * FROM $carry{}.paths ORDER BY depth,edge_keys LIMIT {};",
            self.depth, self.work
        ));
        Ok(sql)
    }
    fn output_body(&self) -> Result<String, CanonicalError> {
        Ok(format!(
            "{} LET $output=array::concat([{{kind:'summary',root:$frozen_root,used:$carry{}.used,truncated:$carry{}.truncated}}],$selected.map(|$path|{{kind:'path',path:$path}}));",
            self.body()?,
            self.depth,
            self.depth
        ))
    }
    fn query_sql(&self) -> Result<String, CanonicalError> {
        Ok(format!(
            "{} SELECT VALUE $this FROM $output;",
            self.output_body()?
        ))
    }
    fn bindings(&self) -> Result<BTreeMap<String, Value>, CanonicalError> {
        let mut bindings = self.neighborhood().bindings("");
        bindings.insert(
            "work".into(),
            Value::Number(surrealdb_types::Number::Int(i64::from(self.work))),
        );
        match &self.anchor {
            GraphAnchor::StudyPoint {
                revision,
                generation,
                ..
            } => {
                bindings.insert(
                    "selected_revision".into(),
                    crate::canonical_codec::encode_uint(*revision)?,
                );
                bindings.insert(
                    "selected_generation".into(),
                    crate::canonical_codec::encode_uint(*generation)?,
                );
            }
            GraphAnchor::AnalysisNode { revision, .. } => {
                bindings.insert("selected_revision".into(), Value::String(revision.clone()));
            }
            _ => return Err(invalid("unsupported path owner")),
        }
        Ok(bindings)
    }
    /// Same bounded SQL/bindings used by execution, suitable for passive plan inspection.
    pub fn description(&self) -> Result<serde_json::Value, CanonicalError> {
        Ok(
            serde_json::json!({"sql_body":self.query_sql()?,"bindings":self.bindings()?,"scope":"simple paths and closing cycle edges in one readonly snapshot; finite work and output caps"}),
        )
    }
    fn explained(&self, analyze: bool) -> Result<String, CanonicalError> {
        if analyze {
            // v4's ANALYZE lowerer needs a pipeline around imperative block expressions.
            // The complete same body stays inside EXPLAIN; no setup query executes first.
            Ok(format!(
                "EXPLAIN ANALYZE FORMAT JSON SELECT VALUE {{{} RETURN $output;}} FROM [0];",
                self.output_body()?
            ))
        } else {
            Ok(format!("EXPLAIN FORMAT JSON {{{}}};", self.query_sql()?))
        }
    }
}
impl CanonicalStore {
    /// Inspect the same complete block; static EXPLAIN does not execute its LET/subquery work.
    pub async fn inspect_graph_paths(
        &self,
        request: &CanonicalGraphPathsRequest,
        analyze: bool,
    ) -> Result<serde_json::Value, CanonicalError> {
        request.validate()?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {
            let bindings=request.bindings()?;
            let sql=request.explained(analyze)?;
            let mut response=protected_query("canonical_explanations::path_plan",||Ok(self.db.metadata_query(MetadataResponse::DiagnosticPlan,sql.clone()).bind(bindings.clone()))).await?;
            let server_version=response.server_version().map(str::to_owned);
            let captured=crate::query_explanation::NativeExplanationResponse::capture(&response,if analyze {crate::query_explanation::NativeExplanationResponseKind::ProfiledOperation}else{crate::query_explanation::NativeExplanationResponseKind::Inspection})?;
            let plan=response.take::<Value>(0)?;
            let mut explanation=crate::query_explanation::NativeQueryExplanation::new(
                crate::query_explanation::NativeExplanationIdentity {issuer:self.issuing_store()?.to_owned(),data_cut:serde_json::json!({"anchor":request.anchor,"depth":request.depth,"work":request.work}),server_version,server_recipe:None,server_invocation:None},serde_json::json!({"operation":request.description()?,"inspection_sql":sql}),&plan,analyze)?;
            explanation.response=Some(captured);
            Ok(serde_json::json!({"explanation":explanation,"raw_plan":plan,"scope":"native plan of the entire bounded path block; missing inner operator counters remain unknown"}))
        })).await
    }
    /// Read the first bounded page; use `begin_graph_paths` to retain continuation.
    pub async fn graph_paths(
        &self,
        request: &CanonicalGraphPathsRequest,
    ) -> Result<CanonicalGraphPathsDocument, CanonicalError> {
        self.begin_graph_paths(request)
            .await?
            .next_page()?
            .ok_or(CanonicalError::IncompleteResponse)
    }
    /// Freeze one finite native snapshot; its activity drains at EOF before publication.
    /// Later pages retain allocation and the original deadline, with no live read authority.
    pub async fn begin_graph_paths(
        &self,
        request: &CanonicalGraphPathsRequest,
    ) -> Result<GraphPathsRead, CanonicalError> {
        request.validate()?;
        let activity = self.admit_activity()?;
        let deadline = original_deadline(REQUEST_TIMEOUT);
        activity
            .scope(within_clock(deadline, async {
                let bindings = request.bindings()?;
                let sql = request.query_sql()?;
                let batch = MetadataResponse::GraphPaths.limits();
                let mut total = batch;
                // Every protobuf node has at least a tag and length/value byte; the cumulative
                // 4MiB transport cap is authoritative even for high-multiplicity bounded paths.
                total.nodes = total.wire_bytes / 2;
                let completed = self
                    .db
                    .metadata_read_query(MetadataResponse::GraphPaths, sql)
                    .bind(bindings)
                    .response_limits(total)
                    .batch_response_limits(batch)
                    .into_owned()
                    .consume_batches(PathBatches {
                        request: request.clone(),
                        root: None,
                        paths: Vec::with_capacity(request.work as usize),
                        summary: None,
                        retained: size_of::<GraphPathsRead>()
                            + (request.work as usize) * size_of::<GraphPath>(),
                    })
                    .await?;
                let staged = completed.into_consumer();
                let (expanded, truncated) =
                    staged.summary.ok_or(CanonicalError::IncompleteResponse)?;
                let root = staged.root.ok_or(CanonicalError::IncompleteResponse)?;
                if staged.paths.len() != expanded as usize {
                    return Err(CanonicalError::IncompleteResponse);
                }
                // Account every possible retained page header before publishing the owner.
                // Path/node/edge buffers move to pages; only root/request headers are copied.
                let pages = staged.paths.len().max(1).div_ceil(request.limit as usize);
                let headers = pages.saturating_mul(path_page_header_bytes(request, &root));
                if staged.retained.saturating_add(headers) > crate::canonical::MESSAGE_BYTES {
                    return Err(CanonicalError::PayloadLimit);
                }
                Ok(GraphPathsRead {
                    request: request.clone(),
                    root,
                    paths: staged.paths.into(),
                    expanded,
                    truncated,
                    emitted: false,
                    emitted_paths: 0,
                    deadline,
                    _allocation: crate::canonical::transport::current_response_owner(),
                })
            }))
            .await
    }
}
/// Caller-held continuation over a completely received bounded native snapshot.
/// The owner retains allocation and the original deadline after native work drains at EOF.
/// Frozen metadata conveys no live admission; no serializable cursor can recreate this cut.
#[derive(Debug)]
pub struct GraphPathsRead {
    request: CanonicalGraphPathsRequest,
    root: GraphNode,
    paths: std::collections::VecDeque<GraphPath>,
    expanded: u32,
    truncated: bool,
    emitted: bool,
    emitted_paths: u32,
    deadline: tokio::time::Instant,
    _allocation: Option<crate::transport::client::ResponseAllocation>,
}
impl GraphPathsRead {
    /// Return the next frozen page, refusing reuse beyond the original window.
    pub fn next_page(&mut self) -> Result<Option<CanonicalGraphPathsDocument>, CanonicalError> {
        if tokio::time::Instant::now() >= self.deadline {
            return Err(invalid(
                "original graph snapshot observation deadline expired",
            ));
        }
        if self.paths.is_empty() && self.emitted {
            return Ok(None);
        }
        self.emitted = true;
        let count = self.paths.len().min(self.request.limit as usize);
        let paths = self.paths.drain(..count).collect();
        let page_offset = self.emitted_paths;
        self.emitted_paths = self
            .emitted_paths
            .checked_add(u32::try_from(count).map_err(|_| CanonicalError::PayloadLimit)?)
            .ok_or(CanonicalError::PayloadLimit)?;
        let remaining_paths =
            u32::try_from(self.paths.len()).map_err(|_| CanonicalError::PayloadLimit)?;
        let more = remaining_paths != 0;
        Ok(Some(CanonicalGraphPathsDocument {version:pse_model::document::Version,request:self.request.clone(),root:self.root.clone(),paths,expanded:self.expanded,page_offset,remaining_paths,complete:!self.truncated&&!more,incomplete_reason:if self.truncated {Some("work_limit".into())}else if more {Some("result_limit".into())}else {None},evidence_scope:"one frozen native readonly snapshot; complete means the declared simple-path depth only, never full dependency closure or scientific eligibility; payload acquisition remains separate".into(),_allocation:self._allocation.clone()}))
    }
}
struct PathBatches {
    request: CanonicalGraphPathsRequest,
    root: Option<GraphNode>,
    paths: Vec<GraphPath>,
    summary: Option<(u32, bool)>,
    retained: usize,
}
impl crate::transport::client::BatchConsumer for PathBatches {
    fn consume<'a>(
        &'a mut self,
        batch: crate::transport::client::TentativeBatch<'a>,
    ) -> crate::transport::client::BatchFuture<'a> {
        Box::pin(async move {
            (|| -> Result<(), CanonicalError> {
                if batch.values.len() > 64 {
                    return Err(CanonicalError::PayloadLimit);
                }
                for value in batch.values {
                    if matches!(value, Value::None | Value::Null) {
                        continue;
                    }
                    let Value::Object(mut item) = value.clone() else {
                        return Err(CanonicalError::IncompleteResponse);
                    };
                    let kind = crate::canonical_codec::decode_string(
                        crate::canonical_codec::required(&mut item, "kind")?,
                    )?;
                    if kind == "summary" {
                        if self.summary.is_some() || !self.paths.is_empty() {
                            return Err(CanonicalError::IncompleteResponse);
                        }
                        let used = crate::canonical_codec::decode_uint(
                            crate::canonical_codec::required(&mut item, "used")?,
                        )?;
                        let truncated = crate::canonical_codec::decode_boolean(
                            crate::canonical_codec::required(&mut item, "truncated")?,
                        )?;
                        if used > u64::from(self.request.work) {
                            return Err(CanonicalError::PayloadLimit);
                        }
                        self.root = Some(decode_root(
                            &self.request,
                            crate::canonical_codec::required(&mut item, "root")?,
                        )?);
                        self.summary = Some((
                            u32::try_from(used).map_err(|_| CanonicalError::PayloadLimit)?,
                            truncated,
                        ));
                    } else if kind == "path" {
                        if self.summary.is_none() || self.paths.len() >= self.request.work as usize
                        {
                            return Err(CanonicalError::IncompleteResponse);
                        }
                        let path = decode_path(
                            &self.request,
                            self.root
                                .as_ref()
                                .ok_or(CanonicalError::IncompleteResponse)?,
                            crate::canonical_codec::required(&mut item, "path")?,
                        )?;
                        self.retained = self.retained.saturating_add(path_heap_bytes(&path));
                        if self.retained > crate::canonical::MESSAGE_BYTES {
                            return Err(CanonicalError::PayloadLimit);
                        }
                        self.paths.push(path);
                    } else {
                        return Err(CanonicalError::IncompleteResponse);
                    }
                }
                Ok(())
            })()
            .map_err(|error| surrealdb_types::Error::internal(error.to_string()))
        })
    }
}
fn decode_root(
    request: &CanonicalGraphPathsRequest,
    value: Value,
) -> Result<GraphNode, CanonicalError> {
    let Value::Object(mut root) = value else {
        return Err(CanonicalError::IncompleteResponse);
    };
    let key =
        crate::canonical_codec::decode_string(crate::canonical_codec::required(&mut root, "key")?)?;
    let role = crate::canonical_codec::decode_string(crate::canonical_codec::required(
        &mut root, "role",
    )?)?;
    let revision = crate::canonical_codec::required(&mut root, "revision")?;
    let (table, selected, revision) = match &request.anchor {
        GraphAnchor::StudyPoint {
            point,
            revision: expected,
            ..
        } => {
            let actual = crate::canonical_codec::decode_uint(revision)?;
            if actual != *expected {
                return Err(CanonicalError::IncompleteResponse);
            }
            ("canonical_study_points", point, actual.to_string())
        }
        GraphAnchor::AnalysisNode {
            node,
            revision: expected,
            ..
        } => {
            let actual = crate::canonical_codec::decode_string(revision)?;
            if actual != *expected {
                return Err(CanonicalError::IncompleteResponse);
            }
            ("canonical_analysis_nodes", node, actual)
        }
        _ => return Err(CanonicalError::IncompleteResponse),
    };
    if key != *selected {
        return Err(CanonicalError::IncompleteResponse);
    }
    Ok(GraphNode {
        identity: node(table, &key),
        role,
        revision: Some(revision),
    })
}
fn graph_node_heap_bytes(value: &GraphNode) -> usize {
    value
        .identity
        .capacity()
        .saturating_add(value.role.capacity())
        .saturating_add(value.revision.as_ref().map_or(0, String::capacity))
}
fn path_heap_bytes(path: &GraphPath) -> usize {
    let nodes = path.nodes.iter().fold(
        path.nodes.capacity() * size_of::<GraphNode>(),
        |bytes, node| bytes.saturating_add(graph_node_heap_bytes(node)),
    );
    path.edges.iter().fold(
        nodes.saturating_add(path.edges.capacity() * size_of::<GraphEdge>()),
        |bytes, edge| {
            bytes
                .saturating_add(edge.identity.capacity())
                .saturating_add(edge.source.capacity())
                .saturating_add(edge.target.capacity())
                .saturating_add(match &edge.role {
                    GraphEdgeRole::AnalysisRelation { relation } => relation.capacity(),
                    _ => 0,
                })
                .saturating_add(edge.evidence.as_ref().map_or(0, |evidence| {
                    evidence
                        .artifact
                        .capacity()
                        .saturating_add(evidence.field.as_ref().map_or(0, String::capacity))
                }))
        },
    )
}
fn path_page_header_bytes(request: &CanonicalGraphPathsRequest, root: &GraphNode) -> usize {
    let anchor = match &request.anchor {
        GraphAnchor::StudyPoint { point, .. } => point.capacity(),
        GraphAnchor::AnalysisNode {
            analysis,
            node,
            revision,
        } => analysis
            .capacity()
            .saturating_add(node.capacity())
            .saturating_add(revision.capacity()),
        _ => 0,
    };
    size_of::<CanonicalGraphPathsDocument>()
        .saturating_add(anchor)
        .saturating_add(graph_node_heap_bytes(root))
        .saturating_add(512)
        .saturating_add(request.limit as usize * size_of::<GraphPath>())
}
fn decode_path(
    request: &CanonicalGraphPathsRequest,
    root: &GraphNode,
    row: Value,
) -> Result<GraphPath, CanonicalError> {
    let Value::Object(mut row) = row else {
        return Err(CanonicalError::IncompleteResponse);
    };
    let cycle = crate::canonical_codec::decode_boolean(crate::canonical_codec::required(
        &mut row, "cycle",
    )?)?;
    let Value::Array(raw_nodes) = crate::canonical_codec::required(&mut row, "nodes")? else {
        return Err(CanonicalError::IncompleteResponse);
    };
    let Value::Array(raw_edges) = crate::canonical_codec::required(&mut row, "edges")? else {
        return Err(CanonicalError::IncompleteResponse);
    };
    if raw_edges.is_empty()
        || raw_edges.len() > request.depth as usize
        || raw_nodes.len() != raw_edges.len() + 1
    {
        return Err(CanonicalError::IncompleteResponse);
    }
    let keys = raw_nodes
        .into_iter()
        .map(crate::canonical_codec::decode_string)
        .collect::<Result<Vec<_>, _>>()?;
    let mut edges = Vec::new();
    for (index, edge) in raw_edges.into_iter().enumerate() {
        let Value::Object(edge) = edge else {
            return Err(CanonicalError::IncompleteResponse);
        };
        let mut question = request.neighborhood();
        match &mut question.anchor {
            GraphAnchor::StudyPoint { point, .. } => point.clone_from(&keys[index]),
            GraphAnchor::AnalysisNode { node, .. } => node.clone_from(&keys[index]),
            _ => return Err(CanonicalError::IncompleteResponse),
        }
        edges.push(decode_edge(&question, edge)?.1);
    }
    let table = if matches!(request.anchor, GraphAnchor::StudyPoint { .. }) {
        "canonical_study_points"
    } else {
        "canonical_analysis_nodes"
    };
    let nodes = keys
        .into_iter()
        .enumerate()
        .map(|(index, key)| {
            if index == 0 {
                root.clone()
            } else {
                GraphNode {
                    identity: node(table, &key),
                    role: "recorded_neighbor".into(),
                    revision: None,
                }
            }
        })
        .collect();
    Ok(GraphPath {
        nodes,
        edges,
        cycle,
    })
}

#[cfg(all(test, feature = "canonical-maintenance"))]
#[allow(
    clippy::unwrap_used,
    reason = "parser controls require the complete checked native statement to compile"
)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn bounded_path_pages_refuse_expired_original_window() {
        let request = CanonicalGraphPathsRequest {
            version: pse_model::document::Version,
            anchor: GraphAnchor::AnalysisNode {
                analysis: "analysis".into(),
                node: "node".into(),
                revision: "revision".into(),
            },
            scope: GraphScope::RecordedSources,
            depth: 1,
            work: 1,
            limit: 1,
        };
        let mut read = GraphPathsRead {
            request,
            root: GraphNode {
                identity: "node".into(),
                role: "root".into(),
                revision: None,
            },
            paths: std::collections::VecDeque::new(),
            expanded: 0,
            truncated: false,
            emitted: false,
            emitted_paths: 0,
            deadline: tokio::time::Instant::now(),
            _allocation: None,
        };
        assert!(read.next_page().is_err());
    }
    #[test]
    fn bounded_path_and_nonexecuting_explain_blocks_parse_as_native_statements() {
        for scope in [GraphScope::RecordedSources, GraphScope::RetainedConsumers] {
            let request = CanonicalGraphPathsRequest {
                version: pse_model::document::Version,
                anchor: GraphAnchor::AnalysisNode {
                    analysis: "analysis".into(),
                    node: "node".into(),
                    revision: "revision".into(),
                },
                scope,
                depth: 8,
                work: 4096,
                limit: 64,
            };
            for analyzed in [false, true] {
                let sql = request.explained(analyzed).unwrap();
                surrealdb_expr::parser::parse(&sql).unwrap();
                assert!(sql.starts_with(if analyzed {
                    "EXPLAIN ANALYZE FORMAT JSON SELECT VALUE {"
                } else {
                    "EXPLAIN FORMAT JSON {"
                }));
            }
            let mut too_deep = request;
            too_deep.depth = 9;
            assert!(too_deep.body().is_err());
        }
    }
}
