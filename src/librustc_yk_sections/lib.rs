// Copyright 2018 King's College London.
// Created by the Software Development Team <http://soft-dev.org/>.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(box_patterns)]

extern crate rustc;
extern crate rustc_yk_link;
extern crate rustc_codegen_utils;
extern crate byteorder;

use std::env;

/// Are Yorick debug sections enabled?
pub fn with_yk_debug_sections() -> bool {
    match env::var("YK_DEBUG_SECTIONS") {
        Ok(_) => true,
        _ => false,
    }
}

pub mod mir_cfg;
