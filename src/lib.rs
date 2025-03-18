use std::ffi::c_int;

use reg::register_tokenizer;
use rusqlite::ffi::*;

mod query;
mod reg;
#[cfg(test)]
mod test;
mod tokenize;

#[macro_export]
macro_rules! not_ok_return {
    ($rc:expr) => {{
        if $rc != SQLITE_OK as c_int {
            return $rc;
        }
    }};
}

pub(crate) static mut SQLITE3_PLUGIN_API: *mut sqlite3_api_routines = std::ptr::null_mut();

/// SAFETY: This function shound ONLY be called by SQLite
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqlite3_extension_init(
    db: *mut sqlite3,
    pz_err_msg: *mut *mut char,
    p_api: *mut sqlite3_api_routines,
) -> c_int {
    unsafe {
        if !p_api.is_null() {
            SQLITE3_PLUGIN_API = p_api;
        }

        not_ok_return!(register_tokenizer::<tokenize::Simple>(
            b"baize\0", db, pz_err_msg
        ));
        #[cfg(feature = "jieba")]
        register_tokenizer::<tokenize::Jieba>(b"jieba\0", db, pz_err_msg)
    }
}
