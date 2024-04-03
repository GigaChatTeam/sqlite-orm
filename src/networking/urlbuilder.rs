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

//! This is probably the module I am ashamed of the most. For internal use only
//!
//! # Goal
//! provides the `UrlBuilder` struct which is a builder-style url wrapper.
//!
//! # Things I am ashamed of
//! * macros. they look ugly.
//! * repetativeness. even with macros, the code is repetative.
//! * the fact that functions consume and return ownerships (this is inevitable
//!   for builders but I
//! still hate this)
//! * the fact that DEFAULT INITIALIZATION CAN NOT BE DONE FUCKING BULLSHIT (I
//!   just realized that i
//! can just add `new` method to url struct but whatever. Will put it in todo i
//! guess.)
//! * names. I don't think anyone cares though.
//! * this is fucking unreadable

use url;

#[derive(Clone)]
pub struct UrlWithExtensions {
    address: url::Url,
    headers: Vec<(String, String)>,
    method: Option<String>,
}

#[derive(Clone)]
/// Builder for `Url` from `url` crate. For internal use only.
pub enum RequestBuilder {
	Url(UrlWithExtensions),
	None,
}

impl Default for RequestBuilder {
	fn default() -> Self {
		Self::new()
	}
}

/// Macro to generate builder functions. For internal use only.
///
/// # Note
/// I am so sorry for this
macro_rules! builder_func {
	($method_own:ident, $method_foreign:ident, $pass:expr) => {
		pub fn $method_own(self, $method_own: &str) -> Self {
			match self {
				Self::Url(mut url) => {
					url.address.$method_foreign($pass);
					Self::Url(url)
				}
				Self::None => self,
			}
		}
	};
}

/// Marco to generate builder functions (again). For internal use only.
///
/// # Note
/// And for this one, I am sorry as well
macro_rules! builder_func_result {
	($method_own:ident, $method_foreign:ident, $pass:expr, $type:ty) => {
		pub fn $method_own(self, $method_own: $type) -> Self {
			match self {
				Self::Url(mut url) => match url.address.$method_foreign($pass) {
					Ok(_) => Self::Url(url),
					Err(_) => Self::None,
				},
				Self::None => self,
			}
		}
	};
}

impl RequestBuilder {
	pub fn new() -> RequestBuilder {
		// TODO: add `new` implementation for builder
		// using pasre method like this sucks
		RequestBuilder::Url(UrlWithExtensions{
            address: url::Url::parse("http://127.0.0.1").unwrap(),
            headers: Vec::new(),
            method: None,
        })
	}
	pub fn query_pairs(self, pairs: Vec<(String, String)>) -> Self {
		match self {
			Self::Url(mut url) => {
				url.address.query_pairs_mut().extend_pairs(pairs);
				Self::Url(url)
			}
			Self::None => self,
		}
	}
	pub fn param(self, key: &str, value: &str) -> Self {
		match self {
			Self::Url(mut url) => {
				url.address.query_pairs_mut().append_pair(key, value);
				Self::Url(url)
			}
			Self::None => self,
		}
	}
    pub fn header(self, header: String, value: String) -> Self {
        match self {
            Self::Url(mut url) => {
                url.headers.push((header, value));
                Self::Url(url)
            }
            Self::None => self,
        }
    }
	pub fn build(self) -> Option<ureq::Request> {
		match self {
			Self::Url(url) => {
                let method = url.method.unwrap_or(String::from("GET"));
                let mut req = ureq::request_url(method.as_str(), &url.address);
                for (header, value) in url.headers {
                    req = req.set(header.as_str(), value.as_str());
                }
                Some(req)
            },
			Self::None => Option::<ureq::Request>::None,
		}
	}
	builder_func_result!(scheme, set_scheme, scheme, &str);
	builder_func_result!(url, set_host, Some(url), &str);
	builder_func_result!(port, set_port, Some(port), u16);
	builder_func!(path, set_path, path);
	builder_func!(query, set_query, Some(query));
}
