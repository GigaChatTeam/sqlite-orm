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
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    user_name TEXT NOT NULL,
    name_alias TEXT
    -- pfp_path = ":data/icons/users/<id>.ext""
);"#;
    pub const MESSAGES_TABLE: &str = r#"
CREATE TABLE messages (
    channel INTEGER NOT NULL,
    id INTEGER NOT NULL,
    user INTEGER NOT NULL,
    posted_unix INTEGER NOT NULL,
    type INTEGER DEFAULT 1,
    data TEXT,                      -- if type is text, the text itself, if type is media, path to cached files

    PRIMARY KEY (channel, id),
    FOREIGN KEY (channel) REFERENCES channels (id),
    FOREIGN KEY (user) REFERENCES users (id)
);"#;
    pub const CHANNELS_TABLE: &str = r#"
CREATE TABLE channels (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    avatar INTEGER,
    enabled BOOLEAN,

    FOREIGN KEY avatar REFERENCES media (cache_id)
);"#;
    pub const ACCOUNTS_TABLE: &str = r#"
CREATE TABLE accounts (
    client INTEGER PRIMARY KEY,
    secret TEXT NOT NULL,
    key TEXT NOT NULL,

    FOREIGN KEY (client) references users (id)
);"#;
    /// это для себя
    pub const CONFIG_TABLE: &str = r#"
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT
);"#;
    pub const MEDIA_TABLE: &str = r#"
CREATE TABLE media (
    cache_id INTEGER PRIMARY KEY,
    absolute_path TEXT,
);"#;
    pub const MEDIA_LINK_TABLE: &str = r#"
CREATE TABLE media_link (
    channel INTEGER NOT NULL,
    message_id INTEGER NOT NULL,
    link INTEGER NOT NULL,

    PRIMARY KEY (channel, message_id),
    FOREIGN KEY (channel, message_id) REFERENCES messages (channel, id),
    FOREIGN KEY (link) REFERENCES media (cache_id)
);"#;
    pub const PERMISSIONS_TABLE: &str = r#"
CREATE TABLE permissions (
    channel INTEGER,
    user INTEGER,
    permission INTEGER,

    PRIMARY KEY (channel, user, permission),
    FOREIGN KEY (channel) REFERENCES channels (id),
    FOREIGN KEY (user) REFERENCES users (id)
);"#;
}

/// cbindgen:ignore
pub mod insert {
    pub const MESSAGE: &str = r#"
INSERT
    INTO messages (channel, id, user, posted_unix, type, data)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
;"#;
    pub const CHANNEL: &str = r#"
INSERT
    INTO channels (id, name, description, avatar, enabled)
    VALUES (?1, ?2, ?3, ?4, ?5)
;"#;
    pub const MEDIA_ENTRY: &str = r#"
INSERT
    INTO media (absolute_path) VALUES (?1)
;"#;
    pub const MEDIA_LINK: &str = r#"
!!!TODO!!!
;"#;
    pub const PERMISSION: &str = r#"
INSERT 
    INTO permissions(channel, user, permission)
    VALUES (?1, ?2, ?3)
;"#;
}

/// cbindgen:ignore
pub mod misc {
    pub const GET_TABLE_NAMES: &str = r"SELECT name FROM sqlite_master WHERE type='table'";
    pub const DROP: &str = "DROP TABLE ";
}

// pub const : &str = r#""#;
// above serves as a snippet cause i could not configure luasnip (too damn hard)
