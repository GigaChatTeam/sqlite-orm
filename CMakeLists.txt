cmake_minimum_required(VERSION 3.18)

project(gigachat-sqlite-rs 
	VERSION 0.0.0
)

find_program(CARGO cargo REQUIRED)

add_custom_target(libgigachat_sqlite.so ALL
	COMMAND cargo build --release --features multithread
	DEPENDS src/lib.rs
	BYPRODUCTS target/release/libgigachat_sqlite.so
)

if (NOT ${cargo_build_result} EQUAL 0)
	message("result: ${cargo_build_result}")
	message(FATAL_ERROR "Cargo build failed. Exiting")
endif()

