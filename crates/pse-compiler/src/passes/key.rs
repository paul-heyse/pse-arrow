// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Stage keys: the artifact-hash memo key (blueprint §14.3, ADR-0041, ADR-0042).
//!
//! Every declared input is keyed, including an explicit absence. The plan fingerprint is
//! **not** part of the key: the plan is a pure function of the rule plan, the catalog
//! snapshot and the engine profile, all of which are already in it (§14.2 rule 5).
//!
//! Packet C-4 fills this module.
