// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Managed invocation byte observation. Physical launch/settlement remains with its owner.
pub mod finalizer;
pub mod lifecycle;
pub mod payload;
pub mod readiness;
mod request;
pub(crate) use request::decode_path as decode_checkout;
pub mod result;
pub mod streams;
pub mod supervisor;
