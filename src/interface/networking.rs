#![feature(vec_into_raw_parts)]
use crate::interface::common::ptr_to_str;
use crate::interface::urlbuilder::UrlBuilder;


// renamed to differentiate between my json module (prefixe with gc_) and microserde's json module (prefixed with ms_)
use microserde::json as ms_json;
use microserde::{Deserialize, Serialize};

use std::ffi::c_char;
use ureq;

use super::Channel;

/// non-public module that contains structs for json-parsing
///
/// # Notes
/// not for public use
mod gc_json {
	use super::*;

	/// struct to represent "data" field of returned 'channel' request
	///
	/// should work with both meta=true and meta=false (every field that can be disabled with
	/// meta=false is Option<T>)
	#[derive(Serialize, Deserialize, Debug)]
	pub struct ChannelMetaData {
		pub id: u64,
		#[serde(rename = "user-status")]
		pub user_status: u64,
		pub title: Option<String>,
		pub description: Option<String>,
		pub enabled: bool,
		pub public: Option<bool>,
		pub icon: Option<String>,
	}

	/// struct that should contain /user/channels response
	///
	/// I think this is very readable and does not need documentation
	#[derive(Serialize, Deserialize, Debug)]
	pub struct ChannelsResponse {
		pub status: String,
		pub count: u32,
		pub data: Vec<ChannelMetaData>,
	}
}

const MAX_CHANNELS: u32 = 100;

#[repr(i32)]
pub enum NwError {
	/// Unimplemented error handling
	UnknownError = -255,
}

/// A c-style array with Channels.
///
/// # Note
/// should be deallocated in rust, like everything in this codebase
///
/// # Usage
///
#[repr(C)]
pub struct ChannelArray {
	/// Size of the array
	pub size: usize,
    /// Size allocated by rust's Vec. You can safely construct this much elements, but why would
    /// you do this. Generally, this is only used to deallocate memory on the rust side.
    /// The opnly reason to make this `pub` is that maybe someonw will find use case for this. 
    pub alloc: usize,
	/// pointer to the element at index 0
	pub data: *mut Channel,
}

/// a function to load channels with /user/<UID>/channels server request
///
/// # Arguments
///
/// # Returns
///
/// # Example
///
#[no_mangle]
pub extern "C" fn load_channels(
	uid: u64,
	token: *const c_char,
	dlb_url: *const c_char,
) -> ChannelArray {
	let url = UrlBuilder::new()
        .method("http")
        .url(match ptr_to_str(dlb_url){
            Ok(str) => str,
            Err(_) => return ChannelArray{size: 0, alloc: 0, data: std::ptr::null_mut()},
        })
        .port(8084)
        .query("client=5&token=Et9pMkeTo9AYVCeDmzEiLmaHxS5kxtvkqQAoXiGNnfR7nzX9&sort=id&order=asc&meta=true")
        .path("/user/@me/channels")
        .build()
        .expect("?? why ??");

	let mut all_channels: Vec<Channel> = Vec::new();
	loop {
		let request = ureq::request_url("GET", &url);
		let response = request.call().unwrap().into_string().unwrap();
		let structure: gc_json::ChannelsResponse =
			ms_json::from_str(response.as_str()).unwrap();
		all_channels.extend(structure.data.into_iter().map(|x| -> Channel {
			Channel {
				id: x.id,
				title: std::ptr::null_mut(),
				description: std::ptr::null_mut(),
				avatar: std::ptr::null_mut(),
				created: 0,
				created_ns: 0,
				enabled: x.enabled,
			}
		}));
		if structure.count == MAX_CHANNELS {
			break;
		}
	}

    let all_channels = all_channels.into_raw_parts();
	ChannelArray {
		data: all_channels.0,
		size: all_channels.1,
        alloc: all_channels.2,
	}
}
