// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pass records and findings as relations (blueprint §6.13, §23.2).
//!
//! A finding is a relation; a rendered diagnostic is a projection of a finding, never its
//! storage. Pass records are sidecars: they describe an output bundle and are excluded
//! from the membership they describe.
//!
//! Packet C-4 fills this module.
