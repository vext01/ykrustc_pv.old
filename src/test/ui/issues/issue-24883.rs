// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// compile-pass
// skip-codegen
mod a {
    pub mod b { pub struct Foo; }

    pub mod c {
        use super::b;
        pub struct Bar(pub b::Foo);
    }

    pub use self::c::*;
}


fn main() {
    let _ = a::c::Bar(a::b::Foo);
    let _ = a::Bar(a::b::Foo);
}
