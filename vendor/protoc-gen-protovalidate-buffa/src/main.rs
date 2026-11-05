//! Reads a `CodeGeneratorRequest` on stdin, emits a `CodeGeneratorResponse`
//! on stdout.

use std::io::{self, Read, Write};

use buffa::Message;
use buffa_codegen::generated::{
    compiler::{CodeGeneratorRequest, CodeGeneratorResponse},
    descriptor::Edition,
};
use protoc_gen_protovalidate_buffa::{PreparedValidators, prepare};

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
        admitted(&request)?;
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
    admitted(request)?.render()
}

fn admitted(request: &CodeGeneratorRequest) -> anyhow::Result<PreparedValidators> {
    prepare(request).map_err(|error| {
        for diagnostic in error.diagnostics() {
            eprintln!("CEL {:?}: file={} message={} field={} rule={}: {}", diagnostic.kind,
                diagnostic.source_file, diagnostic.message, diagnostic.field.as_deref().unwrap_or("-"),
                diagnostic.rule, diagnostic.reason);
        }
        anyhow::Error::new(error)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preflight_refuses_absent_or_empty_target_populations() {
        let absent = CodeGeneratorRequest { file_to_generate: vec!["missing.proto".into()], ..Default::default() };
        assert!(admitted(&absent).unwrap_err().to_string().contains("missing.proto"));
        assert!(admitted(&CodeGeneratorRequest::default()).is_err());
    }
}
