// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Traversal by operator family (blueprint §7.2).
//!
//! Filled by packet M-4: `NodeVisitor`, so a pass dispatches on the family the
//! operator table declares rather than on a match over 43 opcodes.
