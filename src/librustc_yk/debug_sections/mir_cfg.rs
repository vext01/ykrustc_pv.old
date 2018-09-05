use rustc::ty::TyCtxt;

use rustc_metadata::cstore::{CStore, CrateMetadata};
use rustc::hir::def_id::{CRATE_DEF_INDEX, DefId};
use rustc::hir::def::Def;
use rustc::mir::{Mir, TerminatorKind, BasicBlock};
use rustc::session::Session;

// Edge kinds.
// XXX use u8.
const EDGE_GOTO: u32 = 0;
const EDGE_SWITCHINT: u32 = 1;
const EDGE_RESUME: u32 = 2;
const EDGE_ABORT: u32 = 3;
const EDGE_RETURN: u32 = 4;
const EDGE_UNREACHABLE: u32 = 5;
const EDGE_DROP: u32 = 6;
const EDGE_DROP_AND_REPLACE: u32 = 7;
const EDGE_CALL: u32 = 8;
const EDGE_ASSERT: u32 = 9;
const EDGE_YIELD: u32 = 10;
const EDGE_GENERATOR_DROP: u32 = 11;
const EDGE_FALSE_EDGES: u32 = 12;
const EDGE_FALSE_UNWIND: u32 = 13;

pub fn emit_mir_cfg_section<'a, 'tcx, 'gcx>(tcx: &'a TyCtxt<'a, 'tcx, 'gcx>, cstore: &CStore, sess: &Session) {
    // Iterate crates.
    cstore.iter_crate_data(|k_num, k_md| {
        eprintln!("{:?}", k_num);
        // Iterate top-level items.
        k_md.each_child_of_item(CRATE_DEF_INDEX, |exp| {
            process_def(tcx, k_md, &exp.def);
        }, sess);
    });
}

// XXX kill md?
fn process_def<'a, 'tcx, 'gcx>(tcx: &'a TyCtxt<'a, 'tcx, 'gcx>, _k_md: &CrateMetadata, d: &Def) {
    match d {
        // XXX only top-level functions for now.
        Def::Fn(def_id) => {
            if tcx.is_mir_available(*def_id) {
                process_mir(def_id, tcx.optimized_mir(*def_id));
            } else {
                eprintln!("No MIR for {:?}", d);
            }
        },
        _ => (),
    }
}

fn process_mir(def_id: &DefId, mir: &Mir) {
    for (bb, maybe_bb_data) in mir.basic_blocks().iter_enumerated() {
        if maybe_bb_data.terminator.is_none() {
            continue;
        }
        let bb_data = maybe_bb_data.terminator.as_ref().unwrap();
        match bb_data.kind {
            TerminatorKind::Goto{target: target_bb} => {
                emit_edge(EDGE_GOTO, def_id, bb, Some(def_id), Some(target_bb));
            },
            TerminatorKind::SwitchInt{ref targets, ..} => {
                for target_bb in targets {
                    emit_edge(EDGE_SWITCHINT, def_id, bb, Some(def_id), Some(*target_bb));
                }
            },
            TerminatorKind::Resume => emit_edge(EDGE_RESUME, def_id, bb, None, None),
            TerminatorKind::Abort => emit_edge(EDGE_ABORT, def_id, bb, None, None),
            TerminatorKind::Return => emit_edge(EDGE_RETURN, def_id, bb, None, None),
            TerminatorKind::Unreachable => emit_edge(EDGE_UNREACHABLE, def_id, bb, None, None),
            TerminatorKind::Drop{target: target_bb, unwind: opt_unwind_bb, ..} => {
                emit_edge(EDGE_DROP, def_id, bb, Some(def_id), Some(target_bb));
                if let Some(unwind_bb) = opt_unwind_bb {
                    emit_edge(EDGE_DROP, def_id, bb, Some(def_id), Some(unwind_bb));
                }
            },
            TerminatorKind::DropAndReplace{target: target_bb, unwind: opt_unwind_bb, ..} => {
                emit_edge(EDGE_DROP_AND_REPLACE, def_id, bb, Some(def_id), Some(target_bb));
                if let Some(unwind_bb) = opt_unwind_bb {
                    emit_edge(EDGE_DROP_AND_REPLACE, def_id, bb, Some(def_id), Some(unwind_bb));
                }
            },
            TerminatorKind::Call{cleanup: opt_cleanup_bb, ..} => {
                // XXX need to encode the call target and ret address.
                if let Some(cleanup_bb) = opt_cleanup_bb {
                    emit_edge(EDGE_CALL, def_id, bb, Some(def_id), Some(cleanup_bb));
                }
            },
            TerminatorKind::Assert{target: target_bb, cleanup: opt_cleanup_bb, ..} => {
                emit_edge(EDGE_ASSERT, def_id, bb, Some(def_id), Some(target_bb));
                if let Some(cleanup_bb) = opt_cleanup_bb {
                    emit_edge(EDGE_ASSERT, def_id, bb, Some(def_id), Some(cleanup_bb));
                }
            },
            TerminatorKind::Yield{resume: resume_bb, drop: opt_drop_bb, ..} => {
                emit_edge(EDGE_YIELD, def_id, bb, Some(def_id), Some(resume_bb));
                if let Some(drop_bb) = opt_drop_bb {
                    emit_edge(EDGE_YIELD, def_id, bb, Some(def_id), Some(drop_bb));
                }
            },
            TerminatorKind::GeneratorDrop => emit_edge(EDGE_GENERATOR_DROP, def_id, bb, None, None),
            TerminatorKind::FalseEdges{real_target: real_target_bb, ..} => {
                // Fake edges not considered.
                emit_edge(EDGE_FALSE_EDGES, def_id, bb, Some(def_id), Some(real_target_bb));
            },
            TerminatorKind::FalseUnwind{real_target: real_target_bb, ..} => {
                // Fake edges not considered.
                emit_edge(EDGE_FALSE_UNWIND, def_id, bb, Some(def_id), Some(real_target_bb));
            },
        }
    }
}

fn emit_edge(_kind: u32, _from_def_id: &DefId, _from_bb: BasicBlock,
             _to_def_id: Option<&DefId>, _to_bb: Option<BasicBlock>) {
    eprintln!("kind: {}: {:?} {:?} ---> {:?} {:?}", _kind, _from_def_id, _from_bb, _to_def_id, _to_bb);
}
