#![feature(box_patterns)]

extern crate rustc;
extern crate rustc_metadata;
extern crate rustc_data_structures;
extern crate rustc_yk_link;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate mkstemp;

pub mod mir_cfg;
