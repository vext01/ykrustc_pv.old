use rustc::ty::TyCtxt;
use rustc_yk_datasection::{DataSection, DataSectionObject};
use rustc::hir::def_id::LOCAL_CRATE;

const CRATE_MAP_SECTION_NAME: &str = ".yk_crate_map";

/// Generates a binary object containing a section which describes the crate number to crate name
/// and filename mapping that was used at compile time.
///
/// The format of the section is:
///
///     num_crates: u32
///     crate_mapping[num_crates] {
///         crate_number: unsigned 32-bit,
///         crate_name: null-terminated string,
///         crate_filesystem_path: null-terminated string,
///     }
pub fn emit_crate_map<'a, 'tcx, 'gcx>(tcx: &'a TyCtxt<'a, 'tcx, 'gcx>) -> DataSectionObject {
    let mut sec = DataSection::new(CRATE_MAP_SECTION_NAME);

    // First field in the section is the number of crate records to process.
    let num_crates = tcx.crates().iter().count();
    sec.write_u32((num_crates + 1) as u32);

    // Local crate record.
    sec.write_u32(LOCAL_CRATE.as_u32());
    sec.write_u64(tcx.crate_hash(LOCAL_CRATE).as_u64());

    // Now there's a record for each crate.
    for krate in tcx.crates().iter() {
        sec.write_u32(krate.as_u32());

        let hash = tcx.crate_hash(*krate).as_u64();
        eprintln!("MAP: {:?} ({}) -> 0x{:x}", krate, tcx.crate_name(*krate), hash);
        sec.write_u64(hash);
    }

    sec.compile().unwrap()
}
