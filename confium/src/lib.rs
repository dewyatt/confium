extern crate libloading;
#[macro_use]
pub extern crate slog;
extern crate slog_async;
extern crate slog_stdlog;
extern crate slog_term;

#[macro_use]
pub mod utils;
pub mod error;
pub mod hash;
pub mod stringoptions;

use std::ffi::CString;
use std::os::raw::c_char;
use std::rc::Rc;

use libloading::Library;
use slog::Drain;

use error::{Error, ErrorKind};
use hash::Hash;

type BoxError = Box<dyn std::error::Error>;

type HashCreateFn = extern "C" fn(ffi: *mut FFI, *const c_char, *mut *mut Hash) -> u32;
type HashDestroyFn = extern "C" fn(ffi: *mut FFI, *mut Hash) -> u32;

pub struct HashPlugin {
    create: HashCreateFn,
    destroy: HashDestroyFn,
}

pub struct FFI {
    libraries: Vec<Rc<Library>>,
    logger: slog::Logger,
}

impl FFI {
    pub fn new<L: Into<Option<slog::Logger>>>(logger: L) -> Self {
        FFI {
            libraries: Vec::new(),
            logger: logger
                .into()
                .unwrap_or(slog::Logger::root(slog_stdlog::StdLog.fuse(), o!())),
        }
    }
}

#[no_mangle]
pub extern "C" fn cfm_create(ffi: *mut *mut FFI) -> u32 {
    unsafe {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let log = slog::Logger::root(drain, o!());
        *ffi = Box::into_raw(Box::new(FFI::new(log)));
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_destroy(ffi: *mut FFI) -> u32 {
    unsafe {
        Box::from_raw(ffi);
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_load_plugin(ffi: *mut FFI, c_path: *const c_char) -> u32 {
    if ffi.is_null() || c_path.is_null() {
        return Error {
            kind: ErrorKind::NullPointer {},
            source: None,
        }
        .into();
    }
    let path = unsafe { cstring!(c_path) };
    let lib = Rc::new(match Library::new(path) {
        Ok(l) => l,
        Err(e) => {
            unsafe {
                error!((*ffi).logger, "Failed to load plugin: {}", e);
            }
            return Error {
                kind: ErrorKind::PluginLoadError {},
                source: None,
            }
            .into();
        }
    });
    unsafe {
        let namefn = lib
            .get::<fn() -> *const c_char>(b"cfm_plugin_name\0")
            .unwrap();
        let name = namefn();
        println!("Plugin name: '{}'", cstring!(name));
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_str_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        CString::from_raw(s);
    }
}

#[no_mangle]
pub extern "C" fn cfm_err_get_msg(e: *const Error, msg: *mut *mut c_char) {
    if e.is_null() || msg.is_null() {
        return;
    }
    unsafe {
        *msg = std::ptr::null_mut();
        match CString::new((*e).to_string()) {
            Ok(s) => *msg = s.into_raw(),
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn cfm_err_get_code(e: *const Error, code: *mut u32) {
    if e.is_null() || code.is_null() {
        return;
    }
    unsafe {
        *code = (&*e).into();
    }
}

#[no_mangle]
pub extern "C" fn cfm_err_get_source(e: *const Error, src: *mut *const Error) {
    if e.is_null() || src.is_null() {
        return;
    }
    unsafe {
        /*
        if let Some(&source) = (*e).source {
            *src = *Box::into_raw(source); // TODO
        } else {
            *src = std::ptr::null_mut();
        }
        */
    }
}

/*
void cfm_err_get_msg(cfm_err_t* err, char *msg);
void cfm_err_get_code(cfm_err_t* err, uint32_t *code);
void cfm_err_get_source(cfm_err_t *err, cfm_err_t *src);
void cfm_err_get_backtrace(cfm_err_t *err, char *backtrace);

*/
