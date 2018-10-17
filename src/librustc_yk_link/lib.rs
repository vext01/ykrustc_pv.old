use std::fs;
use std::path::{PathBuf, Path};
use std::process::Command;

/// An extra ELF object file to link into the resulting binary.
pub struct YkExtraLinkObject(PathBuf);

#[cfg(target_arch = "x86_64")]
const BFD_NAME: &'static str = "elf64-x86-64";

#[cfg(target_arch = "x86_64")]
const BFD_ARCH: &'static str = "i386";

impl YkExtraLinkObject {
    /// Creates an ELF object file using the raw binary data stored in the file at `source_path`.
    /// This is achieved by shelling out to the `objcopy(1)` utility to graft the raw data
    /// into a well-formed ELF object file with the data in a section named `sec_name`. The
    /// resulting object is deleted when it falls out of scope.
    pub fn new(source_path: &Path, section_name: &str) -> Self {
        let out_filename = format!("{}.o", source_path.to_str().unwrap());

        let sec_arg = format!(".data={},alloc,load,readonly,data,contents", section_name);
        let mut cmd = Command::new("objcopy");
        cmd.args(&[
            "-I", "binary",
            "-O", BFD_NAME,
            "-B", BFD_ARCH,
            "--rename-section", &sec_arg,
            "-j", ".data",
            source_path.to_str().unwrap(), &out_filename]);

        println!("{:?}", cmd);
        let output = cmd.output().unwrap();
        if !output.status.success() {
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("objcopy failed");
        }
        YkExtraLinkObject(PathBuf::from(out_filename))
    }

    pub fn path(&self) -> &Path {
        &self.0.as_path()
    }
}

impl Drop for YkExtraLinkObject {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
