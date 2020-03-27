use std::path::Path;

fn main() {
    let impl_src = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("nuklear_impl.c");
    let qsort_src = Path::new(std::env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("qsort.c");

    let include = Path::new(std::env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("external")
        .join("nuklear");

    println!("cargo:rerun-if-changed={}", impl_src.display());
    println!("cargo:rerun-if-changed={}", qsort_src.display());
    println!(
        "cargo:rerun-if-changed={}",
        include.join("nuklear.h").display()
    );

    let mut builder = cc::Build::new();
    builder
        .file(impl_src)
        .file(qsort_src)
        .define("NK_INCLUDE_FIXED_TYPES", "1")
        .define("NK_INCLUDE_COMMAND_USERDATA", "1")
        .define("NK_INCLUDE_VERTEX_BUFFER_OUTPUT", "1")
        .define("NK_INCLUDE_FONT_BAKING", "1")
        .define("NK_INCLUDE_DEFAULT_FONT", "1")
        .define("NK_IMPLEMENTATION", "1")
        .include(include)
        .static_flag(true)
        .debug(true)
        .cargo_metadata(true);

    builder.compile("nuklear");
}
