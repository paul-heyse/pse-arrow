// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Reserved typed analysis facts; package guards own all scientific consequences.
use crate::{DeclarationId,Type,Bindings,Result,invalid,specialize::{Environment,Value}};
use pse_ids::SemanticId;
pub use pse_model::generated::enums::ModelingAnalysisRoute as Route;
/// The built-in declaration of the analysis route enumeration.
fn route_id()->DeclarationId{DeclarationId::from(pse_ids::named_id(SemanticId::NIL,"pse.modeling.analysis.route"))}
fn value(route:Route)->Value{Value::Enum{enumeration:route_id(),member:route.as_str().into()}}
pub(crate) fn constant(name:&str)->Option<Value>{
    name.strip_prefix("analysis.").and_then(|s|s.parse::<Route>().ok()).map(value)
}
pub(crate) fn type_of(name:&str)->Option<Type>{
    if name=="analysis.dynamic"{Some(Type::Boolean)}
    else if name=="analysis.route"||constant(name).is_some(){Some(Type::Enum(route_id()))}
    else{None}
}
pub(crate) fn facts(input:&Environment)->Result<Environment>{
    let mut output=input.clone();
    if input.keys().any(|key|key.starts_with("analysis.")&&key!="analysis.route"&&key!="analysis.dynamic"){
        return Err(invalid(SemanticId::NIL,"unknown or read-only analysis fact"));
    }
    let route=match input.get("analysis.route"){
        None=>Route::Steady,
        Some(Value::Enum{enumeration,member})if *enumeration==route_id()=>member.parse::<Route>().map_err(|_|invalid(SemanticId::NIL,"unknown analysis route"))?,
        _=>return Err(invalid(SemanticId::NIL,"analysis route requires its declared enum type")),
    };
    let dynamic=Value::Boolean(route!=Route::Steady);
    if input.get("analysis.dynamic").is_some_and(|v|*v!=dynamic){return Err(invalid(SemanticId::NIL,"analysis.dynamic disagrees with the selected route"));}
    output.insert("analysis.route".into(),value(route));output.insert("analysis.dynamic".into(),dynamic);Ok(output)
}
impl Bindings {
    /// Select one mode. The dynamic Boolean is derived, never an independent decision.
    pub fn with_analysis(mut self,route:Route)->Self{
        self.facts.insert("analysis.route".into(),value(route));
        self.facts.insert("analysis.dynamic".into(),Value::Boolean(route!=Route::Steady));self
    }
    /// Read and validate the selected mode, defaulting to steady analysis.
    pub fn analysis_route(&self)->Result<Route>{
        let facts=facts(&self.facts)?;
        let Value::Enum{member,..}=&facts["analysis.route"] else{return Err(invalid(SemanticId::NIL,"analysis route missing"));};
        member.parse().map_err(|_|invalid(SemanticId::NIL,"analysis route missing"))
    }
}
