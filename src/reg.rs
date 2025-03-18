use std::ffi::{CString, c_char, c_int, c_void};

use rusqlite::ffi::*;

use crate::{SQLITE3_PLUGIN_API, not_ok_return, tokenize::GetTokenizer};

pub unsafe fn register_tokenizer<T: GetTokenizer>(
    name: &[u8],
    db: *mut sqlite3,
    _pz_err_msg: *mut *mut char,
) -> c_int {
    let mut fts5_api_ptr: *mut fts5_api = std::ptr::null_mut();
    not_ok_return!(get_fts5_api(db, &mut fts5_api_ptr));

    unsafe {
        if fts5_api_ptr.is_null() || (*fts5_api_ptr).iVersion < 2 {
            return SQLITE_ERROR as c_int;
        }

        (*fts5_api_ptr).xCreateTokenizer.unwrap()(
            fts5_api_ptr,
            name.as_ptr() as *const c_char,
            fts5_api_ptr as *mut c_void,
            T::get_tokenizer(),
            None,
        )
    }
}

fn get_fts5_api(db: *mut sqlite3, pp_api: *mut *mut fts5_api) -> c_int {
    let mut stmt: *mut sqlite3_stmt = std::ptr::null_mut();
    let sql = CString::new("SELECT fts5(?1)").unwrap();
    unsafe {
        *pp_api = std::ptr::null_mut();

        not_ok_return!((*SQLITE3_PLUGIN_API).prepare.unwrap()(
            db,
            sql.as_ptr(),
            -1,
            &mut stmt,
            std::ptr::null_mut()
        ));

        not_ok_return!((*SQLITE3_PLUGIN_API).bind_pointer.unwrap()(
            stmt,
            1,
            pp_api as *mut c_void,
            b"fts5_api_ptr\0".as_ptr() as *const c_char,
            None,
        ));

        let step_result = (*SQLITE3_PLUGIN_API).step.unwrap()(stmt);
        if step_result != SQLITE_ROW as c_int {
            return step_result;
        }

        (*SQLITE3_PLUGIN_API).finalize.unwrap()(stmt)
    }
}
