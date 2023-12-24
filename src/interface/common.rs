use std::ffi::{CStr, CString, NulError};
use std::str::Utf8Error;

/// Converts pointer to unowned string
/// 
/// Does not own memory! The string must end with '\0' byte!
///
/// # Arguments
/// * `ptr`: pointer to the C string
///
/// # Returns
/// result of CStr::to_str 
///
pub fn ptr_to_str(ptr: *const u8) -> Result<&'static str, Utf8Error> {
    let ptr: &CStr = unsafe { CStr::from_ptr(ptr as *const i8) };
    ptr.to_str()
}

/// Converts std::string::String into *const u8 (const unsigned char*)
/// 
/// clones the string in order to convert it inco CString (which app3nds null terminator to the end
/// of the string) and returns either a NulError or a new, fresh and valid string pointer
/// 
/// # Arguments
/// * str: immutable borrow of the String
///
/// # Returns
/// Result<*const u8, NulError>, where NulError occurs only when there are null bytes in the middle
/// of the original string and *const u8 is a pointer to the beginning of the string
///
pub fn str_to_ptr(str: String) -> Result<*const u8, NulError> {
    let ret = CString::new(str.into_bytes())?
        .into_raw() as *const u8;
    Ok(ret)
}

