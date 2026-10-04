// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The typed settings documents of the native backends (ADR-0113, ADR-0116 Outcomes 1 and
//! 6). Each backend adapter's pse-owned settings type lives here and is compiled whatever
//! native libraries a build links, so the document, its JSON Schema and its identity are the
//! same in every build; a backend a build does not link is refused at admission, never at
//! decoding. The Clarabel settings (`crate::conic::Settings`) and the SCIP settings
//! (`crate::execution::ScipSettings`) are always compiled with their adapters.
//!
//! Conventions every settings document follows, so that its generated Python type is exact:
//! a choice that carries parameters is internally tagged by `kind`; every enumeration is a
//! registry vocabulary; a single-value domain is a validated scalar
//! (`pse_model::scalars`); absent fields take the type's defaults and unknown fields are
//! refused.
pub mod highs;
pub mod ipopt;
pub mod kinsol;
pub mod petsc;
pub mod pounce;
pub mod pounce_convex;
pub mod uno;
