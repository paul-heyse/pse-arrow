// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Compiled symbols and symbol groups (blueprint §6.8).
//!
//! Packet A-3 fills this module. It is declared and wired into
//! [`crate::catalog::assemble`] now so that the assembly order is fixed before the
//! declarations arrive: a module inserted later would reorder the call list, and the
//! call list is the one thing about assembly a reader can check at a glance.

use crate::builder::RegistryBuilder;

/// Declares nothing yet; packet A-3 fills it.
pub fn declare(_builder: &mut RegistryBuilder) {}
