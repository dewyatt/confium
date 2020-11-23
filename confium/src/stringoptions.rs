use std::collections::HashMap;
use std::os::raw::c_char;

use crate::error::Error;

type StringOptions = HashMap<String, String>;

#[no_mangle]
pub extern "C" fn cfm_sopts_create(obj: *mut *mut StringOptions) -> u32 {
    unsafe {
        *obj = Box::into_raw(Box::new(StringOptions::new()));
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_sopts_destroy(obj: *mut StringOptions) -> u32 {
    unsafe {
        Box::from_raw(obj);
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_sopts_clear(obj: *mut StringOptions) -> u32 {
    unsafe {
        (*obj).clear();
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_sopts_set(
    obj: *mut StringOptions,
    c_key: *const c_char,
    c_value: *const c_char,
) -> u32 {
    unsafe {
        let key = cstring!(c_key).to_string();
        let value = cstring!(c_value).to_string();
        (*obj).insert(key, value);
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_sopts_unset(obj: *mut StringOptions, c_key: *const c_char) -> u32 {
    unsafe {
        let key = cstring!(c_key);
        (*obj).remove(key);
    }
    0
}
