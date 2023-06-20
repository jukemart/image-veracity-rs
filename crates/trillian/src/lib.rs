#![feature(async_fn_in_trait)]

#[macro_use]
extern crate derive_builder;

use crate::protobuf::trillian::{LogLeaf, Tree};

pub mod client;
mod protobuf;

// Export some Trillian types
pub type TrillianLogLeaf = LogLeaf;
pub type TrillianTree = Tree;
