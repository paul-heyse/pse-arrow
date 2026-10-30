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
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum Direction {
    Request,
    Response,
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Prepare => GenerationIntent::NewWork,
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
    let (reader_id, request) = match action {
        Action::Prepare => {
            return Err(HostError::Ownership(
                "unreachable product preparation dispatch".into(),
            ));
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
        Action::Discover { reader } => (reader, None),
        Action::Query { reader, request } => (
            reader,
            Some(request_file(&super::source_path(selected, request)?)?),
        ),
    };
    let reader =
        ProductReaderSelection::open(&root, &environment, &reader_id).map_err(product_error)?;
    let (command, payload, category) = match request {
        None => {
            // Captured discovery is a complete transport frame from this same executable.
            #[derive(serde::Deserialize)]
            struct Discovery {
                payload: Box<RawValue>,
            }
            let captured: Discovery =
                serde_json::from_slice(&reader.discovery().map_err(product_error)?)
                    .map_err(io::Error::other)?;
            ("observe discover", captured.payload, "success")
        }
        Some(request) => {
            let reply = reader
                .read(&request, output.invocation_id.as_str(), &environment)
                .map_err(product_error)?;
            if !reply.stderr().is_empty() {
                io::stderr().lock().write_all(reply.stderr())?;
            }
            let category = match reply.category() {
                product_reader::Category::Success => "success",
                product_reader::Category::Refused => "refused",
            };
            ("observe query", reply.payload().to_owned(), category)
        }
    };
    let mut response = Envelope::response(
        command,
        "selected native product response observed",
        ProductView {
            reader: reader.id(),
            category,
            product: &payload,
        },
    );
    response.observation.scope = "captured_product_reader_transport".into();
    response.observation.limitations.push("Scientific identities, outcome, source coverage and validity remain in the original product payload.".into());
    if category == "refused" {
        response.outcome = envelope::CommandOutcome::Usage;
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
