// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generous shared budget for ordinary workflow qualification on the development
//! workstation. This is a ceiling, not an eager allocation. Targeted resource
//! refusal and ownership tests declare their deliberately smaller budgets locally.

/// One shared pool for the workflow's sessions, leaving workstation headroom.
pub(crate) const MEMORY_LIMIT_BYTES: usize = 32 << 30;
