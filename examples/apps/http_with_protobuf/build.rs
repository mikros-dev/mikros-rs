fn main() {
    tonic_build::configure()
        .protoc_arg("--mikros-extensions_out=src")
        .protoc_arg("--mikros-extensions_opt=settings=protoc-gen-mikros-extensions.toml")
        .out_dir("src/generated")
        .extern_path(
            ".google.protobuf.Timestamp",
            "::prost_wkt_types::Timestamp"
        )
        .compile_protos(
            &["proto/card.proto"],
            &["proto", "plugin"],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));

    // Set files that trigger this build process if changed.
    println!("cargo:rerun-if-changed=proto/card.proto");
    println!("cargo:rerun-if-changed=protoc-gen-mikros-extensions.toml");
}