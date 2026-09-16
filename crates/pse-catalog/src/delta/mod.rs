// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delta durability through native DataFusion plans (ADR-0068).
pub mod admission;
pub mod contract;
pub mod dml;
pub mod layout;
mod nested_check;
mod predicates;
pub mod provider;
pub mod publication;
pub mod publication_plan;
pub mod publish;
mod source_spans;
pub mod write;
