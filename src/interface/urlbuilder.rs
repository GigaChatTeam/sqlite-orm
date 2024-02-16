//! This is probably the module I am ashamed of the most. For internal use only
//!
//! # Goal
//! provides the `UrlBuilder` struct which is a builder-style url wrapper.
//!
//! # Things I am ashamed of
//! * macros. they look ugly.
//! * repetativeness. even with macros, the code is repetative.
//! * the fact that functions consume and return ownerships (this is inevitable for builders but I
//! still hate this)
//! * the fact that DEFAULT INITIALIZATION CAN NOT BE DONE FUCKING BULLSHIT (I just realized that i
//! can just add `new` method to url struct but whatever. Will put it in todo i guess.)
//! * names. I don't think anyone cares though.
//! * this is fucking unreadable
//!

use std::default;

use url;

/// Builder for `Url` from `url` crate. For internal use only.
pub enum UrlBuilder {
	Url(url::Url),
    None,
}

/// TODO: add `new` implementation for builder
impl Default for UrlBuilder {
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
        pub fn $method_own(self, $method_own: &str) -> Self{
            match self {
                Self::Url(mut url) => {
                    url.$method_foreign($pass);
                    Self::Url(url)
                },
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
                Self::Url(mut url) => {
                    match url.$method_foreign($pass) {
                        Ok(_) => Self::Url(url),
                        Err(_) => Self::None,
                    }
                },
                Self::None => self,
            }
        } 
    };
}

impl UrlBuilder {
	pub fn new() -> UrlBuilder {
        // TODO: add `new` implementation for builder
        // using pasre method like this sucks
		UrlBuilder::Url(url::Url::parse("http://127.0.0.1").unwrap())
	}
    pub fn query_pairs(self, pairs: Vec<(String, String)>) -> Self {
        match self {
            Self::Url(mut url) => Url(url.query_pairs_mut().extend_pairs(pairs.into_iter())),
            None => self,
        }
    }
    pub fn param(self, key: &str, value: &str) -> Self {
        match self {
            Self::Url(mut url) => Self::Url(url.query_pairs_mut().append_pair(key, value)), 
            None => self,
        }
    }
    pub fn build(self) -> Option<url::Url> {
        match self {
            Self::Url(url) => Some(url),
            Self::None => Option::<url::Url>::None,
        }
    }
    builder_func_result!(scheme, set_scheme, method, &str);
    builder_func_result!(url, set_host, Some(url), &str);
    builder_func_result!(port, set_port, Some(port), u16);
    builder_func!(path, set_path, path);
    builder_func!(query, set_query, Some(query));
}
