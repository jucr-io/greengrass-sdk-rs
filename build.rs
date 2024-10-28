fn main() {
    cxx_build::bridge("src/lib.rs")
        .std("c++14")
        .compile("greenrass-ipc");

    println!("cargo:rerun-if-changed=src/main.rs");
}
