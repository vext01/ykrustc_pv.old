// Copyright (c) 2018 King's College London
// created by the Software Development Team <http://soft-dev.org/>
//
// The Universal Permissive License (UPL), Version 1.0
//
// Subject to the condition set forth below, permission is hereby granted to any
// person obtaining a copy of this software, associated documentation and/or
// data (collectively the "Software"), free of charge and under any and all
// copyright rights in the Software, and any and all patent rights owned or
// freely licensable by each licensor hereunder covering either (i) the
// unmodified Software as contributed to or provided by such licensor, or (ii)
// the Larger Works (as defined below), to deal in both
//
// (a) the Software, and
// (b) any piece of software and/or hardware listed in the lrgrwrks.txt file
// if one is included with the Software (each a "Larger Work" to which the Software
// is contributed by such licensors),
//
// without restriction, including without limitation the rights to copy, create
// derivative works of, display, perform, and distribute the Software and make,
// use, sell, offer for sale, import, export, have made, and have sold the
// Software and the Larger Work(s), and to sublicense the foregoing rights on
// either these or other terms.
//
// This license is subject to the following condition: The above copyright
// notice and either this complete permission notice or at a minimum a reference
// to the UPL must be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate tempfile;

use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write};
use std::env;

const DEFAULT_CC: &'static str = "cc";

#[derive(Debug)]
pub enum DataSectionError {
    // Wrapped errors.
    Io(io::Error),
    // Our errors.
    CCompile(String), // C compilation error.
}

impl From<io::Error> for DataSectionError {
    fn from(err: io::Error) -> Self {
        DataSectionError::Io(err)
    }
}

pub type DataSectionResult<T> = Result<T, DataSectionError>;

/// A reference to an on-disk object file which is deleted when the instance falls out of scope.
#[derive(Debug)]
pub struct DataSectionObject(PathBuf);

impl DataSectionObject {
    pub fn path(&self) -> &Path {
        &self.0.as_path()
    }
}

impl Drop for DataSectionObject {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub struct DataSection {
    sec_name: String,
    source: Vec<String>,
}

/// A structure for building data sections.
impl DataSection {
    pub fn new(sec_name: &str) -> Self {
        let mut source = Vec::new();
        source.push(format!(".section {}", sec_name));
        Self {
            sec_name: String::from(sec_name),
            source
        }
    }

    pub fn sec_name(&self) -> &str {
        &self.sec_name.as_str()
    }

    pub fn write_u8(&mut self, val: u8) {
        self.source.push(format!(".byte {}", val));
    }

    pub fn write_u32(&mut self, val: u32) {
        self.source.push(format!(".4byte {}", val));
    }

    pub fn write_u64(&mut self, val: u64) {
        self.source.push(format!(".8byte {}", val));
    }

    pub fn write_usize(&mut self, val: usize) {
        // Sadly there's no "emit a pointer-sized integer" asm directive.
        #[cfg(target_pointer_width = "64")]
        self.source.push(format!(".8byte {}", val));
        #[cfg(target_pointer_width = "32")]
        self.source.push(format!(".4byte {}", val));
        #[cfg(target_pointer_width = "16")]
        self.source.push(format!(".2byte {}", val));
        #[cfg(target_pointer_width = "8")]
        self.source.push(format!(".byte {}", val));
    }

    pub fn write_str(&mut self, val: &str) {
        self.source.push(format!(".string \"{}\"", val));
    }

    pub fn compile(self) -> DataSectionResult<DataSectionObject> {
        let mut fh = tempfile::Builder::new()
                              .suffix(".s")
                              .tempfile()?;

        for line in self.source {
            writeln!(fh, "{}", line)?;
        }

	let mut output_path = PathBuf::from(fh.path());
	output_path.set_extension(".o");

	let cc = env::var("CC").unwrap_or(String::from(DEFAULT_CC));
        let mut cmd = Command::new(cc);
        cmd.arg("-c")
           .arg("-o")
           .arg(&output_path)
           .arg(fh.path());

        let out = cmd.output()?;
        if !out.status.success() {
            let msg = format!("C compilation error:\n{}", String::from_utf8_lossy(&out.stderr));
            return Err(DataSectionError::CCompile(msg));
        }
        debug_assert!(output_path.exists());

        Ok(DataSectionObject(output_path))
    }
}

#[cfg(test)]
mod tests {
    use super::DataSection;

    #[test]
    fn simple_sec() {
        let mut s = DataSection::new(".test_sec_123");
        s.write_str("hello_world");
        let obj = s.compile().unwrap();
        assert!(obj.path().exists());
        let path = obj.path().to_owned();
        drop(obj);
        assert!(!path.exists());
        // XXX read the object
    }
}
