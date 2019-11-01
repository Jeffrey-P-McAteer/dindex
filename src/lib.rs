/**
 *  dIndex - a distributed, organic, mechanical index for everything
 *  Copyright (C) 2019  Jeffrey McAteer <jeffrey.p.mcateer@gmail.com>
 *  
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 * 
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 * 
 *  You should have received a copy of the GNU General Public License along
 *  with this program; if not, write to the Free Software Foundation, Inc.,
 *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

pub mod config;
pub mod args;
pub mod record;
pub mod actions;

pub mod ext;
pub mod ext_py;

pub mod server;
pub mod server_data_io;

pub mod client;
pub mod http_client;
pub mod data;
pub mod wire;
pub mod signing;
pub mod disp;
pub mod scripting;

pub mod web_scan;

#[cfg(feature = "gui-client")]
pub mod gui_client;

#[macro_use]
extern crate cpython;

#[macro_export]
macro_rules! h_map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

#[macro_export]
macro_rules! py_attr_map_dict(
    ($py:expr, $py_dict:expr, $param_name:expr, $param_val:expr) => {
        {
            if let Err(e) = $py_dict.set_item($py, $param_name, $param_val) {
              println!("[ dindex error ] {:?}", e);
            }
        }
     };
);

#[macro_export]
macro_rules! attr_from_py_dict(
    ($py:expr, $py_dict:expr, $attr_name_s:expr, $def_attr_val:expr, $attr_type:ty) => {
        {
            let attr_capture: $attr_type;
            if let Some(py_val) = $py_dict.get_item($py, $attr_name_s) {
              if let Ok(rust_val) = py_val.extract($py) {
                attr_capture = rust_val;
              }
              else {
                attr_capture = $def_attr_val;
              }
            }
            else {
              attr_capture = $def_attr_val;
            }
            attr_capture
        }
     };
);

