//! Singleton interface for dynamically loaded VISA library.
//!
//! Provides free-standing functions with the same signatures as the static link API,
//! backed by a global [`LibVisa`] instance that is lazily loaded on first use.
//!
//! # Initialization
//!
//! The library is automatically loaded with the platform default path on the first
//! VISA function call. To initialize manually or with a custom path, use
//! [`load_visa_library`] or [`load_visa_library_from_path`] before any other call.
//!
//! # Example
//!
//! ```no_run
//! use visa_sys::*;
//!
//! // Option A: let it auto-load on first use
//! let mut session = 0;
//! unsafe { viOpenDefaultRM(&mut session as ViPSession); }
//!
//! // Option B: manually initialize before use (with error handling)
//! if let Err(e) = load_visa_library() {
//!     eprintln!("VISA not available: {e}");
//!     return;
//! }
//! ```

use std::sync::OnceLock;

use super::*;

static __LIB_VISA: OnceLock<LibVisa> = OnceLock::new();

fn __default_lib_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "VISA.framework/VISA"
    } else if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            "visa64.dll"
        } else {
            "visa32.dll"
        }
    } else {
        "libvisa.so"
    }
}

fn __load_default() -> LibVisa {
    unsafe {
        LibVisa::new(__default_lib_name())
            .expect("failed to load VISA library with the default path; call load_visa_library_from_path() to specify a custom path")
    }
}

/// Manually load the VISA library from the platform default path.
///
/// Returns `Ok(())` if the library was loaded successfully (or was already loaded).
/// Returns `Err` if loading fails, e.g. because the library is not installed.
///
/// This is optional — calling any VISA function will auto-load the library with the
/// default path. Use this function to detect missing libraries before the first call.
pub fn load_visa_library() -> Result<(), libloading::Error> {
    load_visa_library_from_path(__default_lib_name())
}

/// Manually load the VISA library from a custom path.
///
/// Returns `Ok(())` if the library was loaded successfully (or was already loaded).
/// Returns `Err` if loading fails.
///
/// # Panics
///
/// This function does not panic if the library was already loaded (even from a
/// different path) — the first loaded instance is kept.
pub fn load_visa_library_from_path<P: AsRef<std::ffi::OsStr>>(
    path: P,
) -> Result<(), libloading::Error> {
    if __LIB_VISA.get().is_some() {
        return Ok(());
    }
    let lib = unsafe { LibVisa::new(path)? };
    // If another thread loaded first, that's fine — we drop ours.
    let _ = __LIB_VISA.set(lib);
    Ok(())
}

/// Get a reference to the loaded [`LibVisa`] instance, if available.
///
/// Returns `None` if the library has not been loaded yet.
pub fn get_visa_library() -> Option<&'static LibVisa> {
    __LIB_VISA.get()
}

pub unsafe fn viOpenDefaultRM(vi: ViPSession) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viOpenDefaultRM(vi)
}

pub unsafe fn viFindRsrc(
    sesn: ViSession,
    expr: ViConstString,
    vi: ViPFindList,
    retCnt: ViPUInt32,
    desc: *mut ViChar,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viFindRsrc(sesn, expr, vi, retCnt, desc)
}

pub unsafe fn viFindNext(vi: ViFindList, desc: *mut ViChar) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viFindNext(vi, desc)
}

pub unsafe fn viParseRsrc(
    rmSesn: ViSession,
    rsrcName: ViConstRsrc,
    intfType: ViPUInt16,
    intfNum: ViPUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viParseRsrc(rmSesn, rsrcName, intfType, intfNum)
}

pub unsafe fn viParseRsrcEx(
    rmSesn: ViSession,
    rsrcName: ViConstRsrc,
    intfType: ViPUInt16,
    intfNum: ViPUInt16,
    rsrcClass: *mut ViChar,
    expandedUnaliasedName: *mut ViChar,
    aliasIfExists: *mut ViChar,
) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viParseRsrcEx(
        rmSesn,
        rsrcName,
        intfType,
        intfNum,
        rsrcClass,
        expandedUnaliasedName,
        aliasIfExists,
    )
}

pub unsafe fn viOpen(
    sesn: ViSession,
    name: ViConstRsrc,
    mode: ViAccessMode,
    timeout: ViUInt32,
    vi: ViPSession,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viOpen(sesn, name, mode, timeout, vi)
}

pub unsafe fn viClose(vi: ViObject) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viClose(vi)
}

pub unsafe fn viSetAttribute(vi: ViObject, attrName: ViAttr, attrValue: ViAttrState) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viSetAttribute(vi, attrName, attrValue)
}

pub unsafe fn viGetAttribute(
    vi: ViObject,
    attrName: ViAttr,
    attrValue: *mut ::std::os::raw::c_void,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viGetAttribute(vi, attrName, attrValue)
}

pub unsafe fn viStatusDesc(vi: ViObject, status: ViStatus, desc: *mut ViChar) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viStatusDesc(vi, status, desc)
}

pub unsafe fn viTerminate(vi: ViObject, degree: ViUInt16, jobId: ViJobId) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viTerminate(vi, degree, jobId)
}

pub unsafe fn viLock(
    vi: ViSession,
    lockType: ViAccessMode,
    timeout: ViUInt32,
    requestedKey: ViConstKeyId,
    accessKey: *mut ViChar,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viLock(vi, lockType, timeout, requestedKey, accessKey)
}

pub unsafe fn viUnlock(vi: ViSession) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viUnlock(vi)
}

pub unsafe fn viEnableEvent(
    vi: ViSession,
    eventType: ViEventType,
    mechanism: ViUInt16,
    context: ViEventFilter,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viEnableEvent(vi, eventType, mechanism, context)
}

pub unsafe fn viDisableEvent(
    vi: ViSession,
    eventType: ViEventType,
    mechanism: ViUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viDisableEvent(vi, eventType, mechanism)
}

pub unsafe fn viDiscardEvents(
    vi: ViSession,
    eventType: ViEventType,
    mechanism: ViUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viDiscardEvents(vi, eventType, mechanism)
}

pub unsafe fn viWaitOnEvent(
    vi: ViSession,
    inEventType: ViEventType,
    timeout: ViUInt32,
    outEventType: ViPEventType,
    outContext: ViPEvent,
) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viWaitOnEvent(
        vi,
        inEventType,
        timeout,
        outEventType,
        outContext,
    )
}

pub unsafe fn viInstallHandler(
    vi: ViSession,
    eventType: ViEventType,
    handler: ViHndlr,
    userHandle: ViAddr,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viInstallHandler(vi, eventType, handler, userHandle)
}

pub unsafe fn viUninstallHandler(
    vi: ViSession,
    eventType: ViEventType,
    handler: ViHndlr,
    userHandle: ViAddr,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viUninstallHandler(vi, eventType, handler, userHandle)
}

pub unsafe fn viRead(vi: ViSession, buf: ViPBuf, cnt: ViUInt32, retCnt: ViPUInt32) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viRead(vi, buf, cnt, retCnt)
}

pub unsafe fn viReadAsync(vi: ViSession, buf: ViPBuf, cnt: ViUInt32, jobId: ViPJobId) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viReadAsync(vi, buf, cnt, jobId)
}

pub unsafe fn viReadToFile(
    vi: ViSession,
    filename: ViConstString,
    cnt: ViUInt32,
    retCnt: ViPUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viReadToFile(vi, filename, cnt, retCnt)
}

pub unsafe fn viWrite(
    vi: ViSession,
    buf: ViConstBuf,
    cnt: ViUInt32,
    retCnt: ViPUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viWrite(vi, buf, cnt, retCnt)
}

pub unsafe fn viWriteAsync(
    vi: ViSession,
    buf: ViConstBuf,
    cnt: ViUInt32,
    jobId: ViPJobId,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viWriteAsync(vi, buf, cnt, jobId)
}

pub unsafe fn viWriteFromFile(
    vi: ViSession,
    filename: ViConstString,
    cnt: ViUInt32,
    retCnt: ViPUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viWriteFromFile(vi, filename, cnt, retCnt)
}

pub unsafe fn viAssertTrigger(vi: ViSession, protocol: ViUInt16) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viAssertTrigger(vi, protocol)
}

pub unsafe fn viReadSTB(vi: ViSession, status: ViPUInt16) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viReadSTB(vi, status)
}

pub unsafe fn viClear(vi: ViSession) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viClear(vi)
}

pub unsafe fn viSetBuf(vi: ViSession, mask: ViUInt16, size: ViUInt32) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viSetBuf(vi, mask, size)
}

pub unsafe fn viFlush(vi: ViSession, mask: ViUInt16) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viFlush(vi, mask)
}

pub unsafe fn viBufWrite(
    vi: ViSession,
    buf: ViConstBuf,
    cnt: ViUInt32,
    retCnt: ViPUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viBufWrite(vi, buf, cnt, retCnt)
}

pub unsafe fn viBufRead(vi: ViSession, buf: ViPBuf, cnt: ViUInt32, retCnt: ViPUInt32) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viBufRead(vi, buf, cnt, retCnt)
}

pub unsafe fn viVPrintf(
    vi: ViSession,
    writeFmt: ViConstString,
    params: *mut __va_list_tag,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viVPrintf(vi, writeFmt, params)
}

pub unsafe fn viVSPrintf(
    vi: ViSession,
    buf: ViPBuf,
    writeFmt: ViConstString,
    parms: *mut __va_list_tag,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viVSPrintf(vi, buf, writeFmt, parms)
}

pub unsafe fn viVScanf(
    vi: ViSession,
    readFmt: ViConstString,
    params: *mut __va_list_tag,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viVScanf(vi, readFmt, params)
}

pub unsafe fn viVSScanf(
    vi: ViSession,
    buf: ViConstBuf,
    readFmt: ViConstString,
    parms: *mut __va_list_tag,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viVSScanf(vi, buf, readFmt, parms)
}

pub unsafe fn viVQueryf(
    vi: ViSession,
    writeFmt: ViConstString,
    readFmt: ViConstString,
    params: *mut __va_list_tag,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viVQueryf(vi, writeFmt, readFmt, params)
}

pub unsafe fn viIn8(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val8: ViPUInt8,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viIn8(vi, space, offset, val8)
}

pub unsafe fn viOut8(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val8: ViUInt8,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viOut8(vi, space, offset, val8)
}

pub unsafe fn viIn16(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val16: ViPUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viIn16(vi, space, offset, val16)
}

pub unsafe fn viOut16(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val16: ViUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viOut16(vi, space, offset, val16)
}

pub unsafe fn viIn32(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val32: ViPUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viIn32(vi, space, offset, val32)
}

pub unsafe fn viOut32(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val32: ViUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viOut32(vi, space, offset, val32)
}

pub unsafe fn viIn64(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val64: ViPUInt64,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viIn64(vi, space, offset, val64)
}

pub unsafe fn viOut64(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val64: ViUInt64,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viOut64(vi, space, offset, val64)
}

pub unsafe fn viIn8Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val8: ViPUInt8,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viIn8Ex(vi, space, offset, val8)
}

pub unsafe fn viOut8Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val8: ViUInt8,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viOut8Ex(vi, space, offset, val8)
}

pub unsafe fn viIn16Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val16: ViPUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viIn16Ex(vi, space, offset, val16)
}

pub unsafe fn viOut16Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val16: ViUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viOut16Ex(vi, space, offset, val16)
}

pub unsafe fn viIn32Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val32: ViPUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viIn32Ex(vi, space, offset, val32)
}

pub unsafe fn viOut32Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val32: ViUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viOut32Ex(vi, space, offset, val32)
}

pub unsafe fn viIn64Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val64: ViPUInt64,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viIn64Ex(vi, space, offset, val64)
}

pub unsafe fn viOut64Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val64: ViUInt64,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viOut64Ex(vi, space, offset, val64)
}

pub unsafe fn viMoveIn8(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf8: ViAUInt8,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveIn8(vi, space, offset, length, buf8)
}

pub unsafe fn viMoveOut8(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf8: ViAUInt8,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveOut8(vi, space, offset, length, buf8)
}

pub unsafe fn viMoveIn16(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf16: ViAUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveIn16(vi, space, offset, length, buf16)
}

pub unsafe fn viMoveOut16(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf16: ViAUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveOut16(vi, space, offset, length, buf16)
}

pub unsafe fn viMoveIn32(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf32: ViAUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveIn32(vi, space, offset, length, buf32)
}

pub unsafe fn viMoveOut32(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf32: ViAUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveOut32(vi, space, offset, length, buf32)
}

pub unsafe fn viMoveIn64(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf64: ViAUInt64,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveIn64(vi, space, offset, length, buf64)
}

pub unsafe fn viMoveOut64(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf64: ViAUInt64,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveOut64(vi, space, offset, length, buf64)
}

pub unsafe fn viMoveIn8Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf8: ViAUInt8,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveIn8Ex(vi, space, offset, length, buf8)
}

pub unsafe fn viMoveOut8Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf8: ViAUInt8,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveOut8Ex(vi, space, offset, length, buf8)
}

pub unsafe fn viMoveIn16Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf16: ViAUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveIn16Ex(vi, space, offset, length, buf16)
}

pub unsafe fn viMoveOut16Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf16: ViAUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveOut16Ex(vi, space, offset, length, buf16)
}

pub unsafe fn viMoveIn32Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf32: ViAUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveIn32Ex(vi, space, offset, length, buf32)
}

pub unsafe fn viMoveOut32Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf32: ViAUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveOut32Ex(vi, space, offset, length, buf32)
}

pub unsafe fn viMoveIn64Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf64: ViAUInt64,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveIn64Ex(vi, space, offset, length, buf64)
}

pub unsafe fn viMoveOut64Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf64: ViAUInt64,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMoveOut64Ex(vi, space, offset, length, buf64)
}

pub unsafe fn viMove(
    vi: ViSession,
    srcSpace: ViUInt16,
    srcOffset: ViBusAddress,
    srcWidth: ViUInt16,
    destSpace: ViUInt16,
    destOffset: ViBusAddress,
    destWidth: ViUInt16,
    srcLength: ViBusSize,
) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viMove(
        vi, srcSpace, srcOffset, srcWidth, destSpace, destOffset, destWidth, srcLength,
    )
}

pub unsafe fn viMoveAsync(
    vi: ViSession,
    srcSpace: ViUInt16,
    srcOffset: ViBusAddress,
    srcWidth: ViUInt16,
    destSpace: ViUInt16,
    destOffset: ViBusAddress,
    destWidth: ViUInt16,
    srcLength: ViBusSize,
    jobId: ViPJobId,
) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viMoveAsync(
        vi, srcSpace, srcOffset, srcWidth, destSpace, destOffset, destWidth, srcLength, jobId,
    )
}

pub unsafe fn viMoveEx(
    vi: ViSession,
    srcSpace: ViUInt16,
    srcOffset: ViBusAddress64,
    srcWidth: ViUInt16,
    destSpace: ViUInt16,
    destOffset: ViBusAddress64,
    destWidth: ViUInt16,
    srcLength: ViBusSize,
) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viMoveEx(
        vi, srcSpace, srcOffset, srcWidth, destSpace, destOffset, destWidth, srcLength,
    )
}

pub unsafe fn viMoveAsyncEx(
    vi: ViSession,
    srcSpace: ViUInt16,
    srcOffset: ViBusAddress64,
    srcWidth: ViUInt16,
    destSpace: ViUInt16,
    destOffset: ViBusAddress64,
    destWidth: ViUInt16,
    srcLength: ViBusSize,
    jobId: ViPJobId,
) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viMoveAsyncEx(
        vi, srcSpace, srcOffset, srcWidth, destSpace, destOffset, destWidth, srcLength, jobId,
    )
}

pub unsafe fn viMapAddress(
    vi: ViSession,
    mapSpace: ViUInt16,
    mapOffset: ViBusAddress,
    mapSize: ViBusSize,
    access: ViBoolean,
    suggested: ViAddr,
    address: ViPAddr,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMapAddress(vi, mapSpace, mapOffset, mapSize, access, suggested, address)
}

pub unsafe fn viUnmapAddress(vi: ViSession) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viUnmapAddress(vi)
}

pub unsafe fn viMapAddressEx(
    vi: ViSession,
    mapSpace: ViUInt16,
    mapOffset: ViBusAddress64,
    mapSize: ViBusSize,
    access: ViBoolean,
    suggested: ViAddr,
    address: ViPAddr,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMapAddressEx(vi, mapSpace, mapOffset, mapSize, access, suggested, address)
}

pub unsafe fn viPeek8(vi: ViSession, address: ViAddr, val8: ViPUInt8) {
    __LIB_VISA
        .get_or_init(__load_default)
        .viPeek8(vi, address, val8)
}

pub unsafe fn viPoke8(vi: ViSession, address: ViAddr, val8: ViUInt8) {
    __LIB_VISA
        .get_or_init(__load_default)
        .viPoke8(vi, address, val8)
}

pub unsafe fn viPeek16(vi: ViSession, address: ViAddr, val16: ViPUInt16) {
    __LIB_VISA
        .get_or_init(__load_default)
        .viPeek16(vi, address, val16)
}

pub unsafe fn viPoke16(vi: ViSession, address: ViAddr, val16: ViUInt16) {
    __LIB_VISA
        .get_or_init(__load_default)
        .viPoke16(vi, address, val16)
}

pub unsafe fn viPeek32(vi: ViSession, address: ViAddr, val32: ViPUInt32) {
    __LIB_VISA
        .get_or_init(__load_default)
        .viPeek32(vi, address, val32)
}

pub unsafe fn viPoke32(vi: ViSession, address: ViAddr, val32: ViUInt32) {
    __LIB_VISA
        .get_or_init(__load_default)
        .viPoke32(vi, address, val32)
}

pub unsafe fn viPeek64(vi: ViSession, address: ViAddr, val64: ViPUInt64) {
    __LIB_VISA
        .get_or_init(__load_default)
        .viPeek64(vi, address, val64)
}

pub unsafe fn viPoke64(vi: ViSession, address: ViAddr, val64: ViUInt64) {
    __LIB_VISA
        .get_or_init(__load_default)
        .viPoke64(vi, address, val64)
}

pub unsafe fn viMemAlloc(vi: ViSession, size: ViBusSize, offset: ViPBusAddress) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMemAlloc(vi, size, offset)
}

pub unsafe fn viMemFree(vi: ViSession, offset: ViBusAddress) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viMemFree(vi, offset)
}

pub unsafe fn viMemAllocEx(vi: ViSession, size: ViBusSize, offset: ViPBusAddress64) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMemAllocEx(vi, size, offset)
}

pub unsafe fn viMemFreeEx(vi: ViSession, offset: ViBusAddress64) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMemFreeEx(vi, offset)
}

pub unsafe fn viGpibControlREN(vi: ViSession, mode: ViUInt16) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viGpibControlREN(vi, mode)
}

pub unsafe fn viGpibControlATN(vi: ViSession, mode: ViUInt16) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viGpibControlATN(vi, mode)
}

pub unsafe fn viGpibSendIFC(vi: ViSession) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viGpibSendIFC(vi)
}

pub unsafe fn viGpibCommand(
    vi: ViSession,
    cmd: ViConstBuf,
    cnt: ViUInt32,
    retCnt: ViPUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viGpibCommand(vi, cmd, cnt, retCnt)
}

pub unsafe fn viGpibPassControl(vi: ViSession, primAddr: ViUInt16, secAddr: ViUInt16) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viGpibPassControl(vi, primAddr, secAddr)
}

pub unsafe fn viVxiCommandQuery(
    vi: ViSession,
    mode: ViUInt16,
    cmd: ViUInt32,
    response: ViPUInt32,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viVxiCommandQuery(vi, mode, cmd, response)
}

pub unsafe fn viAssertUtilSignal(vi: ViSession, line: ViUInt16) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viAssertUtilSignal(vi, line)
}

pub unsafe fn viAssertIntrSignal(vi: ViSession, mode: ViInt16, statusID: ViUInt32) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viAssertIntrSignal(vi, mode, statusID)
}

pub unsafe fn viMapTrigger(
    vi: ViSession,
    trigSrc: ViInt16,
    trigDest: ViInt16,
    mode: ViUInt16,
) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viMapTrigger(vi, trigSrc, trigDest, mode)
}

pub unsafe fn viUnmapTrigger(vi: ViSession, trigSrc: ViInt16, trigDest: ViInt16) -> ViStatus {
    __LIB_VISA
        .get_or_init(__load_default)
        .viUnmapTrigger(vi, trigSrc, trigDest)
}

pub unsafe fn viUsbControlOut(
    vi: ViSession,
    bmRequestType: ViInt16,
    bRequest: ViInt16,
    wValue: ViUInt16,
    wIndex: ViUInt16,
    wLength: ViUInt16,
    buf: ViConstBuf,
) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viUsbControlOut(
        vi,
        bmRequestType,
        bRequest,
        wValue,
        wIndex,
        wLength,
        buf,
    )
}

pub unsafe fn viUsbControlIn(
    vi: ViSession,
    bmRequestType: ViInt16,
    bRequest: ViInt16,
    wValue: ViUInt16,
    wIndex: ViUInt16,
    wLength: ViUInt16,
    buf: ViPBuf,
    retCnt: ViPUInt16,
) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viUsbControlIn(
        vi,
        bmRequestType,
        bRequest,
        wValue,
        wIndex,
        wLength,
        buf,
        retCnt,
    )
}

pub unsafe fn viPxiReserveTriggers(
    vi: ViSession,
    cnt: ViInt16,
    trigBuses: ViAInt16,
    trigLines: ViAInt16,
    failureIndex: ViPInt16,
) -> ViStatus {
    __LIB_VISA.get_or_init(__load_default).viPxiReserveTriggers(
        vi,
        cnt,
        trigBuses,
        trigLines,
        failureIndex,
    )
}
