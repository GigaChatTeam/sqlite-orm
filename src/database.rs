/* ----------------------------------------------------------------------------
   gc-orm - a library for caching and managing gigachat files
   Copyright (C) 2024 Sotov Konstantin A

   This file is part of gc-orm library.

   This library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Lesser General Public License
   as published by the Free Software Foundation; either version 3 of
   the License, or (at your option) any later version.

   This library is distributed in the hope that it will be useful, but
   WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Lesser General Public License for more details.

   You should have received a copy of the GNU Lesser General Public
   License along with this library; if not, see <http://www.gnu.org/licenses/>.   
   ----------------------------------------------------------------------------
*/


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

#![allow(clippy::similar_names)]

// standard library
use std::ffi::c_char;

// Cargo dependencies
#[cfg(feature = "multithread")]
use {
    r2d2::Pool,
    r2d2_sqlite::SqliteConnectionManager,
};
use rusqlite::{self, params};

use crate::common::ptr_to_str;
// use cpp;

// public re-exports
pub mod sql;
pub mod structs;

pub use structs::*;

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
///     gcdb_init("/home/garfield/.local/share/GigaChat/root.db");
///
///     // use any other functions from now on...
///     if (gcdb_create_database() != 0) {
///         perror("woah, that's unfortunate");
///     }
///     
///     return 0;
/// }
/// ```
#[no_mangle]
pub unsafe extern "C"
fn gcdb_init(dbname: *const c_char) -> i32 {
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
/// extern "C" int32_t gc_create_database();
/// extern "C" int32_t gcdb_init(const char*);
/// <...>
/// std::string database_name = "~/.local/share/GigaChat/gc.db"
/// gcdb_init(database_name.data())
/// if ( (int32_t errors = gc_create_database()) != 0 ) {
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
fn gcdb_create_database() -> i32 {
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
/// handling return value of the gcdb_clear_database function
/// ```cpp
/// #include <message_interface_cpp.h>
/// <...>
/// std::string name = "/home/user/.local/share/GigaChat/cache.db";
/// gcdb_init(name.data());
/// int32_t status = gcdb_clear_database();
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
fn gcdb_clear_database() -> i32 {
    match unsafe {DB_CONNECTION.as_mut()} {
        Some(db) => {
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

fn insert_single_channel(db: &mut rusqlite::Connection, c: &Channel) -> Result<usize, rusqlite::Error> {
    let trans = db.transaction()?;

    let mut result = 0usize;
    let name = ptr_to_str(c.title).ok();
    let description = ptr_to_str(c.description).ok();
    
    trans.prepare_cached(sql::insert::CHANNEL)?
        .execute(params![
            c.id,
            name,
            description,
            Option::<&str>::None,
            c.enabled,
        ])?;

    todo!()
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
    trans.prepare_cached(sql::insert::MESSAGE)?
        .execute(params![
            m.channel,
            "unnamed channel",
        ])?;

    // todo!("also insert users to the users table");

    if m.r#type & MessageType::TXT {
        let mut stmt = trans.prepare_cached(sql::insert::MESSAGE)?;
        let exec_result = stmt.execute(params![
             m.channel, 
             m.sender, 
             m.time, 
             m.time_ns, 
             m.r#type, 
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
fn gcdb_insert_messages(mvec: *const Message, len: usize) -> i32 {
    match unsafe { DB_CONNECTION.as_mut() } {
        Some(db) => {
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

/// A function to read messages from database
#[no_mangle]
pub extern "C"
fn gcdb_get_messages(channel: u64, amount: usize) -> *mut Message {
    todo!()
}

#[cfg(debug_assertions)]
#[no_mangle]
pub extern "C" 
fn test_rust_dynamic_library() {
    println!("THIS IS RUUUUUUST");
}
