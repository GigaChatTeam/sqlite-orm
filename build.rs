use std::str::FromStr;

use cbindgen::{Builder, Config};

extern crate cbindgen;

fn main() {
	let config = Config::from_file("cbindgen.toml")
		.expect("no file `cbindgen.toml` found in working directory");
	let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("???");
	if match Builder::new()
		.with_config(config)
		.with_crate(crate_dir)
		.generate()
	{
		Ok(x) => x.write_to_file("include/gigachat_orm.h"),
		Err(_) => false,
	} {
		println!("успех ёпт");
	} else {
        println!("ващё нихуя не успех");
    }
}
