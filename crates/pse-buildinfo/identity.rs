// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Stable source/configuration framing shared by the build script and isolated tests.
pub(crate) fn digest(mut entries: Vec<(String, Vec<u8>)>) -> pse_ids::ContentHash {
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hash = pse_ids::FramedHasher::new("pse:build-inputs:v1");
    for (name, bytes) in entries {
        hash.str(&name).part(&bytes);
    }
    hash.finish_hash()
}
#[cfg(test)]
mod foundation_unit {
    #[test]
    fn dirty_bytes_names_and_configuration_affect_identity_but_enumeration_order_does_not() {
        let a = ("crates/a/src/lib.rs".into(), b"source".to_vec());
        let b = ("feature:ipopt".into(), b"false".to_vec());
        let original = super::digest(vec![a.clone(), b.clone()]);
        assert_eq!(original, super::digest(vec![b.clone(), a.clone()]));
        assert_ne!(
            original,
            super::digest(vec![(a.0.clone(), b"dirty source".to_vec()), b.clone()])
        );
        assert_ne!(
            original,
            super::digest(vec![("crates/other.rs".into(), a.1.clone()), b.clone()])
        );
        assert_ne!(original, super::digest(vec![a, (b.0, b"true".to_vec())]));
    }
}
