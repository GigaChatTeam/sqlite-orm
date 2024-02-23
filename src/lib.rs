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

#![feature(vec_into_raw_parts)]
pub mod database;
pub mod networking;
pub mod common;

#[cfg(test)]
pub mod testing {
    use std::ffi::{CString, c_char};
    use std::time::SystemTime;
    #[cfg(feature = "multithread")]
    use std::thread::JoinHandle;

    use super::database::*;
    use rand::Rng;
    use random::{self, Source};

    #[test]
    fn create_database() {
        unsafe { dbg!(gigachat_init("./gigachat.db\0".as_ptr() as *const c_char as *const c_char)) };
        assert_eq!(gigachat_create_database(), 6);
    }

    // helper function
    fn gen_rand_msg(gen: &mut random::Xorshift128Plus, x: &CString) -> Message {
        Message {
            r#type: MessageType::TXT as u32,
            data_text: x.as_ptr() as *const c_char,
            data_media: MessageData::Nomedia(()),
            channel: gen.read_u64() % 100,
            sender: gen.read_u64() % 100,
            time: gen.read_u64() % 10000000,
            time_ns: gen.read_u64() % 1000,
            reply_id: 0,
        }
    }

    // write a single message
    #[test]
    fn write_1() {
        unsafe { dbg!(gigachat_init("./gigachat.db\0".as_ptr() as *const c_char)) };
        let m1: Message = Message {
            r#type: MessageType::TXT as u32,
            data_text: "string\0".as_ptr() as *const c_char as *const i8,
            data_media: MessageData::Nomedia(()),
            channel: 0,
            sender: 0,
            time: 1000000,
            time_ns: 0,
            reply_id: 0,
        };
        let messages = vec![m1];
        assert!(gigachat_insert_messages(messages.as_ptr(), messages.len()) >= 0);
    }

    // write a single message from multiple threads
    #[cfg(feature = "multithread")]
    #[test]
    fn write_multithread() {
        unsafe { dbg!(gigachat_init("./gigachat.db\0".as_ptr() as *const c_char)) };
        let mut threads: Vec<JoinHandle<_>> = vec![];
        for i in 1..100 {
            threads.push(std::thread::spawn( move || {
                let mut gen = random::default(rand::thread_rng().gen());
                let x = CString::new(format!("{} N. {} | {}", "multithread_write", i, gen.read_u64())).unwrap();
                let m1 = gen_rand_msg(&mut gen, &x);
                let messages = vec![m1];
                assert_eq!(gigachat_insert_messages(messages.as_ptr(), messages.len()), 1);
            }));
        }
        for t in threads {
            t.join().unwrap();
        }
    }

    // write messages one-by-one in a loop
    #[test]
    fn write_loop() {
        unsafe { dbg!(gigachat_init("./gigachat.db\0".as_ptr() as *const c_char)) };
        let mut amount = 0i32;
        let mut gen = random::default(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        for i in 1..100 {
            let x = CString::new(format!("{} N. {} | {}", "loop_write", i, gen.read_u64())).unwrap();
            let m1 = gen_rand_msg(&mut gen, &x);
            let messages = vec![m1];
            amount += gigachat_insert_messages(messages.as_ptr(), messages.len());
        }
        assert_eq!(amount, 99);
    }

    // write messages as an array
    #[test]
    fn write_array() {
        unsafe { dbg!(gigachat_init("./gigachat.db\0".as_ptr() as *const c_char)) };
        let mut gen = random::default(rand::thread_rng().gen());
        let x = CString::new(format!("{} | {}", "array_write", gen.read_u64())).unwrap();
        let messages: Vec<Message> = std::iter::repeat_with( || gen_rand_msg(&mut gen, &x) )
            .take(100)
            .collect();
        assert_eq!(gigachat_insert_messages(messages.as_ptr(), messages.len()), 100);
    }

    #[test]
    fn clear_database() {
        unsafe { dbg!(gigachat_init("./gigachat.db\0".as_ptr() as *const c_char)) };
        assert_eq!(gigachat_clear_database(), 0);
    }

    #[test] 
    fn load_channels() {
        super::networking::load_channels(0, std::ptr::null(), std::ptr::null());
    }

}
