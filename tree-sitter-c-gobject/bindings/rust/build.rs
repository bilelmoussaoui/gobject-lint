fn main() {
    let qemu = std::env::var_os("CARGO_FEATURE_QEMU").is_some();
    let src_dir = if qemu {
        std::path::Path::new("qemu/src")
    } else {
        std::path::Path::new("src")
    };

    let mut c_config = cc::Build::new();
    c_config.std("c11").include("src");
    if qemu {
        c_config.define("TREE_SITTER_C_GOBJECT_QEMU", None);
    }

    let parser_path = src_dir.join("parser.c");
    c_config.file(&parser_path);
    println!("cargo:rerun-if-changed={}", parser_path.to_str().unwrap());

    let scanner_path = std::path::Path::new("src/scanner.c");
    c_config.file(scanner_path);
    println!("cargo:rerun-if-changed={}", scanner_path.to_str().unwrap());
    println!("cargo:rerun-if-changed=qemu/modifiers.h");

    c_config.compile("tree-sitter-c-gobject");
}
