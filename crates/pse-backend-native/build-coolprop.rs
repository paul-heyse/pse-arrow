// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bind the selected compiled Water image to the declared complete source recipe.
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn authenticate(
    out: &Path,
    root: &Path,
    manifest: &toml::Value,
) -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed=COOLPROP_DIR");
    let prefix = PathBuf::from(
        env::var_os("COOLPROP_DIR").ok_or("CoolProp requires explicit native preparation")?,
    );
    let receipt = prefix.join(".complete.json");
    let image = prefix.join("lib/libCoolProp.so");
    for path in [&receipt, &image] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let bytes = fs::read(&receipt)?;
    let document: serde_json::Value = serde_json::from_slice(&bytes)?;
    let pin = &manifest["workspace"]["metadata"]["pse"]["coolprop"];
    let expected = serde_json::to_value(pin)?;
    let patch = root.join(
        pin["patch"]
            .as_str()
            .ok_or("CoolProp patch declaration absent")?,
    );
    println!("cargo:rerun-if-changed={}", patch.display());
    if document["version"] != 1
        || document["identity"]["kind"] != "coolprop"
        || document["identity"]["pin"] != expected
        || document["identity"]["construction"]["patch-sha256"]
            != super::build_source::sha256(&fs::read(patch)?)
        || document["files"]["lib/libCoolProp.so"]
            != super::build_source::sha256(&fs::read(&image)?)
    {
        return Err(
            "CoolProp artifact does not match the declared source/patch/image recipe".into(),
        );
    }
    let supplier = super::build_source::authenticated(root, "coolprop-sys")?;
    let mut identity = serde_json::json!({"native": document, "supplier": supplier});
    identity["image_path"] = image.to_string_lossy().as_ref().into();
    super::build_receipt::write_if_changed(
        &out.join("coolprop-profile.json"),
        &serde_json::to_vec(&identity)?,
    )?;
    Ok(())
}
