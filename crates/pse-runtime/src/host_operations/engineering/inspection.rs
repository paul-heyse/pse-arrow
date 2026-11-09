// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independently compiled read-only modeling inspection world.

/// Source-derived inspection records and borrowed views.
#[path = "../../generated/engineering-inspection/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator surface")]
pub mod pb;
#[path = "../../generated/engineering-inspection/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator spelling and packaging modules")]
mod validation;

/// The selected wire types and their shared imports.
pub mod types {
    pub use super::pb::pse::engineering::v1::*;
    pub use super::pb::pse::engineering::v1::{FILE_DESCRIPTOR_SET_BYTES, descriptor_pool};
    pub use pse_host::engineering_types::pb::pse::engineering::v1::*;

    /// Register every Rust message owned by the selected inspection world.
    pub fn register_types(registry: &mut buffa::type_registry::TypeRegistry) {
        pse_host::engineering_types::pb::pse::engineering::v1::register_types(registry);
        super::pb::pse::engineering::v1::register_types(registry);
    }

    #[doc(hidden)]
    pub mod __buffa {
        #[doc(hidden)]
        pub mod oneof {
            pub use super::super::super::pb::pse::engineering::v1::__buffa::oneof::*;
            pub use pse_host::engineering_types::pb::pse::engineering::v1::__buffa::oneof::*;
        }

        #[doc(hidden)]
        pub mod view {
            pub use super::super::super::pb::pse::engineering::v1::__buffa::view::*;
            pub use pse_host::engineering_types::pb::pse::engineering::v1::__buffa::view::*;

            #[doc(hidden)]
            pub mod oneof {
                pub use super::super::super::super::pb::pse::engineering::v1::__buffa::view::oneof::*;
                pub use pse_host::engineering_types::pb::pse::engineering::v1::__buffa::view::oneof::*;
            }
        }
    }
}

use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
use pse_contract_proto::support::*;

#[path = "../../generated/engineering-inspection/native-adapters.rs"]
#[rustfmt::skip]
mod adapters;
pub use adapters::{decode_modeling_inspection, encode_modeling_inspection};

/// Complete descriptor import world of the model-inspection projection.
pub const DESCRIPTORS: &[u8] = include_bytes!("../../generated/engineering-inspection/descriptor.bin");
