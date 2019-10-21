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

use url::{Url};

use std::fs::File;
use std::io::prelude::*;

use crate::config::Config;
use crate::data::Data;
use crate::record::Record;

pub fn read_stored_records(config: &Config, data: &mut Data) {
  let uri_s = &config.server_datastore_uri;
  if let Ok(uri) = Url::parse(uri_s) {
    match uri.scheme() {
      "file" => {
        let path = uri.path();
        // This function will return an error if path does not already exist.
        if let Ok(file) = File::open(path) {
          if path.contains(".json") {
            read_stored_records_json_file(file, data);
          }
          else {
            println!("Error: reading server_datastore_uri; unknown filetype '{}'", path);
          }
        }
      }
      "memory" => {
        // This specifies that the server should not read anything in,
        // but instead use memory only to store records.
      }
      unk => {
        println!(
          "Error reading in data: unknown scheme '{}' in given server_datastore_uri={}",
          unk, config.server_datastore_uri
        );
      }
    }
  }
}

pub fn write_stored_records(config: &Config, data: &Data) {
  let uri_s = &config.server_datastore_uri;
  if let Ok(uri) = Url::parse(uri_s) {
    match uri.scheme() {
      "file" => {
        let path = uri.path();
        // This function will create nonexisting files, and truncate existing files when data is written
        if let Ok(file) = File::create(path) {
          if path.contains(".json") {
            write_stored_records_json_file(file, data);
          }
          else {
            println!("Error: reading server_datastore_uri; unknown filetype '{}'", path);
          }
        }
      }
      "memory" => {
        // This specifies that the server should not store records,
        // but instead use memory only to store records.
      }
      unk => {
        println!(
          "Error reading in data: unknown scheme '{}' in given server_datastore_uri={}",
          unk, config.server_datastore_uri
        );
      }
    }
  }
}

pub fn read_stored_records_json_file(mut json_f: File, data: &mut Data) {
  let mut contents = String::new();
  if let Err(e) = json_f.read_to_string(&mut contents) {
    println!("read_stored_records_json_file: {}", e);
    return;
  }
  
  if let Ok(records) = serde_json::from_str::<Vec<Record>>(&contents) {
    for rec in records {
      data.insert(rec);
    }
  }
}

pub fn write_stored_records_json_file(mut json_f: File, data: &Data) {
  // TODO can we serialize without cloning everything OR without locking everything?
  let mut records = vec![];
  for pool in data.record_pools.iter() {
    let read_retries = 5;
    for _ in 0..read_retries {
      if let Ok(pool) = pool.try_read() {
        for rec in pool.iter() {
          records.push(rec.clone());
        }
        break;
      }
    }
  }
  
  let records_json_s = serde_json::to_string(
    &records
  ).expect("Cannot serialize a record");
  
  if let Err(e) = json_f.write_all(records_json_s.as_bytes()) {
    println!("Unable to write new data to db: {}", e);
  }
}

