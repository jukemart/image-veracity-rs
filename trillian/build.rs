fn main() {
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/client")
        .compile(
            &[
                "proto/trillian/trillian_admin_api.proto",
                "proto/trillian/trillian_log_api.proto",
            ],
            &["proto/trillian", "proto/googleapis"],
        )
        .unwrap();
}
