// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delta durability through native DataFusion plans (ADR-0068).
pub mod admission;
pub mod contract;
pub mod dml;
pub mod layout;
mod nested_check;
mod nested_values;
mod predicates;
pub mod provider;
pub mod publication;
pub mod publication_plan;
pub mod publish;
mod quantities;
mod row_checks;
mod source_spans;
pub mod write;

mod references;
