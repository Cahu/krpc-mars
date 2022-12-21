fn main() {
    println!("cargo:rerun-if-changed=protos/krpc.proto");
    println!("cargo:rerun-if-changed=src/krpc.rs");
    protoc_rust::Codegen::new()
        .out_dir("src/")
        .inputs(&["protos/krpc.proto"])
        .run()
        .expect("protoc");
}
