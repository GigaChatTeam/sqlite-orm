
use microserde::{Serialize, Deserialize};

use std::ptr::null as nullptr;

use crate::interface::common::*;
use crate::interface;


trait IntoCStyle<T> {
    fn into_c(&self) -> T;
}

#[derive(Serialize, Deserialize)]
pub struct ChannelDeser {
    id: u64,
    title: String,
    description: Option<String>,
    avatar: Option<String>,
    created: u64,
    created_ns: u32,
    enabled: bool,
} 

impl IntoCStyle<interface::Channel> for ChannelDeser {
    fn into_c(&self) -> interface::Channel {
        interface::Channel { 
            id: self.id,
            title: str_to_ptr(self.title.clone()).unwrap(),
            description: self.description.clone()
                .map_or(nullptr(), |s| {
                    str_to_ptr(s).unwrap_or(nullptr())
                }),
            avatar: self.avatar.clone()
                .map_or(nullptr(), |s| {
                    str_to_ptr(s).unwrap_or(nullptr())
                }),
            created: self.created, 
            created_ns: self.created_ns,
            enabled: self.enabled,
        }
    }
}




