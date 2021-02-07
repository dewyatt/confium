use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_void;

use crate::error::{Error, ErrorCode, ErrorKind};

#[no_mangle]
pub extern "C" fn cfm_err_get_msg(err: *mut c_void, msg: *mut *mut c_char) -> u32 {
    if err.is_null() || msg.is_null() {
        return Error {
            kind: ErrorKind::NullPointer {},
            source: None,
            backtrace: None,
        }
        .into();
    }
    let err = err as *mut Box<dyn std::error::Error>;
    unsafe {
        *msg = std::ptr::null_mut();
        //err.as_ref().unwrap().to_string();
        match CString::new((*err).to_string()) {
            Ok(s) => *msg = s.into_raw(),
            Err(e) => {
                panic!("fail");
                eprintln!("{:?}", e); // TODO: log
            }
        }
        //*msg = CString::new("test").unwrap().into_raw();
        //println!("errstr: {}", (*err).to_string());
        //println!("{}", err.as_ref().unwrap());
        //err.as_ref().unwrap();
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_err_get_code(err: *const c_void, code: *mut u32) -> u32 {
    if err.is_null() || code.is_null() {
        return Error {
            kind: ErrorKind::NullPointer {},
            source: None,
            backtrace: None,
        }
        .into();
    }
    let err = err as *mut Box<dyn std::error::Error>;
    unsafe {
        println!("cfm_err_get_code {:p}", err);
        //std::error::Error::downcast_ref(&*err);
        //(&*err).type_id();
        match (*err).downcast_ref::<Box<Error>>() {
            Some(e) => {
                *code = e.code() as u32;
            }
            None => {
                // TODO log
                eprintln!("Failed to downcast");
                *code = ErrorCode::UNKNOWN as u32;
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_err_get_source(err: *const c_void, src: *mut *const c_void) -> u32 {
    if err.is_null() || src.is_null() {
        return Error {
            kind: ErrorKind::NullPointer {},
            source: None,
            backtrace: None,
        }
        .into();
    }
    let err = err as *mut Box<dyn std::error::Error>;
    unsafe {
        if let Some(source) = (*err).source() {
            *src = Box::into_raw(Box::new(source)) as *mut c_void;
        } else {
            *src = std::ptr::null_mut();
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_err_destroy(err: *mut c_void) {
    if err.is_null() {
        return;
    }
    let err = err as *mut Box<dyn std::error::Error>;
    unsafe {
        Box::from_raw(err);
    }
}

#[no_mangle]
pub extern "C" fn cfm_err_get_backtrace(err: *mut c_void, backtrace: *mut *const c_char) -> u32 {
    if err.is_null() {
        return Error {
            kind: ErrorKind::NullPointer {},
            source: None,
            backtrace: None,
        }
        .into();
    }
    let err = err as *mut Box<dyn std::error::Error>;
    unsafe {
        match (*err).backtrace() {
            Some(bt) => match CString::new(bt.to_string()) {
                Ok(bt) => {
                    *backtrace = bt.into_raw();
                }
                Err(e) => {
                    // TODO
                    panic!("shouldn't be possible");
                }
            },
            None => {
                println!("No backtrace captured");
                *backtrace = std::ptr::null_mut();
            }
        }
    }
    0
}

/*
void cfm_err_get_msg(cfm_err_t* err, char *msg);
void cfm_err_get_code(cfm_err_t* err, uint32_t *code);
void cfm_err_get_source(cfm_err_t *err, cfm_err_t *src);
void cfm_err_get_backtrace(cfm_err_t *err, char *backtrace);

*/
