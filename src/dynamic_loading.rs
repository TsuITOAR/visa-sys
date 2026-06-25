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
//! let mut session: ViSession = 0;
//! let status = unsafe { viOpenDefaultRM(&mut session as ViPSession) };
//! assert_eq!(status, VI_SUCCESS as ViStatus);
//!
//! // Option B: manually initialize before use (with error handling)
//! if let Err(e) = load_visa_library() {
//!     eprintln!("VISA not available: {e}");
//!     return;
//! }
//! ```
//!
//! # Limitations
//!
//! The C *variadic* formatting helpers — `viPrintf`, `viSPrintf`, `viScanf`,
//! `viSScanf` and `viQueryf` — are **not** exposed as free functions here, because
//! a variadic C function cannot be called through a `libloading` function pointer.
//! Use the explicit-`va_list` equivalents instead ([`viVPrintf`], [`viVSPrintf`],
//! [`viVScanf`], [`viVSScanf`], [`viVQueryf`]). The raw variadic symbols are still
//! resolved and reachable on the [`LibVisa`] struct via [`get_visa_library`] if you
//! must call them directly.
//!
//! With dynamic loading, a function is only resolved against the library that is
//! actually loaded. Calling a function that the loaded VISA implementation does not
//! export panics at the call site (see [`get_visa_library`] to probe availability).

// These free functions are thin FFI passthroughs: their safety is that of the
// underlying C call, and their argument counts come verbatim from the VISA API.
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]

use std::sync::OnceLock;

use super::*;

// `ViVAList` has a platform-dependent shape. On most targets the C `va_list` is a
// pointer, which bindgen renders as `*mut c_char`. On some ABIs (e.g. x86_64
// System V and AArch64 Linux) it is a one-element array, which bindgen renders as
// `[__va_list_tag; 1]` while the matching `LibVisa` method parameter decays to
// `*mut __va_list_tag`. `VaListArg` bridges both shapes so the free functions can
// forward a `ViVAList` to the loaded library regardless of how `va_list` is
// represented on the target, without a fragile per-target `cfg` matrix.
trait VaListArg {
    type Ptr;
    fn va_list_ptr(&mut self) -> Self::Ptr;
}
// `va_list` is a pointer (the common case, including all prebindings).
impl<T> VaListArg for *mut T {
    type Ptr = *mut T;
    fn va_list_ptr(&mut self) -> *mut T {
        *self
    }
}
// `va_list` is a one-element array; the callee expects a pointer to its first
// element. `self` is borrowed so the array outlives the FFI call.
impl<T> VaListArg for [T; 1] {
    type Ptr = *mut T;
    fn va_list_ptr(&mut self) -> *mut T {
        self.as_mut_ptr()
    }
}
macro_rules! va_list_arg {
    ($v:expr) => {
        $v.va_list_ptr()
    };
}

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
        LibVisa::new(__default_lib_name()).unwrap_or_else(|e| {
            panic!(
                "failed to load the VISA library from the default path '{}': {e}.\n\
                 Install a VISA runtime, or call load_visa_library_from_path() with a \
                 custom path before any other VISA call. To handle a missing library \
                 without panicking, call load_visa_library() first and check its result.",
                __default_lib_name()
            )
        })
    }
}

/// Access the loaded library, auto-loading it from the platform default path on
/// first use. This is the single entry point the free functions go through.
///
/// # Panics
///
/// Panics if the library has not been loaded yet and loading from the default
/// path fails (e.g. no VISA runtime is installed). Call [`load_visa_library`] or
/// [`load_visa_library_from_path`] beforehand to handle that case gracefully.
#[inline]
fn __lib() -> &'static LibVisa {
    __LIB_VISA.get_or_init(__load_default)
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
/// Returns `Ok(())` if the library was loaded successfully, or `Err` if loading
/// fails.
///
/// The first successful load wins: if the library was already loaded — including
/// an auto-load triggered by an earlier VISA call, or a load from a different
/// path — this is a no-op that returns `Ok(())` and the original instance is
/// kept. Call this (or [`load_visa_library`]) before any other VISA function to
/// be sure your path is the one used.
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
    __lib().viOpenDefaultRM(vi)
}

pub unsafe fn viFindRsrc(
    sesn: ViSession,
    expr: ViConstString,
    vi: ViPFindList,
    retCnt: ViPUInt32,
    desc: *mut ViChar,
) -> ViStatus {
    __lib().viFindRsrc(sesn, expr, vi, retCnt, desc)
}

pub unsafe fn viFindNext(vi: ViFindList, desc: *mut ViChar) -> ViStatus {
    __lib().viFindNext(vi, desc)
}

pub unsafe fn viParseRsrc(
    rmSesn: ViSession,
    rsrcName: ViConstRsrc,
    intfType: ViPUInt16,
    intfNum: ViPUInt16,
) -> ViStatus {
    __lib().viParseRsrc(rmSesn, rsrcName, intfType, intfNum)
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
    __lib().viParseRsrcEx(
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
    __lib().viOpen(sesn, name, mode, timeout, vi)
}

pub unsafe fn viClose(vi: ViObject) -> ViStatus {
    __lib().viClose(vi)
}

pub unsafe fn viSetAttribute(vi: ViObject, attrName: ViAttr, attrValue: ViAttrState) -> ViStatus {
    __lib().viSetAttribute(vi, attrName, attrValue)
}

pub unsafe fn viGetAttribute(
    vi: ViObject,
    attrName: ViAttr,
    attrValue: *mut ::std::os::raw::c_void,
) -> ViStatus {
    __lib().viGetAttribute(vi, attrName, attrValue)
}

pub unsafe fn viStatusDesc(vi: ViObject, status: ViStatus, desc: *mut ViChar) -> ViStatus {
    __lib().viStatusDesc(vi, status, desc)
}

pub unsafe fn viTerminate(vi: ViObject, degree: ViUInt16, jobId: ViJobId) -> ViStatus {
    __lib().viTerminate(vi, degree, jobId)
}

pub unsafe fn viLock(
    vi: ViSession,
    lockType: ViAccessMode,
    timeout: ViUInt32,
    requestedKey: ViConstKeyId,
    accessKey: *mut ViChar,
) -> ViStatus {
    __lib().viLock(vi, lockType, timeout, requestedKey, accessKey)
}

pub unsafe fn viUnlock(vi: ViSession) -> ViStatus {
    __lib().viUnlock(vi)
}

pub unsafe fn viEnableEvent(
    vi: ViSession,
    eventType: ViEventType,
    mechanism: ViUInt16,
    context: ViEventFilter,
) -> ViStatus {
    __lib().viEnableEvent(vi, eventType, mechanism, context)
}

pub unsafe fn viDisableEvent(
    vi: ViSession,
    eventType: ViEventType,
    mechanism: ViUInt16,
) -> ViStatus {
    __lib().viDisableEvent(vi, eventType, mechanism)
}

pub unsafe fn viDiscardEvents(
    vi: ViSession,
    eventType: ViEventType,
    mechanism: ViUInt16,
) -> ViStatus {
    __lib().viDiscardEvents(vi, eventType, mechanism)
}

pub unsafe fn viWaitOnEvent(
    vi: ViSession,
    inEventType: ViEventType,
    timeout: ViUInt32,
    outEventType: ViPEventType,
    outContext: ViPEvent,
) -> ViStatus {
    __lib().viWaitOnEvent(vi, inEventType, timeout, outEventType, outContext)
}

pub unsafe fn viInstallHandler(
    vi: ViSession,
    eventType: ViEventType,
    handler: ViHndlr,
    userHandle: ViAddr,
) -> ViStatus {
    __lib().viInstallHandler(vi, eventType, handler, userHandle)
}

pub unsafe fn viUninstallHandler(
    vi: ViSession,
    eventType: ViEventType,
    handler: ViHndlr,
    userHandle: ViAddr,
) -> ViStatus {
    __lib().viUninstallHandler(vi, eventType, handler, userHandle)
}

pub unsafe fn viRead(vi: ViSession, buf: ViPBuf, cnt: ViUInt32, retCnt: ViPUInt32) -> ViStatus {
    __lib().viRead(vi, buf, cnt, retCnt)
}

pub unsafe fn viReadAsync(vi: ViSession, buf: ViPBuf, cnt: ViUInt32, jobId: ViPJobId) -> ViStatus {
    __lib().viReadAsync(vi, buf, cnt, jobId)
}

pub unsafe fn viReadToFile(
    vi: ViSession,
    filename: ViConstString,
    cnt: ViUInt32,
    retCnt: ViPUInt32,
) -> ViStatus {
    __lib().viReadToFile(vi, filename, cnt, retCnt)
}

pub unsafe fn viWrite(
    vi: ViSession,
    buf: ViConstBuf,
    cnt: ViUInt32,
    retCnt: ViPUInt32,
) -> ViStatus {
    __lib().viWrite(vi, buf, cnt, retCnt)
}

pub unsafe fn viWriteAsync(
    vi: ViSession,
    buf: ViConstBuf,
    cnt: ViUInt32,
    jobId: ViPJobId,
) -> ViStatus {
    __lib().viWriteAsync(vi, buf, cnt, jobId)
}

pub unsafe fn viWriteFromFile(
    vi: ViSession,
    filename: ViConstString,
    cnt: ViUInt32,
    retCnt: ViPUInt32,
) -> ViStatus {
    __lib().viWriteFromFile(vi, filename, cnt, retCnt)
}

pub unsafe fn viAssertTrigger(vi: ViSession, protocol: ViUInt16) -> ViStatus {
    __lib().viAssertTrigger(vi, protocol)
}

pub unsafe fn viReadSTB(vi: ViSession, status: ViPUInt16) -> ViStatus {
    __lib().viReadSTB(vi, status)
}

pub unsafe fn viClear(vi: ViSession) -> ViStatus {
    __lib().viClear(vi)
}

pub unsafe fn viSetBuf(vi: ViSession, mask: ViUInt16, size: ViUInt32) -> ViStatus {
    __lib().viSetBuf(vi, mask, size)
}

pub unsafe fn viFlush(vi: ViSession, mask: ViUInt16) -> ViStatus {
    __lib().viFlush(vi, mask)
}

pub unsafe fn viBufWrite(
    vi: ViSession,
    buf: ViConstBuf,
    cnt: ViUInt32,
    retCnt: ViPUInt32,
) -> ViStatus {
    __lib().viBufWrite(vi, buf, cnt, retCnt)
}

pub unsafe fn viBufRead(vi: ViSession, buf: ViPBuf, cnt: ViUInt32, retCnt: ViPUInt32) -> ViStatus {
    __lib().viBufRead(vi, buf, cnt, retCnt)
}

#[allow(unused_mut)]
pub unsafe fn viVPrintf(vi: ViSession, writeFmt: ViConstString, mut params: ViVAList) -> ViStatus {
    __lib().viVPrintf(vi, writeFmt, va_list_arg!(params))
}

#[allow(unused_mut)]
pub unsafe fn viVSPrintf(
    vi: ViSession,
    buf: ViPBuf,
    writeFmt: ViConstString,
    mut parms: ViVAList,
) -> ViStatus {
    __lib().viVSPrintf(vi, buf, writeFmt, va_list_arg!(parms))
}

#[allow(unused_mut)]
pub unsafe fn viVScanf(vi: ViSession, readFmt: ViConstString, mut params: ViVAList) -> ViStatus {
    __lib().viVScanf(vi, readFmt, va_list_arg!(params))
}

#[allow(unused_mut)]
pub unsafe fn viVSScanf(
    vi: ViSession,
    buf: ViConstBuf,
    readFmt: ViConstString,
    mut parms: ViVAList,
) -> ViStatus {
    __lib().viVSScanf(vi, buf, readFmt, va_list_arg!(parms))
}

#[allow(unused_mut)]
pub unsafe fn viVQueryf(
    vi: ViSession,
    writeFmt: ViConstString,
    readFmt: ViConstString,
    mut params: ViVAList,
) -> ViStatus {
    __lib().viVQueryf(vi, writeFmt, readFmt, va_list_arg!(params))
}

pub unsafe fn viIn8(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val8: ViPUInt8,
) -> ViStatus {
    __lib().viIn8(vi, space, offset, val8)
}

pub unsafe fn viOut8(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val8: ViUInt8,
) -> ViStatus {
    __lib().viOut8(vi, space, offset, val8)
}

pub unsafe fn viIn16(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val16: ViPUInt16,
) -> ViStatus {
    __lib().viIn16(vi, space, offset, val16)
}

pub unsafe fn viOut16(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val16: ViUInt16,
) -> ViStatus {
    __lib().viOut16(vi, space, offset, val16)
}

pub unsafe fn viIn32(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val32: ViPUInt32,
) -> ViStatus {
    __lib().viIn32(vi, space, offset, val32)
}

pub unsafe fn viOut32(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val32: ViUInt32,
) -> ViStatus {
    __lib().viOut32(vi, space, offset, val32)
}

pub unsafe fn viIn64(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val64: ViPUInt64,
) -> ViStatus {
    __lib().viIn64(vi, space, offset, val64)
}

pub unsafe fn viOut64(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    val64: ViUInt64,
) -> ViStatus {
    __lib().viOut64(vi, space, offset, val64)
}

pub unsafe fn viIn8Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val8: ViPUInt8,
) -> ViStatus {
    __lib().viIn8Ex(vi, space, offset, val8)
}

pub unsafe fn viOut8Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val8: ViUInt8,
) -> ViStatus {
    __lib().viOut8Ex(vi, space, offset, val8)
}

pub unsafe fn viIn16Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val16: ViPUInt16,
) -> ViStatus {
    __lib().viIn16Ex(vi, space, offset, val16)
}

pub unsafe fn viOut16Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val16: ViUInt16,
) -> ViStatus {
    __lib().viOut16Ex(vi, space, offset, val16)
}

pub unsafe fn viIn32Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val32: ViPUInt32,
) -> ViStatus {
    __lib().viIn32Ex(vi, space, offset, val32)
}

pub unsafe fn viOut32Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val32: ViUInt32,
) -> ViStatus {
    __lib().viOut32Ex(vi, space, offset, val32)
}

pub unsafe fn viIn64Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val64: ViPUInt64,
) -> ViStatus {
    __lib().viIn64Ex(vi, space, offset, val64)
}

pub unsafe fn viOut64Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    val64: ViUInt64,
) -> ViStatus {
    __lib().viOut64Ex(vi, space, offset, val64)
}

pub unsafe fn viMoveIn8(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf8: ViAUInt8,
) -> ViStatus {
    __lib().viMoveIn8(vi, space, offset, length, buf8)
}

pub unsafe fn viMoveOut8(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf8: ViAUInt8,
) -> ViStatus {
    __lib().viMoveOut8(vi, space, offset, length, buf8)
}

pub unsafe fn viMoveIn16(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf16: ViAUInt16,
) -> ViStatus {
    __lib().viMoveIn16(vi, space, offset, length, buf16)
}

pub unsafe fn viMoveOut16(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf16: ViAUInt16,
) -> ViStatus {
    __lib().viMoveOut16(vi, space, offset, length, buf16)
}

pub unsafe fn viMoveIn32(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf32: ViAUInt32,
) -> ViStatus {
    __lib().viMoveIn32(vi, space, offset, length, buf32)
}

pub unsafe fn viMoveOut32(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf32: ViAUInt32,
) -> ViStatus {
    __lib().viMoveOut32(vi, space, offset, length, buf32)
}

pub unsafe fn viMoveIn64(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf64: ViAUInt64,
) -> ViStatus {
    __lib().viMoveIn64(vi, space, offset, length, buf64)
}

pub unsafe fn viMoveOut64(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress,
    length: ViBusSize,
    buf64: ViAUInt64,
) -> ViStatus {
    __lib().viMoveOut64(vi, space, offset, length, buf64)
}

pub unsafe fn viMoveIn8Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf8: ViAUInt8,
) -> ViStatus {
    __lib().viMoveIn8Ex(vi, space, offset, length, buf8)
}

pub unsafe fn viMoveOut8Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf8: ViAUInt8,
) -> ViStatus {
    __lib().viMoveOut8Ex(vi, space, offset, length, buf8)
}

pub unsafe fn viMoveIn16Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf16: ViAUInt16,
) -> ViStatus {
    __lib().viMoveIn16Ex(vi, space, offset, length, buf16)
}

pub unsafe fn viMoveOut16Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf16: ViAUInt16,
) -> ViStatus {
    __lib().viMoveOut16Ex(vi, space, offset, length, buf16)
}

pub unsafe fn viMoveIn32Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf32: ViAUInt32,
) -> ViStatus {
    __lib().viMoveIn32Ex(vi, space, offset, length, buf32)
}

pub unsafe fn viMoveOut32Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf32: ViAUInt32,
) -> ViStatus {
    __lib().viMoveOut32Ex(vi, space, offset, length, buf32)
}

pub unsafe fn viMoveIn64Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf64: ViAUInt64,
) -> ViStatus {
    __lib().viMoveIn64Ex(vi, space, offset, length, buf64)
}

pub unsafe fn viMoveOut64Ex(
    vi: ViSession,
    space: ViUInt16,
    offset: ViBusAddress64,
    length: ViBusSize,
    buf64: ViAUInt64,
) -> ViStatus {
    __lib().viMoveOut64Ex(vi, space, offset, length, buf64)
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
    __lib().viMove(
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
    __lib().viMoveAsync(
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
    __lib().viMoveEx(
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
    __lib().viMoveAsyncEx(
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
    __lib().viMapAddress(vi, mapSpace, mapOffset, mapSize, access, suggested, address)
}

pub unsafe fn viUnmapAddress(vi: ViSession) -> ViStatus {
    __lib().viUnmapAddress(vi)
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
    __lib().viMapAddressEx(vi, mapSpace, mapOffset, mapSize, access, suggested, address)
}

pub unsafe fn viPeek8(vi: ViSession, address: ViAddr, val8: ViPUInt8) {
    __lib().viPeek8(vi, address, val8)
}

pub unsafe fn viPoke8(vi: ViSession, address: ViAddr, val8: ViUInt8) {
    __lib().viPoke8(vi, address, val8)
}

pub unsafe fn viPeek16(vi: ViSession, address: ViAddr, val16: ViPUInt16) {
    __lib().viPeek16(vi, address, val16)
}

pub unsafe fn viPoke16(vi: ViSession, address: ViAddr, val16: ViUInt16) {
    __lib().viPoke16(vi, address, val16)
}

pub unsafe fn viPeek32(vi: ViSession, address: ViAddr, val32: ViPUInt32) {
    __lib().viPeek32(vi, address, val32)
}

pub unsafe fn viPoke32(vi: ViSession, address: ViAddr, val32: ViUInt32) {
    __lib().viPoke32(vi, address, val32)
}

pub unsafe fn viPeek64(vi: ViSession, address: ViAddr, val64: ViPUInt64) {
    __lib().viPeek64(vi, address, val64)
}

pub unsafe fn viPoke64(vi: ViSession, address: ViAddr, val64: ViUInt64) {
    __lib().viPoke64(vi, address, val64)
}

pub unsafe fn viMemAlloc(vi: ViSession, size: ViBusSize, offset: ViPBusAddress) -> ViStatus {
    __lib().viMemAlloc(vi, size, offset)
}

pub unsafe fn viMemFree(vi: ViSession, offset: ViBusAddress) -> ViStatus {
    __lib().viMemFree(vi, offset)
}

pub unsafe fn viMemAllocEx(vi: ViSession, size: ViBusSize, offset: ViPBusAddress64) -> ViStatus {
    __lib().viMemAllocEx(vi, size, offset)
}

pub unsafe fn viMemFreeEx(vi: ViSession, offset: ViBusAddress64) -> ViStatus {
    __lib().viMemFreeEx(vi, offset)
}

pub unsafe fn viGpibControlREN(vi: ViSession, mode: ViUInt16) -> ViStatus {
    __lib().viGpibControlREN(vi, mode)
}

pub unsafe fn viGpibControlATN(vi: ViSession, mode: ViUInt16) -> ViStatus {
    __lib().viGpibControlATN(vi, mode)
}

pub unsafe fn viGpibSendIFC(vi: ViSession) -> ViStatus {
    __lib().viGpibSendIFC(vi)
}

pub unsafe fn viGpibCommand(
    vi: ViSession,
    cmd: ViConstBuf,
    cnt: ViUInt32,
    retCnt: ViPUInt32,
) -> ViStatus {
    __lib().viGpibCommand(vi, cmd, cnt, retCnt)
}

pub unsafe fn viGpibPassControl(vi: ViSession, primAddr: ViUInt16, secAddr: ViUInt16) -> ViStatus {
    __lib().viGpibPassControl(vi, primAddr, secAddr)
}

pub unsafe fn viVxiCommandQuery(
    vi: ViSession,
    mode: ViUInt16,
    cmd: ViUInt32,
    response: ViPUInt32,
) -> ViStatus {
    __lib().viVxiCommandQuery(vi, mode, cmd, response)
}

pub unsafe fn viAssertUtilSignal(vi: ViSession, line: ViUInt16) -> ViStatus {
    __lib().viAssertUtilSignal(vi, line)
}

pub unsafe fn viAssertIntrSignal(vi: ViSession, mode: ViInt16, statusID: ViUInt32) -> ViStatus {
    __lib().viAssertIntrSignal(vi, mode, statusID)
}

pub unsafe fn viMapTrigger(
    vi: ViSession,
    trigSrc: ViInt16,
    trigDest: ViInt16,
    mode: ViUInt16,
) -> ViStatus {
    __lib().viMapTrigger(vi, trigSrc, trigDest, mode)
}

pub unsafe fn viUnmapTrigger(vi: ViSession, trigSrc: ViInt16, trigDest: ViInt16) -> ViStatus {
    __lib().viUnmapTrigger(vi, trigSrc, trigDest)
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
    __lib().viUsbControlOut(vi, bmRequestType, bRequest, wValue, wIndex, wLength, buf)
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
    __lib().viUsbControlIn(
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
    __lib().viPxiReserveTriggers(vi, cnt, trigBuses, trigLines, failureIndex)
}
