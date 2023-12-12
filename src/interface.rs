
//! Interface that implements most public database-related functions
//!
//! # Purpose
//! The purpose of this module is to provide FFI exports for the API
//! 
//! # Important note
//! This module is not 100% FFI-compatible out of the box! For example, in C you would need to
//! write tagged union compatible with `cbindgen` produced output for enum types!
//! Any contributions to FFI are welcome. They will be accepted in `include/` directory of this
//! project. Please don't request changes to cbindgen output.
//!

mod sql;
use std::ffi::CStr;
use std::str::Utf8Error;
use std::ops;
// use cpp;

/// Flag enum to represent what a message contains
///
/// # Usage
/// It is intended to be used as flag enum. For example, this code will store an image that has
/// both text and image associated with it:
/// ```cpp
/// write_msg({MessageType::TEXT | MessageType::IMAGE, "", ...})
///
/// ```
///
#[repr(C)]
#[derive(Debug)]
pub enum MessageType {
    /// most generic text message
    TXT         = 1 << 0,
    /// an image in any of supported formats
    IMG         = 1 << 1,
    /// a video in any of supported formats
    VID         = 1 << 2,
    /// graphics interchange format (short, looping video)
    GIF         = 1 << 3,
    /// any of audio formats, except for voice message
    MUS         = 1 << 4,
    /// audio message (voice message)
    AMSG        = 1 << 5,
    /// video message (like circles from telegram)
    VMSG        = 1 << 6,
    /// a sticker (small image without boundaries)
    STCKR       = 1 << 7,
    /// a reaction: single unicode character displayed below message. This message type must always
    /// have a reply field as a valid message ID. the message to shich reaction is replying is a
    /// target of a reaction message
    REACT       = 1 << 8,
    /// an arbitrary amount of media that is displayed in a single message. media can be of any of
    /// the types above except TXT, STCKR and REACT. Some of the formats (like MUS, AMSG, VMSG)
    /// will probavly have strange behaviour
    MEDGROUP    = 1 << 9,
}

macro_rules! impl_bitor {
    ($t1:ty, $t2:ty) => {
        impl ops::BitOr<$t2> for $t1 {
            type Output = u32;
            fn bitor(self, rhs: $t2) -> Self::Output {
                (self as Self::Output) | (rhs as Self::Output)
            }
        }
    };
}

impl_bitor!(MessageType, MessageType);
impl_bitor!(MessageType, u32);
impl_bitor!(u32, MessageType);

macro_rules! impl_bitand {
    ($t1:ty, $t2:ty) => {
        impl ops::BitAnd<$t2> for $t1 {
            type Output = bool;
            fn bitand(self, rhs: $t2) -> Self::Output {
                ((self as u32) & (rhs as u32)) != 0
            }
        }
    };
}

impl_bitand!(MessageType, MessageType);
impl_bitand!(MessageType, u32);
impl_bitand!(u32, MessageType);

/// enum to represent what type of data is stored in database
///
/// I didn't find application for this but it is there I guess
///
#[repr(C)]
#[derive(Debug)]
pub enum EntryType {
    /// Any message-related database entry
    MESSAGE,
    /// Any user-related database entry
    USER,
    /// Any file-related database entry
    FILE,
    /// Any database entry
    /// Abstract thing added for no reason
    ENTRY, 
}

/// enum to represent the type of media being sent/stored
///
/// Required as a field for Media struct
/// Note: Does not represent whether the, for example, audio will be a voice message of music
/// player entry. It only cares about format. I will consider to remove it in future
#[repr(C)]
#[derive(Debug)]
pub enum MediaType{
    /// A Video
    VID,
    /// An Image
    IMG,
    /// A GIF
    GIF,
    /// A piece of audio
    AUD,
}


/// A struct to reresent coordinates of a Media entry in MediaGroup
///
/// could as well have been u32 or [u8, 4]. indexes are the following:
#[repr(C)]
#[derive(Debug)]
pub struct MediaCoordinates {
    /// x position of top left corner inside attachement
    xp: u8,
    /// y position of top left corner inside attachement
    yp: u8,
    /// x span inside attachement
    xs: u8,
    /// y span inside attachement
    ys: u8,
}

/// A struct to represent Media entry 
///
/// This struct is intended to be used as array with MediaGroup MessageType or as an individual
/// struct. It may store path to thumbnain (preview) of the image/audio/any other media type.
///
/// For example, if a Media entry is a voice message, the struct will be initialized as following:
/// ```
/// // show there is no preview for audio message (however there can be one)
/// Media{MediaType::AUD, "~/.local/...", std::ptr::null(), {0,0,1,1}}
/// ```
///
#[repr(C)]
#[derive(Debug)]
pub struct Media {
    /// The type of media. Can only be one at a time since MediaType is not a flag enum
    pub r#type: MediaType,
    /// Path to the file. Relative to the cache directory (e.g. "audio/150920203T145701.ogg"
    /// instead of "~/.local/share/GigaChat/audio/150920203T145701.ogg")
    pub path: *const i8,
    /// Path to preview image. Now it makes sense only for VID, IMG, GIF and MUS MediaType. For MUS
    /// it would be album cover, for IMG it would be path to low-res version of image. Null pointer
    /// if no preview is available
    pub preview: *const i8,
    /// coordinates of the media in group. refer to MediaCoordinates documentation for more
    /// information
    pub coordinates: MediaCoordinates,
}

#[repr(C)]
#[derive(Debug)]
pub struct MediaArrayType {
    size: usize,
    data: *const Media,
}

/// enum to represent data of any type of message.
///
/// Based on the type the Message should contain one of these
///
/// Future considerations: add `location` type
#[repr(C)]
#[derive(Debug)]
pub enum MessageData {
    Nomedia(()),
    Media(Media),
    MediaArray(MediaArrayType),
}

/// A struct to represent any type of Message.
///
/// Message can contain any number of attachements. MessageType is a flag enum and the contents of
/// the message is deterined by `type` field
#[derive(Debug)]
#[repr(C)]
pub struct Message {
    /// type of the message. use MessageType enum with BitAnd (&) to represent the contents
    pub r#type: u32,
    /// data_text is used to store raw string that the client receives with the message. can be
    /// empty.
    pub data_text: *const i8,
    /// data_media is either a Media struct or c-style array of Media structs.
    pub data_media: MessageData,
    /// ID of an author of the message
    pub sender: u64,
    /// time in UNIX seconds
    pub time: u64,
    /// time in nanoseconds excluding whole seconds (actual_nanoseconds - UNIX_SECONDS*10^9)
    pub time_ns: u64,
    /// ID of the message to which the current message is replying. 0 if this is not a reply.
    pub reply_id: u64,
}


#[no_mangle]
pub extern "C" 
fn write_to_db(m: Message) {
    dbg!(m);
}

/// Converts pointer to unowned string
/// DOES NOT OWN POINTER MEMORY!
///
/// # Arguments
/// * `ptr`: pointer to the C string
///
/// # Returns
/// result of CStr::to_str 
pub fn ptr_to_str(ptr: *const u8) -> Result<&'static str, Utf8Error> {
    let ptr: &CStr = unsafe { CStr::from_ptr(ptr as *const i8) };
    ptr.to_str()
}

/// Creates database at path `dbname`
///
/// Creates every necessary table if they do not exist (may be used to fix integrity of database)
///  
/// # Arguments 
/// * `dbname`: path to database being created ( TODO: check whether path should exist)
///
/// # Returns 
/// * i8 ( = c_char ): success status.
/// * * 0 = success
/// * * -1 (0xFF) = could not open database
/// * * any positive number = amount of errors occured duirng creating tables 
///     (this showing up is either FFI or library's fault)
///
/// # Example
/// ```cpp
/// extern "C" int8_t create_database(const char*);
/// <...>
/// std::string database_name = "~/.local/share/gigachat/gc.db"
/// if ( (int8_t errors = create_database(database_name.data())) != 0 ) 
///     handle_errors(errors);
/// proceed();
/// <...>
/// ```
#[no_mangle]
pub extern "C"
fn create_database(dbname: *const u8) -> i8 {
    let dbname = match ptr_to_str(dbname) {
        Ok(name) => name,
        Err(error) => panic!("Rust's `create_database` terminated with error: {error} due to invalid `dbname` argument"),
    };
    let db = match rusqlite::Connection::open(dbname) {
        Ok(db) => db,
        Err(_) => return -1,
    };

    let mut return_value = 0i8;
    let statements = &[ 
        sql::CREATE_USERS_TABLE, 
        sql::CREATE_ACCOUNTS_TABLE, 
        sql::CREATE_CHANNELS_TABLE, 
        sql::CREATE_MESSAGES_TABLE,
        sql::CREATE_CONFIG_TABLE
    ];

    for &i in statements {
        match db.execute(i, []) {
            Ok(_) => {},
            Err(_) => return_value += 1,
        }
    }

    return return_value;
}


