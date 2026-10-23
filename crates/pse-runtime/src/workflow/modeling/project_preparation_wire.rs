// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical boundary projection of the existing structural bindings, with explicit tags.
use pse_modeling::{Bindings, DeclarationId, specialize::Value};
use pse_ids::SemanticId;
use std::collections::{BTreeMap,BTreeSet};
#[derive(serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct PreparationBindings {
    formulation:PreparationFormulation,
    arguments:BTreeMap<String,PreparationStructuralValue>,
    scope:BTreeMap<String,PreparationStructuralValue>,
    facts:Vec<(PreparationFact,PreparationStructuralValue)>,
    demand:Vec<String>,
}
#[derive(serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct PreparationFormulation {omitted:BTreeSet<SemanticId>,elastic:BTreeMap<SemanticId,PreparationStructuralValue>}
#[derive(serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
enum PreparationFact {Route{},Dynamic{},ObjectiveLevel{},Stage{name:String}}
impl From<&pse_modeling::analysis::Fact> for PreparationFact {fn from(f:&pse_modeling::analysis::Fact)->Self{match f{pse_modeling::analysis::Fact::Route=>Self::Route{},pse_modeling::analysis::Fact::Dynamic=>Self::Dynamic{},pse_modeling::analysis::Fact::ObjectiveLevel=>Self::ObjectiveLevel{},pse_modeling::analysis::Fact::Stage(name)=>Self::Stage{name:name.clone()}}}}
impl From<PreparationFact> for pse_modeling::analysis::Fact {fn from(f:PreparationFact)->Self{match f{PreparationFact::Route{}=>Self::Route,PreparationFact::Dynamic{}=>Self::Dynamic,PreparationFact::ObjectiveLevel{}=>Self::ObjectiveLevel,PreparationFact::Stage{name}=>Self::Stage(name)}}}
#[derive(serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(tag="value_kind",rename_all="snake_case",deny_unknown_fields)]
enum PreparationStructuralValue {
    Boolean{value:bool},Integer{value:i64},Number{bits:u64,quantity:pse_quantity::QuantityTypeId},
    Coordinate{id:SemanticId,bits:u64,quantity:pse_quantity::QuantityTypeId},Text{value:String},
    Entity{id:DeclarationId,kind:DeclarationId},
    Enumeration{enumeration:DeclarationId,member:SemanticId},
    Identifier{scheme:DeclarationId,value:String},
    Definition{id:DeclarationId,bindings:BTreeMap<String,PreparationStructuralValue>},
    Function{id:DeclarationId},Row{table:DeclarationId,names:Vec<String>,fields:Vec<PreparationStructuralValue>},
    Set{items:Vec<PreparationStructuralValue>},Tuple{items:Vec<PreparationStructuralValue>},
    QuantityType{id:pse_quantity::QuantityTypeId},ReferenceState{id:pse_quantity::ReferenceStateId},Missing{},
}
impl From<&Value> for PreparationStructuralValue {
    fn from(v:&Value)->Self {match v {
        Value::Boolean(value)=>Self::Boolean{value:*value},Value::Integer(value)=>Self::Integer{value:*value},
        Value::Number{bits,quantity}=>Self::Number{bits:*bits,quantity:*quantity},Value::Coordinate{id,bits,quantity}=>Self::Coordinate{id:*id,bits:*bits,quantity:*quantity},
        Value::Text(value)=>Self::Text{value:value.clone()},Value::Entity{id,kind}=>Self::Entity{id:*id,kind:*kind},
        Value::Enum{enumeration,member}=>Self::Enumeration{enumeration:*enumeration,member:*member},Value::Identifier{scheme,value}=>Self::Identifier{scheme:*scheme,value:value.clone()},
        Value::Definition{id,bindings}=>Self::Definition{id:*id,bindings:bindings.iter().map(|(k,v)|(k.clone(),v.into())).collect()},Value::Function(id)=>Self::Function{id:*id},
        Value::Row{table,names,fields}=>Self::Row{table:*table,names:names.to_vec(),fields:fields.iter().map(Into::into).collect()},
        Value::Set(items)=>Self::Set{items:items.iter().map(Into::into).collect()},Value::Tuple(items)=>Self::Tuple{items:items.iter().map(Into::into).collect()},
        Value::QuantityType(id)=>Self::QuantityType{id:*id},Value::ReferenceState(id)=>Self::ReferenceState{id:*id},Value::Missing=>Self::Missing{},
    }}
}
impl From<PreparationStructuralValue> for Value {
    fn from(v:PreparationStructuralValue)->Self {match v {
        PreparationStructuralValue::Boolean{value}=>Self::Boolean(value),PreparationStructuralValue::Integer{value}=>Self::Integer(value),
        PreparationStructuralValue::Number{bits,quantity}=>Self::Number{bits,quantity},PreparationStructuralValue::Coordinate{id,bits,quantity}=>Self::Coordinate{id,bits,quantity},
        PreparationStructuralValue::Text{value}=>Self::Text(value),PreparationStructuralValue::Entity{id,kind}=>Self::Entity{id,kind},
        PreparationStructuralValue::Enumeration{enumeration,member}=>Self::Enum{enumeration,member},PreparationStructuralValue::Identifier{scheme,value}=>Self::Identifier{scheme,value},
        PreparationStructuralValue::Definition{id,bindings}=>Self::Definition{id,bindings:bindings.into_iter().map(|(k,v)|(k,v.into())).collect()},PreparationStructuralValue::Function{id}=>Self::Function(id),
        PreparationStructuralValue::Row{table,names,fields}=>Self::Row{table,names:names.into(),fields:fields.into_iter().map(Into::into).collect()},
        PreparationStructuralValue::Set{items}=>Self::Set(items.into_iter().map(Into::into).collect()),PreparationStructuralValue::Tuple{items}=>Self::Tuple(items.into_iter().map(Into::into).collect()),
        PreparationStructuralValue::QuantityType{id}=>Self::QuantityType(id),PreparationStructuralValue::ReferenceState{id}=>Self::ReferenceState(id),PreparationStructuralValue::Missing{}=>Self::Missing,
    }}
}
pub(super) fn serialize<S:serde::Serializer>(v:&Bindings,s:S)->Result<S::Ok,S::Error>{
    use serde::Serialize;
    PreparationBindings{formulation:PreparationFormulation{omitted:v.formulation.omitted.clone(),elastic:v.formulation.elastic.iter().map(|(k,v)|(*k,v.into())).collect()},arguments:v.arguments.iter().map(|(k,v)|(k.clone(),v.into())).collect(),scope:v.scope.iter().map(|(k,v)|(k.clone(),v.into())).collect(),facts:v.facts.iter().map(|(k,v)|(k.into(),v.into())).collect(),demand:v.demand.clone()}.serialize(s)
}
pub(super) fn deserialize<'de,D:serde::Deserializer<'de>>(d:D)->Result<Bindings,D::Error>{
    use serde::Deserialize;let v=PreparationBindings::deserialize(d)?;
    let mut facts=BTreeMap::new();for (k,v) in v.facts{if facts.insert(k.into(),v.into()).is_some(){return Err(serde::de::Error::custom("duplicate structural fact"));}}
    Ok(Bindings{formulation:pse_modeling::specialize::Formulation{omitted:v.formulation.omitted,elastic:v.formulation.elastic.into_iter().map(|(k,v)|(k,v.into())).collect()},arguments:v.arguments.into_iter().map(|(k,v)|(k,v.into())).collect(),scope:v.scope.into_iter().map(|(k,v)|(k,v.into())).collect(),facts,demand:v.demand})
}
