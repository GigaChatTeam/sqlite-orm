pub mod interface;

#[cfg(test)]
pub mod testing {
    use crate::interface;

    use super::interface::*;
    #[test]
    fn test_of_test() {
        println!("test launched!");
        assert_eq!(2+2, 4);
        let _x = 69;
    }
    #[test] 
    fn read_write() {
        let m: Message = Message {
            r#type: MessageType::TXT as u32,
            data_text: "string".as_ptr() as *const _ as *const i8,
            data_media: MessageData::Nomedia(()),
            sender: 0,
            time: 0,
            time_ns: 0,
            reply_id: 0,
        };
        dbg!(m);
    }

    #[test]
    fn get_users() {
        interface::networking::load_channels(1, "justanothercatgirl\0".as_ptr(), "https://example.com\0".as_ptr())
    }

    #[test]
    fn creation() {
        assert_eq!(create_database("/home/main/.local/share/gigaqt/gigaqt.db\0".as_ptr()), 0);
    }

}

