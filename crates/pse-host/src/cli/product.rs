// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opaque product delegation; scientific request types remain in the selected product.
use super::{GenerationIntent, HostError, OutputOptions, checkout_root};
use crate::{
    envelope::{self, Envelope},
    environment::Environment,
    product_reader::{self, Contract, ProductReaderSelection},
};
use clap::{Subcommand, ValueEnum};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::value::RawValue;
use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Explicitly build and capture a product reader; never performed by passive reads.
    Prepare,
    /// Discover the operations and schema associations of one captured reader.
    Discover {
        #[arg(long)]
        reader: String,
    },
    /// Read a directional schema from the selected product, without reconstructing it.
    Schema {
        #[arg(long)]
        reader: String,
        #[arg(value_enum)]
        contract: Direction,
    },
    /// Delegate a product-owned JSON request; discover/schema describe its accepted grammar.
    Query {
        #[arg(long)]
        reader: String,
        /// Existing JSON file containing only the product request, not a transport wrapper.
        #[arg(long)]
        request: PathBuf,
    },
    /// Explicitly admit a product-owned diagnostic capture or selected evidence publication.
    Investigate {
        #[arg(long)]
        reader: String,
        /// Product-owned investigate request; passive query never grants attachment permission.
        #[arg(long)]
        request: PathBuf,
    },
    /// Follow opaque product-issued passive continuations; closing the view only stops observation.
    Follow {
        #[arg(long)]
        reader: String,
        /// Initial product follow request; resume coordinates remain product-owned.
        #[arg(long)]
        request: PathBuf,
        /// Maximum whole snapshots emitted before observation stops.
        #[arg(long, default_value_t = 32, value_parser = clap::value_parser!(u32).range(1..))]
        samples: u32,
        /// Minimum polling interval; native progress never awaits this renderer.
        #[arg(long, default_value_t = 1000, value_parser = clap::value_parser!(u64).range(1..))]
        interval_ms: u64,
        /// Observer allowance; its expiry does not time out the original workload.
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..))]
        timeout: u64,
    },
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum Direction {
    Request,
    Response,
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Prepare | Self::Investigate { .. } => GenerationIntent::NewWork,
            _ => GenerationIntent::Passive,
        }
    }
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct PreparedReader {
    /// Resolver-owned selection key; never an arbitrary executable path.
    reader: String,
    /// Original source context at preparation, not current-source freshness.
    source_identity: String,
    /// Captured executable content identity, independent of a mutable target directory.
    executable_sha256: String,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct ProductView<'a> {
    /// Exact selected immutable product reader.
    reader: &'a str,
    /// Product transport category, independent of the observed scientific outcome.
    category: &'a str,
    /// Original lexical product payload; the product schema owns its scientific meaning.
    product: &'a RawValue,
    /// Opaque product-issued continuation when the selected operation supports one.
    next_request: Option<&'a RawValue>,
}
/// Each whole stream record declares observer lifecycle independently of scientific state.
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct FollowView<'a> {
    reader: &'a str,
    observer_state: &'static str,
    sample: usize,
    product: Option<&'a RawValue>,
    next_request: Option<&'a RawValue>,
    summary: Option<&'a product_reader::follow::Summary>,
}
fn product_error(error: product_reader::ProductReaderError) -> HostError {
    HostError::ProductReader(Box::new(error))
}
fn request_file(path: &Path) -> Result<Box<RawValue>, HostError> {
    const MAXIMUM: usize = 1024 * 1024;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAXIMUM as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAXIMUM {
        return Err(HostError::Ownership(
            "product request exceeds 1 MiB transport extent".into(),
        ));
    }
    serde_json::from_slice(&bytes).map_err(|error| {
        HostError::Ownership(format!(
            "product request is invalid JSON ({:?}, line {}, column {})",
            error.classify(),
            error.line(),
            error.column()
        ))
    })
}
pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let environment: Environment = std::env::vars_os().collect();
    if let Action::Follow {
        reader,
        request,
        samples,
        interval_ms,
        timeout,
    } = &action
    {
        let reader =
            ProductReaderSelection::open(&root, &environment, reader).map_err(product_error)?;
        let request = request_file(&super::source_path(selected, request.clone())?)?;
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(*timeout))
            .ok_or_else(|| {
                HostError::Ownership("observer deadline exceeds monotonic horizon".into())
            })?;
        return follow(
            reader,
            environment,
            request,
            product_reader::follow::Limits {
                samples: *samples as usize,
                interval: Duration::from_millis(*interval_ms),
                deadline,
            },
            output,
        );
    }
    if matches!(&action, Action::Prepare) {
        let reader = product_reader::prepare(&root, &environment).map_err(product_error)?;
        return Ok(output.emit(Envelope::response(
            "observe prepare",
            "native product reader explicitly prepared",
            PreparedReader {
                reader: reader.id().to_owned(),
                source_identity: reader.source_identity().map_err(product_error)?,
                executable_sha256: reader.executable().sha256.clone(),
            },
        ))?);
    }
    let (reader_id, request, investigation) = match action {
        Action::Prepare => {
            return Err(HostError::Ownership(
                "unreachable product preparation dispatch".into(),
            ));
        }
        Action::Follow { .. } => {
            return Err(HostError::Ownership("unreachable follow dispatch".into()));
        }
        Action::Schema { reader, contract } => {
            let selection = ProductReaderSelection::open(&root, &environment, &reader)
                .map_err(product_error)?;
            let schema = selection
                .schema(match contract {
                    Direction::Request => Contract::Request,
                    Direction::Response => Contract::Response,
                })
                .map_err(product_error)?;
            // This is the product's actual directional schema, not the host envelope schema.
            writeln!(io::stdout().lock(), "{}", schema.get())?;
            return Ok(0);
        }
        Action::Discover { reader } => (reader, None, false),
        Action::Query { reader, request } => (
            reader,
            Some(request_file(&super::source_path(selected, request)?)?),
            false,
        ),
        Action::Investigate { reader, request } => (
            reader,
            Some(request_file(&super::source_path(selected, request)?)?),
            true,
        ),
    };
    let reader =
        ProductReaderSelection::open(&root, &environment, &reader_id).map_err(product_error)?;
    let (command, payload, category, next_request) = match request {
        None => {
            // Captured discovery is a complete transport frame from this same executable.
            #[derive(serde::Deserialize)]
            struct Discovery {
                payload: Box<RawValue>,
            }
            let captured: Discovery =
                serde_json::from_slice(&reader.discovery().map_err(product_error)?)
                    .map_err(io::Error::other)?;
            ("observe discover", captured.payload, "success", None)
        }
        Some(request) => {
            let reply = if investigation {
                reader.investigate(&request, output.invocation_id.as_str(), &environment)
            } else {
                reader.read(&request, output.invocation_id.as_str(), &environment)
            }
            .map_err(product_error)?;
            if !reply.stderr().is_empty() {
                io::stderr().lock().write_all(reply.stderr())?;
            }
            let category = match reply.category() {
                product_reader::Category::Success => "success",
                product_reader::Category::Refused => "refused",
            };
            (
                if investigation {
                    "observe investigate"
                } else {
                    "observe query"
                },
                reply.payload().to_owned(),
                category,
                reply.next_request().map(ToOwned::to_owned),
            )
        }
    };
    let mut response = Envelope::response(
        command,
        "selected native product response observed",
        ProductView {
            reader: reader.id(),
            category,
            product: &payload,
            next_request: next_request.as_deref(),
        },
    );
    response.observation.scope = "captured_product_reader_transport".into();
    response.observation.limitations.push("Scientific identities, outcome, source coverage and validity remain in the original product payload.".into());
    if category == "refused" {
        response.outcome = envelope::CommandOutcome::Usage;
    }
    Ok(output.emit(response)?)
}

fn follow(
    reader: ProductReaderSelection,
    environment: Environment,
    request: Box<RawValue>,
    limits: product_reader::follow::Limits,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    if output.presentation.fields.is_some() || output.presentation.budget.is_some() {
        return Err(HostError::Ownership("follow emits producer-owned whole snapshots; select detail in the product request rather than --fields/--budget".into()));
    }
    let reader = Arc::new(reader);
    let control = pse_diagnostics::Cancellation::default();
    let mut count = 0;
    let mut refused = false;
    let summary = super::runtime()?.block_on(async {
        let mut interrupt =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        let observing = product_reader::follow::run(
            reader.clone(),
            Arc::new(environment),
            output.invocation_id.as_str().into(),
            request,
            limits,
            control.clone(),
            |reply| {
                count += 1;
                refused |= reply.category() == product_reader::Category::Refused;
                if !reply.stderr().is_empty() {
                    io::stderr().lock().write_all(reply.stderr())?;
                }
                let mut response = Envelope::response(
                    "observe follow",
                    "whole product follow sample",
                    FollowView {
                        reader: reader.id(),
                        observer_state: "observing",
                        sample: count,
                        product: Some(reply.payload()),
                        next_request: reply.next_request(),
                        summary: None,
                    },
                );
                response.observation.scope = "captured_product_reader_transport".into();
                if refused {
                    response.outcome = envelope::CommandOutcome::Usage;
                }
                output.emit(response)?;
                Ok(())
            },
        );
        tokio::pin!(observing);
        let result = tokio::select! {
            result = &mut observing => result,
            _ = interrupt.recv() => { control.cancel(); observing.await },
            _ = terminate.recv() => { control.cancel(); observing.await },
        };
        result.map_err(product_error)
    })?;
    let mut response = Envelope::response(
        "observe follow",
        "observation ended; original work retains its own owner",
        FollowView {
            reader: reader.id(),
            observer_state: "stopped",
            sample: count,
            product: None,
            next_request: None,
            summary: Some(&summary),
        },
    );
    response.observation.scope = "captured_product_reader_transport".into();
    response.observation.timed_out = matches!(summary.stop, product_reader::follow::Stop::Deadline);
    if refused {
        response.outcome = envelope::CommandOutcome::Usage;
    }
    if matches!(summary.stop, product_reader::follow::Stop::Cancelled) {
        response.outcome = envelope::CommandOutcome::Interrupted;
    }
    Ok(output.emit(response)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_transport_retains_lexical_numbers_and_duplicate_keys() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("request.json");
        let original = r#"{"operation":"custom","x":1.00000000000000000000001,"x":2}"#;
        std::fs::write(&path, original).unwrap();
        assert_eq!(request_file(&path).unwrap().get(), original);
        std::fs::write(&path, b"invalid").unwrap();
        assert!(request_file(&path).is_err());
        std::fs::write(&path, vec![b' '; 1024 * 1024 + 1]).unwrap();
        assert!(request_file(&path).is_err());
    }
}
