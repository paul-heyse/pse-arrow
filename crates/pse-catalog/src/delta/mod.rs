// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delta durability through native DataFusion plans (ADR-0068).
pub mod admission;
pub mod attempt;
mod changes;
pub mod contract;
mod dependencies;
pub mod dml;
pub mod layout;
pub(crate) mod lease;
pub(crate) mod leased;
pub mod maintenance;
mod nested_check;
mod nested_values;
mod numerical;
mod predicates;
pub mod provider;
pub mod publication;
pub mod publication_plan;
pub mod publish;
mod quantities;
mod retention;
mod source_spans;
pub mod write;

mod references;
