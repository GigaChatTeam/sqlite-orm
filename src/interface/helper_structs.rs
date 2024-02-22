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




