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
use std::time::{Duration};
use std::io::prelude::*;

use crate::config::Config;
use crate::config::Server;
use crate::config::ServerProtocol;
use crate::record::Record;
use crate::actions::Action;
use crate::wire::WireData;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ListenAction {
  Continue, EndListen
}

impl ListenAction {
  pub fn parse(s: &str) -> ListenAction {
    if s == "Continue" {
      ListenAction::Continue
    }
    else if s == "EndListen" {
      ListenAction::EndListen
    }
    else {
      ListenAction::EndListen
    }
  }
}

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
      publish_udp_server_sync(config, server, rec);
    }
    ServerProtocol::UNIX => {
      publish_unix_server_sync(config, server, rec);
    }
    ServerProtocol::WEBSOCKET => {
      publish_websocket_server_sync(config, server, rec);
    }
    ServerProtocol::MULTICAST => {
      publish_udp_server_sync(config, server, rec);
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
      if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting TCP read timeout: {}", e);
      }
      if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(256))) {
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

pub fn publish_udp_server_sync(_config: &Config, server: &Server, rec: &Record) {
  use std::net::UdpSocket;
  
  let wire_data = WireData {
    action: Action::publish,
    record: rec.clone(),
  };
  
  let server_ip_and_port = format!("{}:{}", server.host, server.port);
  
  match UdpSocket::bind("0.0.0.0:0") {
    Ok(socket) => {
      if let Err(e) = socket.set_read_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting UDP read timeout: {}", e);
      }
      if let Err(e) = socket.set_write_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting UDP write timeout: {}", e);
      }
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = socket.send_to(&bytes, &server_ip_and_port) {
          println!("Error sending WireData to server in publish_udp_server_sync: {}", e);
        }
        // Write the terminal 0xff byte
        if let Err(e) = socket.send_to(&[0xff], &server_ip_and_port) {
          println!("Error sending WireData to server in publish_udp_server_sync: {}", e);
        }
      }
      // At the moment we don't expect data back from the server
    }
    Err(e) => {
      println!("Error in publish_udp_server_sync: {}", e);
    }
  }
  
}

pub fn publish_unix_server_sync(_config: &Config, server: &Server, rec: &Record) {
  use std::os::unix::net::UnixStream;
  
  let wire_data = WireData {
    action: Action::publish,
    record: rec.clone(),
  };
  
  match UnixStream::connect(&server.path) {
    Ok(mut stream) => {
      if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting TCP read timeout: {}", e);
      }
      if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting TCP write timeout: {}", e);
      }
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = stream.write(&bytes) {
          println!("Error sending WireData to server in publish_unix_server_sync: {}", e);
        }
        // Write the terminal 0xff byte
        if let Err(e) = stream.write(&[0xff]) {
          println!("Error sending WireData to server in publish_unix_server_sync: {}", e);
        }
      }
      // At the moment we don't expect data back from the server
    }
    Err(e) => {
      println!("Error in publish_unix_server_sync: {}", e);
    }
  }
  
}


pub fn publish_websocket_server_sync(_config: &Config, server: &Server, rec: &Record) {
  use websocket::client::ClientBuilder;
  use websocket::{OwnedMessage};
  
  let wire_data = WireData {
    action: Action::publish,
    record: rec.clone(),
  };
  
  let ip_and_port = format!("ws://{}:{}", server.host, server.port);
  let mut unconnected_client = ClientBuilder::new(&ip_and_port).expect("Cannot construct websocket client");
  match unconnected_client.connect_insecure() {
    Ok(client) => {
      let (mut _receiver, mut sender) = client.split().unwrap();
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = sender.send_message(&OwnedMessage::Binary(bytes)) {
          println!("Error sending WireData to server in publish_websocket_server_sync: {}", e);
        }
      }
      // At the moment we don't expect data back from the server
    }
    Err(e) => {
      println!("Error in publish_websocket_server_sync: {}", e);
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
  let mut results = match server.protocol {
    ServerProtocol::TCP => {
      query_tcp_server_sync(config, server, query)
    }
    ServerProtocol::UDP => {
      query_udp_server_sync(config, server, query)
    }
    ServerProtocol::UNIX => {
      query_unix_server_sync(config, server, query)
    }
    ServerProtocol::WEBSOCKET => {
      query_websocket_server_sync(config, server, query)
    }
    ServerProtocol::MULTICAST => {
      query_udp_server_sync(config, server, query)
    }
  };
  
  // Now write record.src_server for all records
  for i in 0..results.len() {
    results[i].src_server = Some(server.clone());
  }
  
  return results;
}

pub fn query_tcp_server_sync(_config: &Config, server: &Server, query: &Record) -> Vec<Record> {
  use std::net::TcpStream;
  
  let mut results = vec![];
  
  let wire_data = WireData {
    action: Action::query,
    record: query.clone(),
  };
  
  let ip_and_port = format!("{}:{}", server.host, server.port);
  match TcpStream::connect(&ip_and_port) {
    Ok(mut stream) => {
      if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting TCP read timeout: {}", e);
      }
      if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(256))) {
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
      if server.report_connect_errors {
        println!("Error in query_tcp_server_sync: {}", e);
      }
      return vec![];
    }
  }
  
  return results;
}

pub fn query_udp_server_sync(_config: &Config, server: &Server, query: &Record) -> Vec<Record> {
  use std::net::UdpSocket;
  
  let mut results = vec![];
  
  let wire_data = WireData {
    action: Action::query,
    record: query.clone(),
  };
  
  let server_ip_and_port = format!("{}:{}", server.host, server.port);
  
  match UdpSocket::bind("0.0.0.0:0") {
    Ok(socket) => {
      if let Err(e) = socket.set_read_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting UDP read timeout: {}", e);
      }
      if let Err(e) = socket.set_write_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting UDP write timeout: {}", e);
      }
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = socket.send_to(&bytes, &server_ip_and_port) {
          println!("Error sending WireData to server in query_udp_server_sync: {}", e);
          return vec![];
        }
        // Write the terminal 0xff byte
        if let Err(e) = socket.send_to(&[0xff], &server_ip_and_port) {
          println!("Error sending WireData to server in query_udp_server_sync: {}", e);
          return vec![];
        }
      }
      
      // Read results until connection is closed
      // We use 0xff ("break" stop code in the CBOR spec (rfc 7049))
      // as a deliminator because it is least likely to interfere with CBOR stuff.
      
      //let start = Instant::now(); // TODO add timeout on all these queries
      let mut buff = [0; 16 * 1024];
      let mut overflow_buff = vec![]; // Unused but read-in bytes are appended here
      loop {
        match socket.recv_from(&mut buff) {
          Ok((num_read, _src_socket)) => {
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
            if e.kind() == std::io::ErrorKind::WouldBlock {
              // "fatal" error; we actually probably want to handle
              // this somehow, but for now this is a reliable server-dropped-conn
              // signal.
              break;
            }
            println!("Error reading from UDP: {} (kind={:?})", &e, &e.kind());
            break;
          }
        }
      }
      
    }
    Err(e) => {
      if server.report_connect_errors {
        println!("Error in query_udp_server_sync: {}", e);
      }
      return vec![];
    }
  }
  
  return results;
}

#[cfg(not(unix))]
pub fn query_unix_server_sync(config: &Config, server: &Server, query: &Record) -> Vec<Record> {
  println!("Warning: Cannot query_unix_server_sync because architecture is not unix.");
  return vec![];
}

#[cfg(unix)]
pub fn query_unix_server_sync(_config: &Config, server: &Server, query: &Record) -> Vec<Record> {
  use std::os::unix::net::UnixStream;
  
  let mut results = vec![];
  
  let wire_data = WireData {
    action: Action::query,
    record: query.clone(),
  };
  
  match UnixStream::connect(&server.path) {
    Ok(mut stream) => {
      if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting Unix read timeout: {}", e);
      }
      if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting Unix write timeout: {}", e);
      }
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = stream.write(&bytes) {
          println!("Error sending WireData to server in query_unix_server_sync: {}", e);
          return vec![];
        }
        // Write the terminal 0xff byte
        if let Err(e) = stream.write(&[0xff]) {
          println!("Error sending WireData to server in query_unix_server_sync: {}", e);
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
            println!("Error reading from Unix: {}", e);
            break;
          }
        }
      }
      
    }
    Err(e) => {
      if server.report_connect_errors {
        println!("Error in query_unix_server_sync: {}", e);
      }
      return vec![];
    }
  }
  
  return results;
}

pub fn query_websocket_server_sync(_config: &Config, server: &Server, query: &Record) -> Vec<Record> {
  use websocket::client::ClientBuilder;
  use websocket::{OwnedMessage};
  
  let mut results = vec![];
  
  let wire_data = WireData {
    action: Action::query,
    record: query.clone(),
  };
  
  let ip_and_port = format!("ws://{}:{}", server.host, server.port);
  let mut unconnected_client = ClientBuilder::new(&ip_and_port).expect("Cannot construct websocket client");
  match unconnected_client.connect_insecure() {
    Ok(client) => {
      let (mut receiver, mut sender) = client.split().unwrap();
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = sender.send_message(&OwnedMessage::Binary(bytes)) {
          println!("Error sending WireData to server in query_websocket_server_sync: {}", e);
          return vec![];
        }
      }
      
      // Read results until connection is closed
      // We read one CBOR WireData object per websocket packet
      for resp in receiver.incoming_messages() {
        if let Ok(resp) = resp {
          match resp {
            OwnedMessage::Binary(buff) => {
              if let Ok(wire_res) = serde_cbor::from_slice::<WireData>(&buff[..]) {
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
            }
            unk => {
              println!("Unsupported websocket msg: {:?}", unk);
            }
          }
        }
      }
      
    }
    Err(e) => {
      if server.report_connect_errors {
        println!("Error in query_websocket_server_sync: {}", e);
      }
      return vec![];
    }
  }
  
  return results;
}

pub fn listen_sync<F: Fn(Record) -> ListenAction + Send + Copy>(config: &Config, query: &Record, callback: F) {
  thread::scope(|s| {
    let mut handlers = vec![];
    
    for server in &config.servers {
      let t_server = server.clone();
      handlers.push(s.spawn(move |_| {
        listen_server_sync(config, &t_server, query, callback);
      }));
    }
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
}

pub fn listen_server_sync<F: Fn(Record) -> ListenAction>(config: &Config, server: &Server, query: &Record, callback: F) {
  match server.protocol {
    ServerProtocol::TCP => {
      listen_tcp_server_sync(config, server, query, callback);
    }
    ServerProtocol::UDP => {
      listen_udp_server_sync(config, server, query, callback);
    }
    ServerProtocol::UNIX => {
      listen_unix_server_sync(config, server, query, callback);
    }
    ServerProtocol::WEBSOCKET => {
      listen_websocket_server_sync(config, server, query, callback);
    }
    ServerProtocol::MULTICAST => {
      listen_udp_server_sync(config, server, query, callback);
    }
  }
}

pub fn listen_tcp_server_sync<F: Fn(Record) -> ListenAction>(_config: &Config, server: &Server, query: &Record, callback: F) {
  use std::net::TcpStream;
  
  let wire_data = WireData {
    action: Action::listen,
    record: query.clone(),
  };
  
  let ip_and_port = format!("{}:{}", server.host, server.port);
  match TcpStream::connect(&ip_and_port) {
    Ok(mut stream) => {
      if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting TCP read timeout: {}", e);
      }
      if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting TCP write timeout: {}", e);
      }
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = stream.write(&bytes) {
          println!("Error sending WireData to server in listen_tcp_server_sync: {}", e);
          return;
        }
        // Write the terminal 0xff byte
        if let Err(e) = stream.write(&[0xff]) {
          println!("Error sending WireData to server in listen_tcp_server_sync: {}", e);
          return;
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
                    if callback(wire_res.record) == ListenAction::EndListen {
                      break;
                    }
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
          Err(_e) => {
            // This is usually a timeout and we don't disconnect when listening
            //println!("Error reading from TCP: {}", e);
            //break;
          }
        }
      }
      
    }
    Err(e) => {
      println!("Error in listen_tcp_server_sync: {}", e);
    }
  }
}

pub fn listen_udp_server_sync<F: Fn(Record) -> ListenAction>(_config: &Config, server: &Server, query: &Record, callback: F) {
  use std::net::UdpSocket;
  
  let wire_data = WireData {
    action: Action::listen,
    record: query.clone(),
  };
  
  let server_ip_and_port = format!("{}:{}", server.host, server.port);
  
  match UdpSocket::bind("0.0.0.0:0") {
    Ok(socket) => {
      if let Err(e) = socket.set_read_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting UDP read timeout: {}", e);
      }
      if let Err(e) = socket.set_write_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting UDP write timeout: {}", e);
      }
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = socket.send_to(&bytes, &server_ip_and_port) {
          println!("Error sending WireData to server in publish_udp_server_sync: {}", e);
        }
        // Write the terminal 0xff byte
        if let Err(e) = socket.send_to(&[0xff], &server_ip_and_port) {
          println!("Error sending WireData to server in publish_udp_server_sync: {}", e);
        }
      }
      
      // Read results until connection is closed
      // We use 0xff ("break" stop code in the CBOR spec (rfc 7049))
      // as a deliminator because it is least likely to interfere with CBOR stuff.
      
      //let start = Instant::now(); // TODO add timeout on all these queries
      let mut buff = [0; 16 * 1024];
      let mut overflow_buff = vec![]; // Unused but read-in bytes are appended here
      loop {
        match socket.recv_from(&mut buff) {
          Ok((num_read, _src_socket)) => {
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
                    if callback(wire_res.record) == ListenAction::EndListen {
                      break;
                    }
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
            if e.kind() == std::io::ErrorKind::WouldBlock {
              // "fatal" error; we actually probably want to handle
              // this somehow, but for now this is a reliable server-dropped-conn
              // signal.
              break;
            }
            println!("Error reading from UDP: {} (kind={:?})", &e, &e.kind());
            break;
          }
        }
      }
      
    }
    Err(e) => {
      println!("Error in publish_udp_server_sync: {}", e);
    }
  }
}

#[cfg(not(unix))]
pub fn listen_unix_server_sync<F: Fn(Record) -> ListenAction>(_config: &Config, server: &Server, query: &Record, callback: F) {
  println!("Warning: Cannot listen_unix_server_sync because architecture is not unix.");
  return vec![];
}

#[cfg(unix)]
pub fn listen_unix_server_sync<F: Fn(Record) -> ListenAction>(_config: &Config, server: &Server, query: &Record, callback: F) {
  use std::os::unix::net::UnixStream;
  
  let wire_data = WireData {
    action: Action::listen,
    record: query.clone(),
  };
  
  match UnixStream::connect(&server.path) {
    Ok(mut stream) => {
      if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting Unix read timeout: {}", e);
      }
      if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(256))) {
        println!("Error setting Unix write timeout: {}", e);
      }
      
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = stream.write(&bytes) {
          println!("Error sending WireData to server in listen_unix_server_sync: {}", e);
          return;
        }
        // Write the terminal 0xff byte
        if let Err(e) = stream.write(&[0xff]) {
          println!("Error sending WireData to server in listen_unix_server_sync: {}", e);
          return;
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
                    if callback(wire_res.record) == ListenAction::EndListen {
                      break;
                    }
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
          Err(_e) => {
            // This is usually a timeout and we don't disconnect when listening
            //println!("Error reading from TCP: {}", e);
            //break;
          }
        }
      }
      
    }
    Err(e) => {
      println!("Error in listen_unix_server_sync: {}", e);
    }
  }
}

pub fn listen_websocket_server_sync<F: Fn(Record) -> ListenAction>(_config: &Config, server: &Server, query: &Record, callback: F) {
  use websocket::client::ClientBuilder;
  use websocket::{OwnedMessage};
  
  let wire_data = WireData {
    action: Action::listen,
    record: query.clone(),
  };
  
  let ip_and_port = format!("ws://{}:{}", server.host, server.port);
  let mut unconnected_client = ClientBuilder::new(&ip_and_port).expect("Cannot construct websocket client");
  match unconnected_client.connect_insecure() {
    Ok(client) => {
      let (mut receiver, mut sender) = client.split().unwrap();
          
      if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
        if let Err(e) = sender.send_message(&OwnedMessage::Binary(bytes)) {
          println!("Error sending WireData to server in listen_websocket_server_sync: {}", e);
          return;
        }
      }
      
      // Read results until connection is closed
      // We read one CBOR WireData object per websocket packet
      for resp in receiver.incoming_messages() {
        if let Ok(resp) = resp {
          match resp {
            OwnedMessage::Binary(buff) => {
              if let Ok(wire_res) = serde_cbor::from_slice::<WireData>(&buff[..]) {
                match wire_res.action {
                  Action::end_of_results => {
                    break;
                  }
                  Action::result => {
                    if callback(wire_res.record) == ListenAction::EndListen {
                      break;
                    }
                  }
                  unexpected => {
                    println!("Unexpected action from server, ignoring packet: {}", unexpected);
                  }
                }
              }
            }
            unk => {
              println!("Unsupported websocket msg: {:?}", unk);
            }
          }
        }
      }
      
    }
    Err(e) => {
      println!("Error in listen_websocket_server_sync: {}", e);
    }
  }
}
