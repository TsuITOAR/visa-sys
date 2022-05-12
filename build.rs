use std::env;
use std::path::PathBuf;

fn main() {
    if cfg!(target_arch = "x86_64") {
        println!("cargo:rustc-link-lib=visa64");
        #[cfg(target_os = "windows")]
        {
            let search_path = r#"C:\Program Files (x86)\IVI Foundation\VISA\Win64\Lib_x64\msc"#;
            println!("cargo:rustc-link-search={}", search_path);
        }
    } else if cfg!(target_arch = "x86") {
        println!("cargo:rustc-link-lib=visa32");
        #[cfg(target_os = "windows")]
        {
            let search_path = r#"C:\Program Files (x86)\IVI Foundation\VISA\WinNT\lib\msc"#;
            println!("cargo:rustc-link-search={}", search_path);
        }
    } else {
        unimplemented!("target arch not implemented");
    }
    if let Some(p) = std::env::var_os("VISA_LIB_PATH") {
        p.to_str()
            .map(|p| println!("cargo:rustc-link-search={}", p))
            .unwrap_or_else(|| eprintln!("WARN: illegal value of 'VISA_LIB_PATH'"));
    }

    println!("cargo:rerun-if-changed=wrapper.h");

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
