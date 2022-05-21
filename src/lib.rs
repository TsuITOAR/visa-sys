#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(feature = "bindgen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(feature = "bindgen"))]
include!("./prebind/bindings.rs");

#[cfg(test)]
mod tests {
    use super::*;
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
            let expr = std::ffi::CString::new(b"?*INSTR").unwrap();
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
                    VI_NULL,
                    VI_NULL,
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
