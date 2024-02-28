/* TODO: put license information */

#ifndef GIGACHAT_SQLITE_ORM
#define GIGACHAT_SQLITE_ORM

#pragma once

/* Generated with cbindgen:0.26.0 */

/* THIS FILE IS GENERATED AUTOMATICALLY WITH CBINDGEN. DO NOT EDIT MANUALLY. */
/* ANY CHANGES WILL BE OVERRIDEN. PLEASE MODIFY `build.rs` TO APPLY PATCHES. */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>


// enum to represent the type of media being sent/stored
typedef enum MediaType {
        // A Video
        MediaType_VID,
        // An Image
        MediaType_IMG,
        // A GIF
        MediaType_GIF,
        // A piece of audio
        MediaType_AUD,
        // Must be last for serialization purposes
        MediaType_Sentinel,
} MediaType;

// A struct to reresent coordinates of a Media entry in MediaGroup
typedef struct MediaCoordinates {
        // x position of top left corner inside attachement
        uint8_t xp;
        // y position of top left corner inside attachement
        uint8_t yp;
        // x span inside attachement
        uint8_t xs;
        // y span inside attachement
        uint8_t ys;
} MediaCoordinates;

// A struct to represent Media entry
typedef struct Media {
        // The type of media. Can only be one at a time since MediaType is not a flag enum
        enum MediaType type;
        // Path to the file. Relative to the cache directory (e.g. "audio/150920203T145701.ogg"
        const int8_t *path;
        // Path to preview image. Now it makes sense only for VID, IMG, GIF and MUS MediaType. For MUS
        const int8_t *preview;
        // coordinates of the media in group. refer to MediaCoordinates documentation for more
        struct MediaCoordinates coordinates;
} Media;

// A wrapper for storing Media as C array
typedef struct MediaArrayType {
        uintptr_t size;
        const struct Media *data;
} MediaArrayType;

// enum to represent data of any type of message.
typedef enum MessageData_Tag {
        MessageData_Nomedia,
        MessageData_Media,
        MessageData_MediaArray,
        // Must be last for serialization purposes
        MessageData_Sentinel,
} MessageData_Tag;

typedef struct MessageData {
        MessageData_Tag tag;
        union {
                struct {

                };
                struct {
                        struct Media media;
                };
                struct {
                        struct MediaArrayType media_array;
                };
        };
} MessageData;

// A struct to represent any type of Message.
typedef struct Message {
        // type of the message. use MessageType enum with BitAnd (&) to represent the contents
        uint32_t type;
        // data_text is used to store raw string that the client receives with the message. can be
        const char *data_text;
        // data_media is either a Media struct or c-style array of Media structs.
        struct MessageData data_media;
        // ID of an author of the message
        uint64_t sender;
        // ID of a channel message was sent into
        uint64_t channel;
        // time in UNIX seconds
        uint64_t time;
        // time in nanoseconds excluding whole seconds (actual_nanoseconds - UNIX_SECONDS*10^9)
        uint64_t time_ns;
        // ID of the message to which the current message is replying. 0 if this is not a reply.
        uint64_t reply_id;
} Message;

// A struct that represents an array of permissions
typedef struct Permissions {
        const uint16_t *data;
        uintptr_t size;
} Permissions;

// A Struct to represent a channel inside database.
typedef struct Channel {
        // Unique identifier of the channel
        uint64_t id;
        // Title of the channel (C-string)
        const char *title;
        // Description (optional, should be a null pointer in case is empty)
        const char *description;
        // Profiule picture for the channel (optional, null pointer in case it is not present)
        const char *avatar;
        // Creation time (UNIX seconds)
        bool enabled;
        struct Permissions permissions;
} Channel;

// A c-style array with Channels.
typedef struct ChannelArray {
        // Size of the array. If `arrays_store_errors` feature is enabled: if data is nullptr, size
        intptr_t size;
        // Size allocated by rust's Vec. You can safely construct this much
        intptr_t alloc;
        // pointer to the element at index 0. If nullptr, operation did not succed and `size` field
        struct Channel *data;
} ChannelArray;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

// Initializes the dynamic library. MUST BE CALLED BEFORE ANY OTHER FUNCTION.
 int32_t gigachatdb_init(const char *dbname) ;

// Creates database at path `dbname`
 int32_t gigachatdb_create_database(void) ;

// The function to delete all tables from the database, effectively clearing it up
 int32_t gigachatdb_clear_database(void) ;

// A function to insert any amount of messages into a database
 int32_t gigachatdb_insert_messages(const struct Message *mvec, uintptr_t len) ;

// Frees array of messages allocated by the API
 void gigachatdb_free(struct Message *ptr) ;

// A function to read messages from database
 struct Message *gigachatdb_get_messages(uint64_t channel, uintptr_t amount) ;

 void test_rust_dynamic_library(void) ;

// A function to load all channels with /user/&lt;UID&gt;/channels server request
 struct ChannelArray gigachatnw_load_channels(uint64_t uid, const char *token, const char *dlb_url) ;

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* GIGACHAT_SQLITE_ORM */


