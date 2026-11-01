//! Reads a `CodeGeneratorRequest` on stdin, emits a `CodeGeneratorResponse`
//! on stdout.

use std::io::{self, Read, Write};

use buffa::Message;
use buffa_codegen::generated::{
    compiler::{CodeGeneratorRequest, CodeGeneratorResponse},
    descriptor::Edition,
};
use protoc_gen_protovalidate_buffa::{emit, scan};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args_os().skip(1);
    if let Some(command) = args.next() {
        anyhow::ensure!(command == "--preflight", "expected --preflight descriptor.bin --file source.proto");
        let path = args.next().ok_or_else(|| anyhow::anyhow!("missing descriptor path"))?;
        let bytes = std::fs::read(path)?;
        let descriptors = buffa_codegen::generated::descriptor::FileDescriptorSet::decode_from_slice(&bytes)?;
        let mut files = Vec::new();
        while let Some(flag) = args.next() {
            anyhow::ensure!(flag == "--file", "expected --file");
            files.push(args.next().ok_or_else(|| anyhow::anyhow!("missing --file value"))?.into_string().map_err(|_| anyhow::anyhow!("proto names must be UTF-8"))?);
        }
        anyhow::ensure!(!files.is_empty(), "explicit --file targets are required");
        let request = CodeGeneratorRequest { proto_file: descriptors.file, file_to_generate: files, ..Default::default() };
        admit(&request)?;
        println!("pse-cel-preflight-v1");
        return Ok(());
    }
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    let request = CodeGeneratorRequest::decode_from_slice(&input)?;

    let response = match run(&request) {
        Ok(files) => CodeGeneratorResponse {
            supported_features: Some(1 | 2), // PROTO3_OPTIONAL | SUPPORTS_EDITIONS
            minimum_edition: Some(Edition::EDITION_PROTO2 as i32),
            maximum_edition: Some(Edition::EDITION_2024 as i32),
            file: files,
            error: None,
            ..Default::default()
        },
        Err(e) => CodeGeneratorResponse {
            error: Some(e.to_string()),
            ..Default::default()
        },
    };

    let mut out = Vec::new();
    response.encode(&mut out);
    io::stdout().write_all(&out)?;
    Ok(())
}

fn run(
    request: &CodeGeneratorRequest,
) -> anyhow::Result<Vec<buffa_codegen::generated::compiler::code_generator_response::File>> {
    let opts = parse_opts(request.parameter.as_deref().unwrap_or(""));
    admit(request)?;
    let validators = scan::gather(request)?;
    emit::render_with_options(&validators, &opts)
}

/// Parse the `opt: [...]` parameters passed via `buf.gen.yaml`. Accepts a
/// comma-separated `key=value,flag,...` list (the format protoc / buf use
/// to invoke plugins). Unknown keys are ignored so callers can pass
/// forward-compatible options.
fn parse_opts(parameter: &str) -> emit::Options {
    let mut opts = emit::Options::default();
    for part in parameter.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((k, v)) = part.split_once('=')
            && k.trim() == "proto_module"
        {
            opts.proto_module = v.trim().to_string();
        }
    }
    opts
}

fn admit(request: &CodeGeneratorRequest) -> anyhow::Result<()> {
    anyhow::ensure!(!request.file_to_generate.is_empty(), "explicit authored generation targets are required");
    let mut names = std::collections::BTreeSet::new();
    for file in &request.proto_file {
        let name = file.name.as_deref().ok_or_else(|| anyhow::anyhow!("descriptor has no logical filename"))?;
        anyhow::ensure!(names.insert(name), "duplicate descriptor filename: {name}");
    }
    let mut selected = std::collections::BTreeSet::new();
    for name in &request.file_to_generate {
        anyhow::ensure!(selected.insert(name), "duplicate authored target: {name}");
        anyhow::ensure!(names.contains(name.as_str()), "authored target missing from descriptor: {name}");
    }
    let messages = scan::gather(request)?;
    let diagnostics = emit::cel::preflight(&messages);
    for diagnostic in &diagnostics {
        eprintln!("CEL {:?}: file={} message={} field={} rule={}: {}", diagnostic.kind,
            diagnostic.source_file, diagnostic.message, diagnostic.field.as_deref().unwrap_or("-"),
            diagnostic.rule, diagnostic.reason);
    }
    anyhow::ensure!(diagnostics.is_empty(), "CEL preflight refused {} rule(s)", diagnostics.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preflight_refuses_absent_or_empty_target_populations() {
        let absent = CodeGeneratorRequest { file_to_generate: vec!["missing.proto".into()], ..Default::default() };
        assert!(admit(&absent).unwrap_err().to_string().contains("missing.proto"));
        assert!(admit(&CodeGeneratorRequest::default()).is_err());
    }
}
