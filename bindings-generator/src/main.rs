use std::path::Path;

const SPINE_SRC_PATH: &str = "external/spine-c/spine-c";

fn generate() {
    let out_path = std::env::current_dir().unwrap();

    let bindings = bindgen::Builder::default()
        .header("bindings-generator/spine.h")
        .clang_arg(format!(
            "-I{}",
            Path::new(SPINE_SRC_PATH).join("include").display()
        ))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("spine-sys/src/spine_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    generate();
}
