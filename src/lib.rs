#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));




#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;
    #[test]
    fn open_default_RM() {
        unsafe {
            let session: ViSession;
            assert_eq!(viOpenDefaultRM(&session as ViPSession) < VI_SUCCESS, false);
        }
    }
}
