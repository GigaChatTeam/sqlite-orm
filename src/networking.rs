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

//! A module containing functions related to loading data from DLB server
//!
//! All of the functions use blocking networking. This is not laziness, but a convenience for users
//! of this library. Asynchronous calls would make cross-language interactions a pain in the ass,
//! while internal multithreading will introduce unnecessary overhead and data issues.
//!
//! Unlike [database](crate::database) module, this one does not have to be initialized and does
//! not store anything globally. It is also not affected my `multithread` feature (since there is
//! no shared state in this library). Many of the functions return arrays which is HIGHLT unsafe
//! and great care should be taken to properly deallocate them. 

pub mod gc_json;
pub mod urlbuilder;

use crate::common::{ptr_to_str};

// renamed to differentiate between my json module (prefixe with gc_) and
// microserde's json module (prefixed with ms_)
use microserde::{json as ms_json, Deserialize};

use std::ffi::c_char;
use ureq;

use crate::database::structs;
use urlbuilder::RequestBuilder;

const MAX_CHANNELS: u32 = 150;

/// Am enum representing errors that might occur during network requests
///
/// # Note
/// if the returned error is negative, it means that it should be described in this enum. If
/// returned error is positive - refer to documentation of the function that returned it. In most
/// cases these should be http status codes.
#[repr(i32)]
pub enum NwError {
	/// Unimplemented error handling
	UnknownError = -255,
	/// If the builder for request encountered an error (like invalid character
	/// in URL or in parameter). Should not happen, but here just for case
	RequestBuildError = -9,
	/// Json parsing error
	JsonError = -8,
	/// If the char* contains invalid UTF-8
	InvalidCString = -7,
	/// If server returned Invalid UTF-8 string
	InvalidUTF8 = -6,
	/// If something failed on transport-level (e.g. dns lookup failure or
	/// network is unreachable)
	ConnectionError = -5,
	/// Generic error
	NetworkError = -4,
	/// If connection failed
	Timeout = -3,
	/// If a server returns error
	ServerError = -2,
	/// If UrlBuilder was constructed invalid, or if scheme is not understood by
	/// the request handler
	InvalidUrl = -1,
}

/// Enum to combine NwError and Status code. Used in Basically 1 function. Not
/// for FFI export.
pub enum StatusAndNwError {
	Nw(NwError),
	Status(u16),
}

/// A c-style array with Channels.
///
/// # Note
/// should be deallocated in rust, like everything in this codebase. It assumes usage of rust-nightly-only 
/// API ([this](std::vec::Vec::into_raw_parts) and [this](std::vec::Vec::from_raw_parts)), and
/// that's why is has [alloc](ChannelArray::alloc) field. 
/// # Usage
/// TODO: add example
#[repr(C)]
pub struct ChannelArray {
	/// Size of the array. If `arrays_store_errors` feature is enabled: if data is nullptr, size
    /// stores an error code - if positive, status code from http response, if negative - error as
    /// specified in NwError struct.
	pub size: isize,
	/// Size allocated by rust's Vec. You can safely construct this much
	/// elements, but why would you do this. Generally, this is only used to
	/// deallocate memory on the rust side. The only reason to make this `pub`
	/// is that maybe someone will find use case for this.
	pub alloc: isize,
	/// pointer to the element at index 0. If nullptr, operation did not succed and `size` field 
    /// contains error code (only with `arrays_store_errors` feature enabled, which is default)
	pub data: *mut structs::Channel,
}

/// Abstracts away mapping errors from ureq to this lib's error codes
fn load_from_server<T>(builder: RequestBuilder) -> Result<T, StatusAndNwError>
where
	T: Deserialize,
{
	let request = builder
		.build()
		.ok_or_else(|| StatusAndNwError::Nw(NwError::InvalidUrl))?;
	dbg!(&request);
	let response = request
		.call()
		.map_err(|e: ureq::Error| match e {
			ureq::Error::Status(code, _) => StatusAndNwError::Status(code),
			ureq::Error::Transport(terr) => StatusAndNwError::Nw(match terr.kind() {
				ureq::ErrorKind::InvalidUrl => NwError::InvalidUrl,
				ureq::ErrorKind::UnknownScheme => NwError::InvalidUrl,
				ureq::ErrorKind::InsecureRequestHttpsOnly => NwError::InvalidUrl,
				ureq::ErrorKind::Dns => NwError::ConnectionError,
				ureq::ErrorKind::ConnectionFailed => {
					dbg!(&terr);
					NwError::ConnectionError
				}
				ureq::ErrorKind::TooManyRedirects => NwError::NetworkError,
				ureq::ErrorKind::BadStatus => NwError::NetworkError,
				ureq::ErrorKind::BadHeader => NwError::NetworkError,
				ureq::ErrorKind::Io => {
					dbg!(&terr);
					NwError::ConnectionError
				}
				_ => NwError::UnknownError,
			}),
		})?
		.into_string()
		.map_err(|_| StatusAndNwError::Nw(NwError::InvalidUTF8))?;
	ms_json::from_str::<T>(response.as_str()).map_err(|_| StatusAndNwError::Nw(NwError::JsonError))
}

/// a function that wraps "returning errors inside size of array" in a feature
/// I can easily imagine that someone would not be expecting to receive errors inside the field
/// called "array.size". 
#[allow(unused_variables)]
fn default_array(_err: i32) -> ChannelArray {
	#[cfg(feature = "arrays_store_errors")]
	let size_ = _err as isize;
	#[cfg(not(feature = "arrays_store_errors"))]
	let size_ = 0;

	ChannelArray {
		size: size_,
		alloc: 0,
		data: std::ptr::null_mut(),
	}
}

/// A function to load all channels with /user/&lt;UID&gt;/channels server request
///
/// performs requests to DLB gigachat server. Since server has a cap on how many channels can be
/// sent (at the moment of writing documentation: 150, stored in const MAX_CHANNELS variables), it
/// sends requests in a loop, each time incrementing `offset` parameter by that cap. This function
/// returns [ChannelArray], which is basically a decomposition of `std::vec::Vec<Message>`. Because of
/// that, the function is highly unsafe and may lead to memory leaks if handled improperly.
/// # Arguments
/// * uid: User ID as in gigachat databse. 
/// * token: C string with a token. The memory is treated read-only
/// * dlb_url: A URL to the Data Load Bridge server. Can be either in form `https://address` 
/// or `address`, protocol is replaced with `http` Since servers can be ran locally, this is an
/// argument. In future i will consider either getting rid of it or adding a function that assumes
/// a default address (e.g. dlb.gigachat.com)
/// # Returns
/// * ChannelArray struct. Please refer to [ChannelArray] for more details. 
/// ## If feature `arrays_store_errors` is enabled:
/// [data](ChannelArray::data) being nullptr indicated that there was an error. The error itself in that case is store in
/// [size](ChannelArray::size) field
/// ## Otherwise:
/// 0-initialized [ChannelArray] (all fields are 0) indicates an error. There is no way of knowing
/// what went wrong.
/// # Example
/// TODO
/// # TODO: make that horrifyingly long lambda it's own function
#[no_mangle]
pub extern "C" 
fn gigachatnw_load_channels(uid: u64, token: *const c_char, dlb_url: *const c_char) -> ChannelArray {
	let url = RequestBuilder::new()
		.scheme("http")
		.url(match ptr_to_str(dlb_url) {
			Ok(str) => str,
			Err(_) => return default_array(NwError::InvalidCString as i32),
		})
		.path("/user/@me/channels")
		// .param("sort-type", "asc")
        // .param("sort-field", "activity")
        .param("sort", "asc")
        .param("order", "activity")
		.param("meta", "true")
        .header(
			String::from("Authorization"),
			uid.to_string()
				+ "-" + match ptr_to_str(token) {
				Ok(str) => str,
				Err(_) => return default_array(NwError::InvalidCString as i32),
			},
		);

	let mut all_channels: Vec<structs::Channel> = Vec::new();
	let mut loaded_count: u32 = 0;

	loop {
		let structure = match load_from_server::<gc_json::ChannelsResponse>(
			url.clone().param("offset", loaded_count.to_string().as_str()),
		) {
			Ok(unparsed) => unparsed,
			Err(e) => match e {
				StatusAndNwError::Status(s) => return default_array(s as i32),
				StatusAndNwError::Nw(n) => return default_array(n as i32),
			},
		};
		all_channels.extend(structure.data.into_iter().map(Into::into));
		if structure.count != MAX_CHANNELS {
			break;
		} 
        loaded_count += MAX_CHANNELS;
	}

	let all_channels = all_channels.into_raw_parts();
	ChannelArray {
		data: all_channels.0,
		size: all_channels.1 as isize,
		alloc: all_channels.2 as isize,
	}
}
