extern crate protoc_grpcio;

fn main() {
    println!("cargo:rerun-if-changed={}", "protos");

    protoc_grpcio::compile_grpc_protos(&["mines.proto"], &["protos"], "src")
        .expect("Failed to compile gRPC definitions!");
}