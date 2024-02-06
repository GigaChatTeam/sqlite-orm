use std::str::FromStr;

use cbindgen::{
    Builder,
    Config
};

extern crate cbindgen;

fn main() {
    let config = Config::from_file("cbindgen.toml").expect("no file `cbindgen.toml` found in working directory");
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("???");

    if Builder::new()
        .with_config(config)
        .with_crate(crate_dir)
        .generate()
        .expect("could not generate config")
        .write_to_file("include/gigachat_orm.h") {
            println!("успех ёпт");
        }
}
