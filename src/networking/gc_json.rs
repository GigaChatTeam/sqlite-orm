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

//! non-public module that contains structs for json-parsing
//!
//! # Notes
//! not for public use

use microserde::{Deserialize, Serialize};

use std::ptr::null as nullptr;

use crate::{common::*, database::structs};

trait IntoCStyle<T> {
	fn into_c(self) -> T;
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Forward {
	r#type: String,
	forward_path: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
	id: u64,
	author: Option<u64>,
	editer: bool,
	unix_milli: u64,
	r#type: String,
	data: Option<String>,
	// only IDs
	files: Vec<u64>,
	// media: Vec<Vec<u64>>,
	forward: Option<Forward>,
}

impl Into<structs::Message> for Message {
	fn into(self) -> structs::Message {
		todo!()
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileData {
	pub id: u64,
	pub url: Option<String>,
}

/// struct to represent "data" field of returned 'channel' request
///
/// should work with both meta=true and meta=false (every field that can be
/// disabled with meta=false is `Option<T>`)
#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelMetaData {
	pub id: u64,
	#[serde(rename = "user-status")]
	pub user_status: u64,
	pub title: Option<String>,
	pub description: Option<String>,
	pub public: Option<bool>,
	pub enabled: bool,
	pub icon: Option<FileData>,
}

impl Into<structs::Channel> for ChannelMetaData {
	fn into(self) -> structs::Channel {
		let title_ = transform_string_option(self.title);
		let description_ = transform_string_option(self.description);
		let avatar_ = self
			.icon
			.and_then(|icon| icon.url)
			.and_then(|url| match str_to_ptr(url) {
				Ok(ptr) => Some(ptr),
				Err(_) => None,
			})
			.unwrap_or(nullptr());

		structs::Channel {
			id: self.id,
			title: title_,
			description: description_,
			avatar: avatar_,
			enabled: self.enabled,
			permissions: 0,
		}
	}
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