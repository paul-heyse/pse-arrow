// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Strict native frontmatter reading. Authored document bytes are never serialized.
use std::{borrow::Cow, collections::{BTreeMap, BTreeSet}, fmt, path::Path, sync::OnceLock};
use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, de::{self, MapAccess, SeqAccess, Visitor}};
use serde_saphyr::granit_parser::{Event, Parser, ScalarStyle, Scanner, StrInput, TokenType};
use sha2::{Digest, Sha256};

/// Recursively typed metadata. Numerical values remain numerical, not coerced strings.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Value {
    /// YAML null.
    Null,
    /// YAML boolean.
    Bool(bool),
    /// Signed integer.
    Integer(i64),
    /// Unsigned integer.
    Unsigned(u64),
    /// Floating-point metadata, including original YAML nonfinite values.
    Float(f64),
    /// Authored text or normalized date/timestamp.
    String(String),
    /// Ordered metadata collection.
    Sequence(Vec<Value>),
    /// String-keyed metadata mapping.
    Mapping(BTreeMap<String, Value>),
}
impl Value {
    /// Borrow a string without coercing another metadata type.
    pub fn as_str(&self) -> Option<&str> { match self { Self::String(value) => Some(value), _ => None } }
    /// Borrow an ordered collection.
    pub fn as_sequence(&self) -> Option<&[Self]> { match self { Self::Sequence(value) => Some(value), _ => None } }
    /// Borrow a mapping.
    pub fn as_mapping(&self) -> Option<&BTreeMap<String, Self>> { match self { Self::Mapping(value) => Some(value), _ => None } }
}
struct StringKey(String);
impl<'de> Deserialize<'de> for StringKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyVisitor;
        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = StringKey;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { formatter.write_str("metadata mapping keys must be strings") }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value,E> { Ok(StringKey(value.to_owned())) }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value,E> { Ok(StringKey(value)) }
        }
        deserializer.deserialize_any(KeyVisitor)
    }
}
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MetadataVisitor;
        impl<'de> Visitor<'de> for MetadataVisitor {
            type Value = Value;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { formatter.write_str("scalar, sequence or string-keyed metadata mapping") }
            fn visit_unit<E: de::Error>(self) -> Result<Value,E> { Ok(Value::Null) }
            fn visit_none<E: de::Error>(self) -> Result<Value,E> { Ok(Value::Null) }
            fn visit_bool<E: de::Error>(self,v:bool)->Result<Value,E>{Ok(Value::Bool(v))}
            fn visit_i64<E: de::Error>(self,v:i64)->Result<Value,E>{Ok(Value::Integer(v))}
            fn visit_u64<E: de::Error>(self,v:u64)->Result<Value,E>{Ok(Value::Unsigned(v))}
            fn visit_f64<E: de::Error>(self,v:f64)->Result<Value,E>{Ok(Value::Float(v))}
            fn visit_str<E: de::Error>(self,v:&str)->Result<Value,E>{Ok(Value::String(v.to_owned()))}
            fn visit_string<E: de::Error>(self,v:String)->Result<Value,E>{Ok(Value::String(v))}
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq:A)->Result<Value,A::Error>{
                let mut values=Vec::new(); while let Some(value)=seq.next_element()? { values.push(value); } Ok(Value::Sequence(values))
            }
            fn visit_map<A: MapAccess<'de>>(self,mut map:A)->Result<Value,A::Error>{
                let mut values=BTreeMap::new();
                while let Some((StringKey(key),value))=map.next_entry()? {
                    if values.insert(key.clone(),value).is_some(){return Err(de::Error::custom(format!("duplicate metadata key {key:?}")));}
                }
                Ok(Value::Mapping(values))
            }
        }
        deserializer.deserialize_any(MetadataVisitor)
    }
}

/// One metadata admission failure with its original document context and parser message.
#[derive(Debug)]
pub struct MetadataError { message: String }
impl fmt::Display for MetadataError { fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { f.write_str(&self.message) } }
impl std::error::Error for MetadataError {}
fn invalid(message:impl fmt::Display)->MetadataError { MetadataError{message:message.to_string()} }
/// Result of parsing a document without modifying its source bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct Document<'a> {
    /// Strict interpreted metadata.
    pub metadata: BTreeMap<String, Value>,
    /// Original opening delimiter, metadata and closing delimiter bytes.
    pub prefix: &'a [u8],
    /// Original bytes after the closing delimiter.
    pub body: &'a [u8],
}

/// Read optional or required YAML frontmatter, preserving exact prefix/body bytes.
/// `identity` is the repository-relative owner used solely by sealed legacy exceptions.
/// # Errors
/// Invalid frontmatter/metadata, missing required delimiter, or malformed policy.
pub fn read<'a>(data:&'a [u8],where_:&str,required:bool,identity:Option<&str>,root:&Path)->Result<Document<'a>,MetadataError>{read_with_policy(data,where_,required,identity,root,None)}
/// Read using an already captured legacy policy rather than reopening its file.
/// # Errors
/// The same strict metadata admission failures as `read`.
pub fn read_with_policy<'a>(data:&'a [u8],where_:&str,required:bool,identity:Option<&str>,root:&Path,legacy_policy:Option<&str>)->Result<Document<'a>,MetadataError>{
    let result=(||{
        let mut lines=data.split_inclusive(|byte|*byte==b'\n');
        let first=lines.next().unwrap_or_default();
        if first.strip_suffix(b"\n").unwrap_or(first).strip_suffix(b"\r").unwrap_or(first.strip_suffix(b"\n").unwrap_or(first)) != b"---" {
            if required { return Err(invalid("missing YAML front matter")); }
            return Ok(Document{metadata:BTreeMap::new(),prefix:&[],body:data});
        }
        let mut end=first.len(); let start=end; let mut metadata_end=None;
        for line in lines {
            let source=line.strip_suffix(b"\n").unwrap_or(line).strip_suffix(b"\r").unwrap_or(line.strip_suffix(b"\n").unwrap_or(line));
            if source==b"---" { metadata_end=Some(end); end+=line.len(); break; }
            end+=line.len();
        }
        let metadata_end=metadata_end.ok_or_else(||invalid("unterminated YAML front matter"))?;
        let text=std::str::from_utf8(&data[start..metadata_end]).map_err(invalid)?;
        let encoded=legacy_encoding(text,identity,root,legacy_policy)?;
        let normalized=admit_yaml(&encoded)?;
        let options=serde_saphyr::options! {
            duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
            merge_keys: serde_saphyr::MergeKeyPolicy::Error,
            reject_unsupported_tags: true,
            strict_booleans: true,
            reject_non_finite_typeless_float: false,
            emit_comments: false,
        };
        let value:Value=serde_saphyr::from_str_with_options(&normalized,options).map_err(invalid)?;
        let Value::Mapping(metadata)=value else {return Err(invalid("front matter must contain one mapping"));};
        validate_fields(&metadata,None)?;
        Ok(Document{metadata,prefix:&data[..end],body:&data[end..]})
    })();
    result.map_err(|error:MetadataError|invalid(format!("{where_}: {error}")))
}

/// Validate the existing controlled `doc_*` vocabulary and optional declared topics.
/// # Errors
/// Unknown controlled fields, invalid values, duplicate or undeclared topics.
pub fn validate_fields(values:&BTreeMap<String,Value>,topics:Option<&BTreeSet<String>>)->Result<(),MetadataError>{
    for (key,value) in values {
        if !key.starts_with("doc_"){continue;}
        let allowed:&[&str]=match key.as_str(){
            "doc_role"=>&["contract","instruction","decision","plan","review","reference","evidence","index"],
            "doc_provenance"=>&["authored","generated","observed"],
            "doc_retention"=>&["while-current","while-dependent","owner-managed","protected-unknown"],
            "doc_retirement_trigger"=>&["owner-release","replacement-adopted","producer-policy"],
            "doc_topics"=>{
                let sequence=value.as_sequence().ok_or_else(||invalid("doc_topics must be a list of declared strings"))?;
                let strings:Vec<_>=sequence.iter().map(|value|value.as_str().ok_or_else(||invalid("doc_topics must be a list of declared strings"))).collect::<Result<_,_>>()?;
                let unique:BTreeSet<_>=strings.iter().copied().collect();
                if strings.len()!=unique.len(){return Err(invalid("doc_topics contains duplicate topics"));}
                if let Some(topics)=topics && unique.iter().any(|topic|!topics.contains(*topic)){return Err(invalid("undeclared doc_topics"));}
                continue;
            }
            "doc_owner"|"doc_retention_reason"|"doc_retention_owner"=>&[],
            _=>return Err(invalid(format!("unknown controlled metadata field {key}"))),
        };
        let text=value.as_str().filter(|text|!text.trim().is_empty()).ok_or_else(||invalid(format!("{key} must be a nonempty string")))?;
        if !allowed.is_empty()&&!allowed.contains(&text){return Err(invalid(format!("invalid {key}: {text:?}")));}
    }
    Ok(())
}

fn legacy_encoding<'a>(text:&'a str,identity:Option<&str>,root:&Path,supplied_policy:Option<&str>)->Result<Cow<'a,str>,MetadataError>{
    let Some(identity)=identity.filter(|identity|identity.starts_with("docs/adr/")) else{return Ok(Cow::Borrowed(text));};
    let path=root.join("docs/lifecycle.toml");
    let policy_text=if let Some(policy)=supplied_policy{Cow::Borrowed(policy)}else{
        if !path.is_file(){return Ok(Cow::Borrowed(text));}
        Cow::Owned(std::fs::read_to_string(path).map_err(invalid)?)
    };
    let policy:toml::Value=toml::from_str(&policy_text).map_err(invalid)?;
    let declarations:Vec<_>=policy.get("legacy_yaml").and_then(toml::Value::as_array).into_iter().flatten().filter(|entry|entry.get("path").and_then(toml::Value::as_str)==Some(identity)).collect();
    if declarations.len()>1 {return Err(invalid(format!("duplicate historical YAML encoding declaration: {identity}")));}
    let Some(declaration)=declarations.first() else{return Ok(Cow::Borrowed(text));};
    let scalars=declaration.get("scalars").and_then(toml::Value::as_table).ok_or_else(||invalid("historical scalar declarations must be a table"))?;
    let mut encoded=String::new();
    for line in text.split_inclusive('\n') {
        let source=line.trim_end_matches(['\r','\n']);
        if let Some((key,literal))=source.split_once(':')
            && let Some(hash)=scalars.get(key).and_then(toml::Value::as_str)
            && Sha256::digest(literal.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>()==hash {
                encoded.push_str(key);encoded.push_str(": ");
                encoded.push_str(&serde_json::to_string(literal.trim()).map_err(invalid)?);
                encoded.push_str(&line[source.len()..]);continue;
        }
        encoded.push_str(line);
    }
    Ok(Cow::Owned(encoded))
}

fn admit_yaml(text:&str)->Result<Cow<'_,str>,MetadataError>{
    let mut documents=0; let mut stack:Vec<Option<bool>>=Vec::new();
    let mut date_anchors=BTreeSet::new(); let mut timestamp_tags=BTreeSet::new(); let mut edits=Vec::new();
    for item in Parser::new_from_str(text) {
        let (event,span)=item.map_err(invalid)?;
        let key=stack.last().is_some_and(|frame|*frame==Some(true));
        match event {
            Event::DocumentStart(..)=>{documents+=1;if documents>1{return Err(invalid("front matter must contain one mapping"));}}
            Event::MappingStart(_,_,ref tag)|Event::SequenceStart(_,_,ref tag)=>{
                if key{return Err(invalid("metadata mapping keys must be strings"));}
                if let Some(tag)=tag {check_tag(tag)?;}
                let mapping=matches!(event,Event::MappingStart(..));
                finish_node(&mut stack);stack.push(mapping.then_some(true));
            }
            Event::MappingEnd|Event::SequenceEnd=>{stack.pop();}
            Event::Scalar(value,style,anchor,tag)=>{
                if let Some(tag)=&tag {check_tag(tag)?;}
                let timestamp=tag.as_ref().is_some_and(|tag|tag.suffix_in_namespace("tag:yaml.org,2002:").as_deref()==Some("timestamp"));
                let implicit=tag.is_none()&&style==ScalarStyle::Plain;
                if timestamp||implicit {
                    if let Some(normalized)=normalize_date(&value,timestamp)? {
                        if key{return Err(invalid("metadata mapping keys must be strings"));}
                        if anchor!=0{date_anchors.insert(anchor);}
                        if timestamp {
                            let start=span.tag_start().and_then(|marker|marker.byte_offset()).ok_or_else(||invalid("metadata timestamp tag span absent"))?;
                            timestamp_tags.insert(start);
                        }
                        let start=span.start.byte_offset().ok_or_else(||invalid("metadata scalar byte span absent"))?;
                        let end=span.end.byte_offset().ok_or_else(||invalid("metadata scalar byte span absent"))?;
                        edits.push((start..end,serde_json::to_string(&normalized).map_err(invalid)?));
                    }
                }
                finish_node(&mut stack);
            }
            Event::Alias(anchor)=>{if key&&date_anchors.contains(&anchor){return Err(invalid("metadata mapping keys must be strings"));}finish_node(&mut stack);}
            _=>{}
        }
    }
    if edits.is_empty(){return Ok(Cow::Borrowed(text));}
    // The supplier deliberately refuses a timestamp-tagged scalar as a String.
    // After calendar validation, retag only the matching parser-identified tokens
    // in the private decoding input; preserve anchors, source bytes and all other tags.
    if !timestamp_tags.is_empty() {
        for token in Scanner::new(StrInput::new(text)) {
            let token=token.map_err(invalid)?;
            if matches!(token.token_type(),TokenType::Tag(..)) {
                let range=token.span().byte_range().ok_or_else(||invalid("metadata tag byte span absent"))?;
                if timestamp_tags.remove(&range.start) {edits.push((range,"!!str".to_owned()));}
            }
        }
        if !timestamp_tags.is_empty(){return Err(invalid("metadata timestamp tag token absent"));}
    }
    edits.sort_by(|left,right|right.0.start.cmp(&left.0.start));
    let mut rewritten=text.to_owned();
    for (range,value) in edits{rewritten.replace_range(range,&value);}
    Ok(Cow::Owned(rewritten))
}
fn finish_node(stack:&mut [Option<bool>]){if let Some(Some(key))=stack.last_mut(){*key=!*key;}}
fn check_tag(tag:&serde_saphyr::granit_parser::Tag)->Result<(),MetadataError>{
    if tag.core_suffix().is_some() || tag.suffix_in_namespace("tag:yaml.org,2002:").as_deref()==Some("timestamp") {return Ok(());}
    Err(invalid(format!("unsupported metadata tag {tag}")))
}
fn normalize_date(value:&str,explicit:bool)->Result<Option<String>,MetadataError>{
    static DATE:OnceLock<Result<Regex,regex::Error>>=OnceLock::new();
    let pattern=DATE.get_or_init(||Regex::new(r"^([0-9]{4})-([0-9]{1,2})-([0-9]{1,2})(?:(?:[Tt]|[ \t]+)([0-9]{1,2}):([0-9]{2}):([0-9]{2})(?:\.([0-9]*))?[ \t]*(Z|[+-][0-9]{1,2}(?::[0-9]{2})?)?)?$" )).as_ref().map_err(invalid)?;
    let Some(captures)=pattern.captures(value) else {return if explicit{Err(invalid("invalid metadata timestamp"))}else{Ok(None)};};
    // YAML resolves date-only implicit scalars only with two-digit month/day.
    if !explicit && captures.get(4).is_none() && (captures[2].len() != 2 || captures[3].len() != 2) {return Ok(None);}
    let parse=|index|captures.get(index).ok_or_else(||invalid("timestamp field absent"))?.as_str().parse::<u32>().map_err(invalid);
    let year=i32::try_from(parse(1)?).map_err(invalid)?;
    if year == 0 {return Err(invalid("invalid metadata date"));}
    let date=NaiveDate::from_ymd_opt(year,parse(2)?,parse(3)?).ok_or_else(||invalid("invalid metadata date"))?;
    let base=format!("{:04}-{:02}-{:02}",date.year(),date.month(),date.day());
    if captures.get(4).is_none(){return Ok(Some(base));}
    let time=NaiveTime::from_hms_opt(parse(4)?,parse(5)?,parse(6)?).ok_or_else(||invalid("invalid metadata timestamp time"))?;
    let mut result=format!("{base}T{:02}:{:02}:{:02}",time.hour(),time.minute(),time.second());
    // Match Python datetime.isoformat(): six fractional digits, UTC as +00:00.
    let fraction=captures.get(7).map_or("",|value|value.as_str());
    let fraction=format!("{fraction:0<6}"); let fraction=&fraction[..6];
    if fraction.bytes().any(|byte|byte!=b'0'){result.push('.');result.push_str(fraction);}
    if let Some(zone)=captures.get(8).map(|value|value.as_str()) {
        if zone=="Z" {result.push_str("+00:00");}
        else {
            let sign=&zone[..1];let (hours,minutes)=zone[1..].split_once(':').unwrap_or((&zone[1..],"0"));
            let hours=hours.parse::<u32>().map_err(invalid)?;let minutes=minutes.parse::<u32>().map_err(invalid)?;
            if hours>=24||minutes>=60{return Err(invalid("invalid metadata timestamp offset"));}
            if hours==0&&minutes==0 {result.push_str("+00:00");} else{result.push_str(&format!("{sign}{hours:02}:{minutes:02}"));}
        }
    }
    Ok(Some(result))
}
