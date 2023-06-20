fn main() {
    std::env::set_var("PROTOC", protobuf_src::protoc());
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/protobuf")
        .compile(
            &[
                "proto/trillian_admin_api.proto",
                "proto/trillian_log_api.proto",
            ],
            &["proto/", "proto/third_party/googleapis"],
        )
        .unwrap();
}
