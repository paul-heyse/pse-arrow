// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Semantic producers and selected source synchronization; no parallel semantic graph.
use super::{
    Answer, Configuration, Error, Kind, Query, Result, Section, service::SessionStatus, transport,
};
use crate::source_read::{SourceRequest, SourceText};
use lspf::{
    ServerHandle,
    types::{self, request::Request},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

pub(super) struct Engine {
    pub(super) server: ServerHandle,
    pub(super) evidence: Arc<Mutex<transport::Evidence>>,
    pub(super) child: transport::ChildOwner,
    pub(super) driver: tokio::task::JoinHandle<lspf::Result<lspf::Outcome>>,
    pub(super) configuration: Configuration,
    pub(super) backend: String,
    generation: AtomicU64,
    active: AtomicUsize,
    documents: Mutex<BTreeMap<PathBuf, (String, i32)>>,
    context: Arc<super::context::Context>,
    encoding: String,
    configuration_exchange: Arc<transport::ConfigurationExchange>,
}
pub(super) fn protocol(error: impl std::fmt::Display) -> Error {
    Error::Protocol(error.to_string())
}
pub(super) async fn request<R: Request>(server: &ServerHandle, params: Value) -> Result<Value> {
    let params = serde_json::from_value::<R::Params>(params)?;
    let answer = server
        .request::<R>(params)
        .await
        .map_err(|error| match error {
            lspf::ClientError::Cancelled => Error::Cancelled,
            lspf::ClientError::Remote(error) => Error::Refusal(super::Failure {
                phase: super::FailurePhase::Provider,
                observed_cancellation: None,
                state: match error.code {
                    -32800 | -32802 => super::FailureState::Cancelled,
                    -32801 => super::FailureState::Stale,
                    -32601 => super::FailureState::Unsupported,
                    -32002 => super::FailureState::Unavailable,
                    _ => super::FailureState::Failed,
                },
                message: error.message,
                backend_code: Some(error.code),
                backend_data: error.data,
            }),
            error => protocol(error),
        })?;
    Ok(serde_json::to_value(answer)?)
}
fn notify<N: types::notification::Notification>(
    server: &ServerHandle,
    params: Value,
) -> Result<()> {
    server
        .notify::<N>(serde_json::from_value(params)?)
        .map_err(protocol)
}
macro_rules! extension {
    ($name:ident, $method:literal) => {
        enum $name {}
        impl Request for $name {
            type Params = Value;
            type Result = Value;
            const METHOD: &'static str = $method;
        }
    };
}
extension!(ExpandMacro, "rust-analyzer/expandMacro");
extension!(RelatedTests, "rust-analyzer/relatedTests");
extension!(Ssr, "experimental/ssr");
extension!(Predicate, "rust-analyzer/evaluatePredicate");
extension!(AnalyzerStatus, "rust-analyzer/analyzerStatus");
enum NativeServerStatus {}
impl types::notification::Notification for NativeServerStatus {
    type Params = Value;
    const METHOD: &'static str = "experimental/serverStatus";
}
// The pinned analyzer adds snippet edits/group fields beyond standard CodeAction.
// Retain these as native evidence instead of losing them in a standard-only decode.
extension!(NativeActions, "textDocument/codeAction");
extension!(NativeActionResolve, "codeAction/resolve");

pub(super) fn uri(path: &Path) -> Result<String> {
    url::Url::from_file_path(path)
        .map(String::from)
        .map_err(|()| Error::Protocol("source path cannot be represented as a file URI".into()))
}
impl Engine {
    pub(super) async fn start(
        checkout: &Path,
        configuration: Configuration,
        stderr: std::fs::File,
    ) -> Result<Self> {
        // Capture effective Cargo inputs and install preparation watches before any analyzer
        // loading/build effects. Quiescence never discards intervening input changes.
        let captured_context =
            Arc::new(super::context::Context::capture(checkout, &configuration)?);
        let version = std::process::Command::new("rust-analyzer")
            .arg("--version")
            .current_dir(checkout)
            .output()?;
        if !version.status.success() {
            return Err(Error::Unavailable(
                String::from_utf8_lossy(&version.stderr).into(),
            ));
        }
        let backend = String::from_utf8_lossy(&version.stdout).trim().to_string();
        let mut command = std::process::Command::new("rust-analyzer");
        command.current_dir(checkout);
        let capabilities = serde_json::from_value(json!({
            "general":{"positionEncodings":["utf-8","utf-16"]},
            "workspace":{"configuration":true,"applyEdit":false,"workspaceEdit":{"documentChanges":true,"resourceOperations":["create","rename","delete"]},"fileOperations":{"willRename":true}},
            "textDocument":{"definition":{"linkSupport":true},"typeDefinition":{"linkSupport":true},"implementation":{"linkSupport":true},"hover":{"contentFormat":["plaintext","markdown"]},"completion":{"completionItem":{"snippetSupport":false,"resolveSupport":{"properties":["documentation","detail","additionalTextEdits"]}}},"codeAction":{"dataSupport":true,"codeActionLiteralSupport":{"codeActionKind":{"valueSet":["quickfix","refactor","refactor.extract","refactor.inline","refactor.rewrite"]}},"resolveSupport":{"properties":["edit"]}},"inlayHint":{"resolveSupport":{"properties":["tooltip","textEdits","label.tooltip","label.location"]}}},
            "experimental":{"serverStatusNotification":true}
        }))?;
        let mut build_scripts = json!({"enable":configuration.build_macros,"rebuildOnSave":false});
        if configuration.build_macros {
            let entry = crate::generation::EntryGuard::acquire_current()
                .map_err(|error| Error::Host(Box::new(error)))?;
            let control = entry.prepared().ok_or_else(|| {
                Error::Unavailable(
                    "macro preparation requires the original immutable native generation".into(),
                )
            })?;
            let adapter = control
                .executable(checkout, crate::generation::HostTool::Correctness)
                .map_err(|error| Error::Host(Box::new(error)))?;
            let adapter = adapter.to_str().ok_or_else(|| {
                Error::Unavailable(
                    "Cargo build-script override requires a Unicode executable path".into(),
                )
            })?;
            let mut arguments = vec![
                adapter.to_string(),
                "check".into(),
                "--workspace".into(),
                "--all-targets".into(),
                "--message-format=json".into(),
                "--locked".into(),
            ];
            if !configuration.features.is_empty() {
                arguments.extend(["--features".into(), configuration.features.join(",")]);
            }
            if configuration.no_default_features {
                arguments.push("--no-default-features".into());
            }
            if let Some(target) = &configuration.target {
                arguments.extend(["--target".into(), target.clone()]);
            }
            build_scripts["overrideCommand"] = json!(arguments);
        }
        let mut cargo = json!({"autoreload":true,"buildScripts":build_scripts, "features":configuration.features, "noDefaultFeatures":configuration.no_default_features});
        if let Some(target) = &configuration.target {
            cargo["target"] = json!(target);
        }
        let options = json!({
            "linkedProjects":[configuration.project.join("Cargo.toml")], "checkOnSave":false,
            "cargo":cargo,"procMacro":{"enable":configuration.build_macros}
        });
        let configuration_exchange =
            Arc::new(transport::ConfigurationExchange::new(options.clone()));
        let (transport, child, evidence) = transport::spawn(
            &mut command,
            uri(&configuration.project)?,
            stderr,
            configuration_exchange.clone(),
        )?;
        let response_configuration = configuration_exchange.clone();
        let activation_configuration = configuration_exchange.clone();
        let activation_context = captured_context.clone();
        let client = lspf::Client::builder(capabilities)
            .initialization_options(options)
            .request::<types::ConfigurationRequest, _, _>(move |_, params, _| {
                let configuration = response_configuration.clone();
                async move {
                    let options = configuration
                        .options
                        .lock()
                        .map_err(|error| lspf::LspError::Internal(error.to_string()))?
                        .clone();
                    Ok(params
                        .items
                        .iter()
                        .map(|item| {
                            if item.section.as_deref() == Some("rust-analyzer") {
                                options.clone()
                            } else {
                                Value::Null
                            }
                        })
                        .collect())
                }
            })
            .notification::<NativeServerStatus, _, _>(move |context, params| {
                let configuration = activation_configuration.clone();
                let captured_context = activation_context.clone();
                async move {
                    if params["quiescent"] != true
                        || configuration
                            .phase
                            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                            .is_err()
                    {
                        return;
                    }
                    let result = async {
                        {
                            configuration.options.lock().map_err(protocol)?["cargo"]["autoreload"] =
                                json!(false);
                        }
                        let server = context.server();
                        // Pinned RA ignores notification parameters and fetches workspace/configuration.
                        notify::<types::DidChangeConfigurationNotification>(
                            &server,
                            json!({"settings":null}),
                        )?;
                        while configuration.phase.load(Ordering::SeqCst) != 2 {
                            configuration.written.notified().await;
                        }
                        // The adapter observed the actual config response write. This ordered roundtrip
                        // follows RA's synchronous config-response callback, rather than assuming an ack.
                        request::<AnalyzerStatus>(&server, json!({"textDocument":null})).await?;
                        captured_context.seal()?;
                        configuration.phase.store(3, Ordering::SeqCst);
                        Ok::<(), Error>(())
                    }
                    .await;
                    if let Err(error) = result {
                        if let Ok(mut issue) = configuration.issue.lock() {
                            *issue = Some(error.to_string());
                        }
                        configuration.phase.store(4, Ordering::SeqCst);
                    }
                }
            })
            .build(transport)
            .map_err(protocol)?;
        let connection = tokio::time::timeout(Duration::from_secs(45), client.connect())
            .await
            .map_err(protocol)?
            .map_err(protocol)?;
        let server = connection.server();
        let driver = tokio::spawn(connection.serve());
        let encoding = {
            let state = evidence.lock().map_err(protocol)?;
            let result = state.result.as_ref().ok_or_else(|| {
                Error::Protocol("actual initialize result was not retained".into())
            })?;
            serde_json::to_value(&result.capabilities.position_encoding)?
                .as_str()
                .unwrap_or("utf-16")
                .to_string()
        };
        if encoding != "utf-8" && encoding != "utf-16" {
            return Err(Error::Unavailable(format!(
                "unimplemented negotiated encoding {encoding}"
            )));
        }
        Ok(Self {
            server,
            evidence,
            child,
            driver,
            configuration,
            backend,
            generation: AtomicU64::new(1),
            active: AtomicUsize::new(0),
            documents: Mutex::new(BTreeMap::new()),
            context: captured_context,
            encoding,
            configuration_exchange,
        })
    }
    pub(super) fn status(&self) -> Result<SessionStatus> {
        let context = self.check_context();
        let evidence = self.evidence.lock().map_err(protocol)?;
        let result = evidence
            .result
            .as_ref()
            .ok_or_else(|| Error::Protocol("initialize result missing".into()))?;
        Ok(SessionStatus {
            configuration: self.configuration.clone(), backend: self.backend.clone(),
            position_encoding: self.encoding.clone(), generation: self.generation.load(Ordering::SeqCst),
            server_capabilities: serde_json::to_value(&result.capabilities)?,
            backend_status: evidence.status.clone(),
            active_requests:self.active.load(Ordering::SeqCst),
            context_current:context.is_ok(),context_issue:context.err().map(|error|error.to_string()),
            automatic_reload_disabled:self.configuration_exchange.phase.load(Ordering::SeqCst)==3,
            preparation_issue:self.configuration_exchange.issue.lock().map_err(protocol)?.clone(),
            preparation: if self.configuration.build_macros { "explicit check --workspace --all-targets through original correctness adapter; automatic checking and build-script rerun-on-save disabled. Native package-tree and Cargo-declared rerun-input watches fence preparation; any input/watch change requires stop and explicit prepare. Undeclared external build-script/proc-macro I/O is unknown, not exact compiler-input freshness." } else { "restricted: automatic checking, build scripts and proc macros disabled" }.into(),
        })
    }
    fn check_context(&self) -> Result<()> {
        self.context.check()
    }
    fn synchronize(&self, source: &SourceText) -> Result<u64> {
        self.check_context()?;
        let path = source.identity().path().to_owned();
        let uri = uri(&path)?;
        let digest = source.identity().source_sha256().to_string();
        let mut documents = self.documents.lock().map_err(protocol)?;
        match documents.get(&path) {
            Some((old, _)) if old == &digest => {}
            Some((_, version)) => {
                let version = version
                    .checked_add(1)
                    .ok_or_else(|| Error::Protocol("document version exhausted".into()))?;
                notify::<types::DidChangeTextDocumentNotification>(
                    &self.server,
                    json!({"textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":source.text()}]}),
                )?;
                documents.insert(path, (digest, version));
                self.generation.fetch_add(1, Ordering::SeqCst);
            }
            None => {
                notify::<types::DidOpenTextDocumentNotification>(
                    &self.server,
                    json!({"textDocument":{"uri":uri,"languageId":"rust","version":1,"text":source.text()}}),
                )?;
                documents.insert(path, (digest, 1));
                self.generation.fetch_add(1, Ordering::SeqCst);
            }
        }
        Ok(self.generation.load(Ordering::SeqCst))
    }
    fn refresh_open_documents(&self) -> Result<()> {
        let paths: Vec<PathBuf> = self
            .documents
            .lock()
            .map_err(protocol)?
            .keys()
            .cloned()
            .collect();
        for path in paths {
            match (SourceRequest::Working { path: path.clone() }).acquire() {
                Ok(source) => {
                    self.synchronize(&source)?;
                }
                Err(crate::source_read::SourceReadError::Io(error))
                    if error.kind() == std::io::ErrorKind::NotFound =>
                {
                    notify::<types::DidCloseTextDocumentNotification>(
                        &self.server,
                        json!({"textDocument":{"uri":uri(&path)?}}),
                    )?;
                    notify::<types::DidChangeWatchedFilesNotification>(
                        &self.server,
                        json!({"changes":[{"uri":uri(&path)?,"type":3}]}),
                    )?;
                    self.documents.lock().map_err(protocol)?.remove(&path);
                    self.generation.fetch_add(1, Ordering::SeqCst);
                }
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
    fn check_generation(&self, generation: u64) -> Result<()> {
        self.check_context()?;
        if self.generation.load(Ordering::SeqCst) != generation {
            return Err(Error::Stale);
        }
        for (path, (digest, _)) in self.documents.lock().map_err(protocol)?.iter() {
            if file_digest(path)?.as_ref() != Some(digest) {
                return Err(Error::Stale);
            }
        }
        Ok(())
    }
    pub(super) async fn workspace_symbols(
        &self,
        query: super::WorkspaceQuery,
    ) -> Result<super::WorkspaceAnswer> {
        if query.query.contains("::") {
            return Err(Error::Unavailable("The selected LSP workspace-symbol protocol does not establish native qualified module-path matching; select an explicit native snapshot for this query.".into()));
        }
        if query.include_dependencies {
            return Err(Error::Unavailable("The selected LSP workspace-symbol protocol does not establish dependency-root search coverage; select a prepared native snapshot for that explicit domain.".into()));
        }
        struct Active<'a>(&'a Engine);
        impl Drop for Active<'_> {
            fn drop(&mut self) {
                self.0.release_query();
            }
        }
        self.active.fetch_add(1, Ordering::SeqCst);
        let _active = Active(self);
        if self.configuration_exchange.phase.load(Ordering::SeqCst) != 3 {
            return Err(Error::Unavailable(
                "explicit analyzer preparation is pending".into(),
            ));
        }
        self.refresh_open_documents()?;
        self.check_context()?;
        let status = self.status()?;
        if status
            .backend_status
            .as_ref()
            .and_then(|value| value["quiescent"].as_bool())
            == Some(false)
        {
            return Err(Error::Unavailable(
                "analyzer workspace reload is pending".into(),
            ));
        }
        // An unopened native candidate has no byte binding yet. Synchronize the
        // discovered files through this same owner and repeat the native request;
        // never attach a post-query filesystem digest to an earlier native range.
        let mut bound = None;
        for _ in 0..4 {
            let generation = self.generation.load(Ordering::SeqCst);
            if query
                .if_generation
                .is_some_and(|expected| expected != generation)
            {
                return Err(Error::Stale);
            }
            let mut native = request::<types::WorkspaceSymbolRequest>(
                &self.server,
                json!({"query":query.query}),
            )
            .await?;
            let mut snapshots = BTreeMap::new();
            for symbol in native.as_array().into_iter().flatten() {
                let Some(uri) = symbol.pointer("/location/uri").and_then(Value::as_str) else {
                    continue;
                };
                let Ok(path) = url::Url::parse(uri).map_err(protocol)?.to_file_path() else {
                    continue;
                };
                if snapshots.contains_key(&path) {
                    continue;
                }
                let source = SourceRequest::Working { path: path.clone() }.acquire()?;
                self.synchronize(&source)?;
                snapshots.insert(path, source.identity().source_sha256().to_owned());
            }
            if generation != self.generation.load(Ordering::SeqCst) {
                continue;
            }
            self.check_generation(generation)?;
            for symbol in native.as_array_mut().into_iter().flatten() {
                let digest = symbol
                    .pointer("/location/uri")
                    .and_then(Value::as_str)
                    .and_then(|uri| url::Url::parse(uri).ok())
                    .and_then(|uri| uri.to_file_path().ok())
                    .and_then(|path| snapshots.get(&path));
                if let (Some(digest), Some(object)) = (digest, symbol.as_object_mut()) {
                    object.insert("source_sha256".into(), json!(digest));
                }
            }
            bound = Some((generation, native));
            break;
        }
        let (generation,native)=bound.ok_or_else(||Error::Unavailable("native symbol candidates changed while binding exact source; repeat the query against a stable selected context".into()))?;
        Ok(super::WorkspaceAnswer {
            session: super::AnalysisContext {
                backend: status.backend,
                configuration: status.configuration,
                position_encoding: status.position_encoding,
                backend_status: status.backend_status,
                preparation: status.preparation,
            },
            generation,
            observed: super::protocol_facts::symbols(&native, None).into_iter().take(query.limit).collect(),
            symbols: section("workspace_symbols", native, query.limit),
            limitations: vec![
                "Native workspace-symbol candidates retain provider ranking and configuration; document symbols and rustdoc item IDs are not interchangeable identities.".into(),
                "Backend search may be incomplete and cannot select an exact Cargo compiler-unit instance. No compilation or semantic preparation was started by this query.".into(),
            ],
        })
    }
    pub(super) async fn query(&self, query: Query) -> Result<Answer> {
        struct Active<'a>(&'a Engine);
        impl Drop for Active<'_> {
            fn drop(&mut self) {
                self.0.release_query();
            }
        }
        self.active.fetch_add(1, Ordering::SeqCst);
        let _active = Active(self);
        let mut query = query;
        for _ in 0..4 {
            if let Some(answer) = self.query_once(query.clone()).await? {
                return Ok(answer);
            }
            // Native result inputs were added through this same owner. The
            // original input source guard is unchanged; fence the expanded world.
            query.if_generation = Some(self.generation.load(Ordering::SeqCst));
        }
        Err(Error::Unavailable(
            "native location sources did not stabilize; repeat the selected query".into(),
        ))
    }
    async fn query_once(&self, query: Query) -> Result<Option<Answer>> {
        if self.configuration_exchange.phase.load(Ordering::SeqCst) != 3 {
            return Err(Error::Unavailable("explicit initial preparation or post-activation configuration is pending; inspect session status".into()));
        }
        self.refresh_open_documents()?;
        if self
            .evidence
            .lock()
            .map_err(protocol)?
            .status
            .as_ref()
            .and_then(|status| status["quiescent"].as_bool())
            == Some(false)
        {
            return Err(Error::Unavailable("analyzer workspace preparation/reload is pending; inspect code session status and reacquire".into()));
        }
        let file = match query.file.canonicalize() {
            Ok(file) => file,
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound && query.if_source.is_some() =>
            {
                return Err(Error::Stale);
            }
            Err(error) => return Err(error.into()),
        };
        if !file.starts_with(&self.configuration.project) {
            return Err(Error::Unavailable(
                "source is outside selected project root".into(),
            ));
        }
        let source = SourceRequest::Working { path: file.clone() }.acquire()?;
        if query
            .if_source
            .as_ref()
            .is_some_and(|expected| expected != source.identity().source_sha256())
        {
            return Err(Error::Stale);
        }
        if query
            .if_generation
            .is_some_and(|expected| expected != self.generation.load(Ordering::SeqCst))
        {
            return Err(Error::Stale);
        }
        let mut generation = self.synchronize(&source)?;
        let position = position(source.text(), query.line, query.column, &self.encoding)?;
        let params = json!({"textDocument":{"uri":uri(&file)?},"position":position});
        let end_line = query.end_line.unwrap_or(query.line);
        if end_line < query.line {
            return Err(Error::Source(
                crate::source_read::SourceReadError::InvalidRange,
            ));
        }
        let line_text = source
            .text()
            .split('\n')
            .nth(end_line - 1)
            .ok_or(Error::Source(
                crate::source_read::SourceReadError::InvalidRange,
            ))?;
        let range = json!({"start":{"line":query.line-1,"character":0},"end":self::position(source.text(),end_line,line_text.strip_suffix('\r').unwrap_or(line_text).chars().count()+1,&self.encoding)?});
        let mut sections = Vec::new();
        let mut proposal = None;
        let limit = query.limit;
        match &query.kind {
            Kind::Importable { .. } => {
                return Err(Error::unsupported(
                    "the selected live LSP does not expose native visibility/import-path facts; explicitly select a prepared native snapshot",
                ));
            }
            Kind::Explain => {
                let (hover, definition, type_definition) = tokio::join!(
                    request::<types::HoverRequest>(&self.server, params.clone()),
                    request::<types::DefinitionRequest>(&self.server, params.clone()),
                    request::<types::TypeDefinitionRequest>(&self.server, params.clone())
                );
                sections.push(section("hover", hover?, limit));
                sections.push(section("definition", definition?, limit));
                sections.push(section("type_definition", type_definition?, limit));
                if query.end_line.is_some() {
                    sections.push(section(
                        "selected_inlay_annotations",
                        request::<types::InlayHintRequest>(
                            &self.server,
                            json!({"textDocument":{"uri":uri(&file)?},"range":range}),
                        )
                        .await?,
                        limit,
                    ));
                }
            }
            Kind::Refs => {
                let mut params = params.clone();
                params["context"] = json!({"includeDeclaration":true});
                let references = request::<types::ReferencesRequest>(&self.server, params).await?;
                let total = references.as_array().map_or(0, Vec::len);
                let mut files = BTreeMap::<String, Vec<Value>>::new();
                if let Some(references) = references.as_array() {
                    for reference in references.iter().take(limit) {
                        let file = reference["uri"]
                            .as_str()
                            .ok_or_else(|| protocol("native reference has no URI"))?;
                        files
                            .entry(file.into())
                            .or_default()
                            .push(reference.clone());
                    }
                }
                sections.push(Section{category:"references_by_file".into(),evidence:json!({"files":files.into_iter().map(|(uri,references)|json!({"uri":uri,"references":references})).collect::<Vec<_>>(),"total_references":total,"shown_references":total.min(limit)}),omitted:total.saturating_sub(limit),handles:vec![]});
            }
            Kind::Implementations => sections.push(section(
                "implementations",
                request::<types::ImplementationRequest>(&self.server, params).await?,
                limit,
            )),
            Kind::Symbols => sections.push(section(
                "document_symbols",
                request::<types::DocumentSymbolRequest>(
                    &self.server,
                    json!({"textDocument":{"uri":uri(&file)?}}),
                )
                .await?,
                limit,
            )),
            Kind::Macro => sections.push(section(
                "macro_expansion",
                request::<ExpandMacro>(&self.server, params).await?,
                limit,
            )),
            Kind::Tests => sections.push(section(
                "related_test_candidates",
                request::<RelatedTests>(&self.server, params).await?,
                limit,
            )),
            Kind::Candidates => sections.push(section(
                "completion_candidates",
                request::<types::CompletionRequest>(&self.server, params).await?,
                limit,
            )),
            Kind::Candidate { index } => {
                if query.if_source.is_none() || query.if_generation.is_none() {
                    return Err(Error::InvalidRequest(
                        "completion resolution requires --if-source and --if-generation".into(),
                    ));
                }
                let items = request::<types::CompletionRequest>(&self.server, params).await?;
                let items = items
                    .as_array()
                    .or_else(|| items.get("items").and_then(Value::as_array));
                let item = items
                    .and_then(|items| items.get(*index))
                    .ok_or_else(|| Error::Unavailable("completion candidate is absent".into()))?;
                check_candidate(&query, item)?;
                sections.push(section(
                    "resolved_completion",
                    request::<types::CompletionResolveRequest>(&self.server, item.clone()).await?,
                    limit,
                ));
            }
            Kind::Layout => {
                return Err(Error::unsupported(
                    "structured layout requires an explicitly selected native snapshot",
                ));
            }
            Kind::Predicate { text } => {
                let mut params = params.clone();
                params["text"] = json!(text);
                sections.push(section(
                    "predicate_evaluation",
                    request::<Predicate>(&self.server, params).await?,
                    limit,
                ));
            }
            Kind::Hints | Kind::Hint { .. } => {
                let hints = request::<types::InlayHintRequest>(
                    &self.server,
                    json!({"textDocument":{"uri":uri(&file)?},"range":range}),
                )
                .await?;
                if let Kind::Hint { index } = query.kind {
                    if query.if_source.is_none() || query.if_generation.is_none() {
                        return Err(Error::InvalidRequest(
                            "lazy hint resolution requires --if-source and --if-generation".into(),
                        ));
                    }
                    let hint = hints
                        .as_array()
                        .and_then(|hints| hints.get(index))
                        .ok_or_else(|| Error::Unavailable("selected hint is absent".into()))?;
                    check_candidate(&query, hint)?;
                    sections.push(section(
                        "resolved_hint",
                        request::<types::InlayHintResolveRequest>(&self.server, hint.clone())
                            .await?,
                        limit,
                    ));
                } else {
                    sections.push(section("selected_inlay_annotations", hints, limit));
                }
            }
            Kind::Ssr { rule } => {
                let params = json!({"query":rule,"parseOnly":false,"textDocument":{"uri":uri(&file)?},"position":position,"selections":[range]});
                let first = request::<Ssr>(&self.server, params.clone()).await?;
                self.prepare_edit_inputs(&first)?;
                generation = self.generation.load(Ordering::SeqCst);
                let edit = request::<Ssr>(&self.server, params).await?;
                proposal = Some(self.proposal(&edit, generation)?);
            }
            Kind::Calls => {
                let items =
                    request::<types::CallHierarchyPrepareRequest>(&self.server, params).await?;
                if let Some(items) = items.as_array() {
                    for item in items.iter().take(limit) {
                        let (incoming, outgoing) = tokio::join!(
                            request::<types::CallHierarchyIncomingCallsRequest>(
                                &self.server,
                                json!({"item":item})
                            ),
                            request::<types::CallHierarchyOutgoingCallsRequest>(
                                &self.server,
                                json!({"item":item})
                            )
                        );
                        sections.push(section("incoming_calls_one_hop", incoming?, limit));
                        sections.push(section("outgoing_calls_one_hop", outgoing?, limit));
                    }
                }
                sections.push(section("call_hierarchy_roots", items, limit));
            }
            Kind::Rename { name } => {
                let prepared =
                    request::<types::PrepareRenameRequest>(&self.server, params.clone()).await?;
                if prepared.is_null() {
                    return Err(Error::Unavailable(
                        "selected position cannot be renamed".into(),
                    ));
                }
                let mut params = params.clone();
                params["newName"] = json!(name);
                let first = request::<types::RenameRequest>(&self.server, params.clone()).await?;
                self.prepare_edit_inputs(&first)?;
                generation = self.generation.load(Ordering::SeqCst);
                let edit = request::<types::RenameRequest>(&self.server, params).await?;
                proposal = Some(self.proposal(&edit, generation)?);
                sections.push(section("prepare_rename", prepared, limit));
            }
            Kind::FileRename { destination } => {
                let destination = if destination.is_absolute() {
                    destination.clone()
                } else {
                    self.configuration.project.join(destination)
                };
                let params = json!({"files":[{"oldUri":uri(&file)?,"newUri":uri(&destination)?}]});
                let first =
                    request::<types::WillRenameFilesRequest>(&self.server, params.clone()).await?;
                self.prepare_edit_inputs(&first)?;
                generation = self.generation.load(Ordering::SeqCst);
                let mut edit =
                    request::<types::WillRenameFilesRequest>(&self.server, params).await?;
                if edit.is_null() {
                    edit = json!({});
                }
                if edit["documentChanges"].is_null() {
                    let mut documents = Vec::new();
                    if let Some(changes) = edit["changes"].as_object() {
                        for (uri, edits) in changes {
                            documents.push(
                                json!({"textDocument":{"uri":uri,"version":null},"edits":edits}),
                            );
                        }
                    }
                    edit = json!({"documentChanges":documents});
                }
                edit["documentChanges"]
                    .as_array_mut()
                    .ok_or_else(|| protocol("file rename edits have no ordered document changes"))?
                    .push(
                        json!({"kind":"rename","oldUri":uri(&file)?,"newUri":uri(&destination)?}),
                    );
                proposal = Some(self.proposal(&edit, generation)?);
            }
            Kind::Assists | Kind::Assist { .. } => {
                let params = json!({"textDocument":{"uri":uri(&file)?},"range":{"start":position,"end":position},"context":{"diagnostics":[]}});
                let actions = request::<NativeActions>(&self.server, params.clone()).await?;
                if let Kind::Assist { index } = query.kind {
                    if query.if_source.is_none() || query.if_generation.is_none() {
                        return Err(Error::InvalidRequest("selected assist requires --if-source and --if-generation from discovery".into()));
                    }
                    let action = actions
                        .as_array()
                        .and_then(|items| items.get(index))
                        .ok_or_else(|| {
                            Error::Unavailable("selected assist index is absent".into())
                        })?;
                    check_candidate(&query, action)?;
                    let resolved = if action.get("edit").is_some() {
                        action.clone()
                    } else {
                        request::<NativeActionResolve>(&self.server, action.clone()).await?
                    };
                    let edit = resolved.get("edit").ok_or_else(|| {
                        Error::Unavailable(
                            "assist has no workspace edit; command execution is not supported"
                                .into(),
                        )
                    })?;
                    self.prepare_edit_inputs(edit)?;
                    generation = self.generation.load(Ordering::SeqCst);
                    let current = request::<NativeActions>(&self.server, params).await?;
                    let matching: Vec<&Value> = current
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|candidate| {
                            if action["data"]["id"].is_string() {
                                candidate["data"]["id"] == action["data"]["id"]
                            } else {
                                candidate["title"] == action["title"]
                            }
                        })
                        .collect();
                    if matching.len() != 1 {
                        return Err(Error::Stale);
                    }
                    let action = matching[0];
                    let resolved = if action.get("edit").is_some() {
                        action.clone()
                    } else {
                        request::<NativeActionResolve>(&self.server, action.clone()).await?
                    };
                    let edit = resolved.get("edit").ok_or_else(|| {
                        Error::Unavailable("resolved assist has no workspace edit".into())
                    })?;
                    proposal = Some(self.proposal(edit, generation)?);
                    sections.push(section("selected_assist", resolved, limit));
                } else {
                    sections.push(section("assist_candidates", actions, limit));
                }
            }
        }
        let mut target_paths = std::collections::BTreeSet::new();
        for section in &sections {
            if matches!(
                section.category.as_str(),
                "definition"
                    | "type_definition"
                    | "implementations"
                    | "references_by_file"
                    | "incoming_calls_one_hop"
                    | "outgoing_calls_one_hop"
                    | "call_hierarchy_roots"
            ) {
                collect_location_paths(&section.evidence, &mut target_paths)?;
            }
        }
        let mut location_sources = Vec::new();
        for path in target_paths {
            let selected = SourceRequest::Working { path }.acquire()?;
            self.synchronize(&selected)?;
            location_sources.push(super::LocationSource {
                path: selected.identity().path().into(),
                source_sha256: selected.identity().source_sha256().into(),
            });
        }
        if generation != self.generation.load(Ordering::SeqCst) {
            return Ok(None);
        }
        self.check_generation(generation)?;
        if (SourceRequest::Working { path: file.clone() })
            .acquire()?
            .identity()
            != source.identity()
        {
            return Err(Error::Stale);
        }
        let status = self.status()?;
        let session = super::AnalysisContext {
            backend: status.backend,
            configuration: status.configuration,
            position_encoding: status.position_encoding,
            backend_status: status.backend_status,
            preparation: status.preparation,
        };
        let available = proposal.as_ref().is_some_and(|proposal| {
            !proposal.candidates().is_empty() || !proposal.file_operations().is_empty()
        }) || sections.iter().any(|section| match &section.evidence {
            Value::Null => false,
            Value::Array(items) => !items.is_empty(),
            Value::Object(object) => object
                .get("total_references")
                .and_then(Value::as_u64)
                .map_or_else(
                    || {
                        object
                            .get("items")
                            .and_then(Value::as_array)
                            .is_none_or(|items| !items.is_empty())
                    },
                    |total| total > 0,
                ),
            _ => true,
        });
        let source_uri = uri(&file)?;
        let (observed, omitted_relationships) =
            super::protocol_facts::project(&sections, &source_uri)?;
        Ok(Some(Answer { session, source: source.identity().clone(),source_uri,source_byte_offset:crate::source_read::byte_offset(source.text(),query.line,query.column)?,availability:if available{super::Availability::Available}else{super::Availability::NoMatch}, generation, position:serde_json::from_value(position)?, observed,omitted_relationships,
            native_types:None,
            sections, location_sources,proposal,
            limitations: vec![
                "Analyzer evidence is not Cargo/Clippy/borrow-check acceptance. Macro/build-script preparation is explicit in session context; missing or failed preparation can make answers incomplete.".into(),
                "The analyzer selects Cargo cfg/test contexts; LSP cannot select an exact compiler-unit/crate instance. Ambiguous same-file contexts remain backend-selected.".into(),
                "Generation fences selected open documents and effective Cargo/workspace inputs; macro preparation additionally refuses package-tree or declared rerun-input changes. Undeclared external build-script/proc-macro I/O is unknown. Native watchers own other source invalidation; this is not an atomic cross-file filesystem snapshot.".into(),
                "Typed captures, adjustments and reference categories are unavailable through the pinned protocol. Macro expansion has invocation fallback, not an exact expansion source map.".into(),
                "Indirect dispatch, callback/FFI edges and universal trait applicability are not established. Test/completion candidates are non-exhaustive; no analyzer-produced command is executed.".into(),
            ] }))
    }
    fn release_query(&self) {
        // Keep this session's original source/generation history. Discarding it
        // at idle would let changed targets reuse an old semantic generation.
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
    fn prepare_edit_inputs(&self, edit: &Value) -> Result<()> {
        for path in super::edit::input_paths(edit, &self.configuration.project)? {
            self.synchronize(&SourceRequest::Working { path }.acquire()?)?;
        }
        Ok(())
    }
    fn proposal(
        &self,
        edit: &Value,
        generation: u64,
    ) -> Result<crate::source_proposal::ChangeProposal> {
        self.check_generation(generation)?;
        let proposal = super::edit::proposal(
            &self.configuration,
            &self.backend,
            generation,
            &self.encoding,
            edit,
            |path, version| {
                let source = SourceRequest::Working {
                    path: path.to_owned(),
                }
                .acquire()?;
                if self
                    .documents
                    .lock()
                    .map_err(protocol)?
                    .get(source.identity().path())
                    .is_none_or(|(digest, current)| {
                        digest != source.identity().source_sha256()
                            || version.is_some_and(|version| version != i64::from(*current))
                    })
                {
                    return Err(Error::Stale);
                }
                Ok(source)
            },
        )?;
        self.check_generation(generation)?;
        Ok(proposal)
    }
    pub(super) async fn shutdown(mut self) -> Result<()> {
        let mut failures = Vec::new();
        match tokio::time::timeout(Duration::from_secs(10), self.server.shutdown()).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => failures.push(format!("shutdown: {error}")),
            Err(error) => failures.push(format!("shutdown deadline: {error}")),
        }
        if let Err(error) = self.server.exit() {
            failures.push(format!("exit: {error}"));
        }
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        loop {
            match self.child.0.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        failures.push(format!("analyzer exit: {status}"));
                    }
                    break;
                }
                Ok(None) if tokio::time::Instant::now() < deadline => {
                    tokio::time::sleep(Duration::from_millis(25)).await
                }
                waiting => {
                    if let Err(error) = waiting {
                        failures.push(format!("observe original analyzer: {error}"));
                    }
                    if let Err(error) = self.child.0.kill() {
                        failures.push(format!("kill original analyzer: {error}"));
                    }
                    if let Err(error) = self.child.0.wait() {
                        failures.push(format!("reap original analyzer: {error}"));
                    }
                    failures.push("analyzer did not exit gracefully before deadline".into());
                    break;
                }
            }
        }
        match tokio::time::timeout(Duration::from_secs(10), &mut self.driver).await {
            Ok(Ok(Ok(_))) => {}
            Ok(Ok(Err(error))) => failures.push(format!("transport driver: {error}")),
            Ok(Err(error)) => failures.push(format!("transport task: {error}")),
            Err(error) => {
                failures.push(format!("transport task deadline: {error}"));
                self.driver.abort();
                let _ = self.driver.await;
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(protocol(failures.join("; ")))
        }
    }
}
pub(super) fn file_digest(path: &Path) -> Result<Option<String>> {
    match std::fs::read(path) {
        Ok(bytes) => {
            use sha2::Digest as _;
            Ok(Some(
                sha2::Sha256::digest(bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
fn collect_location_paths(
    value: &Value,
    paths: &mut std::collections::BTreeSet<PathBuf>,
) -> Result<()> {
    match value {
        Value::Object(object) => {
            for key in ["uri", "targetUri"] {
                if let Some(uri) = object.get(key).and_then(Value::as_str) {
                    let uri = url::Url::parse(uri).map_err(protocol)?;
                    if let Ok(path) = uri.to_file_path() {
                        paths.insert(path.canonicalize()?);
                    }
                }
            }
            for child in object.values() {
                collect_location_paths(child, paths)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_location_paths(item, paths)?;
            }
        }
        _ => {}
    }
    Ok(())
}
pub(super) fn section(category: &str, mut evidence: Value, limit: usize) -> Section {
    let records = if evidence.is_array() {
        evidence.as_array_mut()
    } else {
        evidence.get_mut("items").and_then(Value::as_array_mut)
    };
    let omitted = if let Some(records) = records {
        let omitted = records.len().saturating_sub(limit);
        records.truncate(limit);
        omitted
    } else {
        0
    };
    let records = evidence
        .as_array()
        .or_else(|| evidence.get("items").and_then(Value::as_array));
    let handles = if matches!(
        category,
        "completion_candidates" | "assist_candidates" | "selected_inlay_annotations"
    ) {
        records
            .into_iter()
            .flatten()
            .enumerate()
            .filter_map(|(index, record)| {
                serde_json::to_string(record)
                    .ok()
                    .map(|record| super::CandidateHandle {
                        index,
                        fingerprint: crate::source_read::source_digest(&record),
                    })
            })
            .collect()
    } else {
        vec![]
    };
    Section {
        category: category.into(),
        evidence,
        omitted,
        handles,
    }
}
pub(super) fn check_candidate(query: &Query, candidate: &Value) -> Result<()> {
    let expected=query.if_candidate.as_ref().ok_or_else(||Error::InvalidRequest("lazy selection requires --if-candidate fingerprint from discovery; an index is not an identity".into()))?;
    if expected != &crate::source_read::source_digest(&serde_json::to_string(candidate)?) {
        return Err(Error::Stale);
    }
    Ok(())
}
pub(super) fn position(text: &str, line: usize, column: usize, encoding: &str) -> Result<Value> {
    let offset = crate::source_read::byte_offset(text, line, column)?;
    let start = text[..offset].rfind('\n').map_or(0, |at| at + 1);
    let character = if encoding == "utf-8" {
        offset - start
    } else {
        text[start..offset].encode_utf16().count()
    };
    Ok(json!({"line":line-1,"character":character}))
}
pub(super) fn byte_position(text: &str, position: &Value, encoding: &str) -> Result<usize> {
    byte_protocol_position(text, &serde_json::from_value(position.clone())?, encoding)
}
pub(super) fn byte_protocol_position(
    text: &str,
    position: &super::ProtocolPosition,
    encoding: &str,
) -> Result<usize> {
    let line = usize::try_from(position.line).map_err(protocol)?;
    let character = usize::try_from(position.character).map_err(protocol)?;
    let start = crate::source_read::byte_offset(text, line + 1, 1)?;
    let raw = text[start..].split('\n').next().unwrap_or("");
    let line_text = raw.strip_suffix('\r').unwrap_or(raw);
    let mut units = 0;
    for (byte, ch) in line_text.char_indices() {
        if units == character {
            return Ok(start + byte);
        }
        units += if encoding == "utf-8" {
            ch.len_utf8()
        } else {
            ch.len_utf16()
        };
        if units > character {
            return Err(Error::Source(
                crate::source_read::SourceReadError::InvalidRange,
            ));
        }
    }
    if units == character {
        Ok(start + line_text.len())
    } else {
        Err(Error::Source(
            crate::source_read::SourceReadError::InvalidRange,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scalar_positions_roundtrip_negotiated_encodings_and_crlf() {
        let source = "a😀é\r\nβz\n";
        assert_eq!(
            position(source, 1, 3, "utf-8").unwrap(),
            json!({"line":0,"character":5})
        );
        assert_eq!(
            position(source, 1, 3, "utf-16").unwrap(),
            json!({"line":0,"character":3})
        );
        for encoding in ["utf-8", "utf-16"] {
            for (line, column) in [(1, 1), (1, 2), (1, 3), (1, 4), (2, 1), (2, 2), (2, 3)] {
                let position = position(source, line, column, encoding).unwrap();
                assert_eq!(
                    byte_position(source, &position, encoding).unwrap(),
                    crate::source_read::byte_offset(source, line, column).unwrap()
                );
            }
        }
        assert!(byte_position(source, &json!({"line":0,"character":2}), "utf-16").is_err());
    }
    #[test]
    fn budgets_keep_native_records_indivisible() {
        let answer = section(
            "references",
            json!([{"range":{"start":{"line":1,"character":2},"end":{"line":1,"character":8}}},{"uri":"second"}]),
            1,
        );
        assert_eq!(answer.omitted, 1);
        assert_eq!(answer.evidence[0]["range"]["end"]["character"], 8);
    }
    #[test]
    fn lazy_candidate_identity_refuses_changed_or_reordered_native_records() {
        let records = json!([{"label":"first","position":{"line":1,"character":2}},{"label":"second","position":{"line":1,"character":4}}]);
        let discovery = section("selected_inlay_annotations", records.clone(), 2);
        let query = Query {
            kind: Kind::Hint { index: 0 },
            file: PathBuf::new(),
            line: 2,
            column: 3,
            if_source: Some("source".into()),
            if_generation: Some(1),
            if_candidate: Some(discovery.handles[0].fingerprint.clone()),
            limit: 2,
            end_line: None,
        };
        assert!(check_candidate(&query, &records[0]).is_ok());
        assert!(matches!(
            check_candidate(&query, &records[1]),
            Err(Error::Stale)
        ));
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn real_analyzer_resolves_unicode_entities_previews_edits_and_survives_cancel() {
        let fixture = tempfile::tempdir().unwrap();
        std::fs::create_dir(fixture.path().join("src")).unwrap();
        std::fs::write(
            fixture.path().join("Cargo.toml"),
            "[package]\nname='semantic-control'\nversion='0.0.0'\nedition='2024'\n[workspace]\n",
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("Cargo.lock"),
            "version = 4\n[[package]]\nname = \"semantic-control\"\nversion = \"0.0.0\"\n",
        )
        .unwrap();
        let original = "pub struct Left;\npub struct Right;\nimpl Left { pub fn same(&self) -> u32 { 1 } }\nimpl Right { pub fn same(&self) -> u32 { 2 } }\npub fn use_both() -> u32 { let café = Left; café.same() + Right.same() }\n#[test] fn checks() { assert_eq!(use_both(), 3); }\n";
        let file = fixture.path().join("src/lib.rs");
        std::fs::write(&file, original).unwrap();
        let configuration = Configuration {
            backend: super::super::Backend::Lsp,
            project: fixture.path().to_owned(),
            features: vec![],
            no_default_features: false,
            target: None,
            build_macros: false,
            test_context: false,
        };
        let checkout = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let engine = Engine::start(
            &checkout,
            configuration,
            std::fs::File::create(fixture.path().join("backend.stderr")).unwrap(),
        )
        .await
        .unwrap();
        let line = original.lines().nth(4).unwrap();
        let column = line[..line.find(".same").unwrap() + 1].chars().count() + 1;
        let query = |kind| Query {
            kind,
            file: file.clone(),
            line: 5,
            column,
            if_source: None,
            if_generation: None,
            if_candidate: None,
            limit: 12,
            end_line: None,
        };
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        let answer = loop {
            match engine.query(query(Kind::Explain)).await {
                Ok(answer)
                    if answer
                        .sections
                        .iter()
                        .find(|section| section.category == "definition")
                        .unwrap()
                        .evidence
                        .as_array()
                        .is_some_and(|items| !items.is_empty()) =>
                {
                    break answer;
                }
                result => {
                    assert!(
                        tokio::time::Instant::now() < deadline,
                        "analyzer never resolved fixture: {result:?}"
                    );
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        };
        let definition = &answer
            .sections
            .iter()
            .find(|section| section.category == "definition")
            .unwrap()
            .evidence;
        assert_eq!(definition[0]["targetSelectionRange"]["start"]["line"], 2);
        let renamed = engine
            .query(query(Kind::Rename {
                name: "left_only".into(),
            }))
            .await
            .unwrap();
        let proposal = renamed.proposal.unwrap();
        assert_eq!(proposal.selected().len(), 2);
        assert!(
            proposal
                .candidates()
                .iter()
                .all(|candidate| candidate.replacement() == "left_only")
        );
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            original,
            "preview must not apply writes"
        );
        let pid = rustix::process::Pid::from_child(&engine.child.0);
        rustix::process::kill_process(pid, rustix::process::Signal::STOP).unwrap();
        let canceled = tokio::time::timeout(
            Duration::from_millis(50),
            request::<types::HoverRequest>(
                &engine.server,
                json!({"textDocument":{"uri":uri(&file).unwrap()},"position":answer.position}),
            ),
        )
        .await;
        rustix::process::kill_process(pid, rustix::process::Signal::CONT).unwrap();
        assert!(
            canceled.is_err(),
            "pending wire request should be abandoned"
        );
        assert!(
            engine.query(query(Kind::Explain)).await.is_ok(),
            "request cancellation must preserve session"
        );
        std::fs::write(
            &file,
            original.replace("let café = Left", "let café = Right"),
        )
        .unwrap();
        let mut old = query(Kind::Explain);
        old.if_source = Some(answer.source.source_sha256().into());
        assert!(matches!(engine.query(old).await, Err(Error::Stale)));
        let fresh = loop {
            let mut current = query(Kind::Explain);
            current.column += 1;
            match engine.query(current).await {
                Ok(answer) => break answer,
                Err(_) => {
                    assert!(tokio::time::Instant::now() < deadline);
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        };
        assert!(fresh.generation > answer.generation);
        let definition = &fresh
            .sections
            .iter()
            .find(|section| section.category == "definition")
            .unwrap()
            .evidence;
        assert_eq!(definition[0]["targetSelectionRange"]["start"]["line"], 3);
        std::fs::write(
            fixture.path().join("Cargo.toml"),
            "[package]\nname='changed-context'\nversion='0.0.0'\nedition='2024'\n[workspace]\n",
        )
        .unwrap();
        assert!(matches!(
            engine.query(query(Kind::Explain)).await,
            Err(Error::Stale)
        ));
        engine.shutdown().await.unwrap();
    }
}
