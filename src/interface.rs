//! mod interface | Interface that implements most public database-related functions

mod sql;
use std::ffi::CStr;
use std::str::Utf8Error;


/// Converts pointer to unowned string
/// DOES NOT OWN POINTER MEMORY!
/// # Arguments
/// * `ptr`: pointer to the C string
///
/// # Returns
/// result of CStr::to_str 
fn ptr_to_str(ptr: *const u8) -> Result<&'static str, Utf8Error> {
    let ptr: &CStr = unsafe { CStr::from_ptr(ptr as *const i8) };
    ptr.to_str()
}

/// Creates database at path `dbname`
/// Creates every necessary table if they do not exist (may be used to fix integrity of database)
///  
/// # Arguments 
/// * `dbname`: path to database being created ( TODO: check whether path should exist)
///
/// # Returns 
/// * i8 ( = c_char ): success status.
/// * * 0 = success
/// * * -1 (0xFF) = could not open database
/// * * any positive number = amount of errors occured duirng creating tables 
///     (this showing up is either FFI or library's fault)
#[no_mangle]
pub extern "C"
fn create_database(dbname: *const u8) -> i8 {
    let dbname = match ptr_to_str(dbname) {
        Ok(name) => name,
        Err(error) => panic!("Rust's `create_database` terminated with error: {error} due to invalid `dbname` argument"),
    };
    let db = match rusqlite::Connection::open(dbname) {
        Ok(db) => db,
        Err(_) => return -1,
    };

    let mut return_value = 0i8;
    let statements = &[ 
        sql::CREATE_USERS_TABLE, 
        sql::CERATE_ACCOUNTS_TABLE, 
        sql::CREATE_CHANNELS_TABLE, 
        sql::CREATE_MESSAGES_TABLE,
        sql::CREATE_CONFIG_TABLE
    ];
    for &i in statements {
        match db.execute(i, []) {
            Ok(_) => {},
            Err(_) => return_value += 1,
        }
    }
    return return_value;
}

