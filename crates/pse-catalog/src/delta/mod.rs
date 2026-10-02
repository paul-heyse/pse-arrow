// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delta durability through native DataFusion plans (ADR-0068).
mod actions;
pub mod admission;
pub mod attempt;
pub mod candidate;
pub mod changes;
pub mod collect;
pub mod contract;
pub mod dependencies;
pub mod discovery;
pub mod dml;
mod field_check;
pub mod layout;
pub(crate) mod leased;
pub mod maintenance;
pub mod manifest;
mod operation;
pub mod provider;
pub mod publication;
pub mod publication_plan;
pub mod scope;
pub mod settlement;
pub mod ticket;
pub mod write;
mod write_evidence;
