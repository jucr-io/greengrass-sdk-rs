use std::env::var;

fn main() {
    // Build the C++ SDK.
    use cmake::Config;

    let sdk_profile = match var("PROFILE").unwrap().as_str() {
        "release" => "Release",
        _ => "Debug",
    };
    let dst = Config::new("aws-iot-device-sdk-cpp-v2")
        .profile(sdk_profile)
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();
    println!("cargo:rustc-link-search=native={}/lib64", dst.display());

    // Link to the AWS IoT SDK libraries

    // C libraries
    println!("cargo:rustc-link-lib=static:+whole-archive=crypto");
    println!("cargo:rustc-link-lib=static:+whole-archive=s2n");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-io");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-iot");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-common");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-event-stream");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-checksums");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-cal");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-s3");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-mqtt");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-auth");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-http");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-compression");
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-c-sdkutils");

    // C++ libraries
    println!("cargo:rustc-link-lib=static:+whole-archive=aws-crt-cpp");
    println!("cargo:rustc-link-lib=static:+whole-archive=GreengrassIpc-cpp");
    println!("cargo:rustc-link-lib=static:+whole-archive=EventstreamRpc-cpp");

    // Now build the glue code.
    let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap();
    cxx_build::bridge("src/ffi.rs")
        .include(manifest_dir)
        .file("src/aws.cc")
        .std("c++23")
        .compile("greenrass-ipc");

    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/aws.cc");
    println!("cargo:rerun-if-changed=include/aws.h");
}
