/* ----------------------------------------------------------------------------
   gigachat-orm - a library for caching and managing gigachat files
   Copyright (C) 2024 Sotov Konstantin A

   This file is part of gigachat-orm library.

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

//! A module that stores and names SQL queries.
//! Should not be used as API
//! 
//! # Note
//! If a user for some reason whants to change configuration manually,
//! they will have to use SQLite client. This will probably break things,
//! so giving acces via json seems really stupid.
//!

/// cbindgen:ignore
pub mod create {
    pub const USERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    user_name TEXT NOT NULL,
    name_alias TEXT
    -- pfp_path = ":data/icons/users/<id>.ext""
);"#;
    pub const MESSAGES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS messages (
    channel INTEGER NOT NULL,
    user INTEGER NOT NULL,
    posted_unix INTEGER NOT NULL,
    posted_ns INTEGER DEFAULT 0,    -- can be empty for most cases
    type INTEGER DEFAULT 1,
    data TEXT,                      -- if type is text, the text itself, if type is media, path to cached files

    PRIMARY KEY (channel, posted_unix, posted_ns),
    FOREIGN KEY (channel) REFERENCES channels(id),
    FOREIGN KEY (user) REFERENCES users(id)
);"#;
    pub const CHANNELS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS channels (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    permissions INTEGER DEFAULT 0
    -- icon_path = ":data/icons/channels/<id>.ext"
);"#;
    pub const ACCOUNTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS accounts (
    client INTEGER PRIMARY KEY,
    secret TEXT NOT NULL,
    key TEXT NOT NULL,

    FOREIGN KEY (client) REFERENCES users (id)
);"#;
    pub const CONFIG_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS config (
    name TEXT PRIMARY KEY,
    value TEXT
);"#;
    pub const MEDIA_LINK_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS media_link (
    time_unix INTEGER NOT NULL,
    time_ns INTEGER NOT NULL,
    channel INTEGER NOT NULL,
    link TEXT NOT NULL,

    PRIMARY KEY (channel, time_unix, time_ns),
    FOREIGN KEY (channel) REFERENCES channels(id),
    FOREIGN KEY (time_unix, time_ns) REFERENCES messages(posted_unix, posted_ns)
);"#;
}

/// cbindgen:ignore
pub mod insert {
    pub const MESSAGE_DATA: &str = r#"
INSERT OR IGNORE 
    INTO messages (channel, user, posted_unix, posted_ns, type, data)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6);
"#;
    pub const MESSAGE_CHANNEL: &str = r#"
INSERT OR IGNORE
    INTO channels (id, name)
    VALUES (?1, ?2);
"#;
    pub const MEDIA: &str = r#"

"#;
}

/// cbindgen:ignore
pub mod misc {
    pub const GET_TABLE_NAMES: &str = r"SELECT name FROM sqlite_master WHERE type='table'";
    pub const DROP: &str = "DROP TABLE ";
}

// pub const : &str = r#""#;
// above serves as a snippet cause i could not configure luasnip (too damn hard)
