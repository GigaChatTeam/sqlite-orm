
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

// standard library
use std::ops;

// Cargo dependencies
use rusqlite::{self, params};

use self::common::ptr_to_str;
// use cpp;

// private imports
mod helper_structs;

// public re-exports
pub mod sql;
pub mod networking;
pub mod common;


/// A Struct to represent a channel inside database. 
///
/// Fields `description` and `avatar` are optional, and since this struct needs to be
/// FFI-compatible, null pointers are used instead of Option<> type
///
/// Note: This struct has 
#[repr(C)]
#[derive(Debug)]
pub struct Channel {
    /// Unique identifier of the channel
    pub id: u64,
    /// Title of the channel (C-string)
    pub title: *const u8,
    /// Description (optional, should be a null pointer in case is empty)
    pub description: *const u8,
    /// Profiule picture for the channel (optional, null pointer in case it is not present)
    pub avatar: *const u8,
    /// Creation time (UNIX seconds)
    pub created: u64,
    /// Creation time in nanoseconds (actual_nanoseconds - seconds*10^9: nanoseconds without whole
    /// seconds)
    pub created_ns: u32,
    /// Whether the listening to it is enabled (`listening` in context of GigaChat means that you
    /// can read messages and load history from it)
    pub enabled: bool,
}

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
    pub xp: u8,
    /// y position of top left corner inside attachement
    pub yp: u8,
    /// x span inside attachement
    pub xs: u8,
    /// y span inside attachement
    pub ys: u8,
}

/// A struct to represent Media entry 
///
/// This struct is intended to be used as array with MediaGroup MessageType or as an individual
/// struct. It may store path to thumbnain (preview) of the image/audio/any other media type.
///
/// For example, if a Media entry is a voice message, the struct will be initialized as following:
/// ```rust
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

/// A wrapper for storing Media as C array
///
/// Has no methods.
#[repr(C)]
#[derive(Debug)]
pub struct MediaArrayType {
    pub size: usize,
    pub data: *const Media,
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
    /// ID of a channel message was sent into
    pub channel: u64,
    /// time in UNIX seconds
    pub time: u64,
    /// time in nanoseconds excluding whole seconds (actual_nanoseconds - UNIX_SECONDS*10^9)
    pub time_ns: u64,
    /// ID of the message to which the current message is replying. 0 if this is not a reply.
    pub reply_id: u64,
}

/// Enum to represent errors that might happen when calling functions from this module\
///
/// This is not intended to be used as a return value directlry. Instead, this should be casted to
/// i32 (c_int). It contains only negatuve values because positive are reserved for counters
/// (like amount of columns affected)
///
/// NOTE: Maybe the representation type will be changed to bigger integer types, as well as 
#[repr(i32)]
pub enum DbError {
    QueryError = -6,
    StatementError = -5,
    CouldNotStartTransaction = -4,
    CoundNotEndTransaction = -3,
    InvalidCString = -2,
    CouldNotOpen = -1,
}

/// A function to panic with formatting. For internal-use only. 
fn panic_with_message<T: std::error::Error>(function: &str, err: T, reason: &str) -> ! {
    println!("Rust's `{function}` terminated with the error: `{err}` due to {reason}");
    panic!()
}

/// A function to open database
fn open_db(path: *const u8) -> Result<rusqlite::Connection, DbError> {
    let dbname = ptr_to_str(path).map_err(|_| DbError::InvalidCString)?;
    rusqlite::Connection::open(dbname).map_err(|_| DbError::CouldNotOpen)
}

#[no_mangle]
pub extern "C" 
fn write_to_db(m: Message) {
    dbg!(m);
}

/// Creates database at path `dbname`
///
/// Creates every necessary table if they do not exist (may be used to fix integrity of database)
///  
/// # Arguments 
/// * `dbname`: path to database being created ( TODO: check whether path should exist)
///
/// # Returns 
/// * i32 ( = c_int ): success status.
/// * * 0 = success
/// * * any negative number = the i32 representation of fields in DbError enum
/// * * any positive number = amount of errors occured duirng creating tables
///     (this showing up is either FFI or library's fault)
///
/// # Example
/// ```cpp
/// extern "C" int32_t create_database(const char*);
/// <...>
/// std::string database_name = "~/.local/share/GigaChat/gc.db"
/// if ( (int32_t errors = create_database(database_name.data())) != 0 ) {
///     if (errors > 0) {
///         std::cout << errors << " tables could not be created!"
///         shit_happened();
///     }
///     switch (static_cast<DbError>(error)) {
///         /* <handle each individual enum member> */
///         default: std::cerr << "how did that even happen";
///     }
/// }
/// proceed();
/// <...>
/// ```
#[no_mangle]
pub extern "C"
fn create_database(dbname: *const u8) -> i32 {
    let db = match open_db(dbname) {
        Ok(db) => db,
        Err(e) => return e as i32,
    };

    let mut return_value = 0i32;
    let statements = &[ 
        sql::create::USERS_TABLE, 
        sql::create::ACCOUNTS_TABLE, 
        sql::create::CHANNELS_TABLE, 
        sql::create::MESSAGES_TABLE,
        sql::create::CONFIG_TABLE,
        sql::create::MEDIA_LINK_TABLE,
    ];
    
    for &i in statements {
        match db.execute_batch(i) {
            Ok(_) => (),
            Err(_) => {
                dbg!(&i);
                return_value += 1
            }
        }
    }

    return return_value;
}

fn load_names(connection: &mut rusqlite::Connection) -> Result<Vec<String>, i32> {
    if let Ok(mut nm) = connection.prepare(sql::misc::GET_TABLE_NAMES) {
        match nm.query_map([], |name| name.get(0)) {
            Ok(map) => Ok(map.filter_map(|i| i.ok()).collect()),
            Err(_) => return Err(DbError::QueryError as i32),
        }
    } else { 
        return Err(DbError::StatementError as i32) 
    }
}

/// The function to delete all tables from the database, effectively clearing it up
///
/// Note: This function does not clear local cached files!
///
/// # Arguments
/// * `dbname`: path to the database file.
///
/// # Returns
/// * i32 ( = c_char ): success status
/// * * 0 = success
/// * * any negative number = i32 representation of DbError enum
/// * * any positive number = amount of tables that had errors in deletion (this indicates that there
/// is a FFI or library's fault)
///
/// # Example
/// handling return value of the clear_database function
/// ```cpp
/// #include <message_interface_cpp.h>
/// <...>
/// std::string name = "/home/user/.local/share/GigaChat/cache.db";
/// int32_t status = clear_database(name.data());
/// if (status != 0) {
///     if (status > 0) std::cout << status << " tables were not deleted successfully";
///     else {
///         switch (static_cast<DbError>(status)) {
///             /* handle errors from the DbError enum */
///             default: std::cout << "unknown error";
///         }
///     }
/// }
/// <...>
/// ```
///
#[no_mangle]
pub extern "C" 
fn clear_database(dbname: *const u8) -> i32 {
    match open_db(dbname) {
        Ok(mut db) => {
            let names: Vec<String> = match load_names(&mut db){
                Ok(name) => name,
                Err(error) => return error,
            };
            let mut transaction = match db.transaction() {
                Ok(trans) => trans,
                Err(_) => return DbError::CouldNotStartTransaction as i32,
            };

            transaction.set_drop_behavior(rusqlite::DropBehavior::Commit);
            
            let mut error_count = names.len() as i32;
            for i in names {
                match transaction.execute(sql::misc::DROP, rusqlite::params![i]) {
                    Ok(amount) => error_count -= (amount != 0) as i32,
                    Err(_) => error_count += 1,
                };
            }

            match transaction.commit() {
                Ok(_) => error_count,
                Err(_) => DbError::CoundNotEndTransaction as i32,
            }
        },
        Err(e) => e as i32,
    }
}


/// Not FFI-compatible function to insert a single message into database and cache this statement.
///
/// Should not be used outside of library. Use `insert_messages_to_database` instead.
///
/// # Arguments
/// * db: mutable borrow of database connection
/// * m: 
fn insert_single_message(db: &mut rusqlite::Connection, m: Message) ->  Result<(), Box<dyn std::error::Error>> {
    let trans = db.transaction()?;
    if m.r#type & MessageType::TXT {
        let mut stmt = trans.prepare_cached(sql::insert::MESSAGE_DATA)?;
        stmt.execute(params![
             m.channel, 
             m.sender, 
             m.time, 
             m.time_ns, 
             m.r#type, 
             ptr_to_str(m.data_text as *const u8)?
        ])?;
    }
    if let MessageData::Media(media) = m.data_media {
        let mut stmt = trans.prepare_cached(sql::insert::MEDIA)?;
        todo!("ADD CHECK FOR Message::r#type");
        todo!("IMPLEMENT MEDIA VALIDITY CHECK AND INSERTING");
        stmt.execute(params![
            
        ])?;
    }
    else if let MessageData::MediaArray(array) = m.data_media {
        todo!();
    }
    
    trans.commit()?;
    Ok(())
}

/// A function to insert any amount ofmessages into a database
///
/// # Arguments
/// * dbname: path to database (C-style string)
/// * mvec: message vector. A C-style array of `Message` structs that should be constructed in the
/// language calling this library. must have no less than `len` valid Messages.
/// * len ( in C: size_t): amount of messages in the array `mvec`. If mvec is a raw memory adderss,
/// the last message will be located at `mvec + (sizeof(Message) * (len-1))`
///
/// # Returns
/// i8: error_status
/// * any negative number = error: an i32 representation of DbError enum member
/// * any positive number = amount of inserted messages
///
/// # Example
/// <nothing here yet...>
#[no_mangle]
pub extern "C"
fn insert_messages_to_database(dbname: *const u8, mvec: *const Message, len: usize) -> i32 {
    match open_db(dbname) {
        Ok(mut connection) => {
            todo!("use function insert_single_message");
        },
        Err(_) => return DbError::CouldNotOpen as i32,
    }
}

