fn main() {
    #[cfg(feature = "bindgen")]
    bindgen::bindgen();
}

#[cfg(feature = "bindgen")]
mod bindgen {
    use std::env;
    use std::path::PathBuf;
    const LIB_PATH_VAR: &str = "LIB_VISA_PATH";
    const INCLUDE_PATH_VAR: &str = "INCLUDE_VISA_PATH";
    pub fn bindgen() {
        link_lib();
        add_link_path();
        let include_path =
            PathBuf::from(env::var_os(INCLUDE_PATH_VAR).unwrap_or("./include".into()));
        let bindings = bindgen::Builder::default()
            .header(
                include_path
                    .join("visa.h")
                    .to_str()
                    .expect("path should be valid utf8 string"),
            )
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("Unable to generate bindings");
        let out_path = PathBuf::from(env::var("OUT_DIR").expect("'OUT_DIR' should be set"));
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    fn link_lib() {
        let lib = if cfg!(target_arch = "x86_64") {
            "visa64"
        } else if cfg!(target_arch = "x86") {
            "visa32"
        } else {
            unimplemented!("target arch not implemented");
        };
        println!("cargo:rustc-link-lib={}", lib);
    }
    fn add_link_path() {
        if let Some(p) = env::var_os(LIB_PATH_VAR) {
            p.to_str()
                .map(|p| println!("cargo:rustc-link-search={}", p))
                .unwrap_or_else(|| eprintln!("WARN: illegal value of '{}'", LIB_PATH_VAR));
        } else {
            #[cfg(all(target_arch = "x86", target_os = "windows"))]
            {
                let search_path = r#"C:\Program Files (x86)\IVI Foundation\VISA\WinNT\lib\msc"#;
                println!("cargo:rustc-link-search={}", search_path);
            }
            #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
            {
                let search_path = r#"C:\Program Files (x86)\IVI Foundation\VISA\WinNT\Lib_x64\msc"#;
                println!("cargo:rustc-link-search={}", search_path);
            }
        }
    }
}
