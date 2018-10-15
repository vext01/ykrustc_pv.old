// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(unused_variables)]
#![allow(unreachable_code)]
// Test that we can extract a ! through pattern matching then use it as several different types.

#![feature(never_type)]

fn main() {
    let x: Result<u32, !> = Ok(123);
    match x {
        Ok(z) => (),
        Err(y) => {
            let q: u32 = y;
            let w: i32 = y;
            let e: String = y;
            y
        },
    }
}

