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

use std::io::prelude::*;
use std::sync::atomic::Ordering;

use crate::config::Config;
use crate::data::Data;
use crate::record::Record;
use crate::wire::WireData;
use crate::actions::Action;

use crate::server_data_io::*;

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

pub fn run_tcp_sync(config: &Config, data: &Data) {
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
          // Further housekeeping
          if data.exit_flag.load(Ordering::Relaxed) {
            println!("tcp exiting due to data.exit_flag");
            break;
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

pub fn run_udp_sync(config: &Config, data: &Data) {
  use std::net::UdpSocket;
  use std::io::ErrorKind;
  use std::time::Duration;
  
  let ip_port = format!("{}:{}", config.server_ip, config.server_port);
  println!("udp starting on {}", &ip_port);
  
  match UdpSocket::bind(ip_port) {
    Ok(mut socket) => {
      
      if let Err(e) = socket.set_read_timeout(Some(Duration::from_millis(1024))) {
        println!("Error setting UDP read timeout: {}", e);
      }
      if let Err(e) = socket.set_write_timeout(Some(Duration::from_millis(1024))) {
        println!("Error setting UDP write timeout: {}", e);
      }
      
      let mut incoming_buf = [0u8; 65536];
      
      while !data.exit_flag.load(Ordering::Relaxed) {
        match socket.recv_from(&mut incoming_buf) {
          Ok((num_bytes, src)) => {
              let packet = incoming_buf[0..num_bytes].to_vec();
              if config.is_debug() {
                println!("UDP: {} bytes from {:?}", num_bytes, src);
              }
              handle_udp_conn(&mut socket, src, packet, config, data);
          }
          Err(ref err) if err.kind() != ErrorKind::WouldBlock => {
              println!("UDP Server error: {}", err);
              break;
          }
          Err(_e) => {
              // Usually OS error 11
              //println!("Unknown error: {}", e);
          }
        }
      }
      
    }
    Err(e) => {
      println!("Error starting UDP server: {}", e);
    }
  }
}

pub fn run_unix_sync(config: &Config, data: &Data) {
  use std::os::unix::net::{UnixListener};
  use std::collections::VecDeque;
  use std::path::Path;
  use std::fs;
  
  println!("unix listening to {}", &config.server_unix_socket);
  
  if Path::new(&config.server_unix_socket).exists() {
    if let Err(e) = fs::remove_file(&config.server_unix_socket) {
      println!("Error removing prior unix socket: {}", e);
    }
  }
  
  match UnixListener::bind(&config.server_unix_socket) {
    Ok(socket) => {
      thread::scope(|s| {
        let mut handlers = VecDeque::new();
        handlers.reserve_exact(config.server_threads_in_flight + 4);
        
        for stream in socket.incoming() {
          handlers.push_back(s.spawn(|_| {
            handle_unix_conn(stream, config, data);
          }));
          // Housekeeping
          if handlers.len() > config.server_threads_in_flight {
            let threads_to_join = (handlers.len() as f64 * config.server_threads_in_flight_fraction) as usize;
            // Pop up to threads_to_join thread handles and join on them
            for _ in 0..threads_to_join {
              if let Some(h) = handlers.pop_front() {
                if let Err(e) = h.join() {
                  println!("Error joining Unix Socket thread: {:?}", e);
                }
              }
            }
          }
          // Further housekeeping
          if data.exit_flag.load(Ordering::Relaxed) {
            println!("Unix exiting due to data.exit_flag");
            break;
          }
        }
        
        for h in handlers {
          h.join().unwrap();
        }
        
      }).unwrap();
    }
    Err(e) => {
      println!("Error starting Unix server: {}", e);
    }
  }
}

fn handle_udp_conn(socket: &mut std::net::UdpSocket, src: std::net::SocketAddr, mut packet: Vec<u8>, config: &Config, data: &Data) {
  use std::sync::{Arc, Mutex};
  
  // UDP assumes every packet is a single CBOR message which is allowed to end in 0xff
  if packet.last().eq(&Some(&0xff)) {
    packet.pop();
  }
  if packet.len() < 1 {
    return; // Do nothing, likely a stray 0xff that got put in a 2nd packet
  }
  
  // Parse bytes to WireData
  match serde_cbor::from_slice::<WireData>(&packet[..]) {
    Err(e) => {
      println!("Error reading WireData from UDP client: {}", e);
      return;
    }
    Ok(wire_data) => {
      if config.is_debug() {
          println!("got connection, wire_data={:?}", wire_data);
        }
        match wire_data.action {
          Action::query => {
            let ts_socket = Arc::new(Mutex::new(socket));
            data.search_callback(&wire_data.record.create_regex_map(), |result| {
              let wire_data = WireData {
                action: Action::result,
                record: result.clone(),
              };
              if let Ok(bytes) = serde_cbor::to_vec(&wire_data) {
                if let Ok(stream) = ts_socket.lock() {
                  if let Err(e) = stream.send_to(&bytes, &src) {
                    println!("Error sending result to UDP client: {}", e);
                    return false; // stop querying, client has likely exited
                  }
                  // Write packet seperation byte
                  if let Err(e) = stream.send_to(&[0xff], &src) {
                    println!("Error sending result to UDP client: {}", e);
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
              if let Ok(stream) = ts_socket.lock() {
                if let Err(e) = stream.send_to(&bytes, &src) {
                  println!("Error sending result to UDP client: {}", e);
                }
                // Write packet seperation byte
                if let Err(e) = stream.send_to(&[0xff], &src) {
                  println!("Error sending result to UDP client: {}", e);
                }
              }
            }
          }
          Action::publish => {
            if ! wire_data.record.is_empty() {
              data.insert(wire_data.record);
            }
            // For now just dump entire Data to storage whenever something is added
            // TODO optimize etc etc
            write_stored_records(config, &data);
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
            if ! wire_data.record.is_empty() {
              data.insert(wire_data.record);
            }
            // For now just dump entire Data to storage whenever something is added
            // TODO optimize etc etc
            write_stored_records(config, &data);
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

fn handle_unix_conn(stream: Result<std::os::unix::net::UnixStream, std::io::Error>, config: &Config, data: &Data) {
  use std::time::Duration;
  use std::sync::{Arc, Mutex};
  use crate::h_map;
  
  if let Ok(mut stream) = stream {
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
            if ! wire_data.record.is_empty() {
              data.insert(wire_data.record);
            }
            // For now just dump entire Data to storage whenever something is added
            // TODO optimize etc etc
            write_stored_records(config, &data);
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
