fn main() {
    #[cfg(not(any(docsrs, feature = "proc")))]
    {
        link_lib();
        add_link_path();
    }
    #[cfg(feature = "bindgen")]
    bindgen::bindgen();
}

#[cfg(not(any(docsrs, feature = "proc")))]
fn link_lib() {
    use std::env;
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let lib = match (&*target_arch, &*target_os) {
        ("x86_64", "macos") => "framework=VISA",
        ("x86_64", _) => "visa64",
        ("x86", _) => "visa32",
        _ => {
            unimplemented!("target arch {} not implemented", target_arch)
        }
    };
    println!("cargo:rustc-link-lib={}", lib);
}

#[cfg(not(any(docsrs, feature = "proc")))]
fn add_link_path() {
    const LIB_PATH_VAR: &str = "LIB_VISA_PATH";
    use std::env;
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let prefix = if target_os == "macos" {
        "framework="
    } else {
        ""
    };
    if let Some(p) = env::var_os(LIB_PATH_VAR) {
        match p.to_str() {
            Some(p) => println!("cargo:rustc-link-search={}{}", prefix, p),
            None => println!("cargo:warning=illegal value of '{}'", LIB_PATH_VAR),
        }
    } else {
        #[cfg(all(target_arch = "x86", target_os = "windows"))]
        {
            let search_path = r#"C:\Program Files (x86)\IVI Foundation\VISA\WinNT\lib\msc"#;
            println!("cargo:rustc-link-search={search_path}");
        }
        #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
        {
            let search_path = r#"C:\Program Files (x86)\IVI Foundation\VISA\WinNT\Lib_x64\msc"#;
            println!("cargo:rustc-link-search={search_path}");
        }
        #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
        {
            let search_path = "framework=/Library/Frameworks";
            println!("cargo:rustc-link-search={search_path}");
        }
    }
}

#[cfg(feature = "bindgen")]
mod bindgen {
    use std::env;
    use std::path::PathBuf;
    const INCLUDE_PATH_VAR: &str = "INCLUDE_VISA_PATH";
    pub fn bindgen() {
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
}
