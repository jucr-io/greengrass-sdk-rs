use std::env::var;

fn main() {
    // Build the C++ SDK.
    let mut config = cmake::Config::new("aws-iot-device-sdk-cpp-v2");
    config.define("BUILD_SHARED_LIBS", "OFF");

    let sdk_profile = match var("PROFILE").unwrap().as_str() {
        "release" => "Release",
        _ => "Debug",
    };
    config.profile(sdk_profile);

    let target = var("TARGET").unwrap();
    let host = var("HOST").unwrap();
    if target != host {
        // The C++ SDK is expecting a target triplet, without the vendor part.
        let target = target.replace("-unknown", "");
        config
            .define("CMAKE_CROSSCOMPILING", "TRUE")
            // The prepackaged internal crypto library doens't build for aarch64.
            .define("USE_OPENSSL", "ON")
            .target(&target);
    }

    let dst = config.build();

    println!("cargo:rustc-link-search=native={}/lib64", dst.display());
    println!("cargo:rustc-link-search=native={}/lib", dst.display());

    // Link to the AWS IoT SDK libraries

    // C libraries
    println!("cargo:rustc-link-lib=dylib=crypto");
    println!("cargo:rustc-link-lib=dylib=ssl");
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
    let aws_sdk_include = format!("{}/include", dst.display());
    cxx_build::bridge("src/ffi.rs")
        .include(manifest_dir)
        .include(aws_sdk_include)
        .file("src/aws.cc")
        .std("c++17")
        .compile("greenrass-ipc");

    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/aws.cc");
    println!("cargo:rerun-if-changed=include/aws.h");
}
