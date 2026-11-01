// SPDX-License-Identifier: MIT OR Apache-2.0
#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

struct Inspect;
impl Callbacks for Inspect {
    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        let mut owned = Vec::<String>::new();
        for def in tcx.hir_body_owners() {
            let name = tcx.def_path_str(def);
            if tcx.is_closure_like(def.to_def_id()) {
                for capture in tcx.closure_captures(def) {
                    owned.push(format!(
                        "CAPTURE {name} place={} mode={:?} cause={:?}",
                        capture.to_string(tcx),
                        capture.info.capture_kind,
                        capture.info.capture_kind_expr_id
                    ));
                }
                owned.push(format!("ORIGIN {name} {:?}", tcx.closure_kind_origin(def)));
                let coroutine = tcx.mir_promoted(def).0.borrow().coroutine_kind().is_some();
                if coroutine {
                    if let Some(layout) = tcx.mir_coroutine_witnesses(def.to_def_id()) {
                        owned.push(format!(
                            "WITNESS {name} fields={:?} variants={:?} sources={:?} conflicts={:?}",
                            layout.field_tys,
                            layout.variant_fields,
                            layout.variant_source_info,
                            layout.storage_conflicts
                        ));
                    }
                }
            }
            if name.ends_with("partial_move") {
                let promoted = tcx.mir_promoted(def).0;
                owned.push(format!(
                    "EARLY {name} stolen={} phase={:?}",
                    promoted.is_stolen(),
                    promoted.borrow().phase
                ));
                let optimized = tcx.optimized_mir(def);
                owned.push(format!(
                    "LATE {name} promoted_stolen={} optimized_phase={:?}",
                    promoted.is_stolen(),
                    optimized.phase
                ));
            }
        }
        for row in owned {
            println!("{row}");
        }
        Compilation::Stop
    }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    rustc_driver::run_compiler(&args, &mut Inspect);
}
