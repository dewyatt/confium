#![feature(str_split_once)]
#![feature(backtrace)]
#![feature(int_error_matching)]
#![feature(core_intrinsics)]
//TODO: remove
extern crate libloading;
#[macro_use]
pub extern crate slog;
extern crate slog_async;
extern crate slog_stdlog;
extern crate slog_term;

#[macro_use]
pub mod utils;
pub mod error;
pub mod ffi;
pub mod hash;
pub mod stringoptions;

use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs;
use std::io;
use std::io::BufRead;
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::rc::Rc;

use libloading::Library;
use slog::Drain;

use error::{Error, ErrorKind};

//type BoxError = Box<dyn std::error::Error>;

pub struct FFI {
    #[allow(dead_code)] // TODO
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
pub extern "C" fn cfm_create(ffi: *mut *mut c_void) -> u32 {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());
    unsafe {
        *ffi = Box::into_raw(Box::new(FFI::new(log))) as *mut c_void;
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_destroy(ffi: *mut c_void) -> u32 {
    if ffi.is_null() {
        return Error {
            kind: ErrorKind::NullPointer {},
            ..Error::default()
        }
        .into();
    }
    unsafe {
        Box::from_raw(ffi as *mut FFI);
    }
    0
}

#[no_mangle]
pub extern "C" fn cfm_load_plugin(ffi: *mut c_void, c_path: *const c_char) -> u32 {
    if ffi.is_null() || c_path.is_null() {
        return Error {
            kind: ErrorKind::NullPointer {},
            source: None,
            backtrace: None,
        }
        .into();
    }
    let ffi = ffi as *mut FFI;
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
                backtrace: None,
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
pub extern "C" fn cfm_err_get_sample(err: *mut *mut c_void) {
    unsafe {
        /*
        *err = Box::into_raw(Box::new(Box::new(Error {
            kind: ErrorKind::PluginLoadError {},
            source: Some(Box::new(Error {
                kind: ErrorKind::InvalidHexDigit('Z'),
                source: None,
            })),
        }))) as *mut c_void;
        println!("*err: {:#?}", *err);
        */
        /*
        let mut e = Error {
            kind: ErrorKind::PluginLoadError {},
            source: Some(Box::new(Error {
                kind: ErrorKind::InvalidHexDigit('Z'),
                source: None,
            })),
        };
        */
        // trait object
        let boxe: Box<dyn std::error::Error> = Box::new(Error {
            kind: ErrorKind::PluginLoadError {},
            source: Some(Box::new(Error {
                backtrace: None,
                kind: ErrorKind::InvalidHexDigit('Z'),
                source: Some(Box::new(Error {
                    kind: ErrorKind::NullPointer {},
                    source: None,
                    backtrace: None,
                })),
            })),
            backtrace: Some(Backtrace::capture()),
        });
        // thin pointer
        let boxe = Box::new(boxe);
        // raw pointer
        *err = Box::into_raw(boxe) as *mut c_void;
        //*err = Box::into_raw(boxe) as *mut c_void;
        //*err = Box::into_raw(Box::new(boxe)) as *mut c_void;
    }
}

fn parsehex(s: &str) -> Result<u8, Error> {
    if s.is_empty() {
        return Err(Error {
            kind: ErrorKind::InvalidFormat {},
            source: None,
            backtrace: Some(Backtrace::capture()),
        });
    }
    let mut result: u8 = 0;
    for (idx, ch) in s.trim_start_matches("0x").char_indices() {
        let x = match ch.to_digit(16) {
            Some(x) => x,
            None => {
                return Err(Error {
                    kind: ErrorKind::InvalidHexDigit(ch),
                    source: None,
                    backtrace: Some(Backtrace::capture()),
                })
            }
        } as u8;
        result = match result.checked_mul(16) {
            Some(result) => result,
            None => {
                return Err(Error {
                    kind: ErrorKind::Overflow {},
                    source: None,
                    backtrace: Some(Backtrace::capture()),
                })
            }
        };
        result = match result.checked_add(x) {
            Some(result) => result,
            None => {
                return Err(Error {
                    kind: ErrorKind::Overflow {},
                    source: None,
                    backtrace: Some(Backtrace::capture()),
                })
            }
        };
    }
    Ok(result)
}

type OptionsMap = HashMap<String, u8>;

fn load_config(path: &str) -> Result<OptionsMap, Error> {
    let file = fs::File::open(path).ioerr(
        ErrorKind::InvalidConfig { linenum: None },
        Some(path.to_string()),
    )?;
    let reader = io::BufReader::new(file);
    let mut options: OptionsMap = HashMap::new();
    for (linenum, line) in reader.lines().enumerate() {
        let line = line.ioerr(
            ErrorKind::InvalidConfig {
                linenum: Some(linenum + 1),
            },
            Some(path.to_string()),
        )?;
        if line.is_empty() {
            continue;
        }
        let option = line.split_once('=').ok_or(Error {
            kind: ErrorKind::InvalidConfig {
                linenum: Some(linenum + 1),
            },
            source: Some(
                Error {
                    kind: ErrorKind::ExpectedToken('='),
                    source: None,
                    backtrace: None,
                }
                .into(),
            ),
            backtrace: Some(Backtrace::capture()),
        })?;
        options.insert(
            option.0.trim().to_string(),
            parsehex(option.1.trim()).map_err(|e| Error {
                kind: ErrorKind::InvalidConfig {
                    linenum: Some(linenum + 1),
                },
                source: Some(e.into()),
                backtrace: Some(Backtrace::capture()),
            })?,
        );
    }
    Ok(options)
}

fn initialize(cfg_path: &str) -> Result<OptionsMap, Error> {
    Ok(load_config(cfg_path).map_err(|e| Error {
        kind: ErrorKind::InitializationFailure {},
        source: Some(e.into()),
        backtrace: Some(Backtrace::capture()),
    })?)
}
pub trait ResultExtIO<T> {
    fn ioerr(self, kind: ErrorKind, path: Option<String>) -> Result<T, Error>;
}

impl<T> ResultExtIO<T> for std::result::Result<T, std::io::Error> {
    fn ioerr(self, kind: ErrorKind, path: Option<String>) -> Result<T, Error> {
        self.map_err(|e| Error {
            kind: kind,
            source: Some(Box::new(Error {
                kind: ErrorKind::Io {
                    path: path,
                    source: e,
                },
                source: None,
                backtrace: None,
            })),
            backtrace: Some(Backtrace::capture()),
        })
    }
}

#[no_mangle]
pub extern "C" fn cfm_app_run(cfgpath: *const c_char, err: *mut *mut c_void) -> u32 {
    if cfgpath.is_null() {
        let e = Error {
            kind: ErrorKind::NullPointer {},
            source: None,
            backtrace: Some(Backtrace::capture()),
        };
        let ret = e.code() as u32;
        if !err.is_null() {
            unsafe {
                *err = Box::into_raw(Box::new(e)) as *mut c_void;
            }
        }
        return ret;
    }
    unsafe {
        match initialize(CStr::from_ptr(cfgpath).to_str().unwrap()) {
            Ok(_) => {}
            Err(e) => {
                let ret = e.code() as u32;
                if !err.is_null() {
                    *err = Box::into_raw(Box::new(e)) as *mut c_void;
                }
                return ret;
            }
        }
    }
    0
}
