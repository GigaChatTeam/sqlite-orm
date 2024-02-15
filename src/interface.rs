
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
//! # Less important note
//! All of the functions from this module are not thread-safe without feature `multithread` enabled.
//! It is recommended to enable it for your project unless you want to have issues with absolutely
//! random crashes, segfaults and database corruptions. Errors found during tests without
//! multithrerad feature enabled:
//! * signal 11, SIGSEGV. Random, hard to reproduce, cause unknown (probably something inside
//! sqlite3.so)
//! * signal 6,  SIGABRT. 2 Possible causes: rusqlite panicking with BorrowMutError or unknown
//! cause (not on the rust side)
//! Note that you can use this module without the feature in a multithreaded application,
//! however you NEED to make sure that the shared library code is ONLY ever called from a single 
//! thread.
//!
//! # Example
//! {{ Not yet provided }}
//!

// standard library
use std::ops;
use std::ffi::c_char;

// Cargo dependencies
#[cfg(feature = "multithread")]
use {
    r2d2::Pool,
    r2d2_sqlite::SqliteConnectionManager,
};
use rusqlite::{self, params};

use self::common::ptr_to_str;
// use cpp;

// private imports
mod helper_structs;

// public re-exports
pub mod sql;
pub mod networking;
pub mod common;
mod urlbuilder;

/// A Struct to represent a channel inside database. 
///
/// Fields `description` and `avatar` are optional, and since this struct needs to be
/// FFI-compatible, null pointers are used instead of Option<> type
///
/// Note: This struct has {{documentation was accidentally lost somewhere in commits but
/// should have been nothing important. Maybe todo: find the deleted docs}}
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
    /// will probably have strange behaviour
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
/// TODO: Remove from the source completely.
/// for now marked as deprecated 
#[deprecated(since = "0.0.0", note = "NO ONE USE THIS ONE GUYS")]
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
/// struct. It may store path to thumbnail (preview) of the image/audio/any other media type.
///
/// For example, if a Media entry is a voice message, the struct will be initialized as following:
/// ```rs
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
/// The core of the API. Every message should be represented by this struct and every Message is
/// writable to database. Database also returns type "message". Overall, a really important guy
/// here.
///
/// # Note:
/// Message can contain any number of attachements. MessageType is a flag enum and the contents of
/// the message is deterined by `type` field
#[derive(Debug)]
#[repr(C)]
pub struct Message {
    /// type of the message. use MessageType enum with BitAnd (&) to represent the contents
    pub r#type: u32,
    /// data_text is used to store raw string that the client receives with the message. can be
    /// empty.
    pub data_text: *const c_char,
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

/// Enum to represent errors that might occur when calling functions from this module (database
/// functions)
///
/// This is not intended to be used as a return value directlry. Instead, this should be casted to
/// i32 (c_int). It contains only negatuve values because positive are reserved.
///
/// The reserved valued are defined as following:
/// * negative value: an error from this enum
/// * 0: success
/// * positive number: status report defined by the function (e.g. amount of inserted rows or
/// sometimes amount of errors. Refer to the documentation of specific function)
///
#[repr(i32)]
pub enum DbError {
    /// Unimplemented error handling
    UnknownError = -255,
    /// This one will be returned if an underlying networking function return any error.
    /// The error from there is not mapped in any way beacuse this should not ever happen
    NetworkModuleError = -10,
    /// Only when feature 'multithread' is enabled. This means that somehow connection pool could
    /// Not read file or reate new file descriptor. 
    ConnectionPoolError = -9,
    /// SQLite statement failed to execute (runtime SQL error)
    SqliteFailure = -8,
    /// Only for init function. Means that function gigachat_init was called multiple times.
    AlreadyInitialized = -7,
    /// function gigachat_init was not called before using other database-related functions.
    Uninitialized = -6,
    /// SQL syntax error (malformed query or something). If it occurs with any function from this
    /// library that does not involve you writing yout own SQL, please open an issue, it is
    /// (probably) developer's fault.
    StatementError = -5,
    /// `rusqlite::Connection::transaction` returned Err somehow
    CouldNotStartTransaction = -4,
    /// `rusqlite::Transaction::commit()` returned Err somehow
    CoundNotEndTransaction = -3,
    /// The C string could not be parsed into rust string (usually invalid UTF-8)
    InvalidCString = -2,
    /// Could not open the database in the init function. 
    CouldNotOpen = -1,
}


/// A function to open database
#[cfg(not(feature = "multithread"))]
fn open_db(path: *const c_char) -> Result<rusqlite::Connection, DbError> {
    let dbname = ptr_to_str(path).map_err(|_| DbError::InvalidCString)?;
    rusqlite::Connection::open(dbname).map_err(|_| DbError::CouldNotOpen)
}

/// A function to open database (multithread variant)
#[cfg(feature = "multithread")]
fn open_db(path: *const c_char) -> Result<SqliteConnectionManager, DbError> {
    let dbname = ptr_to_str(path).map_err(|_| DbError::InvalidCString)?;
    Ok(SqliteConnectionManager::file(dbname))
}

/// The main object for this library. Important: not thread-safe! Thread-safe variant is presented
/// below under #[cfg(feature = "multithread")]
#[cfg(not(feature = "multithread"))]
/// cbindgen:ignore
pub static mut DB_CONNECTION: Option<rusqlite::Connection> = None;
/// The thread-safe vatiant for the main object in this library
#[cfg(feature = "multithread")]
/// cbindgen:ignore
pub static mut DB_CONNECTION: Option<Pool<SqliteConnectionManager>> = None;

/// Initializes the dynamic library. MUST BE CALLED BEFORE ANY OTHER FUNCTION.
///
/// What this funciton does is that it effectively creates all of the necessary global variables
/// (only DB_CONNECTION at the moment). All of the global variables are of Option type and by
/// default are set to None. If the function succeedes, it turns them into Some() or
/// whatever they are supposed to be
///
/// # Arguments
/// * dbname: a C-string where the database should be opened
///
/// # Returns
/// * 0 on success
/// * DbError::Uninitialized (-7) as i32 if the file can not be read
/// * DbError::AlreadyInitialized if the function is called more than once
/// * other errors from DbError enum (like InvalidCString) if they can occur
///
/// # Example 
/// ```C
/// #include <message-db-interface.h> // may be changed in future
///
/// <...>
///
/// int main() {
///     gigachat_database_interface_init("/home/garfield/.local/share/GigaChat/root.db");
///
///     // use any other functions from now on...
///     if (gigachat_create_database() != 0) {
///         perror("woah, that's unfortunate");
///     }
///     
///     return 0;
/// }
/// ```
#[no_mangle]
pub unsafe extern "C"
fn gigachat_init(dbname: *const c_char) -> i32 {
    if DB_CONNECTION.is_some() {
        return DbError::AlreadyInitialized as i32;
    }
    DB_CONNECTION = match open_db(dbname) {
        Ok(db) => {
            #[cfg(not(feature = "multithread"))]
            {
                Some(db)
            }
            #[cfg(feature = "multithread")]
            {
                let x = Pool::new(db);
                if x.is_ok() {
                    Some(x.unwrap())
                } else {
                    return DbError::CouldNotOpen as i32;
                }
            }
        },
        Err(e) => return e as i32,
    };
    0
}

/// Creates database at path `dbname`
///
/// Creates every necessary table if they do not exist (may be used to fix integrity)
///  
/// # Arguments 
/// None
///
/// # Returns 
/// * i32 ( = c_int ): success status.
/// * * any negative number = the i32 representation of fields in DbError enum
/// * * any positive number = amount of successfully created tables
///
/// # Example
/// ```cpp
/// extern "C" int32_t gigachat_create_database();
/// extern "C" int32_t gigachat_database_interface_init(const char*);
/// <...>
/// std::string database_name = "~/.local/share/GigaChat/gc.db"
/// gigachat_database_interface_init(database_name.data())
/// if ( (int32_t errors = gigachat_create_database()) != 0 ) {
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
fn gigachat_create_database() -> i32 {
    let db = match unsafe { DB_CONNECTION.as_mut() } {
        Some(a) => a,
        None => return DbError::Uninitialized as i32,
    };

    let statements = &[ 
        sql::create::USERS_TABLE, 
        sql::create::ACCOUNTS_TABLE, 
        sql::create::CHANNELS_TABLE, 
        sql::create::MESSAGES_TABLE,
        sql::create::CONFIG_TABLE,
        sql::create::MEDIA_LINK_TABLE,
    ];

    #[cfg(feature = "multithread")]
    let db = match db.get() {
        Ok(c) => c,
        Err(_) => return DbError::ConnectionPoolError as i32,
    };

    let mut return_value = 0i32;
    for i in statements {
        match db.execute_batch(i) {
            Ok(_) => { return_value += 1 },
            Err(e) => { dbg!(e); },
        }
    }
    return return_value;
}

fn load_names(connection: &mut rusqlite::Connection) -> Result<Vec<String>, i32> {
    if let Ok(mut nm) = connection.prepare(sql::misc::GET_TABLE_NAMES) {
        match nm.query_map([], |name| name.get(0)) {
            Ok(map) => Ok(map.filter_map(|i| i.ok()).collect()),
            Err(_) => return Err(DbError::SqliteFailure as i32),
        }
    } else { 
        return Err(DbError::StatementError as i32) 
    }
}

/// The function to delete all tables from the database, effectively clearing it up
///
///
/// Note: This function does not clear local cached files! (as if you have alreadyy added support
/// for this LMAO)
///
/// # Arguments
/// None
///
/// # Returns
/// * i32 ( = c_char ): success status
/// * * any negative number = i32 representation of DbError enum
/// * * any positive number = amount of successfully deleted tables
///
/// # Example
/// handling return value of the gigachat_clear_database function
/// ```cpp
/// #include <message_interface_cpp.h>
/// <...>
/// std::string name = "/home/user/.local/share/GigaChat/cache.db";
/// gigachat_database_interface_init(name.data());
/// int32_t status = gigachat_clear_database();
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
fn gigachat_clear_database() -> i32 {
    match unsafe {DB_CONNECTION.as_mut()} {
        Some(mut db) => {
            #[cfg(feature = "multithread")]
            let mut db = match db.get() {
                Ok(c) => c,
                Err(_) => return DbError::ConnectionPoolError as i32,
            };
            let names: Vec<String> = match load_names(&mut db){
                Ok(name) => name,
                Err(error) => return error,
            };
            let mut transaction = match db.transaction() {
                Ok(trans) => trans,
                Err(_) => return DbError::CouldNotStartTransaction as i32,
            };

            transaction.set_drop_behavior(rusqlite::DropBehavior::Commit);
            
            let mut return_value = 0i32;
            for i in names {
                let query = format!("{}{};", sql::misc::DROP, i);
                if transaction.execute_batch(query.as_str()).is_ok() {
                    return_value += 1;
                };
            }

            match transaction.commit() {
                Ok(_) => return_value,
                Err(_) => DbError::CoundNotEndTransaction as i32,
            }
        },
        None => DbError::Uninitialized as i32,
    }
}


/// Not FFI-compatible function to insert a single message into database and cache the statement.
///
/// Should not be used outside of library. Use `insert_messages_to_database` instead.
///
/// # Arguments
/// * db: mutable borrow of database connection
/// * m: Message to be inserted
///
/// # Returns
/// * Result<usize, rusqlite::error>
/// * Ok(usize): amount of rows changed
/// * Err(rusqlite::Error): Error propagation from rusqlite library
///
fn insert_single_message(db: &mut rusqlite::Connection, m: &Message) ->  Result<usize, rusqlite::Error> {
    let mut result = 0usize;
    let trans = db.transaction()?;

    // todo!("load channel if it does not exist");
    trans.prepare_cached(sql::insert::MESSAGE_CHANNEL)?
        .execute(params![
            m.channel,
            "unnamed channel",
        ])?;

    // todo!("also insert users to the users table");

    if m.r#type & MessageType::TXT {
        let mut stmt = trans.prepare_cached(sql::insert::MESSAGE_DATA)?;
        let exec_result = stmt.execute(params![
             m.channel, 
             m.sender, 
             m.time, 
             m.time_ns, 
             m.r#type, 
             ptr_to_str(m.data_text)?
        ]);
        if let Ok(count) = exec_result { result += count };
    }

    if let MessageData::Media(media) = &m.data_media {
        todo!("ADD CHECK FOR Message::r#type");
    }
    else if let MessageData::MediaArray(array) = &m.data_media {
        todo!();
    }
    
    trans.commit()?;
    Ok(result)
}

/// A function to insert any amount of messages into a database
///
/// # Arguments
/// * mvec: message vector. A C-style array of `Message` structs that should be constructed on the
/// user-side (the api treats memory as read-only). must have at least `len` valid Messages
/// * len (in C: size_t): amount of messages in the array `mvec`. If mvec is a raw memory adderss (void pointer),
/// the last message will be located at `mvec + (sizeof(Message) * (len-1))`
///
/// # Returns
/// i8: error_status
/// * any negative number = error: an i32 representation of DbError enum member
/// * any positive number = amount of inserted messages
///
/// # Example
/// {{ nothing here yet... please forgive the developer, he gets no sleep at all :'( }}
#[no_mangle]
pub extern "C"
fn gigachat_insert_messages(mvec: *const Message, len: usize) -> i32 {
    match unsafe { DB_CONNECTION.as_mut() } {
        Some(mut db) => {
            let mut count = 0i32;
            #[cfg(feature = "multithread")]
            let mut db = match db.get() {
                Ok(c) => c,
                Err(_) => return DbError::ConnectionPoolError as i32,
            };

            for i in unsafe { std::slice::from_raw_parts(mvec, len) } {
                match insert_single_message(&mut db, i) {
                    Ok(c) => count += c as i32,
                    Err(e) => match e {
                        rusqlite::Error::SqliteFailure(_, _) => return DbError::SqliteFailure as i32,
                        _ => return DbError::UnknownError as i32,
                    }
                }
            }
            count
        },
        None => DbError::Uninitialized as i32,
    }
}


/// Frees array of messages allocated by the API 
#[no_mangle]
pub unsafe extern "C"
fn gigachat_free(ptr: *mut Message) {
    todo!()
}

/// A function to read messages from database
#[no_mangle]
pub extern "C"
fn gigachat_get_messages(channel: u64, amount: usize) -> *mut Message {
    todo!()
}

#[cfg(feature = "multithread")]
#[no_mangle]
pub extern "C"
fn test(_a: i32) {}

#[cfg(not(feature = "multithread"))]
#[no_mangle]
pub extern "C"
fn test(_a: i32, _b: f64) {}
