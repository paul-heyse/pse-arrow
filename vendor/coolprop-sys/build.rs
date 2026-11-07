fn main() {
    use sha2::{Digest, Sha256};
    use std::{env, fs, path::PathBuf};
    println!("cargo:rerun-if-env-changed=COOLPROP_DIR");
    let prefix = PathBuf::from(env::var_os("COOLPROP_DIR").expect("explicit CoolProp native preparation is required"));
    assert!(prefix.is_absolute(), "CoolProp prepared prefix must be absolute");
    let image = prefix.join("lib/libCoolProp.so");
    let receipt = prefix.join(".complete.json");
    println!("cargo:rerun-if-changed={}", receipt.display());
    println!("cargo:rerun-if-changed={}", image.display());
    let bytes = fs::read(&receipt).expect("completed CoolProp preparation receipt");
    let document: serde_json::Value = serde_json::from_slice(&bytes).expect("valid preparation receipt");
    assert_eq!(document["version"], 1);
    assert_eq!(document["identity"]["kind"], "coolprop");
    let digest: String = Sha256::digest(fs::read(&image).expect("prepared CoolProp image"))
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(document["files"]["lib/libCoolProp.so"], digest);
    fs::write(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("coolprop-receipt.json"), bytes).unwrap();
    println!("cargo:rustc-env=PSE_COOLPROP_IMAGE={}", image.display());
    println!("cargo:rustc-env=PSE_COOLPROP_IMAGE_SHA256={digest}");
    #[cfg(feature = "regen-bindings")]
    {
        use std::{env, path::PathBuf};

        const LIB_NAME: &str = "CoolProp";

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=CoolPropLib.h");
        let target_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let bindings = bindgen::Builder::default()
            .header("CoolPropLib.h")
            .derive_debug(true)
            .derive_default(true)
            .dynamic_library_name(LIB_NAME)
            .dynamic_link_require_all(true)
            .use_core()
            .generate_cstr(true)
            .generate_comments(false)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("bindgen should generate bindings from `CoolPropLib.h`");
        bindings
            .write_to_file(target_dir.join("bindings.rs"))
            .expect("generated bindings should be written to `OUT_DIR`");
    }
}
