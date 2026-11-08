// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generate one current descriptor with the independently published Buffa.

use buffa::Message;
use std::ffi::OsStr;

fn text(argument: &OsStr) -> &str {
    argument.to_str().expect("generator input UTF-8")
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert!(args.len() >= 5);
    let descriptor = std::fs::read(&args[0]).expect("current descriptor");
    let descriptor = buffa_codegen::generated::descriptor::FileDescriptorSet::decode_from_slice(
        &descriptor,
    )
    .expect("decode current descriptor");
    let output = std::path::Path::new(&args[1]);
    let files = [text(&args[2]).to_owned()];
    let mut config = buffa_codegen::CodeGenConfig::default();
    config.lazy_views = text(&args[3]).parse().expect("lazy views selection");
    config.emit_register_fn = text(&args[4]).parse().expect("registration selection");
    let paths = args[5..].chunks_exact(2);
    assert!(paths.remainder().is_empty());
    config.extern_paths = paths
        .map(|pair| (text(&pair[0]).to_owned(), text(&pair[1]).to_owned()))
        .collect();
    for generated in buffa_codegen::generate(&descriptor.file, &files, &config)
        .expect("stock generation from current descriptor")
    {
        std::fs::write(output.join(generated.name), generated.content).expect("write stock output");
    }
}
