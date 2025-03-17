use std::{
    ffi::{c_char, c_int},
    num::NonZero,
};

use sqlite3ext_sys::*;

use crate::not_ok_return;

#[cfg(feature = "jieba")]
mod jieba;
mod simple;

#[cfg(feature = "jieba")]
pub use jieba::Jieba;
pub use simple::Simple;

macro_rules! err_return {
    ($result: expr) => {
        match $result {
            Ok(value) => value,
            Err(rc) => return rc.into(),
        }
    };

    ($result: expr, $error: expr) => {
        match $result {
            Ok(value) => value,
            Err(_) => return $error,
        }
    };
}

trait TokenizerImpl: Sized {
    fn new(args: &[&[u8]]) -> Result<Self, NonZero<i32>>;
    /// Returned tokens must be sorted by start position.
    ///
    /// Colocated tokens must right after the token they are colocated with.
    fn tokenize(&self, text: &str, flags: i32) -> Result<Vec<Token>, NonZero<i32>>;

    unsafe extern "C" fn c_create(
        _arg1: *mut ::std::os::raw::c_void,
        az_arg: *mut *const ::std::os::raw::c_char,
        n_arg: ::std::os::raw::c_int,
        pp_out: *mut *mut Fts5Tokenizer,
    ) -> ::std::os::raw::c_int {
        let mut args = Vec::with_capacity(n_arg as usize);
        for i in 0..n_arg as isize {
            let ptr = unsafe { *az_arg.offset(i) };
            let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
            args.push(c_str.to_bytes());
        }

        let tokenizer = match Self::new(&args) {
            Ok(t) => t,
            Err(rc) => return rc.get() as c_int,
        };

        let tokenizer = Box::into_raw(Box::new(tokenizer));
        unsafe {
            *pp_out = tokenizer as *mut Fts5Tokenizer;
        }

        SQLITE_OK as c_int
    }

    unsafe extern "C" fn c_delete(ptr: *mut Fts5Tokenizer) {
        let _ = unsafe { Box::from_raw(ptr as *mut Self) };
    }

    unsafe extern "C" fn c_tokenize(
        tk_ptr: *mut Fts5Tokenizer,
        p_ctx: *mut ::std::os::raw::c_void,
        flags: ::std::os::raw::c_int,
        p_text: *const ::std::os::raw::c_char,
        n_text: ::std::os::raw::c_int,
        x_token: ::std::option::Option<
            unsafe extern "C" fn(
                p_ctx: *mut ::std::os::raw::c_void,
                tflags: ::std::os::raw::c_int,
                p_token: *const ::std::os::raw::c_char,
                n_token: ::std::os::raw::c_int,
                i_start: ::std::os::raw::c_int,
                i_end: ::std::os::raw::c_int,
            ) -> ::std::os::raw::c_int,
        >,
    ) -> ::std::os::raw::c_int {
        let tokenizer = unsafe { &*(tk_ptr as *const Self) };

        let text = unsafe { std::slice::from_raw_parts(p_text as *const u8, n_text as usize) };
        let text = err_return!(std::str::from_utf8(text), SQLITE_ERROR as c_int);

        let tokens = err_return!(tokenizer.tokenize(text, flags));

        for token in tokens {
            let text = token.text.as_bytes();
            unsafe {
                not_ok_return!(x_token.unwrap()(
                    p_ctx,
                    if token.colocated {
                        FTS5_TOKEN_COLOCATED as c_int
                    } else {
                        0
                    },
                    text.as_ptr() as *const c_char,
                    text.len() as c_int,
                    token.start as c_int,
                    token.end as c_int,
                ))
            };
        }

        SQLITE_OK as c_int
    }
}

pub trait GetTokenizer {
    fn get_tokenizer() -> *mut fts5_tokenizer;
}

impl<T: TokenizerImpl> GetTokenizer for T {
    fn get_tokenizer() -> *mut fts5_tokenizer {
        let tokenizer = fts5_tokenizer {
            xCreate: Some(Self::c_create),
            xDelete: Some(Self::c_delete),
            xTokenize: Some(Self::c_tokenize),
        };

        Box::into_raw(Box::new(tokenizer))
    }
}

#[derive(Debug)]
pub struct Token {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub colocated: bool,
}
