//! FFI bindings to the VISA (Virtual Instrument Software Architecture) library.
//!
//! By default the system VISA library is linked at build time. Enabling the
//! `dynamic_load` feature instead loads it at run time via `libloading`, so the
//! same binary can run with or without VISA installed. In that mode the library
//! auto-loads on the first VISA call (or explicitly via `load_visa_library` /
//! `load_visa_library_from_path`); the free VISA functions keep identical
//! signatures in both modes. See the module docs and the crate README for the
//! initialization model, limitations, and the build-time environment variables
//! (`LIB_VISA_NAME`, `LIB_VISA_PATH`, `INCLUDE_VISA_PATH`).
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// The bindings (generated or pre-generated) are thin FFI surfaces: their `unsafe`
// items carry no `# Safety` docs and some VISA calls take many arguments. Each
// `include!` lives in a `mod` so those allows are scoped to the bindings rather
// than applied crate-wide; `pub use` keeps the API flat at the crate root.
#[cfg(feature = "bindgen")]
#[allow(clippy::missing_safety_doc, clippy::too_many_arguments)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(all(not(feature = "bindgen"), feature = "dynamic_load"))]
#[allow(clippy::missing_safety_doc, clippy::too_many_arguments)]
mod bindings {
    include!("./prebind/bindings_dynamic.rs");
}

#[cfg(all(not(feature = "bindgen"), not(feature = "dynamic_load")))]
#[allow(clippy::missing_safety_doc, clippy::too_many_arguments)]
mod bindings {
    include!("./prebind/bindings.rs");
}

pub use bindings::*;

#[cfg(feature = "dynamic_load")]
mod dynamic_loading;

#[cfg(feature = "dynamic_load")]
pub use dynamic_loading::*;

#[cfg(test)]
mod tests {
    use super::*;

    // Hardware-free: exercises the dynamic-loading plumbing without touching the
    // global singleton (so it is order-independent and runs anywhere). Loading a
    // missing library must return `Err`, not panic.
    #[cfg(feature = "dynamic_load")]
    #[test]
    fn load_missing_library_is_err() {
        let r = unsafe { LibVisa::new("nonexistent-visa-library-xyz123") };
        assert!(r.is_err(), "loading a missing library should return Err");
    }

    // Real end-to-end exercise of the `va_list` path: `viVSPrintf` reads an
    // argument *through* the `va_list` we pass and formats it into a buffer, so a
    // correct result proves the `VaListArg` conversion forwards the list to the
    // loaded library intact. `viVSPrintf` validates the session and needs an
    // instrument one (it formats into our buffer, no device traffic), so we open
    // the first discoverable resource and skip if none is present. The hand-built
    // `va_list` is a pointer to a packed argument area, matching the aarch64-macOS
    // ABI; gated to that host so the layout is valid and verifiable directly.
    #[cfg(all(feature = "dynamic_load", target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn vsprintf_reads_va_list_arg() {
        use std::ffi::{CStr, CString};
        unsafe {
            let mut rm: ViSession = 0;
            assert_eq!(
                viOpenDefaultRM(&mut rm as ViPSession),
                VI_SUCCESS as ViStatus
            );

            let mut find: ViFindList = 0;
            let mut cnt: ViUInt32 = 0;
            let mut desc = [0 as ViChar; VI_FIND_BUFLEN as usize];
            let expr = CString::new("?*INSTR").unwrap();
            let found = viFindRsrc(rm, expr.as_ptr(), &mut find, &mut cnt, desc.as_mut_ptr());
            if found != VI_SUCCESS as ViStatus || cnt == 0 {
                viClose(rm);
                return; // no instrument available; nothing to exercise
            }
            let mut instr: ViSession = 0;
            if viOpen(
                rm,
                desc.as_ptr(),
                VI_NULL as ViAccessMode,
                VI_NULL as ViUInt32,
                &mut instr as ViPSession,
            ) != VI_SUCCESS as ViStatus
            {
                viClose(rm);
                return; // could not open the resource; skip
            }

            // Three `%d` arguments, so the library must `va_arg` three times,
            // advancing through the list. On aarch64 macOS each variadic argument
            // occupies an 8-byte slot in the `va_list`; an `int` sits in the low
            // 4 bytes (little-endian), so the layout is one `u64` slot per value.
            let mut args: [u64; 3] = [1, 22, 333];
            let mut out = [0u8; 64];
            let st = viVSPrintf(
                instr,
                out.as_mut_ptr() as ViPBuf,
                c"%d %d %d".as_ptr() as ViConstString,
                args.as_mut_ptr() as ViVAList,
            );
            viClose(instr);
            viClose(rm);

            assert_eq!(st, VI_SUCCESS as ViStatus, "viVSPrintf returned an error");
            let got = CStr::from_ptr(out.as_ptr() as *const std::os::raw::c_char)
                .to_str()
                .unwrap();
            assert_eq!(got, "1 22 333", "va_list arguments not read correctly");
        }
    }

    #[test]
    fn open_default_RM() {
        let mut session: ViSession = 0;
        unsafe {
            assert_eq!(viOpenDefaultRM(&mut session as ViPSession), VI_SUCCESS as _,);
        }
    }
    #[test]
    fn find_resource() {
        let mut defaultRM: ViSession = 0;
        let mut find_list: ViFindList = 0;
        let mut ret_cnt: ViUInt32 = 0;
        let mut instr_desc: [ViChar; VI_FIND_BUFLEN as usize] = [0; VI_FIND_BUFLEN as usize];
        unsafe {
            assert_eq!(
                viOpenDefaultRM(&mut defaultRM as ViPSession),
                VI_SUCCESS as _,
                "Could not open a session to the VISA Resource Manager!\n"
            );
            let expr = std::ffi::CString::new(*b"?*INSTR").unwrap();
            assert_eq!(
                viFindRsrc(
                    defaultRM,
                    expr.as_ptr(),
                    &mut find_list,
                    &mut ret_cnt,
                    &mut instr_desc as _
                ),
                VI_SUCCESS as _,
                "An error occurred while finding resources.\n"
            );
            eprintln!(
                "{} instruments, serial ports, and other resources found:",
                ret_cnt,
            );
            let mut instr: ViSession = 0;

            loop {
                eprintln!("{} targets to connect", ret_cnt);
                let resource = instr_desc
                    .into_iter()
                    .map(|x| x as u8 as char)
                    .collect::<String>();
                eprintln!("connecting to '{}'", resource);
                let open_status = viOpen(
                    defaultRM,
                    &instr_desc as _,
                    VI_NULL as _,
                    VI_NULL as _,
                    &mut instr as _,
                );
                if open_status != VI_SUCCESS as _ {
                    eprintln!(
                        "An error '{}' occurred opening a session to '{}'\n",
                        open_status,
                        instr_desc
                            .into_iter()
                            .map(|x| x as u8 as char)
                            .collect::<String>()
                    );
                } else {
                    eprintln!("connected to '{}'", resource);
                    viClose(instr);
                }
                ret_cnt -= 1;
                if ret_cnt > 0 {
                    let find_status = viFindNext(find_list, &mut instr_desc as _);
                    if find_status != VI_SUCCESS as _ {
                        eprintln!(
                            "An error '{}' occurred finding the next resource\n.",
                            find_status
                        );
                        break;
                    }
                } else {
                    break;
                }
            }
            viClose(defaultRM);
        }
    }
}
