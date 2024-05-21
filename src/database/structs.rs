use std::ops;
use std::ffi::c_char;

/// A struct that represents an array of permissions
///
/// # Note
/// This struct is not tied to a specific user or channel. This struct should be used in a context
/// of a user-channel pair, otherwise it makes no sense.
#[repr(C)]
#[derive(Debug)]
pub struct Permissions {
    pub data: *const u16,
    pub size: usize,
}

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
    pub title: *const c_char,
    /// Description (optional, should be a null pointer in case is empty)
    pub description: *const c_char,
    /// Profiule picture for the channel (optional, null pointer in case it is not present)
    pub avatar: *const c_char,
    /// Creation time (UNIX seconds)
    /*
    pub created: u64,
    /// Creation time in nanoseconds (actual_nanoseconds - seconds*10^9: nanoseconds without whole
    /// seconds)
    pub created_ns: u32,
    /// Whether the listening to it is enabled (`listening` in context of GigaChat means that you
    /// can read messages and load history from it)
    */
    pub enabled: bool,
    pub permissions: Permissions,
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
    MediaArray(*mut Media),
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
    /// Id of the message inside channel 
    pub id: u64,
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
    /// ID of the message to which the current message is replying. 0 if this is not a reply.
    pub reply_id: *mut u64,
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


