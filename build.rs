use std::env;
use std::path::PathBuf;

fn main() {
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
