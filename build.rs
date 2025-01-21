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

    // アセンブリで書かれたコードをリンク
    link_asmfunc(&out_path);
}

fn link_asmfunc(out_dir: &PathBuf) {
    // amsfuncをコンパイル
    println!("cargo:rerun-if-changed=src/asmfunc.asm");
    let asmfunc_obj = out_dir.join("asmfunc.o");
    let status = Command::new("nasm")
        .args(["-f", "elf64", "-o", asmfunc_obj.to_str().unwrap(), "src/asmfunc.asm"])
        .status()
        .expect("Failed to execute nasm to generate asmfunc.o");
    if !status.success() {
        panic!("Failed to generate asmfunc.o")
    }
    // 静的ライブラリを生成
    let asmfunc_lib = out_dir.join("libasmfunc.a");
    let status = Command::new("ar")
        .args(["crus", asmfunc_lib.to_str().unwrap(), asmfunc_obj.to_str().unwrap()])
        .status()
        .expect("Failed to execute ar to generate asmfunc static library");
    if !status.success() {
        panic!("Failed to generate libasmfunc.a")
    }
    // リンク設定
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=asmfunc");
}
