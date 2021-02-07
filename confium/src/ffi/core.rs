use std::ffi::CString;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn cfm_str_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        CString::from_raw(s);
    }
}
