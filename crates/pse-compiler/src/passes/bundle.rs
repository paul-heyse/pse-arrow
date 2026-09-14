// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Output bundles: complete, immutable, with every declared port (blueprint §14.1).
//!
//! A bundle contains all declared output ports, including explicit empty relations. Each
//! stage publishes a complete replacement rather than overwriting another stage's artifact.
//!
//! Packet C-4 fills this module.
