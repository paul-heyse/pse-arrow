// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated projections of the actual engineering request/response owners.
#[path = "../../generated/engineering/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator surface")]
#[allow(unreachable_pub, reason = "Pinned protobuf supplier emits public scaffolding inside this crate-private wire owner")]
pub(crate) mod broad;
#[path = "../../generated/engineering/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator spelling and packaging modules")]
mod validation;

/// One ergonomic view of the independently compiled wire owners.
pub mod pb {
    #[cfg(test)]
    pub(crate) use super::broad::__buffa_fds;
    /// PSE protobuf packages.
    pub mod pse {
        /// Engineering protobuf package.
        pub mod engineering {
            /// Message names retain their source-selected wire owners.
            pub mod v1 {
                use super::super::super::super::broad;
                use crate::host_operations::engineering::inspection::pb as inspection;
                pub use broad::pse::engineering::v1::*;
                pub use broad::pse::engineering::v1::{FILE_DESCRIPTOR_SET_BYTES, descriptor_pool};
                pub use inspection::pse::engineering::v1::*;
                pub use pse_host::engineering_client::pb::pse::engineering::v1::*;
                pub use pse_host::engineering_types::pb::pse::engineering::v1::*;

                /// Register the generated types from each engineering file owner.
                pub fn register_types(registry: &mut buffa::type_registry::TypeRegistry) {
                    broad::pse::engineering::v1::register_types(registry);
                    inspection::pse::engineering::v1::register_types(registry);
                    pse_host::engineering_client::pb::pse::engineering::v1::register_types(
                        registry,
                    );
                    pse_host::engineering_types::pb::pse::engineering::v1::register_types(registry);
                }

                #[doc(hidden)]
                pub mod __buffa {
                    #[doc(hidden)]
                    pub mod oneof {
                        pub use super::super::broad::pse::engineering::v1::__buffa::oneof::*;
                        pub use super::super::inspection::pse::engineering::v1::__buffa::oneof::*;
                        pub use pse_host::engineering_client::pb::pse::engineering::v1::__buffa::oneof::*;
                        pub use pse_host::engineering_types::pb::pse::engineering::v1::__buffa::oneof::*;
                    }

                    #[doc(hidden)]
                    pub mod view {
                        pub use super::super::broad::pse::engineering::v1::__buffa::view::*;
                        pub use super::super::inspection::pse::engineering::v1::__buffa::view::*;
                        pub use pse_host::engineering_client::pb::pse::engineering::v1::__buffa::view::*;
                        pub use pse_host::engineering_types::pb::pse::engineering::v1::__buffa::view::*;

                        #[doc(hidden)]
                        pub mod oneof {
                            pub use super::super::super::broad::pse::engineering::v1::__buffa::view::oneof::*;
                            pub use super::super::super::inspection::pse::engineering::v1::__buffa::view::oneof::*;
                            pub use pse_host::engineering_client::pb::pse::engineering::v1::__buffa::view::oneof::*;
                            pub use pse_host::engineering_types::pb::pse::engineering::v1::__buffa::view::oneof::*;
                        }
                    }
                }
            }
        }
    }
}
/// Generated typed service and independent client; importing this starts no listener.
#[cfg(feature = "engineering-service")]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, reason = "Pinned service generator surface")]
#[allow(clippy::match_single_binding, reason = "Pinned service generator emits empty streaming branches")]
pub mod services {
    include!("../../generated/engineering/services/pse.engineering.v1.mod.rs");
    pub use pse_host::engineering_client::services::*;
}
/// Complete immutable descriptor identity for the shared engineering contract world.
pub const DESCRIPTORS: &[u8] = include_bytes!("../../generated/engineering/descriptor.bin");
