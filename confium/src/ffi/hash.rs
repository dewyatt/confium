use std::os::raw::c_char;

use crate::hash::Hash;
//use crate::FFI;

//type HashCreateFn = extern "C" fn(ffi: *mut FFI, *const c_char, *mut *mut Hash) -> u32;
//type HashDestroyFn = extern "C" fn(ffi: *mut FFI, *mut Hash) -> u32;

/*
pub struct HashPlugin {
    create: HashCreateFn,
    destroy: HashDestroyFn,
}
*/

#[no_mangle]
pub extern "C" fn cfm_hash_create(_c_name: *const c_char, obj: *mut *mut Hash) -> u32 {
    unsafe {
        *obj = Box::into_raw(Box::new(Hash::new()));
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_hash_destroy(obj: *mut Hash) -> u32 {
    unsafe {
        Box::from_raw(obj);
    }
    0
}
