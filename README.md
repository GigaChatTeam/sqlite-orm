# Overview
## What?
API abstracting away SQL for database interactions
## Why?
To provide easy way for clients to cache messages
## Implementation
Rust + SQLite
## Branches
* `master`:         Currently has NON-THREAD-SAFE implementation!
* `thread-safe`:    I am working on adding thread-safety there. Changes made to master will be merged and patched there.
## Usage
You can call the API functions from any language that supports C functions (extern "C" / cdylib / CDLL / C-FFI)
## Adding support for other languages
C and C++ bindings will be generated with `cbindgen` and stored in `include` directory. Any bindings for other languages should use these headers and the implementations should be placed in `include/<LANGUAGE>` directory, e.g. `include/python/main.py` or `include/java/...`

# Example
`cargo doc --open`. You are welcome.

# Contributing
For now just open a pull request and If it contains anything useful at all it will probably be merged.


