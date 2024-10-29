fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let home = std::env::var("HOME").unwrap();
    let include_dir = format!("{home}/aws/include");
    cxx_build::bridge("src/lib.rs")
        // FIXME: Don't hardcode the path.
        .include(&include_dir)
        .include(manifest_dir)
        .file("src/aws.cc")
        .std("c++14")
        .compile("greenrass-ipc");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/aws.cc");
    println!("cargo:rerun-if-changed=include/aws.h");
}
