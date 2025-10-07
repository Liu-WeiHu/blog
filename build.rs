use std::path::PathBuf;

fn main() {
    let out_dir: PathBuf = "src/proto_gen".into();
    tonic_prost_build::configure()
        .build_client(false)
        .out_dir(&out_dir)
        .file_descriptor_set_path(out_dir.join("file_descriptor_set.bin"))
        .compile_protos(&["proto/api.proto", "proto/common.proto", "proto/request.proto", "proto/response.proto"], &["proto"])
        .expect("failed to compile protos");
}