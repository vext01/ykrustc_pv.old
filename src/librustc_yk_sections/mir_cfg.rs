use rustc::ty::TyCtxt;

use rustc::hir::def_id::DefId;
use rustc::mir::{Mir, TerminatorKind, Operand, Constant};
use rustc::ty::{TyS, TyKind, Const};
use rustc::util::nodemap::DefIdSet;
use std::path::PathBuf;
use mkstemp::TempFile;
use bincode;
use rustc_yk_link::YkExtraLinkObject;
use std::fs;

/// Information about a call target.
#[derive(Serialize)]
enum CallKind {
    Known { crate_hash: u64, def_idx: u32 },
    Unknown,
}

/// Describes how a block is terminated.
#[derive(Serialize)]
enum CfgTerminator {
    Goto { bb: u32 },
    SwitchInt { bbs: Vec<u32> },
    Resume,
    Abort,
    Return,
    Unreachable,
    Drop{bb: u32, unwind_bb: Option<u32> },
    DropAndReplace { bb: u32, unwind_bb: Option<u32> },
    Call { call_kind: CallKind, cleanup_bb: Option<u32> },
    Assert { bb: u32, cleanup_bb: Option<u32> },
    Yield{bb: u32, drop_bb: Option<u32>},
    GeneratorDrop,
    FalseEdges { bb: u32, imaginary_bbs: Vec<u32> },
    FalseUnwind { bb: u32, unwind_bb: Option<u32> },
}

/// Indentifies the start of a MIR block.
#[derive(Serialize)]
struct MirLoc {
    crate_hash: u64,
    def_idx: u32,
    bb: u32,
}

/// A Control Flow Graph (CFG) edge.
#[derive(Serialize)]
struct CfgEdge(MirLoc, CfgTerminator);

const MIR_CFG_SECTION_NAME: &'static str = ".yk_mir_cfg";
const MIR_CFG_TEMPLATE: &'static str = ".ykcfg.XXXXXXXX";

/// Serialises the control flow for the given `DefId`s into a ELF object file and returns a handle for linking.
pub fn emit_mir_cfg_section<'a, 'tcx, 'gcx>(tcx: &'a TyCtxt<'a, 'tcx, 'gcx>, def_ids: &DefIdSet) -> YkExtraLinkObject {
    let mut edges = Vec::new();

    for def_id in def_ids {
        if tcx.is_mir_available(*def_id) {
            edges.extend(process_mir(tcx, def_id, tcx.optimized_mir(*def_id)));
        } else {
            eprintln!("No MIR for {:?}", def_id);
        }
    }

    let mut template = std::env::temp_dir();
    template.push(MIR_CFG_TEMPLATE);
    let fh = TempFile::new(template.to_str().unwrap(), false).unwrap();
    let path = PathBuf::from(fh.path());

    bincode::serialize_into(fh, &edges).unwrap();
    let ret = YkExtraLinkObject::new(&path, MIR_CFG_SECTION_NAME);

    fs::remove_file(path).unwrap();
    ret
}

fn process_mir(tcx: &TyCtxt, def_id: &DefId, mir: &Mir) -> Vec<CfgEdge> {
    let mut edges = Vec::new();

    for (bb, maybe_bb_data) in mir.basic_blocks().iter_enumerated() {
        if maybe_bb_data.terminator.is_none() {
            continue; // XXX find out what that would mean? Assert it can't?
        }

        let loc = MirLoc {
            crate_hash: tcx.crate_hash(def_id.krate).as_u64(),
            def_idx: def_id.index.as_raw_u32(),
            bb: bb.index() as u32,
        };

        let bb_data = maybe_bb_data.terminator.as_ref().unwrap();
        let term = match bb_data.kind {
            TerminatorKind::Goto { target, .. } => CfgTerminator::Goto { bb: target.index() as u32 },
            TerminatorKind::SwitchInt { ref targets, .. } => {
                CfgTerminator::SwitchInt { bbs: targets.iter().map(|t| t.index() as u32).collect::<Vec<u32>>() }
            },
            TerminatorKind::Resume => CfgTerminator::Resume,
            TerminatorKind::Abort => CfgTerminator::Abort,
            TerminatorKind::Return => CfgTerminator::Return,
            TerminatorKind::Unreachable => CfgTerminator::Unreachable,
            TerminatorKind::Drop { target, unwind, .. } => {
                CfgTerminator::Drop {
                    bb: target.index() as u32,
                    unwind_bb: unwind.map(|u| u.index() as u32),
                }
            },
            TerminatorKind::DropAndReplace{target, unwind, ..} => {
                CfgTerminator::DropAndReplace {
                    bb: target.index() as u32,
                    unwind_bb: unwind.map(|u| u.index() as u32),
                }
            },
            TerminatorKind::Call { ref func, cleanup, .. } => {
                // This only supports statically known functions for now.
                let call_kind = if let Operand::Constant(box Constant {
                    literal: Const {
                        ty: &TyS {
                            sty: TyKind::FnDef(target_def_id, _substs), ..
                        }, ..
                    }, ..
                }, ..) = func {
                    CallKind::Known {
                        crate_hash: tcx.crate_hash(target_def_id.krate).as_u64(),
                        def_idx: target_def_id.index.as_raw_u32(),
                    }
                } else {
                    CallKind::Unknown
                };

                CfgTerminator::Call { call_kind, cleanup_bb: cleanup.map(|c| c.index() as u32) }
            },
            TerminatorKind::Assert { target, cleanup, .. } => {
                CfgTerminator::Assert {
                    bb: target.index() as u32,
                    cleanup_bb: cleanup.map(|c| c.index() as u32)
                }
            },
            TerminatorKind::Yield { resume, drop, .. } => {
                CfgTerminator::Yield {
                    bb: resume.index() as u32,
                    drop_bb: drop.map(|d| d.index() as u32),
                }
            },
            TerminatorKind::GeneratorDrop => CfgTerminator::GeneratorDrop,
            TerminatorKind::FalseEdges { real_target, ref imaginary_targets, .. } => {
                CfgTerminator::FalseEdges {
                    bb: real_target.index() as u32,
                    imaginary_bbs: imaginary_targets.iter().map(|t| t.index() as u32).collect(),
                }
            },
            TerminatorKind::FalseUnwind { real_target, unwind, .. } => {
                CfgTerminator::FalseUnwind {
                    bb: real_target.index() as u32,
                    unwind_bb: unwind.map(|u| u.index() as u32),
                }
            },
        };
        edges.push(CfgEdge(loc, term));
    }
    edges
}
