use crate::interface::common::ptr_to_str;
use crate::interface::urlbuilder::UrlBuilder;
use microserde;
use std::ffi::c_char;
use ureq;


#[no_mangle]
pub extern "C" fn load_channels(uid: u64, token: *const c_char, dlb_url: *const c_char) {
    //http://10.242.223.170:8084/user/@me/channels?client=5&token=Et9pMkeTo9AYVCeDmzEiLmaHxS5kxtvkqQAoXiGNnfR7nzX9&sort=activity&order=desc&meta=true
	let url = UrlBuilder::new()
        .method("http")
        .url("10.242.223.170")
        .port(8084)
        .query("?client=5&token=Et9pMkeTo9AYVCeDmzEiLmaHxS5kxtvkqQAoXiGNnfR7nzX9&sort=activity&order=desc&meta=true")
        .path("/user/@me/channels")
        .build()
        .expect("?? why ??");
    let request = ureq::request_url("GET", &url);
    //
    dbg!(request.call().unwrap().into_string().unwrap());

}






