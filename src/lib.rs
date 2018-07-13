// Copyright 2018 Kevin Kaßelmann, David Mehren, Daniel Rose and Frederik Stehli.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed
// except according to those terms.

//! Succinct Tree library with implementations for the succinct trees LOUDS and BP
//! and the Range-Min-Max data structure used by BP. LOUDS and BP also use the
//! Rank/Select data structure from the Rust-Bio crate.
//! Code examples can be found in the submodules.

extern crate bio;
#[macro_use]
extern crate bv;
extern crate id_tree;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate serde;

pub mod bp_tree;
pub mod common;
pub mod louds_tree;
