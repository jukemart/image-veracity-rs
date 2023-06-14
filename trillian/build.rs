fn main() {
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/protobuf")
        .compile(
            &[
                "proto/trillian/trillian_admin_api.proto",
                "proto/trillian/trillian_log_api.proto",
            ],
            &["proto/trillian", "proto/trillian/third_party/googleapis"],
        )
        .unwrap();
}
