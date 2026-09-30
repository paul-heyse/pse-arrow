// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Instantiated physical boundary operations; numerical lowering remains ordinary functions.
use super::*;
use crate::{BoundaryRef, PhysicalOperation, PhysicalRefinement, TransferDirection};

impl Engine<'_, '_> {
    pub(super) fn validate_contextual_equations(&self)->Result<()> {
        let types=self.model.symbols.iter().map(|(id,symbol)|(symbol_name(*id),symbol.ty.clone())).collect::<BTreeMap<_,_>>();
        let functions=self.model.function_contracts(self.p);
        fn check(equation:&Equation,types:&BTreeMap<String,Type>,functions:&CheckedPackage,context:&TypeContext<'_>,at:DeclarationId)->Result<()> {
            match &equation.kind {
                EquationKind::Relation {lhs,rhs,..}=> {
                    let mut contextual=false;
                    for expression in [lhs,rhs] { expression.walk(|expr|match &expr.kind {
                        ExprKind::Path(path)=>{if types.get(&dsl::render_path(path)).is_some_and(|ty|ty.physical_refinement().is_some()){contextual=true;}},
                        ExprKind::NamedCall {name,..}=>{if functions.resolve(at,name).and_then(|id|functions.functions.get(&id)).is_some_and(|f|f.result.physical_refinement().is_some()){contextual=true;}},
                        _=>{},
                    });}
                    if contextual {
                        let left=crate::expression::infer(lhs,types,functions,context,at,None)?;
                        let right=crate::expression::infer(rhs,types,functions,context,at,Some(&left))?;
                        if left!=right {return Err(invalid(at,"equation contextual owners, coordinates or physical roles differ"));}
                    }
                }
                EquationKind::Conditional {then,otherwise,..}=>{check(then,types,functions,context,at)?;check(otherwise,types,functions,context,at)?;}
            }
            Ok(())
        }
        for row in &self.model.equations {check(&row.equation,&types,&functions,self.c,row.lineage.declaration)?;}
        Ok(())
    }
    pub(super) fn resolve_boundary(&self, instance: InstanceId, at: DeclarationId, text: &str, env: &Environment) -> Result<BoundaryRef> {
        let expression = dsl::parse_expr(text).map_err(|error| invalid(at, error.to_string()))?;
        let ExprKind::Path(path) = expression.kind else { return Err(invalid(at, "a boundary must name its actual owner and coordinates")); };
        let (instance, declaration, coordinates) = self.resolve_path(instance, at, &path, env, true)?;
        if self.p.declarations[&declaration].value.kind != Kind::Boundary {
            return Err(invalid(at, "a transfer endpoint must name a boundary declaration"));
        }
        Ok(BoundaryRef::Bound { instance, declaration, coordinates: coordinates.into_iter().map(|(_, value)| value).collect() })
    }
    pub(super) fn bind_physical_owner(&self, ty: &Type, instance: InstanceId, env: &Environment, at: DeclarationId) -> Result<Type> {
        let Type::RefinedQuantity { quantity, refinement: PhysicalRefinement::Transfer { boundary, direction } } = ty else { return Ok(ty.clone()); };
        let BoundaryRef::Declared(declaration) = boundary else { return Ok(ty.clone()); };
        if !self.states[&instance].members.values().any(|member| member == declaration) {
            return Err(invalid(at, "transfer boundary is not a member of the actual owner"));
        }
        let row = &self.p.declarations[declaration];
        let boundary = row.value.boundary.as_ref().ok_or_else(|| invalid(at, "transfer boundary declaration absent"))?;
        let values = boundary.indices.iter().map(|index| env.get(&index.name).cloned().ok_or_else(|| invalid(at, "transfer lacks an actual boundary coordinate"))).collect::<Result<Vec<_>>>()?;
        let coordinates = self.member_coordinates(instance, *declaration, values)?;
        let id = quantity.resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions).map_err(|error| invalid(at, error.to_string()))?;
        let physical = self.c.quantities.quantity_type(id).map_err(|error| invalid(at, error.to_string()))?;
        let kind = self.c.quantities.kind(physical.key.kind).map_err(|error| invalid(at, error.to_string()))?;
        if physical.key.reference_state.is_some() || !kind.extensive || kind.addition_kind != pse_quantity::QuantityAdditionKind::Additive {
            return Err(invalid(at, "a transfer carries a datum-free extensive quantity, not a material reference point"));
        }
        Ok(Type::RefinedQuantity { quantity: quantity.clone(), refinement: PhysicalRefinement::Transfer { boundary: BoundaryRef::Bound { instance, declaration: *declaration, coordinates: coordinates.into_iter().map(|(_,value)|value).collect() }, direction:*direction } })
    }
    pub(super) fn contextual_call(&mut self, instance: InstanceId, at: DeclarationId, name: &str, args: &[Expr], env: &Environment, chain: &[DeclarationId]) -> Result<Option<Expr>> {
        if let Some(function)=self.p.resolve(at,name)
            && let Some(PhysicalOperation::ReferenceTranslation(descriptor))=self.p.functions.get(&function).and_then(|function|function.physical_operation.as_ref()) {
            return self.reference_translation_call(instance,at,function,descriptor.clone(),args,env,chain).map(Some);
        }
        if !matches!(name,"transfer"|"reorient"|"reflect") { return Ok(None); }
        let direction = |expression: &Expr| match dsl::render_expr(expression).as_str() {
            "Into" => Ok(TransferDirection::Into), "OutOf" => Ok(TransferDirection::OutOf),
            _ => Err(invalid(at, "direction must explicitly name Into or OutOf")),
        };
        let (input, target_direction) = match (name,args) {
            ("transfer",[value,_,orientation]) | ("reflect",[_,value,orientation]) => (value,direction(orientation)?),
            ("reorient",[value,orientation]) => (value,direction(orientation)?),
            _ => return Err(invalid(at, "transfer(value,boundary,direction), reorient(value,direction), or reflect(exchange,value,direction) required")),
        };
        let input = self.rewrite(instance, input, env, chain)?;
        let types = self.model.symbols.iter().map(|(id,symbol)|(symbol_name(*id),symbol.ty.clone())).collect::<BTreeMap<_,_>>();
        let actual = crate::expression::infer(&input,&types,&self.model.function_contracts(self.p),self.c,at,None)?;
        let quantity = actual.quantity_scheme().ok_or_else(|| invalid(at,"a physical rate is required"))?.clone();
        let source = actual.physical_refinement().cloned();
        let (result, factor, exchange) = match name {
            "transfer" => {
                if source.is_some() { return Err(invalid(at,"an existing transfer must use reorient or reflect")); }
                let id=quantity.resolve_with_evidence(self.c.quantities,&BTreeMap::new(),self.c.preconditions).map_err(|error|invalid(at,error.to_string()))?;
                let contract=self.c.quantities.quantity_type(id).map_err(|error|invalid(at,error.to_string()))?;
                let kind=self.c.quantities.kind(contract.key.kind).map_err(|error|invalid(at,error.to_string()))?;
                if contract.key.reference_state.is_some() || !kind.extensive || kind.addition_kind != pse_quantity::QuantityAdditionKind::Additive { return Err(invalid(at,"a transfer requires a datum-free extensive rate")); }
                let boundary = self.resolve_boundary(instance,at,&dsl::render_expr(&args[1]),env)?;
                (PhysicalRefinement::Transfer {boundary,direction:target_direction},1,None)
            }
            "reorient" => {
                let (result,factor)=source.as_ref().ok_or_else(||invalid(at,"reorient requires a transfer"))?.reorient(target_direction,at)?;
                (result,factor,None)
            }
            "reflect" => {
                let ExprKind::Path(path)=&args[0].kind else {return Err(invalid(at,"reflection names an exchange declaration"));};
                let (owner,declaration,coordinates)=self.resolve_path(instance,at,path,env,true)?;
                let pair=self.model.exchanges.get(&member_id(owner,declaration,&coordinates)).ok_or_else(||invalid(at,"the selected exchange is not instantiated"))?;
                let source=source.as_ref().ok_or_else(||invalid(at,"reflect requires a transfer"))?;
                let PhysicalRefinement::Transfer {boundary,..}=source else {return Err(invalid(at,"reflect requires a directed transfer"));};
                let target=if boundary==&pair.first {&pair.second} else {&pair.first};
                let (result,factor)=pair.reflect(source,target,target_direction)?;
                (result,factor,Some(pair.declaration))
            }
            _ => return Err(invalid(at,"unknown contextual operation")),
        };
        let output=Type::RefinedQuantity {quantity,refinement:result.clone()};
        let operation=PhysicalOperation::Transfer {source,result,factor,exchange};
        self.physical_transfer_function(at,actual,output,operation,factor,input).map(Some)
    }

    fn reference_translation_call(&mut self,instance:InstanceId,at:DeclarationId,function:DeclarationId,mut descriptor:crate::contextual::ReferenceTranslation,args:&[Expr],env:&Environment,chain:&[DeclarationId])->Result<Expr> {
        use pse_quantity::{CanonicalConversionPlan as Canonical,scheme::Scheme};
        if args.len()!=3 {return Err(invalid(at,"reference translation requires value, composition and actual members"));}
        self.reader.read(self.p,function,self.p.supplies_test_only(function),||crate::provenance::supplied_by(self.p,function))?;
        let contract=&self.p.functions[&function];
        let source_types=self.source_types(at,env)?;
        for ((_,formal),actual) in contract.arguments.iter().zip(args) {
            let actual=crate::expression::infer(actual,&source_types,self.p,self.c,at,Some(formal))?;
            if actual!=*formal {return Err(invalid(at,"reference translation actual contract differs"));}
        }
        let Value::Set(members)=self.eval(at,env,&dsl::render_expr(&args[2]),Some(&contract.arguments[2].1))? else {return Err(invalid(at,"translation requires a finite actual component set"));};
        if members.is_empty() || members.len()!=members.iter().collect::<BTreeSet<_>>().len() {return Err(invalid(at,"translation components must be nonempty and distinct"));}
        let ExprKind::Path(path)=&args[1].kind else {return Err(invalid(at,"translation composition names an actual indexed group"));};
        let weights=self.argument_group(instance,path,env,chain)?;
        if weights.len()!=members.len() || weights.iter().any(|(coordinates,_)|coordinates.len()!=1 || !members.contains(&coordinates[0])) {return Err(invalid(at,"translation composition must cover exactly its actual members"));}
        let mut anchor_env=Environment::new();
        let mut conditions=Vec::new();
        for (name,expression,attribute) in [("translation_temperature",&descriptor.temperature,"temperature"),("translation_pressure",&descriptor.pressure,"pressure")] {
            let quantity=self.p.reference_attribute_type(attribute,function)?;
            let value=self.eval(function,&anchor_env,&dsl::render_expr(expression),Some(&Type::Quantity(Scheme::Concrete(quantity))))?;
            let Value::Number {bits,quantity}=value else {return Err(invalid(at,"reference conditions must be physical constants"));};
            let canonical=Canonical::canonical(self.c.quantities,quantity).and_then(|plan|plan.apply(f64::from_bits(bits))).map_err(|error|invalid(at,error.to_string()))?;
            conditions.push(canonical);anchor_env.insert(name.into(),value);
        }
        let mut selected_weights=Vec::new();
        for (coordinates,weight) in weights {
            self.reserve(1)?;
            let Value::Entity {id:member,kind}=&coordinates[0] else {return Err(invalid(at,"composition member must be an admitted entity"));};
            if !self.p.refines(*kind,descriptor.component_kind) {return Err(invalid(at,"composition member kind differs"));}
            anchor_env.insert("translation_component".into(),coordinates[0].clone());
            let mut anchors=Vec::new();
            let actual=["translation_temperature","translation_pressure","translation_component"].iter().map(|name|dsl::parse_expr(name).map_err(|error|invalid(at,error.to_string()))).collect::<Result<Vec<_>>>()?;
            for anchor in [descriptor.source_anchor,descriptor.target_anchor] {
                let kind=self.function_call(instance,anchor,&actual,&[],&anchor_env,chain)?;
                anchors.push(Expr {kind,span:Span::default()});
            }
            descriptor.anchors.push(crate::contextual::ReferenceAnchorPair {member:member.as_id(),values:[anchors[0].clone(),anchors[1].clone()],temperature:conditions[0],pressure:conditions[1]});
            selected_weights.push(weight);
        }
        pse_quantity::ReferenceTranslation::admit_context(self.c.quantities,descriptor.source,descriptor.target,conditions[0],conditions[1],&descriptor.anchors.iter().map(|anchor|anchor.member).collect::<Vec<_>>(),&descriptor.provenance).map_err(|error|invalid(at,error.to_string()))?;
        let Type::Indexed {element:weight_type,..}=&contract.arguments[1].1 else {return Err(invalid(at,"composition contract must be indexed"));};
        let source_mean=self.reference_anchor_mean(at,&descriptor,true,weight_type,&selected_weights)?;
        let target_mean=self.reference_anchor_mean(at,&descriptor,false,weight_type,&selected_weights)?;
        let value=self.rewrite(instance,&args[0],env,chain)?;
        let binary=|op,lhs,rhs|Expr {kind:ExprKind::Binary {op,lhs:Box::new(lhs),rhs:Box::new(rhs)},span:Span::default()};
        let delta=binary(BinaryOp::Sub,value,source_mean);
        let input=Type::Quantity(Scheme::Delta(Box::new(Scheme::Concrete(descriptor.source))));
        let output=Type::Quantity(Scheme::Delta(Box::new(Scheme::Concrete(descriptor.target))));
        let operation=PhysicalOperation::ReferenceTranslation(descriptor);
        let converted=self.contextual_function(at,vec![("value".into(),input)],output,operation,"value",None,vec![delta])?;
        Ok(binary(BinaryOp::Add,converted,target_mean))
    }

    fn reference_anchor_mean(&mut self,at:DeclarationId,translation:&crate::contextual::ReferenceTranslation,source:bool,weight_type:&Type,weights:&[Expr])->Result<Expr> {
        use pse_quantity::scheme::Scheme;
        let quantity=if source {translation.source}else{translation.target};
        let point=Type::Quantity(Scheme::Concrete(quantity));
        let position=usize::from(!source);
        let reference=translation.anchors[0].values[position].clone();
        let mut terms=Vec::new();
        for index in 0..weights.len() {
            let selected=&translation.anchors[index].values[position];
            terms.push(format!("weight_{index}*(({})-({}))",dsl::render_expr(selected),dsl::render_expr(&reference)));
        }
        let total=(0..weights.len()).map(|index|format!("weight_{index}")).collect::<Vec<_>>().join("+");
        let body=format!("({})+({})/({total})",dsl::render_expr(&reference),terms.join("+"));
        let guard=format!("{} and ({total})>0",(0..weights.len()).map(|index|format!("weight_{index}>=0")).collect::<Vec<_>>().join(" and "));
        let coefficient=self.c.quantities.neutral_dimensionless().ok_or_else(||invalid(at,"reference mean requires a neutral coefficient"))?;
        self.contextual_function(at,(0..weights.len()).map(|index|(format!("weight_{index}"),weight_type.clone())).collect(),point,PhysicalOperation::ReferenceAnchorMean {translation:translation.clone(),source,coefficient},&body,Some(&guard),weights.to_vec())
    }

    fn contextual_function(&mut self,at:DeclarationId,arguments:Vec<(String,Type)>,result:Type,operation:PhysicalOperation,body:&str,guard:Option<&str>,actual:Vec<Expr>)->Result<Expr> {
        self.reserve(1)?;
        let mut hash=FramedHasher::new(pse_ids::Frame::ModelingPhysicalOperationV1);
        hash.id(&at.as_id());operation.frame(&mut hash);hash.str(body).bool(guard.is_some());if let Some(guard)=guard {hash.str(guard);}
        let id=DeclarationId::from(hash.finish_id());let name=format!("f_{}",id.as_id().to_hex());
        self.model.functions.entry(name.clone()).or_insert(crate::Function {physical_operation:Some(operation),reduction:None,validity:guard.map(dsl::parse_predicate).transpose().map_err(|error|invalid(at,error.to_string()))?,envelopes:vec![],validity_reads:crate::envelope::Reads::default(),external:None,continuity:None,id,variables:BTreeSet::new(),arguments,result,body:Some(dsl::parse_expr(body).map_err(|error|invalid(at,error.to_string()))?)});
        Ok(Expr {kind:ExprKind::NamedCall {name,args:actual},span:Span::default()})
    }
    pub(super) fn physical_transfer_function(&mut self, at:DeclarationId, input_type:Type, result:Type, operation:PhysicalOperation, factor:i8, value:Expr)->Result<Expr> {
        let mut h=FramedHasher::new(pse_ids::Frame::ModelingPhysicalOperationV1);
        h.id(&at.as_id()).part(&factor.to_le_bytes());
        for ty in [&input_type,&result] {
            let quantity=ty.quantity_scheme().ok_or_else(||invalid(at,"physical transformation has no numeric payload"))?.resolve_with_evidence(self.c.quantities,&BTreeMap::new(),self.c.preconditions).map_err(|error|invalid(at,error.to_string()))?;
            h.id(&quantity.as_id()).bool(ty.physical_refinement().is_some());
            if let Some(refinement)=ty.physical_refinement(){refinement.frame(&mut h);}
        }
        match &operation {
            PhysicalOperation::Transfer {exchange,..} => {h.str("transfer").bool(exchange.is_some());if let Some(exchange)=exchange {h.id(&exchange.as_id());}}
            PhysicalOperation::TransferMagnitude {..}=>{h.str("contribution");}
            _=>return Err(invalid(at,"not a transfer operation")),
        }
        let id=DeclarationId::from(h.finish_id());
        let name=format!("f_{}",id.as_id().to_hex());
        let body=dsl::parse_expr(if factor<0 {"-value"} else {"value"}).map_err(|error|invalid(at,error.to_string()))?;
        self.model.functions.entry(name.clone()).or_insert(crate::Function {
            physical_operation:Some(operation),reduction:None,validity:None,envelopes:vec![],validity_reads:crate::envelope::Reads::default(),external:None,continuity:None,id,variables:BTreeSet::new(),arguments:vec![("value".into(),input_type)],result,body:Some(body),
        });
        Ok(Expr {kind:ExprKind::NamedCall {name,args:vec![value]},span:Span::default()})
    }
}
