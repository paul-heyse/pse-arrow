// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The shared native path presentation, for PathBuf and OsString owner fields.
use super::NativePath;
use serde::{Deserialize, Serialize};
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

pub(crate) fn serialize<S: serde::Serializer, P: AsRef<OsStr> + ?Sized>(
    path: &P,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    NativePath::from(PathBuf::from(path.as_ref())).serialize(serializer)
}
pub(crate) fn deserialize<'de, D: serde::Deserializer<'de>, P: From<OsString>>(
    deserializer: D,
) -> Result<P, D::Error> {
    Ok(P::from(
        NativePath::deserialize(deserializer)?
            .to_path_buf()
            .into_os_string(),
    ))
}
