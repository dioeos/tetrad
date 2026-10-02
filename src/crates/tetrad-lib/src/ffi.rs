use std::os::raw::c_char;

/// Returns a borrowed pointer to a static, NUL-terminated string.
/// The caller must not modify or free it.
#[unsafe(no_mangle)]
pub extern "C" fn tetrad_init() -> *const c_char {
    c"Hello, World!".as_ptr()
}
