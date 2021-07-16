pub fn main() {
    tonic_build::configure()
        .build_server(false)
        .compile(&["proto/google/datastore/v1/datastore.proto"], &["proto"])
        .unwrap();
}
