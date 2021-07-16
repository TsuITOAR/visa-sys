use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=visa64");
    println!("cargo:rerun-if-changed=wrapper.h");
    let search_path = r#"C:\Program Files\IVI Foundation\VISA\Win64\Lib_x64\msc"#;
    println!("cargo:rustc-link-search={}", search_path);
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
