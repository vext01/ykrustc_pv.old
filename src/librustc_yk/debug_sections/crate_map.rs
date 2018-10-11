use rustc::ty::TyCtxt;
use data_section::{DataSection, DataSectionObject};

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
    sec.write_u32(num_crates as u32);

    // Now there's a record for each crate.
    for krate in tcx.crates().iter() {
        sec.write_u32(krate.as_u32());

        eprintln!("MAP: {:?} -> 0x{}", krate, tcx.crate_disambiguator(*krate).to_fingerprint().to_hex());
        let dis = tcx.crate_disambiguator(*krate).to_fingerprint().as_value();
        sec.write_u64(dis.1);
        sec.write_u64(dis.0);

        //sec.write_str(&tcx.crate_name(*krate).as_str());
        //sec.write_u8(0);

        //let source = tcx.used_crate_source(*krate);
        //let path = if let Some((ref p, _))  = source.rlib {
        //    p.to_str().unwrap()
        //} else {
        //    ""  // Not an rlib.
        //};
        //sec.write_str(path);
    }

    sec.compile().unwrap()
}
