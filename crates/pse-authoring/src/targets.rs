// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse.target_path` parsing and resolution (blueprint §4.4, §6.10).
//!
//! An authored selector path names instances, ports, symbols or equations. P1 resolves it
//! to entity identities at commit; the retained text is the *serialization* of those rows
//! and is re-rendered by `rename`. A change op that edits the text without the identity
//! rows is rejected by P2 (`case.target_text_mismatch`): the text is not a second
//! authority.
//!
//! Packet C-2 fills this module.
