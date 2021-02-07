use std::os::raw::c_char;

pub struct Hash {}

impl Hash {
    pub fn new() -> Hash {
        Hash {}
    }
}

impl Drop for Hash {
    fn drop(&mut self) {}
}

pub type HashCreateFn = extern "C" fn(*const c_char, *mut *mut Hash) -> u32;
