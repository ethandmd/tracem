use std::env;
use std::path::PathBuf;

fn main() {
    #[cfg(feature = "libpfm")]
    {
        //println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
        println!("cargo:rustc-link-lib=pfm");
        let libpfm_bindings = bindgen::Builder::default()
            .header("libpfm_wrapper.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("Unable to generate bindings for libpfm_wrapper.h");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        libpfm_bindings
            .write_to_file(out_path.join("libpfm-sys.rs"))
            .expect("Couldn't write libpfm bindings!");
    }
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings for wrapper.h");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("perf-sys.rs"))
        .expect("Couldn't write bindings!");
}
