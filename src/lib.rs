//! A [`Cargo` build script](http://doc.crates.io/build-script.html) library to handle compilation and inclusion of Windows
//! resources in the most resilient fashion imaginable
//!
//! # Background
//!
//! Including Windows resources seems very easy at first, despite the build scripts' abhorrent documentation:
//! [compile with `windres`, then make linkable with
//! `ar`](https://github.com/nabijaczleweli/cargo-update/commit/ef4346c#diff-a7b0a2dee0126cddf994326e705a91ea).
//!
//! I was very happy with that solution until it was brought to my attention, that [MSVC uses something
//! different](https://github.com/nabijaczleweli/cargo-update/commit/f57e9c3#diff-a7b0a2dee0126cddf994326e705a91ea),
//! and now either `windres`-`ar` combo or `RC.EXE` would be used, which was OK.
//!
//! Later it transpired, that [MSVC is even more incompatible with everything
//! else](https://github.com/nabijaczleweli/cargo-update/commit/39fa758#diff-a7b0a2dee0126cddf994326e705a91ea)
//! by way of not having `RC.EXE` in `$PATH` (because it would only be reasonable to do so),
//! so another MSVC artisan made the script [find the most likely places for `RC.EXE` to
//! be](https://github.com/nabijaczleweli/cargo-update/pull/22), and the script grew yet again,
//! now standing at 100 lines and 3.2 kB.
//!
//! After [copying the build script in its
//! entirety](https://github.com/thecoshman/http/commit/98205a4#diff-a7b0a2dee0126cddf994326e705a91ea)
//! and realising how error-prone that was, then being [nudged by
//! Shepmaster](https://chat.stackoverflow.com/transcript/message/35378953#35378953)
//! to extract it to a crate, here we are.
//!
//! # Usage
//!
//! For the purposes of the demonstration we will assume that the resource file's name
//! is "checksums.rc", but it can be any name relative to the crate root.
//!
//! In `Cargo.toml`:
//!
//! ```toml
//! # The general section with crate name, license, etc.
//! build = "build.rs"
//!
//! [build-dependencies]
//! embed-resource = "1.3"
//! ```
//!
//! In `build.rs`:
//!
//! ```rust,no_run
//! extern crate embed_resource;
//!
//! fn main() {
//!     embed_resource::compile("checksums.rc");
//! }
//! ```
//!
//! # Credit
//!
//! In chronological order:
//!
//! [@liigo](https://github.com/liigo) -- persistency in pestering me and investigating problems where I have failed
//!
//! [@mzji](https://github.com/mzji) -- MSVC lab rat
//!
//! [@TheCatPlusPlus](https://github.com/TheCatPlusPlus) -- knowledge and providing first iteration of manifest-embedding code
//!
//! [@azyobuzin](https://github.com/azyobuzin) -- providing code for finding places where RC.EXE could hide
//!
//! [@retep998](https://github.com/retep998) -- fixing MSVC support
//!
//! [@SonnyX](https://github.com/SonnyX) -- Windows cross-compilation support and testing
//!
//! [@MSxDOS](https://github.com/MSxDOS) -- finding and supplying RC.EXE its esoteric header include paths
//!
//! # Special thanks
//!
//! To all who support further development on [Patreon](https://patreon.com/nabijaczleweli), in particular:
//!
//!   * ThePhD


#[cfg(all(target_os = "windows", target_env = "msvc"))]
extern crate vswhom;
#[cfg(all(target_os = "windows", target_env = "msvc"))]
extern crate winreg;

#[cfg(not(target_os = "windows"))]
mod non_windows;
#[cfg(all(target_os = "windows", target_env = "msvc"))]
mod windows_msvc;
#[cfg(all(target_os = "windows", not(target_env = "msvc")))]
mod windows_not_msvc;

#[cfg(not(target_os = "windows"))]
use self::non_windows::*;
#[cfg(all(target_os = "windows", target_env = "msvc"))]
use self::windows_msvc::*;
#[cfg(all(target_os = "windows", not(target_env = "msvc")))]
use self::windows_not_msvc::*;

use std::env;
use std::path::{Path, PathBuf};


/// Compile the Windows resource file and update the cargo search path if we're on Windows.
///
/// On non-Windows non-Windows-cross-compile-target this does nothing, on non-MSVC Windows and Windows cross-compile targets,
/// this chains `windres` with `ar`,
/// but on MSVC Windows, this will try its hardest to find `RC.EXE` in Windows Kits and/or SDK directories,
/// falling back to [Jon Blow's VS discovery script](https://pastebin.com/3YvWQa5c),
/// and on Windows 10 `%INCLUDE%` will be updated to help `RC.EXE` find `windows.h` and friends.
///
/// # Examples
///
/// In your build script, assuming the crate's name is "checksums":
///
/// ```rust,no_run
/// extern crate embed_resource;
///
/// fn main() {
///     // Compile and link checksums.rc
///     embed_resource::compile("checksums.rc");
/// }
/// ```
#[inline]
pub fn compile<T: AsRef<Path>>(resource_file: T) {
    compile_impl(resource_file.as_ref())
}

fn compile_impl(resource_file: &Path) {
    let comp = ResourceCompiler::new();
    if comp.is_supported() {
        let prefix = &resource_file.file_stem().expect("resource_file has no stem").to_str().expect("resource_file's stem not UTF-8");
        let out_dir = env::var("OUT_DIR").expect("No OUT_DIR env var");

        comp.compile_resource(&out_dir, &prefix, resource_file.to_str().expect("resource_file not UTF-8"));
        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=dylib={}", prefix);
    }
}

/// Find build tools other than the compiler and linker.
///
/// This will try its hardest to find tools such as `MIDL.EXE` in Windows Kits and/or SDK directories.
/// The compilers and linkers can be better found with the `cc` or `vswhom` crates.
pub fn find_windows_sdk_tool<T: AsRef<str>>(tool: T) -> Option<PathBuf> {
    find_windows_sdk_tool_impl(tool.as_ref())
}