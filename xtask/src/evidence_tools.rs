// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit tooling-only evidence lifecycle. Opening never initializes scientific state.
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use pse_operations::evidence::{
    self, EvidenceAccess, EvidenceQuery, EvidenceSnapshot, EvidenceStore, ToolingOptions,
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
#[derive(Debug, Args)]
pub(crate) struct EvidenceArgs {
    /// Selected service state; only its private tooling configuration is opened normally.
    #[arg(long)]
    state: Option<PathBuf>,
    /// Remaining original command allowance, shared by every stage and retry.
    #[arg(long, default_value_t = 30000)]
    timeout_ms: u64,
    #[command(subcommand)]
    command: EvidenceCommand,
}
#[derive(Debug, Subcommand)]
enum EvidenceCommand {
    /// Activate a complete explicitly selected development-knowledge corpus.
    KnowledgeImport {
        #[arg(long)]
        input: PathBuf,
    },
    /// Query full-text excerpts or native graph edges from one exact corpus cut.
    KnowledgeQuery {
        #[arg(long)]
        input: PathBuf,
    },
    /// Explain the same bound native full-text or graph item query.
    KnowledgeExplain {
        #[arg(long)]
        input: PathBuf,
    },
    /// Attest the independent knowledge pointer and generated schema.
    KnowledgeStatus,
    /// Remove only derived knowledge rows, preserving evidence and original sources.
    KnowledgeReset,
    /// Supplied-only query; dispatches before deployment configuration or service activity.
    Offline {
        #[arg(long, required = true)]
        input: Vec<PathBuf>,
        #[arg(long)]
        question: PathBuf,
        #[arg(long)]
        history: bool,
        #[arg(long)]
        observations: Option<PathBuf>,
    },
    /// Compare explicitly supplied source cuts without configuration or a service.
    CompareOffline {
        #[arg(long, required = true)]
        input: Vec<PathBuf>,
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        observations: Option<PathBuf>,
    },
    /// Compare exact activated generations with the same native paired-evidence kernel.
    CompareIndexed {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        observations: Option<PathBuf>,
    },
    /// Ask the same finite question over explicit activated multi-source generations.
    Analyze {
        #[arg(long, required = true)]
        generation: Vec<String>,
        #[arg(long)]
        question: PathBuf,
        #[arg(long)]
        history: bool,
        #[arg(long)]
        observations: Option<PathBuf>,
    },
    /// Explicit root-only initialization of the disposable tooling database.
    Init,
    /// Attest tooling interpretation and report the imported source cut.
    Status,
    /// Import original-owner adapted typed metadata, then activate reconciled membership.
    Import {
        #[arg(long)]
        input: PathBuf,
    },
    /// Query a bounded page from the current complete imported generation.
    Query {
        #[arg(long)]
        generation: Option<String>,
        #[arg(long, default_value_t = 64)]
        limit: usize,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        cursor: Option<PathBuf>,
    },
    /// Discover bounded prior activated and interrupted source cuts.
    Generations {
        #[arg(long, default_value_t = 64)]
        limit: usize,
        #[arg(long)]
        after: Option<String>,
    },
    /// Explicitly drop and reinitialize the derived index; original owners are unchanged.
    Rebuild,
    /// Explicitly drop only the selected disposable database.
    Drop,
}
fn root_credentials(state: &Path) -> Result<(String, String)> {
    #[derive(serde::Deserialize)]
    struct Deployment {
        credentials_file: String,
    }
    #[derive(serde::Deserialize)]
    struct RootCredentials {
        username: String,
        password: String,
    }
    fn read(path: &Path) -> Result<Vec<u8>> {
        let m = std::fs::symlink_metadata(path)?;
        anyhow::ensure!(
            m.is_file() && !m.file_type().is_symlink() && m.len() <= 262144,
            "bounded real credentials required"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            anyhow::ensure!(m.mode() & 0o077 == 0, "credentials must be private");
        }
        Ok(std::fs::read(path)?)
    }
    let d: Deployment = serde_json::from_slice(&read(&state.join("config.json"))?)?;
    let path = Path::new(&d.credentials_file);
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        state.join(path)
    };
    anyhow::ensure!(
        path.parent() == Some(state),
        "service credentials must be a direct private file"
    );
    let c: RootCredentials = serde_json::from_slice(&read(&path)?)?;
    Ok((c.username, c.password))
}
fn activity_guard(state: &Path, options: &ToolingOptions) -> Result<std::fs::File> {
    use std::os::unix::fs::{FileExt, MetadataExt};
    let fd = std::env::var("PSE_TOOLING_ACTIVITY_FD")
        .context("run through just evidence so its actual service activity remains owned")?
        .parse::<u32>()?;
    anyhow::ensure!(fd >= 3, "invalid inherited service activity descriptor");
    let link = PathBuf::from(format!("/proc/self/fd/{fd}"));
    let path = std::fs::read_link(&link)?;
    let expected = state.canonicalize()?.join(".tooling/.activities");
    let stem = path
        .file_stem()
        .and_then(|v| v.to_str())
        .context("invalid activity name")?;
    anyhow::ensure!(
        path.parent() == Some(expected.as_path())
            && path.extension().is_some_and(|v| v == "json")
            && stem.len() == 32
            && stem
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "foreign inherited activity path"
    );
    let inherited = std::fs::File::open(link)?;
    let m = inherited.metadata()?;
    let actual = std::fs::metadata(&path)?;
    let uid = std::fs::metadata("/proc/self")?.uid();
    anyhow::ensure!(
        m.is_file()
            && m.dev() == actual.dev()
            && m.ino() == actual.ino()
            && m.mode() & 0o077 == 0
            && m.uid() == uid
            && m.len() <= 65536,
        "invalid inherited activity ownership"
    );
    let mut bytes = vec![0; m.len().try_into()?];
    inherited.read_exact_at(&mut bytes, 0)?;
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Activity {
        version: u32,
        service_instance_id: String,
    }
    let record: Activity = serde_json::from_slice(&bytes)?;
    anyhow::ensure!(
        record.version == 1
            && record.service_instance_id == options.service_instance_id
            && std::env::var("PSE_TOOLING_SERVICE_INSTANCE_ID")
                .ok()
                .as_deref()
                == Some(options.service_instance_id.as_str()),
        "foreign service activity issuing identity"
    );
    let probe = std::fs::File::open(path)?;
    anyhow::ensure!(
        matches!(probe.try_lock(), Err(std::fs::TryLockError::WouldBlock)),
        "activity descriptor does not retain its original lifetime lock"
    );
    Ok(inherited)
}
pub(crate) fn run(args: EvidenceArgs) -> Result<()> {
    anyhow::ensure!(args.timeout_ms > 0, "positive original allowance required");
    let deadline = Instant::now()
        .checked_add(Duration::from_millis(args.timeout_ms))
        .context("original allowance overflow")?;
    // Offline is a first-class path, before state/config/credentials/service activity.
    if let EvidenceCommand::Offline {
        input,
        question,
        history,
        observations,
    } = &args.command
    {
        let snapshots = input
            .iter()
            .map(|path| pse_operations::evidence_query::EvidenceAnalysis::read_snapshot(path))
            .collect::<Result<Vec<_>, _>>()?;
        let mut analysis =
            pse_operations::evidence_query::EvidenceAnalysis::new(snapshots, *history)?;
        let question = read_question(question)?;
        let observations = read_observations(observations.as_deref())?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let output = runtime.block_on(evidence::within_allowance(
            deadline,
            analysis.ask(&question, &observations),
        ))?;
        println!("{}", serde_json::to_string(&output)?);
        return Ok(());
    }
    if let EvidenceCommand::CompareOffline { input, request, observations } = &args.command {
        let snapshots = input.iter()
            .map(|path| pse_operations::evidence_query::EvidenceAnalysis::read_snapshot(path))
            .collect::<Result<Vec<_>, _>>()?;
        let request = read_comparison_request(request)?;
        let observations = read_observations(observations.as_deref())?;
        let mut analysis = pse_operations::evidence_query::EvidenceAnalysis::new(snapshots, true)?;
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        let output = runtime.block_on(evidence::within_allowance(deadline, analysis.compare(&request, &observations)))?;
        println!("{}", serde_json::to_string(&output)?);
        return Ok(());
    }
    let state = args
        .state
        .as_deref()
        .context("--state is required for service operations")?;
    let options = ToolingOptions::from_state(state)?;
    let _activity = activity_guard(state, &options)?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let output = rt.block_on(evidence::within_allowance(deadline, async {
        match args.command {
            EvidenceCommand::Init => {
                let (user, password) = root_credentials(state)
                    .map_err(|e| evidence::EvidenceError::Configuration(e.to_string()))?;
                EvidenceStore::initialize(&options, &user, &password).await?;
                Ok(serde_json::json!({"initialized":true}))
            }
            EvidenceCommand::Drop | EvidenceCommand::Rebuild => {
                let rebuild = matches!(args.command, EvidenceCommand::Rebuild);
                let (user, password) = root_credentials(state)
                    .map_err(|e| evidence::EvidenceError::Configuration(e.to_string()))?;
                EvidenceStore::drop_database(&options, &user, &password).await?;
                if rebuild {
                    EvidenceStore::initialize(&options, &user, &password).await?;
                }
                Ok(serde_json::json!({"dropped":true,"rebuilt":rebuild}))
            }
            command => {
                let role = if matches!(command, EvidenceCommand::Import { .. } | EvidenceCommand::KnowledgeImport { .. } | EvidenceCommand::KnowledgeReset) {
                    EvidenceAccess::Importer
                } else {
                    EvidenceAccess::Reader
                };
                let store = EvidenceStore::open(&options, role).await?;
                let result = async {
                    match command {
                        EvidenceCommand::KnowledgeStatus => store.knowledge_status().await,
                        EvidenceCommand::KnowledgeReset => {
                            store.knowledge_reset().await?;
                            Ok(serde_json::json!({"knowledge_reset":true,"original_sources_untouched":true}))
                        }
                        EvidenceCommand::KnowledgeImport { input } => {
                            let bytes = supplied_document(&input, 3 * 1024 * 1024).map_err(|cause| evidence::EvidenceError::Configuration(cause.to_string()))?;
                            let corpus: pse_operations::knowledge::KnowledgeCorpus = serde_json::from_slice(&bytes).map_err(|cause| evidence::EvidenceError::Configuration(cause.to_string()))?;
                            serde_json::to_value(store.knowledge_import(&corpus).await?).map_err(|cause| evidence::EvidenceError::Configuration(cause.to_string()))
                        }
                        EvidenceCommand::KnowledgeQuery { input } => {
                            let bytes = supplied_document(&input, 32768).map_err(|cause| evidence::EvidenceError::Configuration(cause.to_string()))?;
                            let request: pse_operations::knowledge::KnowledgeQuery = serde_json::from_slice(&bytes).map_err(|cause| evidence::EvidenceError::Configuration(cause.to_string()))?;
                            serde_json::to_value(store.knowledge_select(&request).await?).map_err(|cause| evidence::EvidenceError::Configuration(cause.to_string()))
                        }
                        EvidenceCommand::KnowledgeExplain { input } => {
                            let bytes = supplied_document(&input, 32768).map_err(|cause| evidence::EvidenceError::Configuration(cause.to_string()))?;
                            let request: pse_operations::knowledge::KnowledgeQuery = serde_json::from_slice(&bytes).map_err(|cause| evidence::EvidenceError::Configuration(cause.to_string()))?;
                            store.knowledge_explain(&request).await
                        }
                        EvidenceCommand::Status => store.status().await,
                        EvidenceCommand::Analyze {
                            generation,
                            question,
                            history,
                            observations,
                        } => {
                            let sources = store.source_cut(&generation, history).await?;
                            let mut analysis =
                                pse_operations::evidence_query::EvidenceAnalysis::new(
                                    sources, history,
                                )?;
                            let question = read_question(&question).map_err(|e| {
                                evidence::EvidenceError::Configuration(e.to_string())
                            })?;
                            let observations =
                                read_observations(observations.as_deref()).map_err(|e| {
                                    evidence::EvidenceError::Configuration(e.to_string())
                                })?;
                            analysis.ask(&question, &observations).await
                        }
                        EvidenceCommand::CompareIndexed { request, observations } => {
                            let request = read_comparison_request(&request).map_err(|e| evidence::EvidenceError::Configuration(e.to_string()))?;
                            let generations = request.baseline.generations.iter().chain(&request.candidate.generations).cloned().collect::<std::collections::BTreeSet<_>>().into_iter().collect::<Vec<_>>();
                            let snapshots = store.source_cut(&generations, true).await?;
                            let observations = read_observations(observations.as_deref()).map_err(|e| evidence::EvidenceError::Configuration(e.to_string()))?;
                            let mut analysis = pse_operations::evidence_query::EvidenceAnalysis::new(snapshots, true)?;
                            serde_json::to_value(analysis.compare(&request, &observations).await?).map_err(|e| evidence::EvidenceError::Configuration(e.to_string()))
                        }
                        EvidenceCommand::Import { input } => {
                            let metadata = std::fs::metadata(&input).map_err(|e| {
                                evidence::EvidenceError::Configuration(e.to_string())
                            })?;
                            if !metadata.is_file() || metadata.len() > 4 * 1024 * 1024 {
                                return Err(evidence::EvidenceError::Configuration(
                                    "bounded snapshot file required".into(),
                                ));
                            }
                            let bytes = std::fs::read(input).map_err(|e| {
                                evidence::EvidenceError::Configuration(e.to_string())
                            })?;
                            let snapshot: EvidenceSnapshot = serde_json::from_slice(&bytes)
                                .map_err(|e| {
                                    evidence::EvidenceError::Configuration(e.to_string())
                                })?;
                            serde_json::to_value(store.import(&snapshot).await?)
                                .map_err(|e| evidence::EvidenceError::Configuration(e.to_string()))
                        }
                        EvidenceCommand::Generations { limit, after } => {
                            store.generations(after.as_deref(), limit).await
                        }
                        EvidenceCommand::Query {
                            generation,
                            limit,
                            kind,
                            status,
                            cursor,
                        } => {
                            let cursor = cursor
                                .map(|path| {
                                    let m = std::fs::metadata(&path).map_err(|e| {
                                        evidence::EvidenceError::Configuration(e.to_string())
                                    })?;
                                    if !m.is_file() || m.len() > 16384 {
                                        return Err(evidence::EvidenceError::Configuration(
                                            "bounded cursor file required".into(),
                                        ));
                                    }
                                    let bytes = std::fs::read(path).map_err(|e| {
                                        evidence::EvidenceError::Configuration(e.to_string())
                                    })?;
                                    serde_json::from_slice(&bytes).map_err(|e| {
                                        evidence::EvidenceError::Configuration(e.to_string())
                                    })
                                })
                                .transpose()?;
                            serde_json::to_value(
                                store
                                    .select(&EvidenceQuery {
                                        generation,
                                        kind,
                                        status,
                                        limit: Some(limit),
                                        cursor,
                                    })
                                    .await?,
                            )
                            .map_err(|e| evidence::EvidenceError::Configuration(e.to_string()))
                        }
                        _ => Err(evidence::EvidenceError::Configuration(
                            "unsupported command".into(),
                        )),
                    }
                }
                .await;
                let drained = store.disconnect().await;
                let output = result?;
                drained?;
                Ok(output)
            }
        }
    }))?;
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

fn read_question(path: &Path) -> Result<pse_operations::evidence_query::EvidenceQuestion> {
    let bytes = supplied_document(path, 65536)?;
    Ok(serde_json::from_slice(&bytes)?)
}
fn read_observations(
    path: Option<&Path>,
) -> Result<Vec<pse_operations::evidence_query::EvidenceSourceObservation>> {
    path.map(|path| Ok(serde_json::from_slice(&supplied_document(path, 262144)?)?))
        .transpose()
        .map(|value| value.unwrap_or_default())
}
fn supplied_document(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let metadata = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() <= limit,
        "bounded explicit document required"
    );
    Ok(std::fs::read(path)?)
}

fn read_comparison_request(path: &Path) -> Result<pse_operations::evidence_query::EvidenceComparisonRequest> {
    Ok(pse_operations::evidence_query::EvidenceComparisonRequest::decode_json(&supplied_document(path, 4 << 20)?)?)
}
