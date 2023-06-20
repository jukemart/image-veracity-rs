# Trillian Client

Workaround warning: 
If underlying protobuf change, running `cargo build` will create an unneeded `google.rpc.rs` file.
The contents of this should match `api->google->rpc.rs`.
