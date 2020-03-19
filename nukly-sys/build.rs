use std::path::Path;

fn main() {
    let src = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("nuklear_impl.c");
    let include = Path::new(std::env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("external")
        .join("nuklear");

    let mut builder = cc::Build::new();
    builder
        .file(src)
        .include(include)
        .static_flag(true)
        .cargo_metadata(true);

    builder.compile("nuklear");
}
