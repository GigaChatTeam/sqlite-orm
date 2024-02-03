pub mod interface;

#[cfg(test)]
pub mod testing {
    use crate::interface;

    use super::interface::*;

    #[test]
    fn create_database() {
        unsafe { dbg!(gigachat_init("./gigachat.db\0".as_ptr())) };
        assert_eq!(gigachat_create_database(), 0);
    }

    #[test] 
    fn read_write() {
        unsafe { dbg!(gigachat_init("./gigachat.db\0".as_ptr())) };
        let m1: Message = Message {
            r#type: MessageType::TXT as u32,
            data_text: "string\0".as_ptr() as *const i8,
            data_media: MessageData::Nomedia(()),
            channel: 0,
            sender: 0,
            time: 1000000,
            time_ns: 0,
            reply_id: 0,
        };
        let m2: Message = Message {
            r#type: MessageType::TXT as u32,
            data_text: "ты пидр\0".as_ptr() as *const i8,
            data_media: MessageData::Nomedia(()),
            channel: 0,
            sender: 1,
            time: 1000001,
            time_ns: 500,
            reply_id: 0,
        };
        let messages = vec![m1, m2];
        assert_eq!(gigachat_insert_messages_to_database(messages.as_ptr(), messages.len()), 2);
    }

    #[test]
    fn clear_database() {
        unsafe { dbg!(gigachat_init("./gigachat.db\0".as_ptr())) };
        assert_eq!(gigachat_clear_database(), 0);
    }

    // #[test]
    // fn get_users() {
    //     interface::networking::load_channels(1, "justanothercatgirl\0".as_ptr(), "https://example.com\0".as_ptr())
    // }

}

