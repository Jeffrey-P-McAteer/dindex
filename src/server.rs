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

use crossbeam_utils::thread;
use url::{Url};

use std::fs::File;
use std::io::prelude::*;

use crate::config::Config;
use crate::data::Data;
use crate::record::Record;
use crate::wire::WireData;
use crate::actions::Action;

pub fn run_sync(config: &Config) {
  let mut data = Data::new(config);
  read_stored_records(config, &mut data);
  let data = data;
  
  thread::scope(|s| {
    let mut handlers = vec![];
    
    handlers.push(s.spawn(|_| {
      run_tcp_sync(config, &data);
    }));
    
    handlers.push(s.spawn(|_| {
      run_udp_sync(config, &data);
    }));
    
    handlers.push(s.spawn(|_| {
      run_unix_sync(config, &data);
    }));
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
}

fn run_tcp_sync(config: &Config, data: &Data) {
  use std::net::TcpListener;
  use std::collections::VecDeque;
  
  let ip_port = format!("{}:{}", config.server_ip, config.server_port);
  println!("tcp starting on {}", &ip_port);
  
  match TcpListener::bind(&ip_port) {
    Ok(listener) => {
      thread::scope(|s| {
        let mut handlers = VecDeque::new();
        handlers.reserve_exact(config.server_threads_in_flight + 4);
        
        for stream in listener.incoming() {
          handlers.push_back(s.spawn(|_| {
            handle_tcp_conn(stream, config, data);
          }));
          // Housekeeping
          if handlers.len() > config.server_threads_in_flight {
            let threads_to_join = (handlers.len() as f64 * config.server_threads_in_flight_fraction) as usize;
            // Pop up to threads_to_join thread handles and join on them
            for _ in 0..threads_to_join {
              if let Some(h) = handlers.pop_front() {
                if let Err(e) = h.join() {
                  println!("Error joining TCP thread: {:?}", e);
                }
              }
            }
          }
        }
        
        for h in handlers {
          h.join().unwrap();
        }
        
      }).unwrap();
    }
    Err(e) => {
      println!("Error starting TCP server: {}", e);
    }
  }
}

fn run_udp_sync(config: &Config, data: &Data) {
  println!("udp starting on 0.0.0.0:{}", config.server_port);
  
}

fn run_unix_sync(config: &Config, data: &Data) {
  
}

fn read_stored_records(config: &Config, data: &mut Data) {
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
      unk => {
        println!(
          "Error reading in data: unknown scheme '{}' in given server_datastore_uri={}",
          unk, config.server_datastore_uri
        );
      }
    }
  }
}

fn write_stored_records(config: &Config, data: &mut Data) {
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
      unk => {
        println!(
          "Error reading in data: unknown scheme '{}' in given server_datastore_uri={}",
          unk, config.server_datastore_uri
        );
      }
    }
  }
}

fn read_stored_records_json_file(mut json_f: File, data: &mut Data) {
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

fn write_stored_records_json_file(mut json_f: File, data: &mut Data) {
  // TODO can we serialize without cloning everything OR without locking everything?
  let mut records = vec![];
  for pool in data.record_pools.iter() {
    let mut read_retries = 5;
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

fn handle_tcp_conn(stream: Result<std::net::TcpStream, std::io::Error>, config: &Config, data: &Data) {
  use std::time::Duration;
  use std::sync::{Arc, Mutex};
  use crate::h_map;
  
  if let Ok(mut stream) = stream {
    if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(1024))) {
      println!("Error setting TCP read timeout: {}", e);
    }
    if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(1024))) {
      println!("Error setting TCP write timeout: {}", e);
    }
    // Clients will send a WireData object and then 0xff ("break" stop code in the CBOR spec (rfc 7049))
    let mut complete_buff: Vec<u8> = vec![];
    let mut buff = [0; 4 * 1024];
    while ! complete_buff.contains(&0xff) {
      match stream.read(&mut buff) {
        Ok(num_read) => {
          complete_buff.extend_from_slice(&buff[0..num_read]);
        }
        Err(e) => {
          println!("Error reading TCP data from client: {}", e);
          return;
        }
      }
    }
    // Remove last 0xff byte from complete_buff
    if complete_buff.last().eq(&Some(&0xff)) {
      complete_buff.pop();
    }
    
    // Parse bytes to WireData
    match serde_cbor::from_slice::<WireData>(&complete_buff[..]) {
      Err(e) => {
        println!("Error reading WireData from TCP client: {}", e);
        return;
      }
      Ok(wire_data) => {
        if config.is_debug() {
          println!("got connection, wire_data={:?}", wire_data);
        }
        match wire_data.action {
          Action::query => {
            let ts_stream = Arc::new(Mutex::new(stream));
            data.search_callback(&wire_data.record.create_regex_map(), |result| {
              let wire_data = WireData {
                action: Action::result,
                record: result.clone(),
              };
              if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
                if let Ok(mut stream) = ts_stream.lock() {
                  if let Err(e) = stream.write(&bytes) {
                    println!("Error sending result to TCP client: {}", e);
                    return false; // stop querying, client has likely exited
                  }
                  // Write packet seperation byte
                  if let Err(e) = stream.write(&[0xff]) {
                    println!("Error sending result to TCP client: {}", e);
                    return false; // stop querying, client has likely exited
                  }
                }
              }
              // We can never have an amplification attack via TCP,
              // so we do not limit the search space.
              return true;
            });
            // Tell clients connection should be closed
            let wire_data = WireData {
              action: Action::end_of_results,
              record: Record::empty()
            };
            if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
              if let Ok(mut stream) = ts_stream.lock() {
                if let Err(e) = stream.write(&bytes) {
                  println!("Error sending result to TCP client: {}", e);
                }
                // Write packet seperation byte
                if let Err(e) = stream.write(&[0xff]) {
                  println!("Error sending result to TCP client: {}", e);
                }
              }
            }
          }
          Action::publish => {
            std::unimplemented!()
          }
          Action::listen => {
            std::unimplemented!()
          }
          unk => {
            println!("Error: unknown action {}", unk);
            return;
          }
        }
      }
    }
    
  }
}


