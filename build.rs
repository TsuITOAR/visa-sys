fn main() {
    println!("cargo::rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    println!("cargo::rerun-if-env-changed=CARGO_CFG_TARGET_OS");
    println!("cargo::rerun-if-env-changed=LIB_VISA_NAME");
    println!("cargo::rerun-if-env-changed=LIB_VISA_PATH");
    println!("cargo::rerun-if-env-changed=INCLUDE_VISA_PATH");
    println!("cargo::rerun-if-env-changed=OUT_DIR");

    #[cfg(not(any(docsrs, feature = "proc", feature = "dynamic_load")))]
    {
        link_lib();
        add_link_path();
    }
    #[cfg(feature = "bindgen")]
    bindgen::bindgen();
}

#[cfg(not(any(docsrs, feature = "proc", feature = "dynamic_load")))]
fn default_lib_name() -> &'static str {
    use std::env;
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    match (&*target_arch, &*target_os) {
        ("x86_64" | "aarch64", "macos") => "framework=VISA",
        (_, "linux") => "visa",
        ("x86_64", _) => "visa64",
        ("x86", _) => "visa32",
        _ => {
            unimplemented!("target arch {} not implemented", target_arch)
        }
    }
}

#[cfg(not(any(docsrs, feature = "proc", feature = "dynamic_load")))]
fn link_lib() {
    const LIB_NAME_VAR: &str = "LIB_VISA_NAME";
    use std::env;
    if let Some(lib_name) = env::var_os(LIB_NAME_VAR) {
        if let Some(l) = lib_name.to_str() {
            println!("cargo:rustc-link-lib={}", l);
            return;
        } else {
            println!("cargo:warning=illegal value of '{}'", LIB_NAME_VAR);
        }
    }
    println!("cargo:rustc-link-lib={}", default_lib_name());
}

#[cfg(not(any(docsrs, feature = "proc", feature = "dynamic_load")))]
fn add_link_path() {
    const LIB_PATH_VAR: &str = "LIB_VISA_PATH";
    use std::env;
    if let Some(p) = env::var_os(LIB_PATH_VAR) {
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
        let prefix = if target_os == "macos" {
            "framework="
        } else {
            ""
        };
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
        #[cfg(all(
            any(target_arch = "x86_64", target_arch = "aarch64"),
            target_os = "macos"
        ))]
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

    // Forces the dynamic-loading symbol-lookup strings to the bare C function
    // names. On macOS bindgen otherwise uses the Mach-O mangled name (a leading
    // underscore), which `dlsym` does not expect; the bare name is correct on
    // every target, so dynamic bindings regenerated on macOS work at runtime and
    // match the portable checked-in prebindings.
    #[cfg(feature = "dynamic_load")]
    #[derive(Debug)]
    struct BareLinkNames;
    #[cfg(feature = "dynamic_load")]
    impl bindgen::callbacks::ParseCallbacks for BareLinkNames {
        fn generated_link_name_override(
            &self,
            item_info: bindgen::callbacks::ItemInfo<'_>,
        ) -> Option<String> {
            match item_info.kind {
                bindgen::callbacks::ItemKind::Function => Some(item_info.name.to_owned()),
                _ => None,
            }
        }
    }

    pub fn bindgen() {
        let include_path =
            PathBuf::from(env::var_os(INCLUDE_PATH_VAR).unwrap_or("./include".into()));
        let header = include_path
            .join("visa.h")
            .to_str()
            .expect("path should be valid utf8 string")
            .to_owned();

        let base = bindgen::Builder::default()
            .header(header)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

        #[cfg(not(feature = "dynamic_load"))]
        let builder = base;

        // For dynamic loading, restrict generation to the VISA API. Without an
        // allowlist bindgen also emits function pointers for unrelated symbols
        // pulled in from system headers (e.g. `__darwin_*` on macOS); with the
        // generated `LibVisa` resolving every field, those would be required
        // symbols that no VISA library exports. `require_all(false)` additionally
        // makes loading tolerant of VISA implementations that omit rarely used
        // functions — a missing symbol only fails if that function is called.
        #[cfg(feature = "dynamic_load")]
        let builder = base
            .parse_callbacks(Box::new(BareLinkNames))
            .allowlist_function("vi.*")
            .allowlist_type("[_]*[Vv]i.*")
            .allowlist_var("_?VI_.*")
            .dynamic_library_name("LibVisa")
            .dynamic_link_require_all(false)
            .override_abi(bindgen::Abi::System, "vi.*");

        let bindings = builder.generate().expect("Unable to generate bindings");
        let out_path = PathBuf::from(env::var("OUT_DIR").expect("'OUT_DIR' should be set"));
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
