use std::path::PathBuf;
use std::process::Command;
use std::env;

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
    // C++で書かれたUSBドライバのライブラリをリンク
    link_usb();
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

fn link_usb() {
    println!("cargo:rerun-if-changed=usb/logger.cpp");
    println!("cargo:rerun-if-changed=usb/wrapper.cpp");
    println!("cargo:rerun-if-changed=usb/libusb.a");
    
    Command::new("make")
        .current_dir("usb")
        .status()
        .expect("Failed to execute make to build USB driver");
    
    // libusb.aをリンク
    let lib_dir = shellexpand::tilde("~/osbook/devenv/x86_64-elf/lib").to_string();
    println!("cargo:rustc-link-search=usb");
    println!("cargo:rustc-link-lib=static=usb");
    println!("cargo:rustc-link-search={}", lib_dir);
    println!("cargo:rustc-link-lib=static=c");
    println!("cargo:rustc-link-lib=static=c++");
    println!("cargo:rustc-link-lib=static=c++abi");
    println!("cargo:rustc-link-lib=static=m");
    println!("cargo:rustc-link-lib=static=freetype");
    
    // let excluded_files = ["main.cpp"];
    // let mikanos_cpp_files = glob::glob("mikanos/kernel/**/*.cpp")
    //     .unwrap()
    //     .map(|x| x.unwrap())
    //     .filter(|x| !excluded_files.contains(&x.file_name().unwrap().to_str().unwrap()))
    //     .collect::<Vec<_>>();
    // 
    // // インクルードパス
    // let includes = [
    //     "~/osbook/devenv/x86_64-elf/include/c++/v1",
    //     "~/osbook/devenv/x86_64-elf/include",
    //     "~/osbook/devenv/x86_64-elf/include/freetype2",
    //     "~/edk2/MdePkg/Include",
    //     "~/edk2/MdePkg/Include/X64"
    // ].map(|x| shellexpand::tilde(x).to_string());
    // // ライブラリパス
    // let lib_dir = shellexpand::tilde("~/osbook/devenv/x86_64-elf/lib").to_string();
    // 
    // println!("cargo:rerun-if-changed=usb/wrapper.cpp");
    // println!("cargo:rustc-link-search={}", lib_dir);
    // println!("cargo:rustc-link-lib=static=c");
    // println!("cargo:rustc-link-lib=static=c++");
    // println!("cargo:rustc-link-lib=static=c++abi");
    // // println!("cargo:rustc-link-lib=static=m");
    // println!("cargo:rustc-link-lib=static=freetype");
    // cc::Build::new()
    //     .cpp(true)
    //     // from buildenv.sh
    //     .includes(includes)
    //     .include("mikanos/kernel")
    //     .flag("-nostdlibinc")
    //     .flag("-D__ELF__")
    //     .flag("-D_LDBL_EQ_DBL")
    //     .flag("-D_GNU_SOURCE")
    //     .flag("-D_POSIX_TIMERS")
    //     .flag("-DEFIAPI=__attribute__((ms_abi))")
    //     // from Makefile
    //     .flag("-O2")
    //     .flag("-fshort-wchar")
    //     .flag("-g")
    //     .flag("--target=x86_64-elf")
    //     .flag("-ffreestanding")
    //     .flag("-mno-red-zone")
    //     .flag("-fno-exceptions")
    //     .flag("-fno-rtti")
    //     .std("c++17")
    //     .cpp_link_stdlib("c++")
    //     // 
    //     .flag("-w")
    //     .flag("-fpermissive") // エラーを警告にする
    //     .files(mikanos_cpp_files)
    //     .file("usb/wrapper.cpp")
    //     .compile("usb")
}
