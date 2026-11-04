// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed representations of the existing native type and local-rule owners.
use super::*;
use super::check::{boolean, child, enumeration, known_fields, text, uint, value};
use pse_contract::boundary_projection::{Float64Domain, Float64Interval, LocalRule, Scalar, TypeRef};

pub(super) const SCALARS: &[&str] = &["NATIVE_SCALAR_UNSPECIFIED", "NATIVE_SCALAR_BOOL", "NATIVE_SCALAR_STRING", "NATIVE_SCALAR_U32", "NATIVE_SCALAR_U8", "NATIVE_SCALAR_U16", "NATIVE_SCALAR_U64", "NATIVE_SCALAR_I16", "NATIVE_SCALAR_I32", "NATIVE_SCALAR_I64", "NATIVE_SCALAR_F32_BITS", "NATIVE_SCALAR_F64_BITS", "NATIVE_SCALAR_BYTES", "NATIVE_SCALAR_SEMANTIC_ID", "NATIVE_SCALAR_CONTENT_HASH", "NATIVE_SCALAR_I128_BITS", "NATIVE_SCALAR_I8"];
pub(super) const FLOAT_PRESETS: &[&str] = &["NATIVE_FLOAT64_PRESET_UNSPECIFIED", "NATIVE_FLOAT64_PRESET_FINITE", "NATIVE_FLOAT64_PRESET_NONNEGATIVE_FINITE", "NATIVE_FLOAT64_PRESET_POSITIVE_FINITE", "NATIVE_FLOAT64_PRESET_POSITIVE_UNIT"];

fn scalar_number(value: Scalar) -> i32 {
    match value { Scalar::Bool => 1, Scalar::String => 2, Scalar::U32 => 3, Scalar::U8 => 4,
        Scalar::U16 => 5, Scalar::U64 => 6, Scalar::I16 => 7, Scalar::I32 => 8, Scalar::I64 => 9,
        Scalar::F32Bits => 10, Scalar::F64Bits => 11, Scalar::Bytes => 12, Scalar::SemanticId => 13, Scalar::ContentHash => 14, Scalar::I128Bits => 15, Scalar::I8 => 16 }
}
fn scalar_value(number: i32) -> Option<Scalar> {
    Some(match number { 1 => Scalar::Bool, 2 => Scalar::String, 3 => Scalar::U32, 4 => Scalar::U8,
        5 => Scalar::U16, 6 => Scalar::U64, 7 => Scalar::I16, 8 => Scalar::I32, 9 => Scalar::I64,
        10 => Scalar::F32Bits, 11 => Scalar::F64Bits, 12 => Scalar::Bytes, 13 => Scalar::SemanticId, 14 => Scalar::ContentHash, 15 => Scalar::I128Bits, 16 => Scalar::I8, _ => return None })
}

impl OptionEncoder {
    pub(super) fn verify_native_schema(&self) -> Result<(), MetadataError> {
        use buffa_descriptor::{FieldKind::{List, Singular}, SingularKind::{Message, Enum, Scalar}, ScalarType::{Bool, Fixed64, Sint64, Uint64, Uint32, String}};
        self.vocabulary("pse.contract.v1.NativeScalar", SCALARS)?;
        self.vocabulary("pse.contract.v1.NativeFloat64Preset", FLOAT_PRESETS)?;
        let message = |name: &str| self.pool.message_index(&format!("pse.contract.v1.{name}")).map(Message).ok_or_else(|| unavailable(format!("missing native contract message {name}")));
        let enumeration = |name: &str| self.pool.enum_index(&format!("pse.contract.v1.{name}")).map(Enum).map(Singular).ok_or_else(|| unavailable(format!("missing native contract vocabulary {name}")));
        let scalar = |kind| Singular(Scalar(kind));
        for (name, expected, oneof) in [
            ("NativeOwnerShape", vec![("record", Singular(message("NativeRecord")?)), ("alternatives", Singular(message("NativeAlternatives")?)), ("string_enum", Singular(message("NativeStringEnum")?))], true),
            ("NativeRecord", vec![("fields", List(message("NativeFieldDeclaration")?))], false),
            ("NativeFieldDeclaration", vec![("name", scalar(String)), ("tag", scalar(Uint32)), ("contract", Singular(message("NativeFieldContract")?)), ("direction", enumeration("Direction")?)], false),
            ("NativeAlternatives", vec![("alternatives", List(message("NativeAlternative")?))], false),
            ("NativeAlternative", vec![("name", scalar(String)), ("tag", scalar(Uint32)), ("fields", List(message("NativeFieldDeclaration")?))], false),
            ("NativeStringEnum", vec![("values", List(Scalar(String)))], false),
            ("NativeFieldContract", vec![("native_type", Singular(message("NativeType")?)), ("local_rules", List(message("NativeLocalRule")?))], false),
            ("NativeType", vec![("scalar", enumeration("NativeScalar")?), ("named_owner", scalar(String)), ("optional_value", Singular(message("NativeType")?)), ("repeated_value", Singular(message("NativeType")?)), ("set_value", Singular(message("NativeType")?)), ("map", Singular(message("NativeMap")?))], true),
            ("NativeMap", vec![("key", Singular(message("NativeType")?)), ("value", Singular(message("NativeType")?))], false),
            ("NativeLocalRule", vec![("required_value", scalar(Bool)), ("positive_duration", scalar(Bool)), ("float64_domain", Singular(message("NativeFloat64Domain")?)), ("signed_range", Singular(message("NativeSignedRange")?)), ("unsigned_range", Singular(message("NativeUnsignedRange")?)), ("length", Singular(message("NativeLength")?)), ("utf8_bytes", Singular(message("NativeLength")?))], true),
            ("NativeFloat64Domain", vec![("preset", enumeration("NativeFloat64Preset")?), ("interval", Singular(message("NativeFloat64Interval")?))], true),
            ("NativeFloat64Interval", vec![("lower_bits", scalar(Fixed64)), ("lower_inclusive", scalar(Bool)), ("upper_bits", scalar(Fixed64)), ("upper_inclusive", scalar(Bool))], false),
            ("NativeSignedRange", vec![("minimum", scalar(Sint64)), ("maximum", scalar(Sint64))], false),
            ("NativeUnsignedRange", vec![("minimum", scalar(Uint64)), ("maximum", scalar(Uint64))], false),
            ("NativeLength", vec![("minimum", scalar(Uint64)), ("maximum", scalar(Uint64))], false),
        ] {
            let descriptor = self.pool.message_by_name(&format!("pse.contract.v1.{name}")).ok_or_else(|| unavailable(format!("missing native contract shape {name}")))?;
            if descriptor.fields().len() != expected.len() || !descriptor.extension_ranges().is_empty()
                || descriptor.oneofs().len() != usize::from(oneof) || (oneof && descriptor.oneofs()[0].name() != "kind")
                || expected.iter().enumerate().any(|(index, (name, kind))| descriptor.field_by_name(name).is_none_or(|field|
                    field.number() != (index + 1) as u32 || field.kind() != *kind || field.is_delimited() || field.oneof_index() != oneof.then_some(0)
                    || field.presence() != if matches!(kind, List(_)) { buffa::editions::FieldPresence::Implicit } else { buffa::editions::FieldPresence::Explicit })) {
                return Err(unavailable(format!("{}: unsupported native contract schema correspondence", descriptor.full_name())));
            }
        }
        Ok(())
    }
    pub(super) fn native_message(&self, name: &str, fields: &[(&str, Value)]) -> Result<DynamicMessage, MetadataError> {
        self.message(&format!("pse.contract.v1.{name}"), fields)
    }
    pub(super) fn native_contract(&self, ty: &TypeRef, rules: &[LocalRule], package: &str) -> Result<DynamicMessage, MetadataError> {
        self.native_message("NativeFieldContract", &[("native_type", Value::Message(self.native_type(ty, package)?)),
            ("local_rules", Value::List(rules.iter().map(|rule| self.native_rule(*rule).map(Value::Message)).collect::<Result<_, _>>()?))])
    }
    fn native_type(&self, ty: &TypeRef, package: &str) -> Result<DynamicMessage, MetadataError> {
        let (name, value) = match ty {
            TypeRef::Scalar(scalar) => ("scalar", Value::EnumNumber(scalar_number(*scalar))),
            TypeRef::Named(name) => ("named_owner", Value::String(if name.contains('.') { name.trim_start_matches('.').into() } else { format!("{package}.{name}") })),
            TypeRef::Optional(inner) => ("optional_value", Value::Message(self.native_type(inner, package)?)),
            TypeRef::Repeated(inner) => ("repeated_value", Value::Message(self.native_type(inner, package)?)),
            TypeRef::Set(inner) => ("set_value", Value::Message(self.native_type(inner, package)?)),
            TypeRef::Map(key, value) => ("map", Value::Message(self.native_message("NativeMap", &[
                ("key", Value::Message(self.native_type(key, package)?)), ("value", Value::Message(self.native_type(value, package)?))])?)),
        };
        self.native_message("NativeType", &[(name, value)])
    }
    fn native_rule(&self, rule: LocalRule) -> Result<DynamicMessage, MetadataError> {
        let (name, value) = match rule {
            LocalRule::Required => ("required_value", Value::Bool(true)),
            LocalRule::PositiveDuration => ("positive_duration", Value::Bool(true)),
            LocalRule::Float64Domain(domain) => ("float64_domain", Value::Message(self.native_float_domain(domain)?)),
            LocalRule::SignedRange { min, max } => ("signed_range", Value::Message(self.native_message("NativeSignedRange", &[("minimum", Value::I64(min)), ("maximum", Value::I64(max))])?)),
            LocalRule::UnsignedRange { min, max } => ("unsigned_range", Value::Message(self.native_range("NativeUnsignedRange", min, max)?)),
            LocalRule::Length { min, max } => ("length", Value::Message(self.native_range("NativeLength", min, max)?)),
            LocalRule::Utf8Bytes { min, max } => ("utf8_bytes", Value::Message(self.native_range("NativeLength", min, max)?)),
        };
        self.native_message("NativeLocalRule", &[(name, value)])
    }
    fn native_range(&self, name: &str, min: u64, max: u64) -> Result<DynamicMessage, MetadataError> {
        self.native_message(name, &[("minimum", Value::U64(min)), ("maximum", Value::U64(max))])
    }
    fn native_float_domain(&self, domain: Float64Domain) -> Result<DynamicMessage, MetadataError> {
        let (name, value) = match domain {
            Float64Domain::Finite => ("preset", Value::EnumNumber(1)),
            Float64Domain::NonnegativeFinite => ("preset", Value::EnumNumber(2)),
            Float64Domain::PositiveFinite => ("preset", Value::EnumNumber(3)),
            Float64Domain::PositiveUnit => ("preset", Value::EnumNumber(4)),
            Float64Domain::Interval(interval) => {
                let (lower, lower_inclusive, upper, upper_inclusive) = interval.endpoints();
                ("interval", Value::Message(self.native_message("NativeFloat64Interval", &[("lower_bits", Value::U64(lower)), ("lower_inclusive", Value::Bool(lower_inclusive)), ("upper_bits", Value::U64(upper)), ("upper_inclusive", Value::Bool(upper_inclusive))])?))
            }
        };
        self.native_message("NativeFloat64Domain", &[(name, value)])
    }
}

pub(super) fn decode(metadata: &DynamicMessage) -> Result<Option<(TypeRef, Vec<LocalRule>)>, MetadataError> {
    if uint(metadata, "metadata_version")? != Some(pse_contract::metadata::VERSION) { return Ok(None); }
    let Some(contract) = child(metadata, "native_contract")? else { return Ok(None); };
    decode_contract(contract).map(Some)
}
pub(super) fn decode_contract(contract: &DynamicMessage) -> Result<(TypeRef, Vec<LocalRule>), MetadataError> {
    known_fields(contract, &["native_type", "local_rules"])?;
    let ty = native_type(required_child(contract, "native_type")?)?;
    let rules = match value(contract, "local_rules")? {
        None => Vec::new(),
        Some(Value::List(rules)) => rules.iter().map(|rule| match rule { Value::Message(rule) => native_rule(rule), _ => Err(error("native local rule is not a typed message")) }).collect::<Result<_, _>>()?,
        _ => return Err(error("native local rules are not a repeated typed declaration")),
    };
    Ok((ty, rules))
}
fn required_child<'a>(message: &'a DynamicMessage, name: &str) -> Result<&'a DynamicMessage, MetadataError> {
    child(message, name)?.ok_or_else(|| error(format!("{}: required {name} is absent", message.message_descriptor().full_name())))
}
fn case<'a>(message: &DynamicMessage, names: &'a [&str]) -> Result<&'a str, MetadataError> {
    known_fields(message, names)?;
    let mut selected = None;
    for name in names { if value(message, name)?.is_some() {
        if selected.replace(*name).is_some() { return Err(error("native declaration has multiple oneof meanings")); }
    } }
    selected.ok_or_else(|| error(format!("{}: required native meaning is absent", message.message_descriptor().full_name())))
}
fn known_enum<T>(message: &DynamicMessage, name: &str, decode: impl FnOnce(i32) -> Option<T>) -> Result<Option<T>, MetadataError> {
    match enumeration(message, name, decode)? {
        Some(pse_contract::metadata::EnumValue::Known(value)) => Ok(Some(value)),
        Some(pse_contract::metadata::EnumValue::Unknown(number)) => Err(unavailable(format!("{}.{name}: unknown native vocabulary {number}", message.message_descriptor().full_name()))),
        None => Ok(None),
    }
}
fn native_type(message: &DynamicMessage) -> Result<TypeRef, MetadataError> {
    let scalar = known_enum(message, "scalar", scalar_value)?;
    Ok(match case(message, &["scalar", "named_owner", "optional_value", "repeated_value", "set_value", "map"])? {
        "scalar" => TypeRef::Scalar(scalar.ok_or_else(|| unavailable("native scalar is unavailable"))?),
        "named_owner" => TypeRef::Named(text(message, "named_owner")?.filter(|name| !name.is_empty()).ok_or_else(|| error("native named owner is empty"))?.into()),
        "optional_value" => TypeRef::Optional(Box::new(native_type(required_child(message, "optional_value")?)?)),
        "repeated_value" => TypeRef::Repeated(Box::new(native_type(required_child(message, "repeated_value")?)?)),
        "set_value" => TypeRef::Set(Box::new(native_type(required_child(message, "set_value")?)?)),
        _ => { let map = required_child(message, "map")?; known_fields(map, &["key", "value"])?; TypeRef::Map(Box::new(native_type(required_child(map, "key")?)?), Box::new(native_type(required_child(map, "value")?)?)) },
    })
}
fn unsigned(message: &DynamicMessage, name: &str) -> Result<u64, MetadataError> {
    match value(message, name)? { Some(Value::U64(value)) => Ok(*value), _ => Err(error(format!("required native {name} is not uint64"))) }
}
fn signed(message: &DynamicMessage, name: &str) -> Result<i64, MetadataError> {
    match value(message, name)? { Some(Value::I64(value)) => Ok(*value), _ => Err(error(format!("required native {name} is not sint64"))) }
}
fn bounds<T: PartialOrd>(min: T, max: T) -> Result<(T, T), MetadataError> {
    if min > max { Err(error("native range minimum exceeds maximum")) } else { Ok((min, max)) }
}
fn native_rule(message: &DynamicMessage) -> Result<LocalRule, MetadataError> {
    Ok(match case(message, &["required_value", "positive_duration", "float64_domain", "signed_range", "unsigned_range", "length", "utf8_bytes"])? {
        name @ ("required_value" | "positive_duration") => {
            if boolean(message, name)? != Some(true) { return Err(error("native unit rule must declare true")); }
            if name == "required_value" { LocalRule::Required } else { LocalRule::PositiveDuration }
        }
        "float64_domain" => LocalRule::Float64Domain(float_domain(required_child(message, "float64_domain")?)?),
        "signed_range" => { let range = required_child(message, "signed_range")?; known_fields(range, &["minimum", "maximum"])?; let (min, max) = bounds(signed(range, "minimum")?, signed(range, "maximum")?)?; LocalRule::SignedRange { min, max } },
        name => {
            let range = required_child(message, name)?; known_fields(range, &["minimum", "maximum"])?;
            let (min, max) = bounds(unsigned(range, "minimum")?, unsigned(range, "maximum")?)?;
            match name { "unsigned_range" => LocalRule::UnsignedRange { min, max }, "length" => LocalRule::Length { min, max }, _ => LocalRule::Utf8Bytes { min, max } }
        }
    })
}
fn float_domain(message: &DynamicMessage) -> Result<Float64Domain, MetadataError> {
    let preset = known_enum(message, "preset", |number| match number { 1 => Some(Float64Domain::Finite), 2 => Some(Float64Domain::NonnegativeFinite), 3 => Some(Float64Domain::PositiveFinite), 4 => Some(Float64Domain::PositiveUnit), _ => None })?;
    match case(message, &["preset", "interval"])? {
        "preset" => preset.ok_or_else(|| unavailable("native float preset is unavailable")),
        _ => {
            let interval = required_child(message, "interval")?;
            known_fields(interval, &["lower_bits", "lower_inclusive", "upper_bits", "upper_inclusive"])?;
            let interval = Float64Interval::new(f64::from_bits(unsigned(interval, "lower_bits")?), boolean(interval, "lower_inclusive")?.ok_or_else(|| error("lower inclusion is absent"))?, f64::from_bits(unsigned(interval, "upper_bits")?), boolean(interval, "upper_inclusive")?.ok_or_else(|| error("upper inclusion is absent"))?);
            if !interval.is_valid() { return Err(error("native binary64 interval is not finite/nonempty")); }
            Ok(Float64Domain::Interval(interval))
        }
    }
}
