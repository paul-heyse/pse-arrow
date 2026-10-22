// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Preserve library order and the provider's dynamic retention scopes from pkg-config.

/// Cargo native-library declarations from one pkg-config `Libs:` payload.
/// Search paths are emitted by the caller; unrelated flags keep their existing handling.
pub(crate) fn libraries(libs: &str, linkage: &str) -> Result<Vec<String>, &'static str> {
    let mut retained = None;
    let mut scopes = Vec::new();
    let mut libraries = Vec::new();
    for token in libs.split_whitespace() {
        if let Some(flags) = token.strip_prefix("-Wl,") {
            for flag in flags.split(',') {
                match flag {
                    "--no-as-needed" => retained = Some(true),
                    "--as-needed" => retained = Some(false),
                    "--push-state" => scopes.push(retained),
                    "--pop-state" => retained = scopes.pop().ok_or("unmatched native linker state pop")?,
                    _ => {}
                }
            }
        } else if let Some(name) = token.strip_prefix("-l") {
            let modifier = if linkage == "dylib" {
                match retained {
                    Some(true) => ":-as-needed",
                    Some(false) => ":+as-needed",
                    None => "",
                }
            } else { "" };
            libraries.push(format!("{linkage}{modifier}={name}"));
        }
    }
    if !scopes.is_empty() { return Err("unmatched native linker state push"); }
    Ok(libraries)
}

/// Source attributes retain the same transitive native-link metadata without
/// requiring the separate rustc CLI `-Z unstable-options` switch on this toolchain.
pub(crate) fn attributes(libraries: &[String]) -> Result<String, &'static str> {
    let mut source = String::new();
    for library in libraries {
        let (kind, name) = library.split_once('=').ok_or("invalid native library declaration")?;
        let (kind, modifiers) = kind.split_once(':').map_or((kind, None), |(kind, modifiers)| (kind, Some(modifiers)));
        source.push_str(&format!("#[link(name = {name:?}, kind = {kind:?}"));
        if let Some(modifiers) = modifiers { source.push_str(&format!(", modifiers = {modifiers:?}")); }
        source.push_str(")]\nunsafe extern \"C\" {}\n");
    }
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_link_retains_the_complete_mkl_dynamic_group_in_source_order() {
        let libs = "-m64 -L${libdir} -Wl,--no-as-needed -lmkl_intel_lp64 -lmkl_gnu_thread -lmkl_core -lgomp -lpthread -lm -ldl";
        assert_eq!(libraries(libs, "dylib").unwrap(), [
            "dylib:-as-needed=mkl_intel_lp64", "dylib:-as-needed=mkl_gnu_thread", "dylib:-as-needed=mkl_core",
            "dylib:-as-needed=gomp", "dylib:-as-needed=pthread", "dylib:-as-needed=m", "dylib:-as-needed=dl",
        ]);
    }
    #[test]
    fn provider_link_scope_restoration_preserves_ordinary_and_static_links() {
        let libs = "-lbefore -Wl,--push-state,--no-as-needed -lretained -Wl,--as-needed -loptional -Wl,--pop-state -lafter";
        assert_eq!(libraries(libs, "dylib").unwrap(), ["dylib=before", "dylib:-as-needed=retained", "dylib:+as-needed=optional", "dylib=after"]);
        assert_eq!(libraries(libs, "static").unwrap(), ["static=before", "static=retained", "static=optional", "static=after"]);
        assert!(libraries("-Wl,--pop-state -lbad", "dylib").is_err());
        assert!(libraries("-Wl,--push-state -lbad", "dylib").is_err());
    }
    #[test]
    fn provider_link_source_attributes_retain_order_and_scope_modifiers() {
        let declarations = libraries("-lbefore -Wl,--no-as-needed -lmkl_core -Wl,--as-needed -lafter", "dylib").unwrap();
        assert_eq!(attributes(&declarations).unwrap(), concat!(
            "#[link(name = \"before\", kind = \"dylib\")]\nunsafe extern \"C\" {}\n",
            "#[link(name = \"mkl_core\", kind = \"dylib\", modifiers = \"-as-needed\")]\nunsafe extern \"C\" {}\n",
            "#[link(name = \"after\", kind = \"dylib\", modifiers = \"+as-needed\")]\nunsafe extern \"C\" {}\n",
        ));
    }
}
