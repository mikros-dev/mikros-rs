fn main() {
    tonic_build::configure()
        .out_dir("src/generated")
        .compile_protos(&["proto/helloworld.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));

    println!("cargo:rerun-if-changed=proto/helloworld.proto");
}