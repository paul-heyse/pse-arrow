// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed immutable table admission, independent of any selected model instance.
use crate::{CheckedPackage,Type,TypeContext,Result,invalid};
use crate::specialize::value::{Evaluator,Value,Environment};
use pse_ids::SemanticId;
use std::collections::{BTreeMap,BTreeSet};
/// Derived table data. Authored declarations remain the durable authority.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Table {
    /// Ordered complete key types.
    pub keys:Vec<Type>,
    /// Whole scalar or heterogeneous row type.
    pub result:Type,
    /// Named heterogeneous columns in source order.
    pub columns:Vec<(String,Type)>,
    /// Unique admitted keys and complete values.
    pub rows:BTreeMap<Vec<Value>,Value>,
    /// Declared fallback, evaluated at admission.
    pub default:Option<Value>,
    /// Missing values remain explicit.
    pub optional:bool,
    /// Dataset declaration supplying each row.
    pub origins:BTreeMap<Vec<Value>,SemanticId>,
}
pub(crate) fn admit(p:&mut CheckedPackage,c:&TypeContext<'_>)->Result<()> {
    let env=Environment::new();
    let mut pending=p.declarations.values().filter(|r|r.value.table.is_some()).map(|r|r.declaration_id).collect::<BTreeSet<_>>();
    while !pending.is_empty(){
        let mut progress=false;let mut failure=None;
        for id in pending.clone(){match table(p,c,&env,id){
            Ok(value)=>{p.tables.insert(id,value);pending.remove(&id);progress=true;},
            Err(error)=>failure=Some(error),
        }}
        if !progress{return Err(failure.unwrap_or_else(||invalid(SemanticId::NIL,"cyclic table data")));}
    }
    for row in p.declarations.values(){
        if let Some(dataset)=&row.value.dataset {
            let target=p.resolve(row.declaration_id,&dataset.table).ok_or_else(||invalid(row.declaration_id,"unknown dataset table"))?;
            if !p.tables.contains_key(&target){return Err(invalid(row.declaration_id,"dataset target is not a table"));}
        }
        if let Some(entity)=&row.value.entity {
            let kind=p.resolve(row.declaration_id,&entity.kind_name).ok_or_else(||invalid(row.declaration_id,"entity kind"))?;
            let members=p.members.get(&kind).ok_or_else(||invalid(kind,"entity kind members"))?;
            let mut supplied=BTreeSet::new();
            for attribute in &entity.attributes {
                if !supplied.insert(&attribute.name){return Err(invalid(row.declaration_id,"duplicate entity attribute"));}
                let member=members.get(&attribute.name).ok_or_else(||invalid(row.declaration_id,"unknown entity attribute"))?;
                let ty=p.types.get(member).ok_or_else(||invalid(*member,"attribute type absent"))?;
                evaluator(p,c,&env,row.declaration_id).text(&attribute.expression,Some(ty))?;
            }
            for (name,member) in members {
                if !supplied.contains(name) && let Some(source)=p.declarations[member].value.binding.as_ref().and_then(|b|b.expression.as_deref()) {
                    evaluator(p,c,&env,*member).text(source,p.types.get(member))?;
                }
                if p.declarations[member].value.kind==pse_model::generated::enums::ModelingDeclarationKind::Attribute && !supplied.contains(name) && p.declarations[member].value.binding.as_ref().is_none_or(|b|b.expression.is_none()){
                    return Err(invalid(row.declaration_id,format!("missing attribute {name}")));
                }
            }
        }
    }
    Ok(())
}
fn evaluator<'a,'b>(p:&'a CheckedPackage,c:&'a TypeContext<'b>,env:&'a Environment,at:SemanticId)->Evaluator<'a,'b>{Evaluator{package:p,physical:c,at,env,limit:100_000,stack:vec![]}}
fn table(p:&CheckedPackage,c:&TypeContext<'_>,env:&Environment,id:SemanticId)->Result<Table>{
    let contract=p.declarations[&id].value.table.as_ref().ok_or_else(||invalid(id,"table contract"))?;
    let names=p.named_types(id);let variables=BTreeSet::new();
    let mut key_names=BTreeSet::new();
    let keys=contract.keys.iter().map(|key|{if !key_names.insert(&key.name){return Err(invalid(id,"duplicate table key name"));}c.resolve(&key.type_name,&variables,&names,id)}).collect::<Result<Vec<_>>>()?;
    let mut column_names=BTreeSet::new();
    let columns=contract.columns.iter().map(|column|{if !column_names.insert(&column.name){return Err(invalid(id,"duplicate table column"));}Ok((column.name.clone(),c.resolve(&column.type_name,&variables,&names,id)?))}).collect::<Result<Vec<_>>>()?;
    let result=if columns.is_empty(){c.resolve(&contract.value_type,&variables,&names,id)?}else{Type::Row(id)};
    let default=contract.default_value.as_ref().map(|source|evaluator(p,c,env,id).text(source,Some(&result))).transpose()?;
    let mut table=Table{keys,result,columns,rows:BTreeMap::new(),origins:BTreeMap::new(),default,optional:contract.missing_policy=="optional"};
    for row in p.declarations.values(){
        let Some(data)=&row.value.dataset else{continue;};
        if p.resolve(row.declaration_id,&data.table)!=Some(id){continue;}
        if data.source.is_empty(){return Err(invalid(row.declaration_id,"dataset provenance is empty"));}
        for entry in &data.rows {
            if entry.keys.len()!=table.keys.len(){return Err(invalid(row.declaration_id,"dataset key arity"));}
            let keys=entry.keys.iter().zip(&table.keys).map(|(source,ty)|evaluator(p,c,env,row.declaration_id).text(source,Some(ty))).collect::<Result<Vec<_>>>()?;
            if table.rows.contains_key(&keys){return Err(invalid(row.declaration_id,"duplicate dataset key"));}
            let value=if table.columns.is_empty(){
                if entry.values.len()!=1{return Err(invalid(row.declaration_id,"scalar table row arity"));}
                evaluator(p,c,env,row.declaration_id).text(&entry.values[0],Some(&table.result))?
            }else{
                if entry.values.len()!=table.columns.len(){return Err(invalid(row.declaration_id,"heterogeneous table row arity"));}
                Value::Row{table:id,fields:entry.values.iter().zip(&table.columns).map(|(source,(name,ty))|Ok((name.clone(),evaluator(p,c,env,row.declaration_id).text(source,Some(ty))?))).collect::<Result<_>>()?}
            };
            table.origins.insert(keys.clone(),row.declaration_id);table.rows.insert(keys,value);
        }
    }
    Ok(table)
}
