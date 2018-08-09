extern crate protoc_grpcio;

const SRC_FILE: &str = "protos/mines.proto";
const TARGET_DIR: &str = "src/protogen";

fn main() {
    println!("cargo:rerun-if-changed={}", SRC_FILE);

    protoc_grpcio::compile_grpc_protos(&[SRC_FILE], &["."], TARGET_DIR)
        .expect("Failed to compile gRPC definitions!");
}