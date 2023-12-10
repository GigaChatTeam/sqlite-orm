#![allow(dead_code)] // this is a temporary measure

/// A module that stores and names SQL queries
pub const CREATE_USERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    user_name TEXT NOT NULL,
    name_alias TEXT
    -- pfp_path = ":data/icons/users/<id>.ext"
);"#;
    pub const CREATE_MESSAGES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS messages (
    channel INTEGER NOT NULL,
    user INTEGER NOT NULL,
    posted_ms INTEGER NOT NULL,     -- TODO: maybe change to integers (seconds & nanoseconds)
    type TEXT DEFAULT "text",
    data TEXT,                      -- if type is text, the text itself, if type is media, path to cached files

    PRIMARY KEY (channel, posted_ms),
    FOREIGN KEY (channel) REFERENCES channels(id),
    FOREIGN KEY (user) REFERENCES users(id)
);"#;
    pub const CREATE_CHANNELS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS channels (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    permissions INTEGER DEFAULT 0
    -- icon_path = ":data/icons/channels/<id>.ext"
);"#;
    pub const CERATE_ACCOUNTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS accounts (
    client INTEGER PRIMARY KEY,
    secret TEXT NOT NULL,
    key TEXT NOT NULL,
    FOREIGN KEY (client) REFERENCES users (id)
);"#;
    /// If a user for some reason whants to change configuration manually,
    /// they will have to use SQLite client. This will probably break things,
    /// so giving acces via json seems really stupid.
    pub const CREATE_CONFIG_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS config (
    name TEXT PRIMARY KEY,
    value TEXT
);"#;
// pub const : &str = r#""#;
// above serves as a snippet cause i could not configure luasnip (too damn hard)
