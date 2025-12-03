fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=../proto/notes.proto");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["../proto/notes.proto"], &["../proto"])?;
    Ok(())
}
