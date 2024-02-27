/* ----------------------------------------------------------------------------
   gigachat-orm - a library for caching and managing gigachat files
   Copyright (C) 2024 Sotov Konstantin A

   This file is part of gigachat-orm library.

   This library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Lesser General Public License
   as published by the Free Software Foundation; either version 3 of
   the License, or (at your option) any later version.

   This library is distributed in the hope that it will be useful, but
   WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Lesser General Public License for more details.

   You should have received a copy of the GNU Lesser General Public
   License along with this library; if not, see <http://www.gnu.org/licenses/>.   
   ----------------------------------------------------------------------------
*/

use std::ffi::{CStr, CString, NulError, c_char};
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
pub fn ptr_to_str(ptr: *const c_char) -> Result<&'static str, Utf8Error> {
    // bro relly? CStr::from_ptr is architecture-dependent? Okay rust this is funny 
    // TODO: change every pointer use to c_char because something funny might happen
    let ptr: &CStr = unsafe { CStr::from_ptr(ptr as *const c_char) };
    ptr.to_str()
}

/// Converts std::string::String into *const u8 (const unsigned char*)
/// 
/// clones the string in order to convert it inco CString (which appends null terminator to the end
/// of the string) and returns either a NulError or a new, fresh and valid string pointer
/// 
/// # Arguments
/// * str: immutable borrow of the String
///
/// # Returns
/// Result<*const u8, NulError>, where NulError occurs only when there are null bytes in the middle
/// of the original string and *const u8 is a pointer to the beginning of the string
///
pub fn str_to_ptr(str: String) -> Result<*const c_char, NulError> {
    let ret = CString::new(str.into_bytes())?
        .into_raw() as *const c_char;
    Ok(ret)
}

/// tries to make a string and if it fails returns nullptr
pub fn transform_string_option(so: Option<String>) -> *const c_char {
    if let Some(s) = so {
        str_to_ptr(s).unwrap_or(std::ptr::null())
    } else {
        std::ptr::null()
    }
}


