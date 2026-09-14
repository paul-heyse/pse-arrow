// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordered structure is the equality oracle; hashes are checked only after that oracle.
use proptest::prelude::*;
use pse_mathir::{ExprGraph, Opcode, Payload};
use pse_quantity::{IndexSet, standard::standard_registry};
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn unrelated_insertion_order_cannot_change_typed_structure(values in prop::collection::vec(-1000i64..1000,1..40)) {
        let registry=standard_registry().unwrap();let quantity=registry.neutral_dimensionless().unwrap();
        let build=|reverse:bool|{
            let mut graph=ExprGraph::new();
            let mut seeds=values.clone();if reverse {seeds.reverse();}
            for value in seeds {graph.insert_typed(Opcode::Const,Payload::IntConst{value},&[],quantity,None).unwrap();}
            let mut root=None;
            for value in &values {
                let node=graph.insert_typed(Opcode::Const,Payload::IntConst{value:*value},&[],quantity,None).unwrap();
                root=Some(if let Some(prior)=root{graph.insert_typed(Opcode::Add,Payload::None,&[prior,node],quantity,None).unwrap()}else{node});
            }
            let indices=graph.iter().map(|(id,_)|(id,IndexSet::new())).collect();
            pse_mathir::number_typed_graph(&graph,&[root.unwrap()],&indices,&registry).unwrap()
        };
        let forward=build(false);let reverse=build(true);
        prop_assert_eq!(forward.roots(),reverse.roots());prop_assert_eq!(forward.len(),reverse.len());
        for ((a_id,a),(b_id,b)) in forward.iter().zip(reverse.iter()) {
            prop_assert_eq!(a_id,b_id);prop_assert_eq!(a.opcode,b.opcode);prop_assert_eq!(&a.payload,&b.payload);
            prop_assert_eq!(&a.children,&b.children);prop_assert_eq!(a.quantity_type,b.quantity_type);
            prop_assert_eq!(&a.free_indices,&b.free_indices);prop_assert_eq!(a.subtree_hash,b.subtree_hash);
        }
    }
}
