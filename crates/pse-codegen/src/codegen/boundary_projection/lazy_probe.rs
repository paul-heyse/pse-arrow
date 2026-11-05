// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Complete deferred wire traversal, from the selected Buffa descriptor world.

use super::*;
use std::{collections::BTreeSet, fmt::Write as _};

/// Exact generation inputs for an allocation-accounted deferred-decode probe.
#[derive(Debug, Clone, Copy)]
pub struct LazyProbeTarget<'a> {
    /// Same final descriptor world used for message generation.
    pub descriptors: &'a FileDescriptorSet,
    /// Selected source file in that world.
    pub selected_file: &'a str,
    /// Same configuration used for message generation.
    pub config: &'a CodeGenConfig,
    /// Consumer path to the selected generated package module.
    pub wire_root: &'a str,
    /// Reachability of the selected root entry points.
    pub visibility: AdapterVisibility,
}

/// Generate complete deferred decoding, without getters that reset allowances.
///
/// The returned root retains its own lazy fields. Descendant temporaries drop
/// after traversal; all backing requests remain charged to the original context.
/// Eager carve-outs are already transitively decoded by the selected maintained
/// decoders. This is neither local-rule validation nor native adoption, and makes
/// no stack, allocator-overhead or complete receiving-envelope claim.
pub fn lazy_decode_probe(
    roots: &[&str],
    target: LazyProbeTarget<'_>,
) -> Result<String, ProjectionCodegenError> {
    if roots.is_empty()
        || !target.config.generate_views
        || !target.config.lazy_views
        || !target.config.allocation_accounted_views
        || !target.config.preserve_unknown_fields
        || target.config.idiomatic_field_names
    {
        return Err(error(
            "lazy probe requires explicit roots and the maintained accounted/preserved view profile with original field names",
        ));
    }
    syn::parse_str::<syn::Path>(target.wire_root)
        .map_err(|source| error(format!("invalid lazy-probe package path: {source}")))?;
    let file = target
        .descriptors
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some(target.selected_file))
        .ok_or_else(|| error("selected lazy-probe descriptor is absent"))?;
    // Edition field encoding can turn messages into eager groups. Refuse until
    // the probe consumes that resolved feature contract, rather than guessing.
    if file.syntax.as_deref() != Some("proto3") {
        return Err(error(
            "lazy probe currently requires the source proto3 profile",
        ));
    }
    let package = file.package.as_deref().unwrap_or_default();
    let requested = vec![target.selected_file.into()];
    let naming = buffa_codegen::context::CodeGenContext::for_generate(
        &target.descriptors.file,
        &requested,
        target.config,
    );
    fn collect<'a>(
        messages: &'a [DescriptorProto],
        parent: &str,
        out: &mut BTreeMap<String, &'a DescriptorProto>,
    ) {
        for message in messages {
            let fqn = format!("{parent}.{}", message.name.as_deref().unwrap_or_default());
            out.insert(fqn.clone(), message);
            collect(&message.nested_type, &fqn, out);
        }
    }
    let mut messages = BTreeMap::new();
    collect(&file.message_type, &format!(".{package}"), &mut messages);
    let view =
        |fqn: &str| local_view_path(&naming, package, target.wire_root, fqn, "LazyView", "'a");
    let mut pending = roots
        .iter()
        .map(|root| format!(".{package}.{root}"))
        .collect::<Vec<_>>();
    let mut selected = BTreeSet::new();
    let mut out = String::new();
    while let Some(fqn) = pending.pop() {
        if !selected.insert(fqn.clone()) {
            continue;
        }
        let message = messages
            .get(&fqn)
            .ok_or_else(|| error(format!("{fqn}: selected lazy message descriptor is absent")))?;
        let ty = view(&fqn)?;
        let _ = writeln!(
            out,
            "impl<'a> ProbeDeferred<'a> for {ty} {{ fn probe_deferred(&self, ctx: buffa::DecodeContext<'_>) -> Result<(),buffa::DecodeError> {{ let _ = ctx;"
        );
        for field in &message.field {
            // Oneof variants, maps and groups are eager, including their full
            // descendants. Their maintained decode route checks child capability.
            if field.r#type != Some(Type::TYPE_MESSAGE)
                || (field.oneof_index.is_some() && field.proto3_optional != Some(true))
            {
                continue;
            }
            let child = field
                .type_name
                .as_deref()
                .ok_or_else(|| error("message field has no type"))?;
            let split = naming
                .rust_type_relative_split(child, package, 0)
                .ok_or_else(|| error(format!("{child}: actual child naming is unavailable")))?;
            if split.is_extern {
                continue;
            }
            let child_message = messages.get(child).ok_or_else(|| {
                error(format!(
                    "{child}: deferred child is outside the selected source file"
                ))
            })?;
            if child_message.options.map_entry == Some(true) {
                continue;
            }
            pending.push(child.into());
            let field_name = field
                .name
                .as_deref()
                .ok_or_else(|| error("deferred field has no name"))?;
            let ident = buffa_codegen::idents::make_field_ident(field_name);
            let method = if field.label == Some(Label::LABEL_REPEATED) {
                "probe_repeated"
            } else {
                "probe_singular"
            };
            let _ = writeln!(out, "{method}(&self.{ident},ctx)?;");
        }
        out.push_str("Ok(()) } }\n");
    }
    for root in roots {
        let ty = view(&format!(".{package}.{root}"))?;
        let visibility = target.visibility.rust();
        let method = snake_case(root);
        let _ = writeln!(
            out,
            "{visibility} fn probe_{method}<'a>(bytes: &'a [u8],ctx:buffa::DecodeContext<'_>) -> Result<{ty},buffa::DecodeError> {{ ctx.require_allocation_accounted(<{ty} as buffa::LazyMessageView>::allocation_accounted())?; let view=<{ty} as buffa::LazyMessageView>::decode_lazy_with_ctx(bytes,ctx)?; view.probe_deferred(ctx)?; Ok(view) }}"
        );
    }
    let helpers = r#"
trait ProbeDeferred<'a>: buffa::LazyMessageView<'a> {
    fn probe_deferred(&self, ctx:buffa::DecodeContext<'_>) -> Result<(),buffa::DecodeError>;
}
fn probe_singular<'a,L:ProbeDeferred<'a>>(field:&buffa::LazyMessageFieldView<'a,L>,ctx:buffa::DecodeContext<'_>)->Result<(),buffa::DecodeError>{
    if let Some((first,rest))=field.fragments().split_first(){
        let ctx=ctx.descend()?;
        ctx.require_allocation_accounted(L::allocation_accounted())?;
        let mut child=L::decode_lazy_with_ctx(first,ctx)?;
        for fragment in rest {
            ctx.require_allocation_accounted(L::allocation_accounted())?;
            child.merge_lazy(fragment,ctx)?;
        }
        child.probe_deferred(ctx)?;
    }
    Ok(())
}
fn probe_repeated<'a,L:ProbeDeferred<'a>>(field:&buffa::LazyRepeatedView<'a,L>,ctx:buffa::DecodeContext<'_>)->Result<(),buffa::DecodeError>{
    for bytes in field.raw_elements(){
        let ctx=ctx.descend()?;
        ctx.require_allocation_accounted(L::allocation_accounted())?;
        let child=L::decode_lazy_with_ctx(bytes,ctx)?;
        child.probe_deferred(ctx)?;
    }
    Ok(())
}
"#;
    let source = format!(
        "// @generated by pse-codegen from the selected final descriptor/config; do not edit.\n{helpers}\n{out}"
    );
    let parsed = syn::parse_file(&source)
        .map_err(|source| error(format!("lazy-probe rendering failed: {source}")))?;
    Ok(prettyplease::unparse(&parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_probe_uses_actual_nested_names_and_refuses_uncovered_profiles() {
        let scalar = FieldDescriptorProto {
            name: Some("type".into()),
            number: Some(1),
            label: Some(Label::LABEL_OPTIONAL),
            r#type: Some(Type::TYPE_STRING),
            ..Default::default()
        };
        let child = DescriptorProto {
            name: Some("Child".into()),
            field: vec![scalar],
            ..Default::default()
        };
        let deferred = FieldDescriptorProto {
            name: Some("child".into()),
            number: Some(1),
            label: Some(Label::LABEL_OPTIONAL),
            r#type: Some(Type::TYPE_MESSAGE),
            type_name: Some(".pse.probe.v1.Root.Child".into()),
            ..Default::default()
        };
        let root = DescriptorProto {
            name: Some("Root".into()),
            field: vec![deferred],
            nested_type: vec![child],
            ..Default::default()
        };
        let descriptors = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("probe.proto".into()),
                package: Some("pse.probe.v1".into()),
                syntax: Some("proto3".into()),
                message_type: vec![root],
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut config = CodeGenConfig::default();
        config.generate_views = true;
        config.lazy_views = true;
        config.allocation_accounted_views = true;
        config.preserve_unknown_fields = true;
        let target = LazyProbeTarget {
            descriptors: &descriptors,
            selected_file: "probe.proto",
            config: &config,
            wire_root: "crate::pb",
            visibility: AdapterVisibility::Parent,
        };
        let source = lazy_decode_probe(&["Root"], target).unwrap();
        assert!(source.contains("root::ChildLazyView<'a>"));
        assert!(source.contains("probe_singular(&self.child, ctx)?"));
        assert!(source.contains("ctx.require_allocation_accounted(L::allocation_accounted())?"));
        assert!(!source.contains(".get()"));
        assert!(!source.contains("to_owned_message"));
        assert!(lazy_decode_probe(&["Absent"], target).is_err());
        let mut unaccounted = config.clone();
        unaccounted.allocation_accounted_views = false;
        assert!(
            lazy_decode_probe(
                &["Root"],
                LazyProbeTarget {
                    config: &unaccounted,
                    ..target
                }
            )
            .is_err()
        );
        let mut renamed = config.clone();
        renamed.idiomatic_field_names = true;
        assert!(
            lazy_decode_probe(
                &["Root"],
                LazyProbeTarget {
                    config: &renamed,
                    ..target
                }
            )
            .is_err()
        );
        let mut editions = descriptors.clone();
        editions.file[0].syntax = Some("editions".into());
        assert!(
            lazy_decode_probe(
                &["Root"],
                LazyProbeTarget {
                    descriptors: &editions,
                    ..target
                }
            )
            .is_err()
        );
    }
}
