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
//! All of the functions here use blocking networking. This is not laziness, but a
//! convenience for users of this library. Asynchronous calls would make
//! cross-language interactions a pain in the ass, while internal multithreading
//! will introduce unnecessary overhead and data issues.
//! (so if you get something like `NetworkingOnMainThreadError` in your android application, this
//! is entirely yours and noone else's fault)
//!
//! Unlike [database](crate::database) module, this one does not have to be
//! initialized and does not store anything globally. It is also not affected my
//! `multithread` feature (since there is no shared state in this library). Many
//! of the functions return arrays which is memory-unsafe and great care should
//! be taken to properly deallocate them.

pub mod gc_json;
pub mod urlbuilder;

use crate::{common::ptr_to_str, memory};

// renamed to differentiate between my json module (prefixe with gc_) and
// microserde's json module (prefixed with ms_)
use microserde::{json as ms_json, Deserialize};

use std::{ffi::c_char, ptr::null_mut};
use ureq;

use crate::database::structs;
use urlbuilder::RequestBuilder;

const MAX_CHANNELS: u32 = 150;

/// Am enum representing errors that might occur during network requests
///
/// # Note
/// if the returned error is negative, it means that it should be described in
/// this enum. If returned error is positive - refer to documentation of the
/// function that returned it. In most cases these should be http status codes.
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

/// Abstracts away mapping errors from ureq to this lib's error codes
fn load_from_server<T>(builder: RequestBuilder) -> Result<T, StatusAndNwError>
where
	T: Deserialize,
{
	let request = builder.build().ok_or(StatusAndNwError::Nw(NwError::InvalidUrl))?;
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

/// A function to load all channels with /user/&lt;UID&gt;/channels server
/// request
///
/// performs requests to DLB gigachat server. Since server has a cap on how many
/// channels can be sent (at the moment of writing documentation: 150, stored in
/// const MAX_CHANNELS variables), it sends requests in a loop, each time
/// incrementing `offset` parameter by that cap. This function returns
/// an array as defined in [memory] module.
/// # Arguments
/// * uid: User ID as in gigachat databse.
/// * token: C string with a token. The memory is treated read-only.
/// * dlb_url: A URL to the Data Load Bridge server. Can be either in form `https://address`
/// or `address`, protocol is replaced with `http` anyways. <br>
/// &Tab;Since servers can be ran locally, this is an argument. In future i will
/// consider either getting rid of it or adding a function that assumes a
/// default address (e.g. dlb.gigachat.com)
/// # Returns
/// * An array of [Channel](structs) structs. Please refer to documentation in [memory] for more
/// details.
/// # Errors 
/// currently not implemented, but there is "adding a function to return last error" in my TODO
/// list. So the function will not return any error information, another function will try to
/// return global error variable (like errno).
/// ## MAYBE A CONSIDERATION IN FUTURE?
/// making `arrays_return_errors` feature to return something else instead of a nullptr.
/// maybe.
/// # Example
/// no example yet.
/// # TODO:
/// - make that horrifyingly long lambda it's own function
/// - which one did you mean mf i don't see one
#[no_mangle]
pub extern "C" fn gigachatnw_load_channels(
	uid: u64,
	token: *const c_char,
	dlb_url: *const c_char,
) -> *mut structs::Channel {
	let url = RequestBuilder::new()
		.scheme("http")
		.url(match ptr_to_str(dlb_url) {
			Ok(str) => str,
			Err(_) => return null_mut(),
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
				Err(_) => return null_mut(),
			},
		);

	let mut all_channels: Vec<structs::Channel> = Vec::new();
	let mut loaded_count: u32 = 0;

	loop {
		let structure = match load_from_server::<gc_json::ChannelsResponse>(
			url.clone().param("offset", loaded_count.to_string().as_str()),
		) {
			Ok(unparsed) => unparsed,
			Err(_) => {
				// TODO: set ERROR variable
				return null_mut();
			}
		};
		all_channels.extend(structure.data.into_iter().map(Into::into));
		if structure.count != MAX_CHANNELS {
			break;
		}
		loaded_count += MAX_CHANNELS;
	}

	unsafe { memory::copy_from_slice::<structs::Channel>(all_channels.as_slice()) }
}


