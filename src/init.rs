use rusqlite::{
    Connection,
    Error,
    Result
};

use std::ffi::{
    CStr,
    CString
};

pub const X: i32 = 69;

pub fn create_database() -> Result<(), Error> {
    println!("henlo");
    Ok(())
}
