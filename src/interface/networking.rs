
use ureq;
use super::common::ptr_to_str;

#[no_mangle]
pub extern "C" fn load_channels(uid: u64, token: *const u8, dlb_url: *const u8) {
    let url = std::format!(
            "{}?client={}&token={}", 
            ptr_to_str(dlb_url).unwrap(),
            uid,
            ptr_to_str(token).unwrap()
        );
    let data = match ureq::get(url.as_str()).call() {
        Ok(resp) => {
            if resp.status() == 200_u16 {
                resp.into_string().unwrap()
            } else {
                std::format!("ERROR: {}", resp.status().to_string())
            }
        },
        Err(e) => {
            e.to_string()
        },
    };
    println!("the result of calling the request: {data}");
    // return struct that I have not yet defined
}


