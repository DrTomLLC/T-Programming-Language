//! T-Lang standard library: exposes `tlang_print` and `tlang_println` for backends.

/// Print a UTF-8 string slice without a trailing newline.
/// Backends call this via the FFI or link directly.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tlang_print(ptr: *const u8, len: usize) {
    // Safety: assume backends pass a valid UTF-8 pointer+length
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    if let Ok(s) = std::str::from_utf8(bytes) {
        print!("{}", s);
    }
}

/// Print a UTF-8 string slice with a trailing newline.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tlang_println(ptr: *const u8, len: usize) {
    // Safety: We trust that the caller provides a valid UTF-8 pointer and length
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    if let Ok(s) = std::str::from_utf8(bytes) {
        println!("{}", s);
    }
}
