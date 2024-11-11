use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=/usr/include/x86_64-linux-gnu/");

    let bindings = bindgen::Builder::default()
        .header("../MikanLoaderPkg/frame_buffer_config.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .use_core()
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}
