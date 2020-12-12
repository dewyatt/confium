#[macro_escape]
macro_rules! cstring {
    ($str:ident) => {{
        if ($str.is_null()) {
            return Error {
                kind: $crate::error::ErrorKind::NullPointer {},
                source: None,
            }
            .into();
        }
        match std::ffi::CStr::from_ptr($str).to_str() {
            Ok(s) => s,
            Err(_) => {
                return Error {
                    kind: $crate::error::ErrorKind::InvalidUTF8 {},
                    source: None,
                }
                .into();
            }
        }
    }};
}
