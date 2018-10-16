use rustc::ty::TyCtxt;

use rustc::hir::def_id::DefId;
use rustc::mir::{Mir, TerminatorKind, BasicBlock, Operand, Constant};
use rustc::ty::{TyS, TyKind, Const};
use rustc_yk_datasection::{DataSection, DataSectionObject};
use rustc::util::nodemap::DefIdSet;

// Edge kinds.
const GOTO: u8 = 0;
const SWITCHINT: u8 = 1;
const RESUME: u8 = 2;
const ABORT: u8 = 3;
const RETURN: u8 = 4;
const UNREACHABLE: u8 = 5;
const DROP_NO_UNWIND: u8 = 6;
const DROP_WITH_UNWIND: u8 = 7;
const DROP_AND_REPLACE_NO_UNWIND: u8 = 8;
const DROP_AND_REPLACE_WITH_UNWIND: u8 = 9;
const CALL_NO_CLEANUP: u8 = 10;
const CALL_WITH_CLEANUP: u8 = 11;
const CALL_UNKNOWN_NO_CLEANUP: u8 = 12;
const CALL_UNKNOWN_WITH_CLEANUP: u8 = 13;
const ASSERT_NO_CLEANUP: u8 = 14;
const ASSERT_WITH_CLEANUP: u8 = 15;
const YIELD_NO_DROP: u8 = 16;
const YIELD_WITH_DROP: u8 = 17;
const GENERATOR_DROP: u8 = 18;
const FALSE_EDGES: u8 = 19;
const FALSE_UNWIND: u8 = 20;
const NO_MIR: u8 = 254;
const SENTINAL: u8 = 255;

const MIR_CFG_SECTION_NAME: &'static str = ".yk_mir_cfg";

pub fn emit_mir_cfg_section<'a, 'tcx, 'gcx>(
    tcx: &'a TyCtxt<'a, 'tcx, 'gcx>,
    def_ids: &DefIdSet) -> DataSectionObject
{
    let mut sec = DataSection::new(MIR_CFG_SECTION_NAME);

    for def_id in def_ids {
        if tcx.is_mir_available(*def_id) {
            process_mir(tcx, &mut sec, def_id, tcx.optimized_mir(*def_id));
        } else {
            sec.write_u8(NO_MIR);
            sec.write_u64(tcx.crate_hash(def_id.krate).as_u64());
            sec.write_u32(def_id.index.as_raw_u32());
        }
    }

    sec.write_u8(SENTINAL);
    sec.compile().unwrap()
}

fn process_mir(tcx: &TyCtxt, sec: &mut DataSection, def_id: &DefId, mir: &Mir) {
    for (bb, maybe_bb_data) in mir.basic_blocks().iter_enumerated() {
        if maybe_bb_data.terminator.is_none() {
            continue; // XXX find out what that would mean? Assert it can't?
        }
        let bb_data = maybe_bb_data.terminator.as_ref().unwrap();

        match bb_data.kind {
            // GOTO: <simple static edge>
            TerminatorKind::Goto{target: target_bb} => {
                emit_simple_static_edge(tcx, sec, GOTO, def_id, bb, target_bb);
            },
            // SWITCHINT: crate_num: u32, def_idx: u32, from_bb: u32, num_targets: usize, target_bb0, ..., target_bbN
            TerminatorKind::SwitchInt{ref targets, ..} => {
                sec.write_u8(SWITCHINT);
                sec.write_u64(tcx.crate_hash(def_id.krate).as_u64());
                sec.write_u32(def_id.index.as_raw_u32());
                sec.write_u32(bb.index() as u32);
                sec.write_usize(targets.len());
                for target_bb in targets {
                    sec.write_u32(target_bb.index() as u32);
                }
            },
            // RESUME, ABORT, RETURN, UNREACHABLE: <simple dynamic edge>
            TerminatorKind::Resume => emit_simple_dynamic_edge(tcx, sec, RESUME, def_id, bb),
            TerminatorKind::Abort => emit_simple_dynamic_edge(tcx, sec, ABORT, def_id, bb),
            TerminatorKind::Return => emit_simple_dynamic_edge(tcx, sec, RETURN, def_id, bb),
            TerminatorKind::Unreachable => emit_simple_dynamic_edge(tcx, sec, UNREACHABLE, def_id, bb),

            // DROP_NO_UNWIND: <simple static edge>
            // DROP_WITH_UNWIND: <simple static edge> + unwind_bb: u32
            TerminatorKind::Drop{target: target_bb, unwind: opt_unwind_bb, ..} => {
                if let Some(unwind_bb) = opt_unwind_bb {
                    emit_simple_static_edge(tcx, sec, DROP_WITH_UNWIND, def_id, bb, target_bb);
                    sec.write_u32(unwind_bb.index() as u32);
                } else {
                    emit_simple_static_edge(tcx, sec, DROP_NO_UNWIND, def_id, bb, target_bb);
                }
            },
            // DROP_AND_REPLACE_NO_UNWIND: <simple static edge>
            // DROP_AND_REPLACE_UNWIND, <simple static edge> + unwind_bb: u32
            TerminatorKind::DropAndReplace{target: target_bb, unwind: opt_unwind_bb, ..} => {
                if let Some(unwind_bb) = opt_unwind_bb {
                    emit_simple_static_edge(tcx, sec, DROP_AND_REPLACE_WITH_UNWIND, def_id, bb, target_bb);
                    sec.write_u32(unwind_bb.index() as u32);
                } else {
                    emit_simple_static_edge(tcx, sec, DROP_AND_REPLACE_NO_UNWIND, def_id, bb, target_bb);
                }
            },
            TerminatorKind::Call{ref func, cleanup: opt_cleanup_bb, ..} => {
                // XXX
                if let Operand::Constant(box Constant {
                    literal: Const {
                        ty: &TyS {
                            sty: TyKind::FnDef(target_def_id, _substs), ..
                        }, ..
                    }, ..
                }, ..) = func {
                    // A statically known call target.
                    if opt_cleanup_bb.is_some() {
                        sec.write_u8(CALL_WITH_CLEANUP);
                    } else {
                        sec.write_u8(CALL_NO_CLEANUP);
                    }

                    // Source.
                    sec.write_u64(tcx.crate_hash(def_id.krate).as_u64());
                    sec.write_u32(def_id.index.as_raw_u32());
                    sec.write_u32(bb.index() as u32);

                    // Destination.
                    sec.write_u64(tcx.crate_hash(target_def_id.krate).as_u64());
                    sec.write_u32(target_def_id.index.as_raw_u32()); // Assume destination bb is 0.

                    // Cleanup (if any).
                    if let Some(cleanup_bb) = opt_cleanup_bb {
                        sec.write_u32(cleanup_bb.index() as u32);
                    }
                } else {
                    // It's a kind of call that we can't statically know the target of.
                    if opt_cleanup_bb.is_some() {
                        sec.write_u8(CALL_UNKNOWN_WITH_CLEANUP);
                    } else {
                        sec.write_u8(CALL_UNKNOWN_NO_CLEANUP);
                    }

                    // Source.
                    sec.write_u64(tcx.crate_hash(def_id.krate).as_u64());
                    sec.write_u32(def_id.index.as_raw_u32());
                    sec.write_u32(bb.index() as u32);

                    // Cleanup (if any).
                    if let Some(cleanup_bb) = opt_cleanup_bb {
                        sec.write_u32(cleanup_bb.index() as u32);
                    }
                }
            },
            // ASSERT_NO_CLEANUP: <simple static edge>
            // ASSERT_WITH_CLEANUP: <simple static edge> + cleanup_bb: u32
            TerminatorKind::Assert{target: target_bb, cleanup: opt_cleanup_bb, ..} => {
                if let Some(cleanup_bb) = opt_cleanup_bb {
                    emit_simple_static_edge(tcx, sec, ASSERT_WITH_CLEANUP, def_id, bb, target_bb);
                    sec.write_u32(cleanup_bb.index() as u32);
                } else {
                    emit_simple_static_edge(tcx, sec, ASSERT_NO_CLEANUP, def_id, bb, target_bb);
                }
            },
            // YIELD_NO_DROP: <simple static edge>
            // YIELD_WITH_DROP: <simple static edge> + drop_bb: u32
            TerminatorKind::Yield{resume: resume_bb, drop: opt_drop_bb, ..} => {
                if let Some(drop_bb) = opt_drop_bb {
                    emit_simple_static_edge(tcx, sec, YIELD_WITH_DROP, def_id, bb, resume_bb);
                    sec.write_u32(drop_bb.index() as u32);
                } else {
                    emit_simple_static_edge(tcx, sec, YIELD_NO_DROP, def_id, bb, resume_bb);
                }
            },
            TerminatorKind::GeneratorDrop => emit_simple_dynamic_edge(tcx, sec, GENERATOR_DROP, def_id, bb),
            TerminatorKind::FalseEdges{real_target: real_target_bb, ..} => {
                // Fake edges not considered.
                emit_simple_static_edge(tcx, sec, FALSE_EDGES, def_id, bb,real_target_bb);
            },
            TerminatorKind::FalseUnwind{real_target: real_target_bb, ..} => {
                // Fake edges not considered.
                emit_simple_static_edge(tcx, sec, FALSE_UNWIND, def_id, bb, real_target_bb);
            },
        }
    }
}

/// Emit a simple edge with a statically known destination.
fn emit_simple_static_edge(tcx: &TyCtxt, sec: &mut DataSection, kind: u8, def_id: &DefId,
                    from_bb: BasicBlock, to_bb: BasicBlock) {
    sec.write_u8(kind);
    sec.write_u64(tcx.crate_hash(def_id.krate).as_u64());
    sec.write_u32(def_id.index.as_raw_u32());
    sec.write_u32(from_bb.index() as u32);
    sec.write_u32(to_bb.index() as u32);
}

/// Emit a simple edge whose destination isn't statically known.
fn emit_simple_dynamic_edge(tcx: &TyCtxt, sec: &mut DataSection, kind: u8, def_id: &DefId,
                             from_bb: BasicBlock) {
    sec.write_u8(kind);
    sec.write_u64(tcx.crate_hash(def_id.krate).as_u64());
    sec.write_u32(def_id.index.as_raw_u32());
    sec.write_u32(from_bb.index() as u32);
}
