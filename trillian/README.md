# Trillian Client

To create github.com/google/trillian project to build protobuf files:

First get Google API and Google Trillian Protobufs:

```shell
cd proto

git submodule add -f https://github.com/google/trillian

git submodule update --remote

git config submodule.trillian/proto/trillian.active false
```

Workaround warning: 
If underlying protobuf change, running `cargo build` will create an unneeded `google.rpc.rs` file.
The contents of this should match `api->google->rpc.rs`.
