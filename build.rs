// Names the .so's ABI generation for the dynamic linker: `omarchy_lib_theme::api::ffi` is the C
// contract this identifies, and its layout is what decides when this number moves.
//
// `rustc-cdylib-link-arg` reaches only the cdylib output — the staticlib and rlib this crate also
// produces carry no SONAME to set.
fn main() {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,libomarchy_lib_theme.so.1");
}
