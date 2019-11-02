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

use structopt::StructOpt;

use libc::c_char;
use std::ffi::CStr;

use crate::record::Record;
use crate::config;
use crate::config::Config;
use crate::args::Args;
use crate::disp;
use crate::client;

#[no_mangle]
pub extern fn dindex_args() -> *mut Args {
  Box::into_raw(Box::new( Args::from_args() ))
}

#[no_mangle]
pub extern fn dindex_args_free(args: *mut Args) {
  if args.is_null() {
    return;
  }
  unsafe {
    Box::from_raw(args);
  }
}

#[no_mangle]
pub extern fn dindex_config(args: *mut Args) -> *mut Config {
  let c = if args.is_null() {
    // default structopt calls exit() if bad args so we create empty args
    // if the given action is bad
    let a = Args::from_iter_safe( std::env::args() );
    match a {
      Ok(a) => {
        config::read_config( &a )
      }
      Err(_e) => {
        config::read_config( &Args::empty() )
      }
    }
  }
  else {
    config::read_config( unsafe{ &(*args) } )
  };
  Box::into_raw(Box::new(c))
}

#[no_mangle]
pub extern fn dindex_config_free(config: *mut Config) {
  if config.is_null() {
    return;
  }
  unsafe {
    Box::from_raw(config);
  }
}

#[no_mangle]
pub extern fn dindex_record_empty() -> *mut Record {
  Box::into_raw(Box::new( Record::empty() ))
}

#[no_mangle]
pub extern fn dindex_record_free(rec: *mut Record) {
  if rec.is_null() {
    return;
  }
  unsafe {
    Box::from_raw(rec);
  }
}


#[no_mangle]
pub extern fn dindex_record_put(rec_ptr: *mut Record, key: *const c_char, val: *const c_char) {
  if rec_ptr.is_null() || key.is_null() || val.is_null() {
      return;
  }
  let key = unsafe { CStr::from_ptr(key) }.to_string_lossy().to_string();
  let val = unsafe { CStr::from_ptr(val) }.to_string_lossy().to_string();
  unsafe {
    (*rec_ptr).p.insert(key, val);
  }
}

#[no_mangle]
pub extern fn dindex_record_display(config: *mut Config, rec_ptr: *mut Record) {
  if rec_ptr.is_null() || config.is_null() {
      println!("NULL Record/Config");
  }
  else {
      unsafe {
        disp::print_results_ref(&(*config), &vec![&(*rec_ptr)]);
      }
  }
}

type RecordVec = Vec<Record>;

#[no_mangle]
pub extern fn dindex_record_vec_free(rec: *mut RecordVec) {
  if rec.is_null() {
    return;
  }
  unsafe {
    Box::from_raw(rec);
  }
}

#[no_mangle]
pub extern fn dindex_record_display_vec(config: *mut Config, rec_ptr: *mut RecordVec) {
  if rec_ptr.is_null() || config.is_null() {
      println!("NULL Record/Config");
  }
  else {
      unsafe {
        disp::print_results(&(*config), &(*rec_ptr));
      }
  }
}

#[no_mangle]
pub extern fn dindex_client_query_sync(config: *mut Config, rec_ptr: *mut Record) -> *mut RecordVec {
  if config.is_null() || rec_ptr.is_null() {
    return std::ptr::null_mut();
  }
  else {
    unsafe {
      let results = client::query_sync(&(*config), &(*rec_ptr));
      Box::into_raw(Box::new( results ))
    }
  }
}

#[no_mangle]
pub extern fn dindex_client_publish_sync(config: *mut Config, rec_ptr: *mut Record) {
  if config.is_null() || rec_ptr.is_null() {
    return;
  }
  else {
    unsafe {
      client::publish_sync(&(*config), &(*rec_ptr));
    }
  }
}

// Return should map to a ListenAction
type ListenCallback = extern "C" fn(*mut Record) -> *const c_char;

#[no_mangle]
pub extern fn dindex_client_listen_sync(config: *mut Config, rec_ptr: *mut Record, callback: ListenCallback) {
  if config.is_null() || rec_ptr.is_null() {
    return;
  }
  else {
    unsafe {
      client::listen_sync(&(*config), &(*rec_ptr), |rec| {
        let cstr_action = callback(Box::into_raw(Box::new(rec)));
        if ! cstr_action.is_null() {
          let str_action = CStr::from_ptr(cstr_action).to_string_lossy();
          return client::ListenAction::parse(&str_action);
        }
        else {
          return client::ListenAction::EndListen;
        }
      });
    }
  }
}
