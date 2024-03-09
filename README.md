# Overview
## TODO 
* Implement a `C array` wrapper already, I've procrastinated on this long enough
* Move nightly-only features behind a compilation feature 
* Load channels
* Implement integrity checks
* Check for consistency in function names
* test memory leaks and async more
* test from FFI 
* restructure SQL database 
* implement json -> C structs through `into` trait
## What?
API abstracting away SQL for database interactions
## Why?
To provide easy way for clients to cache messages
## Implementation
Rust + SQLite
## Branches
* `master`:         The main working branch, Thread-safe API was merged into master
* `temp`:           unfinished commits that will be squeezed into master 
## (Rust) features
* `multithread`: support for calling this library from multiple threads. Without this feature multithreading causes undefined behavior
## Usage
You can call the API functions from any language that supports C functions (extern "C" / cdylib / CDLL / C-FFI)
## Adding support for other languages
C and C++ bindings will be generated with `cbindgen` and stored in `include` directory. Any bindings for other languages should use these headers and the implementations should be placed in `include/<LANGUAGE>` directory, e.g. `include/python/main.py` or `include/java/...`
## Build
For now this library requires rust-nightly, because one of the functions uses `std::vec::Vec::into_raw_parts`, which is nightly-only experimental API. In future it should be moved behind a compilation feature for rust-stable compatability
* `cargo build --release` is you are planning on using this library, since functions that are exclusively for debugging are hidden behind `#[cfg(debug_assertions)]`
* `cargo build` if you are planning on developing/contributing to library 
# Example
`cargo doc --open` in source directory. You are welcome.

# Contributing
For now just open a pull request and If it contains anything useful at all it will probably be merged.


