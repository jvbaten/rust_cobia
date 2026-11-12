# Rust binding for COBIA

(C) Jasper, AmsterCHEM 2026

This workspace implements rust bindings for COBIA.

CAPE-OPEN consists of a series of specifications to expand the range of application of process simulation technologies. 
The CAPE-OPEN specifications specify a set of software interfaces that allow plug and play inter-operability between
a given process modelling environment (PME) and a third-party process modelling component (PMC).

The CAPE-OPEN specifications are supported by the non-profit organization CO-LaN: http://www.colan.org

The COBIA middle-ware is a platform independent object model and request broker implementation to
facilitate the inter-operation of CAPE-OPEN compliant PMEs and PMCs.

# License

This project is provided under the MIT license. See LICENSE for more details.

# Prerequisites

To build the COBIA crate, the following must be installed on your computer:

* The COBIA SDK (available for download from [https://colan.repositoryhosting.com/trac/colan_cobia/downloads](https://colan.repositoryhosting.com/trac/colan_cobia/downloads) 
* The required ingredients for bindgen, including a valid CLANG installation, see [https://rust-lang.github.io/rust-bindgen/requirements.html](https://rust-lang.github.io/rust-bindgen/requirements.html)

# Cross-compiling

The build script reads these environment variables in addition to `COBIA_INCLUDE`:

* `COBIA_CC`, `COBIA_AR`: C compiler and archiver used to build the C binding for the target. Defaults: `clang` and `ar`, or the tools in `LIBCLANG_PATH` for Windows targets.
* `COBIA_SKIP_CODEGEN`: when set (to anything but `0`), the committed `src/cobia_enums.rs`, `src/cape_open/mod.rs` and `src/cape_open_1_2/mod.rs` are used as they are instead of being regenerated. Regenerating builds and runs the `cidl2rs` host tool against an installed COBIA runtime, which cannot be done when cross-compiling.

For example, to build for 64-bit Windows from Linux or macOS with MinGW-w64, put this in `.cargo/config.toml`:

```toml
[env]
COBIA_INCLUDE = "/path/to/COBIA/CLIENT"
COBIA_SKIP_CODEGEN = "1"
COBIA_CC = "x86_64-w64-mingw32-gcc"
COBIA_AR = "x86_64-w64-mingw32-ar"

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
rustflags = ["-C", "link-args=-ladvapi32 -lole32 -lshell32 -lshlwapi -loleaut32"]
```

and point bindgen at the target and its headers:

```sh
export BINDGEN_EXTRA_CLANG_ARGS="--target=x86_64-pc-windows-gnu -isystem /usr/x86_64-w64-mingw32/include"
cargo build --target x86_64-pc-windows-gnu --release
```

For 32-bit Windows, use `i686-pc-windows-gnu` and the `i686-w64-mingw32-` tools. On a case-sensitive file system the MinGW headers are lower case, while the COBIA headers include `Shlobj.h` and `Shlwapi.h`; symbolic links `Shlobj.h -> shlobj.h` and `Shlwapi.h -> shlwapi.h` in the MinGW include directory solve this.


