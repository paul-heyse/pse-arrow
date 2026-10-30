// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::observation::{Diagnostics, read_checkpoint};
use pse_diagnostics::observation::{Outcome, Policy};
mod canonical;
mod publication;

fn request(correlation: &str, request: Request) -> Vec<u8> {
    serde_json::to_vec(&ReaderRequest {
        protocol: Protocol::ProductReader,
        version: Version,
        correlation: correlation.into(),
        request,
    })
    .unwrap()
}
