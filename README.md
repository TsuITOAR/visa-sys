# visa-sys

[![crates.io](https://img.shields.io/crates/v/visa-sys.svg)](https://crates.io/crates/visa-sys)
[![docs.rs](https://docs.rs/visa-sys/badge.svg)](https://docs.rs/visa-sys)
[![license](https://img.shields.io/crates/l/visa-sys.svg)](#license)

FFI bindings to the [VISA](https://www.ivifoundation.org/) (Virtual Instrument
Software Architecture) library.

The crate ships pre-generated bindings, so a C toolchain / `libclang` is **not**
required for a normal build. The VISA runtime itself (NI-VISA, Keysight, R&S, …)
must be available on the target machine.

## Linking modes

### Default: link at build time

By default the crate links the system VISA library at build time. The library
name and search path are chosen per platform and can be overridden with
environment variables:

| Variable             | Purpose                                            |
| -------------------- | -------------------------------------------------- |
| `LIB_VISA_NAME`      | Override the library name passed to the linker.    |
| `LIB_VISA_PATH`      | Add a library search path (a framework on macOS).  |
| `INCLUDE_VISA_PATH`  | Header directory used when regenerating with `bindgen`. |

Enable the `bindgen` feature to regenerate the bindings from `visa.h` at build
time instead of using the checked-in ones.

### `dynamic_load`: load at run time

```toml
[dependencies]
visa-sys = { version = "0.1", features = ["dynamic_load"] }
```

With the `dynamic_load` feature the VISA shared library is **not** linked at
build time. Instead it is opened at run time with [`libloading`], so the same
binary can run on machines with or without VISA installed. The free functions
(`viOpenDefaultRM`, `viOpen`, …) keep the same signatures as the linked build, so
switching modes needs no code changes.

```rust
use visa_sys::*;

// The library auto-loads from the platform default path on the first call:
let mut session = 0;
let status = unsafe { viOpenDefaultRM(&mut session as ViPSession) };

// …or load it explicitly to handle a missing runtime without panicking:
if let Err(e) = load_visa_library() {
    eprintln!("VISA not available: {e}");
}
// A custom path is also supported:
// load_visa_library_from_path("/opt/visa/libvisa.so")?;
```

Default search names: `VISA.framework/VISA` (macOS), `visa64.dll` / `visa32.dll`
(Windows), `libvisa.so` (other).

#### Behaviour and limitations

- **Auto-load panics on a missing library.** A VISA call auto-loads on first use
  and panics if the default path cannot be opened — analogous to a linked build
  failing to start. Call `load_visa_library()` first to detect this gracefully.
- **Tolerant of partial implementations.** Symbols are resolved lazily; a VISA
  runtime that omits a rarely-used function still works for the functions it does
  export. Calling an unsupported function panics at the call site.
- **No variadic helpers.** `viPrintf`, `viSPrintf`, `viScanf`, `viSScanf` and
  `viQueryf` are unavailable as free functions (a C-variadic function cannot be
  called through a function pointer). Use the explicit-`va_list` variants
  `viVPrintf`, `viVSPrintf`, `viVScanf`, `viVSScanf`, `viVQueryf`.

[`libloading`]: https://crates.io/crates/libloading

## License

Licensed under either of MIT or Apache-2.0 at your option.
