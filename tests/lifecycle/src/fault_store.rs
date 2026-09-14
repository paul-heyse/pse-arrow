// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The fault-injecting `ObjectStore` wrapper and its plan — fail-before, truncate,
//! flip-byte, one-shot precondition, cancel-at-step (blueprint §20.1, §24.1 "Lifecycle");
//! filled by packet B-faults.
