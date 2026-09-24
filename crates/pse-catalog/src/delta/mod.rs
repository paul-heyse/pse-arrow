// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delta durability through native DataFusion plans (ADR-0068).
mod actions;
pub mod admission;
pub mod attempt;
pub mod changes;
pub mod contract;
pub mod dependencies;
pub mod dml;
mod field_check;
pub mod layout;
pub(crate) mod lease;
pub(crate) mod leased;
pub mod maintenance;
mod operation;
pub mod provider;
pub mod publication;
pub mod publication_plan;
pub mod publish;
pub mod retention;
pub mod settlement;
pub mod write;
mod write_evidence;
