extern crate bindgen;
extern crate cc;

use std::env;
use std::path::{Path, PathBuf};

fn main () {
    let default_include_dir = PathBuf::from("/usr/include/php");
    let include_dir = env::var_os("PHP_INCLUDE_DIR").map(PathBuf::from).unwrap_or(default_include_dir);

    generate_pphp_helper(&include_dir);
    generate_php_bindings(&include_dir);
}

fn generate_pphp_helper(include_dir: &PathBuf) {
    println!("cargo:rerun-if-changed=src/pphp_helper.c");
    let includes = ["/", "/TSRM", "/Zend", "/main"].iter().map(|d| {
        format!("{}{}", include_dir.to_string_lossy(), d)
    }).collect::<Vec<String>>();

    let mut builder = cc::Build::new();
    builder.file("src/pphp_helper.c");

    for include in includes {
        builder.include(include);
    }

    builder.compile("libpphp_helper.a");
}

fn generate_php_bindings(include_dir: &PathBuf) {
    println!("cargo:rerun-if-env-changed=PHP_INCLUDE_DIR");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=src/_php_bindings.rs");
    if Path::exists(Path::new("src/_php_bindings.rs")) {
        // already generated the bindings
        return;
    }

    if !include_dir.exists() {
        panic!(
            "PHP include directory does not exist: {}",
            include_dir.to_string_lossy()
        );
    }

    let includes = ["/", "/TSRM", "/Zend", "/main"].iter().map(|d| {
        format!("-I{}{}", include_dir.to_string_lossy(), d)
    }).collect::<Vec<String>>();

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(includes)
        .hide_type("FP_NAN")
        .hide_type("FP_INFINITE")
        .hide_type("FP_ZERO")
        .hide_type("FP_SUBNORMAL")
        .hide_type("FP_NORMAL")
        .hide_type("max_align_t")
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/_php_bindings.rs")
        .expect("Couldn't write bindings!");
}
