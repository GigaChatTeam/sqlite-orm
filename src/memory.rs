//! A module to allocate memory, mainly for arrays
//!
//! # ARRAY STRUCTURE
//! [`usize asize` | `usize esize` | `data: bytes[asize*esize]` ]
//! `asize`: amount of elements (allocated size divided by element size.
//! `esize`: element size.
//! both of them can be accessed with special functions.
//! `data`: start of the array
//! ## How functions return arrays
//! They return a pointer to the `data`, so that you can access array elements
//! like normal in C. metadata is stored "to the left" of the pointer. while you
//! can access metadata directly, you should use special functions
//! [gigachatmem_array_size] and [gigachatmem_array_element_size] ## How you
//! should pass arrays to function All functions are aware of this array type,
//! so you should pass it just like you receive it: without manually subtracting
//! anything to the pointer <br> Actually, you should never try to access the
//! metadata directly. just treat this like a normal array, except you can get
//! it's size from a function and you should deallocate it with this library.
//!
//! # Note
//! private functions may be generic. Public functions use private functions
//! instantiations, so this should be fine.
//! Right?
//! Will find out during testing.
//!
//! # Safety
//! everything in this module is unsafe, due to either dereferencing a raw
//! pointer or using allocation API. Private functions may cast memory to slices
//! and vice versa.
//!
//! # Source
//! Functions are grouped like this on purpose, there is 1 line between groups
//! and no lines between function in group. Please don't format this.    :w

use crate::database::structs;
use std::{
	alloc::{alloc, dealloc, Layout},
	ffi::c_void,
	mem::size_of,
	ptr::null_mut,
};

/// Holds all possible types that can be allocated. Will be extended in future.
#[repr(C)]
#[non_exhaustive]
pub enum Type {
	Message,
	Channel,
	/// The only one that requires explaination. This means "raw bytes". so if
	/// you pass it to allocation function, you are essentially allocating array
	/// of bytes.
	Raw,
}

/// use 1 byte as alignment by default
/// change source to change that.
pub const GLOB_MEM_ALIGN: usize = 1 << 3;
/// size of the `usize` type. Used by several functions.
const USIZE_SIZE: usize = size_of::<usize>();

/// Private API.
unsafe fn alloc_array<T>(size: usize) -> Option<*mut T> {
	let t_size = size_of::<T>();
	let layout = Layout::from_size_align(t_size * size + 2 * USIZE_SIZE, GLOB_MEM_ALIGN);
	if layout.is_err() {
		return None;
	}
	let layout = layout.unwrap();
	Some(alloc(layout).byte_add(2 * USIZE_SIZE) as *mut T)
}

/// frees any array returned by this module
///
/// # Arguments
/// * `arr`: pointer to the beginning of the array (to the `data` field, not to
///   the metadata). It
/// being null does not cause undefined behavior. But why would you pass a
/// nullptr to a `free` function is a mystery.
///
/// # Returns
/// * Nothing :)
///
/// # Safety
/// Should not fail, unless you pass an invalid pointer. If you do, shit WILL
/// happen.
///
/// # Examples
/// You will figure this out, I believe in you.
#[no_mangle]
pub unsafe extern "C" fn gcmm_free_array(arr: *mut c_void) {
	if arr.is_null() {
		return;
	}
	let arr = arr.byte_sub(2 * USIZE_SIZE) as *mut usize;
	let layout = Layout::from_size_align(*arr * *arr.add(1), GLOB_MEM_ALIGN).unwrap();
	dealloc(arr as *mut u8, layout);
}

/// allocates an array of specified type and size.
///
/// # Arguments
/// * `t`: enum "which type would you like to allocate". If you pass unsupported
///   member, nullptr is
/// returned
/// * `n`: size of array (how much T's can array hold).
///
/// # Returns
/// * a void pointer to the newly allocated data. It is your responsibbility to
///   cast it to whatever
/// type you need.
///
/// # Safety
/// Unsafe, because it uses allocation API. just be sure not to pass n that is
/// bigger than your RAM
///
/// # How this works
/// This function switches on t and calls a separate function for each possible
/// t. yes, really.
///
/// # Examples
/// TODO, this one actually may need explaination
#[no_mangle]
pub unsafe extern "C" fn gcmm_alloc_array(t: Type, n: usize) -> *mut c_void {
	#[allow(unreachable_patterns)]
	match t {
		Type::Message => gcmm_alloc_Messages(n) as *mut c_void,
		Type::Channel => gcmm_alloc_Channels(n) as *mut c_void,
        Type::Raw => alloc_array::<u8>(n).unwrap_or(null_mut()) as *mut c_void,
		_ => null_mut(), /* Why would rust complain on me matching an unreachable pattern on
		                  * non_exhaustive enum */
	}
}

/// returns the size of the array
///
/// # Arguments
/// `arr`: a valid pointer to array
///
/// # Returns
/// `usize`: size of the array
///
/// # Safety:
/// dereferences a raw pointer. if valid, should be no problem.
///
/// # Examples
/// sorry, not today. one day i will finish documentation, maybe...
#[no_mangle]
pub unsafe extern "C" fn gcmm_array_size(arr: *const c_void) -> usize {
	let arr = arr.byte_sub(2 * USIZE_SIZE) as *const usize;
	*arr
}
/// returns the size of one element in the array
///
/// # Arguments
/// `arr`: a valid pointer to array
///
/// # Returns
/// `usize`: size of the element
///
/// # Safety:
/// dereferences a raw pointer. if valid, should be no problem.
///
/// # Examples
/// sorry, not today. one day i will finish documentation, maybe...
#[no_mangle]
pub unsafe extern "C" fn gcmm_array_element_size(arr: *mut c_void) -> usize {
	let arr = arr.byte_sub(2 * USIZE_SIZE) as *const usize;
	*arr.add(1)
}

/// I am lazy and rust allows this. 
macro_rules! gen_allocator {
    ($struct:ident,$name:ident) => {
        /// A function to allocate memory for [$struct](crate::database::structs::$struct)
        /// 
        /// # Arguments 
        /// * `n`: size of the array. 
        ///
        /// # Returns 
        /// * pointer to allocated memory 
        /// 
        /// # Safety
        /// * uses memory allocation API, so has to be unsafe 
        ///
        /// # More details 
        /// see [gcmm_alloc_array]
        ///
        /// # Note 
        /// This documentation is generated from macro, so substitution may be incorrect, i don't
        /// know how to fix this. Anyways, you should guess what this function returns from it's name
        #[no_mangle]
        pub unsafe extern "C" 
        fn $name(n: usize) -> *mut structs::$struct {
            let memory = alloc_array::<structs::$struct>(n);
            memory.unwrap_or(null_mut()) as *mut structs::$struct
        }
    };
}

gen_allocator!(Channel, gcmm_alloc_Channels);
gen_allocator!(Message, gcmm_alloc_Messages);
gen_allocator!(Media, gcmm_alloc_Media);

