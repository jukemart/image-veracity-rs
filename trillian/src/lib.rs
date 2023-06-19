#![feature(async_fn_in_trait)]

#[macro_use]
extern crate derive_builder;

use crate::protobuf::trillian::LogLeaf;

pub mod client;
mod protobuf;

// Export some Trillian types
pub type TrillianLogLeaf = LogLeaf;
