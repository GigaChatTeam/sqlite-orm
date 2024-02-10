use crate::interface::common::ptr_to_str;
use crate::interface::urlbuilder::UrlBuilder;
use microserde;
use std::ffi::c_char;
use ureq;

#[no_mangle]
pub extern "C" fn load_channels(uid: u64, token: *const c_char, dlb_url: *const c_char) {
	let url = UrlBuilder::new()
        .method("http")
        .url("127.0.0.1")
        .port(8084)
        .query("?index=a&penis=b")
        .path("/lol/lmao")
        .build();
	let request = ureq::builder().build();
}






