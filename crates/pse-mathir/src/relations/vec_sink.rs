// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! An in-memory sink and source over plain vectors (blueprint §6.9, §24.1).
//!
//! Filled by packet M-5: `VecSink`, the test double implementing both
//! `MathRelationSink` and `MathRelationSource`, so an emit → load → emit round trip can be
//! asserted without an Arrow dependency.
