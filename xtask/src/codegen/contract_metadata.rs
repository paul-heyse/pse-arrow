// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One source-compiler bootstrap for complete typed option import closures.
use anyhow::{Result, bail};
use buffa::Message;
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use std::{path::Path, process::Command};

/// Compile actual source schemas, preserving raw custom options and genuine source
/// information. This helper has no native runtime or scientific Rust dependency.
pub(super) fn imports(root: &Path) -> Result<FileDescriptorSet> {
    let output = Command::new("buf").args(["build", "--as-file-descriptor-set", "-o", "-"])
        .current_dir(root.join("crates/pse-host/proto")).output()?;
    if !output.status.success() { bail!("compile pinned validation imports: {}", String::from_utf8_lossy(&output.stderr)); }
    let mut set = FileDescriptorSet::decode_from_slice(&output.stdout)?;
    set.file.retain(|file| file.name.as_deref().is_some_and(|name| !name.starts_with("pse/")));
    for source in ["crates/pse-contract/proto", "crates/pse-schema/proto"] {
        let output = Command::new("buf").args(["build", source, "--as-file-descriptor-set", "-o", "-"]).current_dir(root).output()?;
        if !output.status.success() { bail!("compile source-owned contract options {source}: {}", String::from_utf8_lossy(&output.stderr)); }
        for file in FileDescriptorSet::decode_from_slice(&output.stdout)?.file {
            if let Some(existing) = set.file.iter().find(|existing| existing.name == file.name) {
                if existing != &file { bail!("compiled option import collision: {:?}", file.name); }
            } else { set.file.push(file); }
        }
    }
    Ok(set)
}
