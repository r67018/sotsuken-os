use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // hankaku.txtからhankaku.binを生成
    let status = Command::new("./tools/makefont.py")
        .args(["-o", "hankaku.bin", "hankaku.txt"])
        .status()
        .expect("Failed to execute script to generate hankaku.bin");
    if !status.success() {
        panic!("Failed to generate hankaku.bin");
    }

    // hankaku.binからhankaku.oを生成
    let status = Command::new("objcopy")
        .args(["-I", "binary", "-O", "elf64-x86-64", "-B", "i386:x86-64", "hankaku.bin", "hankaku.o"])
        .status()
        .expect("Failed to execute objcopy to generate hankaku.o");
    if !status.success() {
        panic!("Failed to generate hankaku.o");
    }

    // C++からRustへのFFIを生成
    println!("cargo:rustc-link-search=/usr/include/x86_64-linux-gnu/");
    let bindings = bindgen::Builder::default()
        .header("MikanLoaderPkg/frame_buffer_config.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .use_core()
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}
