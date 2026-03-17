use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let vendor = root.join("vendor").join("src");
    let parser = vendor.join("parser.c");
    let header = vendor.join("tree_sitter").join("parser.h");

    if !parser.exists() || !header.exists() {
        panic!(
            "tree-sitter grammar missing. Run `npm run setup` at repo root to fetch parser sources."
        );
    }

    println!("cargo:rerun-if-changed={}", parser.display());
    println!("cargo:rerun-if-changed={}", header.display());

    let mut build = cc::Build::new();
    build.include(vendor.clone()).file(parser).warnings(false);

    let scanner = vendor.join("scanner.c");
    if scanner.exists() {
        println!("cargo:rerun-if-changed={}", scanner.display());
        build.file(scanner);
    }

    build.compile("tree-sitter-autohotkey");
}
