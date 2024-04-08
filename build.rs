use std::env;
use std::path::PathBuf;

fn main() {
    let perf_bindings = bindgen::Builder::default()
        .header("perf_wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings for perf_wrapper.h");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    perf_bindings
        .write_to_file(out_path.join("perf-sys.rs"))
        .expect("Couldn't write perf bindings!");
    println!("cargo:rustc-link-lib=numa");
    let numa_bindings = bindgen::Builder::default()
        .header("numa_wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings for numa_wrapper.h");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    numa_bindings
        .write_to_file(out_path.join("numa-sys.rs"))
        .expect("Couldn't write numa bindings!");
}
