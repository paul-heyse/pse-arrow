// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared selected SUNDIALS build inputs for KINSOL and IDAS, including IDAS-only builds.
use sundials_sys as ffi;
/// Selected native source/configuration/provider fingerprint. The deployed executable's
/// separate file observation binds the actual static SUNDIALS bytes. The embedded
/// source receipt was checked against the selected maintained tree during build.
/// Missing qualified provider evidence
/// leaves direct native operation available but cannot authorize a child handshake.
pub(crate) fn build() -> Option<pse_ids::ContentHash> {
    let profile = NativeProfile {
        sundials_version: [
            ffi::SUNDIALS_VERSION_MAJOR,
            ffi::SUNDIALS_VERSION_MINOR,
            ffi::SUNDIALS_VERSION_PATCH,
        ],
        klu_version: [
            suitesparse_sys::KLU_MAIN_VERSION,
            suitesparse_sys::KLU_SUB_VERSION,
            suitesparse_sys::KLU_SUBSUB_VERSION,
        ],
        abi_widths: [
            u64::try_from(size_of::<ffi::sunrealtype>()).ok()?,
            u64::try_from(size_of::<ffi::sunindextype>()).ok()?,
        ],
        metadata: [
            ffi::SUNDIALS_VERSION,
            ffi::SUNDIALS_VERSION_LABEL,
            ffi::SUN_C_COMPILER,
            ffi::SUN_C_COMPILER_VERSION,
            ffi::SUN_C_COMPILER_FLAGS,
            ffi::SUN_BUILD_TYPE,
            ffi::SUN_TPL_LIST,
            ffi::SUN_KLU_VERSION,
        ],
    };
    profile_identity(
        include_bytes!(concat!(env!("OUT_DIR"), "/kinsol-klu-provider.json")),
        include_str!(concat!(env!("OUT_DIR"), "/sundials-source.txt")),
        &profile,
    )
}
#[derive(Clone)]
struct NativeProfile<'a> {
    sundials_version: [u32; 3],
    klu_version: [u32; 3],
    abi_widths: [u64; 2],
    // Typed bindgen byte arrays, including their native terminators. In particular
    // SUN_KLU_VERSION may be empty; the linked KLU header version remains explicit.
    metadata: [&'a [u8]; 8],
}
fn profile_identity(
    provider: &[u8],
    source: &str,
    profile: &NativeProfile<'_>,
) -> Option<pse_ids::ContentHash> {
    if source.trim().is_empty() {
        return None;
    }
    // The build helper binds a coherent admitted prefix. Native admission owns
    // receipt/file validation; this hook does not independently inspect archives.
    // Refuse absent/malformed embedded evidence without inventing a fallback pin.
    let document: serde_json::Value = serde_json::from_slice(provider).ok()?;
    for field in ["identity", "files"] {
        if !document
            .get(field)?
            .as_object()
            .is_some_and(|values| !values.is_empty())
        {
            return None;
        }
    }
    let mut hasher = pse_ids::FramedHasher::new(pse_ids::Frame::NativeBuildV1);
    hasher.str("sundials-profile.v1").part(provider).str(source);
    for value in profile
        .sundials_version
        .into_iter()
        .chain(profile.klu_version)
    {
        hasher.u64(u64::from(value));
    }
    for value in profile.abi_widths {
        hasher.u64(value);
    }
    for value in profile.metadata {
        hasher.part(value);
    }
    Some(hasher.finish_hash())
}

#[cfg(test)]
mod build_tests {
    use super::*;
    const PROVIDER: &[u8] = br#"{"identity":{"source":"qualified-provider"},"files":{"lib/libklu.a":"native-admission-digest"}}"#;
    fn profile() -> NativeProfile<'static> {
        NativeProfile {
            sundials_version: [7, 1, 1],
            klu_version: [2, 3, 3],
            abi_widths: [8, 8],
            metadata: [
                b"version\0",
                b"label\0",
                b"compiler\0",
                b"compiler-version\0",
                b"flags\0",
                b"build-type\0",
                b"TPL\0",
                b"\0",
            ],
        }
    }
    #[test]
    fn native_profile_identity_requires_provider_and_source_evidence() {
        let profile = profile();
        for provider in [
            b"".as_slice(),
            b"not json",
            b"null",
            b"[]",
            b"{}",
            br#"{"identity":{},"files":{"lib":1}}"#,
            br#"{"identity":{"source":1},"files":{}}"#,
        ] {
            assert!(profile_identity(provider, "declared source", &profile).is_none());
        }
        assert!(profile_identity(PROVIDER, " \n", &profile).is_none());
        assert!(profile_identity(PROVIDER, "declared source", &profile).is_some());
    }
    #[test]
    fn native_profile_identity_binds_all_native_inputs_without_host_source_provenance() {
        let original = profile();
        let identity = profile_identity(PROVIDER, "declared source", &original).unwrap();
        assert_eq!(
            profile_identity(PROVIDER, "declared source", &original),
            Some(identity)
        );
        assert_ne!(
            profile_identity(PROVIDER, "changed source", &original),
            Some(identity)
        );
        let changed_provider = br#"{"identity":{"source":"different-provider"},"files":{"lib/libklu.a":"native-admission-digest"}}"#;
        assert_ne!(
            profile_identity(changed_provider, "declared source", &original),
            Some(identity)
        );
        let changed_file = br#"{"identity":{"source":"qualified-provider"},"files":{"lib/libklu.a":"changed-native-file"}}"#;
        assert_ne!(
            profile_identity(changed_file, "declared source", &original),
            Some(identity)
        );
        for index in 0..3 {
            let mut changed = original.clone();
            changed.sundials_version[index] += 1;
            assert_ne!(
                profile_identity(PROVIDER, "declared source", &changed),
                Some(identity)
            );
            let mut changed = original.clone();
            changed.klu_version[index] += 1;
            assert_ne!(
                profile_identity(PROVIDER, "declared source", &changed),
                Some(identity)
            );
        }
        for index in 0..2 {
            let mut changed = original.clone();
            changed.abi_widths[index] += 1;
            assert_ne!(
                profile_identity(PROVIDER, "declared source", &changed),
                Some(identity)
            );
        }
        for index in 0..8 {
            let mut changed = original.clone();
            changed.metadata[index] = b"changed\0";
            assert_ne!(
                profile_identity(PROVIDER, "declared source", &changed),
                Some(identity)
            );
        }
    }
    #[test]
    fn selected_sundials_build_reaches_idas_without_kinsol() {
        #[cfg(feature = "idas")]
        assert_eq!(
            crate::execution::adapter(crate::solve::Backend::Idas).build(),
            build()
        );
        #[cfg(feature = "kinsol")]
        assert_eq!(
            crate::execution::adapter(crate::solve::Backend::Kinsol).build(),
            build()
        );
        assert!(
            build().is_some(),
            "managed selected-provider controls require a build identity"
        );
    }
}
