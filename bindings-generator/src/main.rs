use std::path::Path;

const SRC_PATH: &str = "external/nuklear";

fn generate() {
    let out_path = std::env::current_dir().unwrap();

    let bindings = bindgen::Builder::default()
        .header("bindings-generator/wrapper.h")
        .clang_arg(format!("-I{}", Path::new(SRC_PATH).display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("nukly-sys/nuklear_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    generate();
}
