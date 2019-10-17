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

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::io::prelude::*;

use crate::config::Config;
use crate::config::Server;
use crate::config::ServerProtocol;
use crate::record::Record;
use crate::actions::Action;
use crate::wire::WireData;

pub fn publish_sync(config: &Config, query: &Record) {
  thread::scope(|s| {
    let mut handlers = vec![];
    
    for server in &config.servers {
      let t_server = server.clone();
      handlers.push(s.spawn(move |_| {
        publish_server_sync(config, &t_server, query);
      }));
    }
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
  
}

pub fn publish_server_sync(config: &Config, server: &Server, rec: &Record) {
  match server.protocol {
    ServerProtocol::TCP => {
      publish_tcp_server_sync(config, server, rec);
    }
    ServerProtocol::UDP => {
      std::unimplemented!()
    }
    ServerProtocol::UNIX => {
      std::unimplemented!()
    }
  }
}

pub fn publish_tcp_server_sync(_config: &Config, server: &Server, rec: &Record) {
  use std::net::TcpStream;
  
  let wire_data = WireData {
    action: Action::publish,
    record: rec.clone(),
  };
  
  let ip_and_port = format!("{}:{}", server.host, server.port);
  match TcpStream::connect(&ip_and_port) {
    Ok(mut stream) => {
      if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(1024))) {
        println!("Error setting TCP read timeout: {}", e);
      }
      if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(1024))) {
        println!("Error setting TCP write timeout: {}", e);
      }
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = stream.write(&bytes) {
          println!("Error sending WireData to server in publish_tcp_server_sync: {}", e);
        }
        // Write the terminal 0xff byte
        if let Err(e) = stream.write(&[0xff]) {
          println!("Error sending WireData to server in publish_tcp_server_sync: {}", e);
        }
      }
      // At the moment we don't expect data back from the server
    }
    Err(e) => {
      println!("Error in publish_tcp_server_sync: {}", e);
    }
  }
  
}

pub fn query_sync(config: &Config, query: &Record) -> Vec<Record> {
  let results: Arc<Mutex<Vec<Record>>> = Arc::new(Mutex::new(vec![]));
  
  thread::scope(|s| {
    let mut handlers = vec![];
    
    for server in &config.servers {
      let t_results = results.clone();
      let t_server = server.clone();
      handlers.push(s.spawn(move |_| {
        let res_v = query_server_sync(config, &t_server, query);
        if let Ok(mut t_results) = t_results.lock() {
          t_results.extend_from_slice(&res_v[..]);
        }
      }));
    }
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
  
  return Arc::try_unwrap(results).unwrap().into_inner().unwrap();
}

pub fn query_server_sync(config: &Config, server: &Server, query: &Record) -> Vec<Record> {
  match server.protocol {
    ServerProtocol::TCP => {
      return query_tcp_server_sync(config, server, query);
    }
    ServerProtocol::UDP => {
      std::unimplemented!()
    }
    ServerProtocol::UNIX => {
      std::unimplemented!()
    }
  }
}

pub fn query_tcp_server_sync(config: &Config, server: &Server, query: &Record) -> Vec<Record> {
  use std::net::TcpStream;
  
  let mut results = vec![];
  
  let wire_data = WireData {
    action: Action::query,
    record: query.clone(),
  };
  
  let ip_and_port = format!("{}:{}", server.host, server.port);
  match TcpStream::connect(&ip_and_port) {
    Ok(mut stream) => {
      if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(1024))) {
        println!("Error setting TCP read timeout: {}", e);
      }
      if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(1024))) {
        println!("Error setting TCP write timeout: {}", e);
      }
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = stream.write(&bytes) {
          println!("Error sending WireData to server in query_tcp_server_sync: {}", e);
          return vec![];
        }
        // Write the terminal 0xff byte
        if let Err(e) = stream.write(&[0xff]) {
          println!("Error sending WireData to server in query_tcp_server_sync: {}", e);
          return vec![];
        }
      }
      
      // Read results until connection is closed
      // We use 0xff ("break" stop code in the CBOR spec (rfc 7049))
      // as a deliminator because it is least likely to interfere with CBOR stuff.
      
      let mut buff = [0; 16 * 1024];
      let mut overflow_buff = vec![]; // Unused but read-in bytes are appended here
      loop {
        match stream.read(&mut buff) {
          Ok(num_read) => {
            let new_bytes_read = &buff[0..num_read];
            let mut parse_buff = overflow_buff.clone();
            parse_buff.extend_from_slice(new_bytes_read);
            let parse_buff = parse_buff; // All unparsed bytes
            // Jump to the next 0xff byte and parse data from 0..ff_i as a WireData object
            if let Some(ff_i) = parse_buff.iter().position(|&r| r == 0xff) {
              // There exists a 0xff at ff_i, use it to get the CBOR bytes
              let mut cbor_slice = vec![];
              cbor_slice.extend_from_slice(&parse_buff[0..ff_i]);
              // Remove last 0xff byte from cbor_slice
              if cbor_slice.last().eq(&Some(&0xff)) {
                cbor_slice.pop();
              }
              if let Ok(wire_res) = serde_cbor::from_slice::<WireData>(&cbor_slice) {
                //println!("wire_res={:?}", wire_res);
                match wire_res.action {
                  Action::end_of_results => {
                    break;
                  }
                  Action::result => {
                    results.push(wire_res.record);
                  }
                  unexpected => {
                    println!("Unexpected action from server, ignoring packet: {}", unexpected);
                  }
                }
              }
              // TODO improve below
              // For the moment we always throw out bytes reguardless of if they were a valid WireData or not
              overflow_buff = vec![];
              overflow_buff.extend_from_slice(&parse_buff[ff_i+1..]);
            }
            else {
              // There is no 0xff, we must read more data.
              // For the moment store all data in overflow_buff
              overflow_buff = parse_buff.clone();
            }
            
          }
          Err(e) => {
            println!("Error reading from TCP: {}", e);
            break;
          }
        }
      }
      
    }
    Err(e) => {
      println!("Error in query_tcp_server_sync: {}", e);
      return vec![];
    }
  }
  
  return results;
}
