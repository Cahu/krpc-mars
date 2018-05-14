extern crate protoc_rust;

fn main() {
    println!("cargo:rerun-if-changed=protos/krpc.proto");
    println!("cargo:rerun-if-changed=src/krpc.rs");
    protoc_rust::run(protoc_rust::Args {
        out_dir:   "src/",
        input:     &["protos/krpc.proto"],
        includes:  &[],
        customize: protoc_rust::Customize {
            ..Default::default()
        },
    }).expect("protoc");
}
