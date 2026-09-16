// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared exact source bindings and ordinary native plan constructors.
pub(super) use crate::passes::native_sources::{
    Sources, append, c, error, filter, invalid, join, project, require, sid,
};
