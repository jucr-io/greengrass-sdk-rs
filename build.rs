fn main() {
    cxx_build::bridge("src/lib.rs")
        // FIXME: Don't hardcode the path.
        .include("/home/zeenix/checkout/aws/include")
        .std("c++14")
        .compile("greenrass-ipc");

    println!("cargo:rerun-if-changed=src/main.rs");
}
