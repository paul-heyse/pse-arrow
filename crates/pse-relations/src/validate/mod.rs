// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The recursive field, schema and batch validators (blueprint §4.4, §5.3 step 1).
//!
//! Registration is passive: review E1 showed that an ordinary `SELECT` can bypass an
//! extension-type factory. These validators are the active half — they run at provider and
//! bundle admission, around the ordered engine analyzers, and on every published output
//! batch, and they return violating keys plural rather than the first failure.
//!
//! Packet A-7 fills this module.
