/**
 *  dIndex - a distributed, organic, mechanical index for everything
 *  Copyright (C) 2019  Jeffrey McAteer <jeffrey.p.mcateer@gmail.com>
 *  
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; version 2 of the License only.
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
use websocket;

use std::io::prelude::*;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::process;
use std::fs;

use crate::config::Config;
use crate::data::{Data, Listener};
use crate::record::Record;
use crate::wire::WireData;
use crate::actions::Action;

use crate::server_data_io::*;

use crate::h_map;

pub fn run_sync(config: &Config) {
  // Write PID to config.server_pid_file
  let our_pid_s = format!("{}", process::id());
  if let Err(e) = fs::write(&config.server_pid_file, our_pid_s.as_str()) {
    println!("Error writing to PID file: {}", e);
  }
  println!("Server PID: {}", our_pid_s);
  
  let mut data = Data::new(config);
  read_stored_records(config, &mut data);
  let data = data;
  
  thread::scope(|s| {
    let mut handlers = vec![];
    
    if config.server_listen_tcp {
      handlers.push(s.spawn(|_| {
        run_tcp_sync(config, &data);
      }));
    }
    
    if config.server_listen_udp {
      handlers.push(s.spawn(|_| {
        run_udp_sync(config, &data);
      }));
    }
    
    if config.server_listen_unix {
      handlers.push(s.spawn(|_| {
        run_unix_sync(config, &data);
      }));
    }
    
    if config.server_listen_websocket {
      handlers.push(s.spawn(|_| {
        run_websocket_sync(config, &data);
      }));
    }
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
}

pub fn run_tcp_sync(config: &Config, data: &Data) {
  use std::net::TcpListener;
  use std::collections::VecDeque;
  
  let ip_port = format!("{}:{}", config.server_ip, config.server_port);
  if !config.server_extra_quiet {
    println!("tcp starting on {}", &ip_port);
  }
  
  match TcpListener::bind(&ip_port) {
    Ok(listener) => {
      thread::scope(|s| {
        let mut handlers = VecDeque::new(); //
        handlers.reserve_exact(config.server_threads_in_flight + 4);
        
        for stream in listener.incoming() {
          handlers.push_back(s.spawn(|_| {
            handle_tcp_conn(stream, config, data);
          }));
          // Housekeeping
          if handlers.len() > config.server_threads_in_flight {
            // First try to avoid deadlocks by calling trim_invalid_listeners
            handlers.push_back(s.spawn(|_| {
              data.trim_invalid_listeners();
            }));
            let threads_to_join = (handlers.len() as f64 * config.server_threads_in_flight_fraction) as usize;
            // Pop up to threads_to_join thread handles and join on them
            for _ in 0..threads_to_join {
              if let Some(h) = handlers.pop_front() {
                println!(" calling h.join()...");
                if let Err(e) = h.join() {
                  println!("Error joining TCP thread: {:?}", e);
                }
              }
            }
            println!(" Done joining threads!");
          }
          // Further housekeeping
          if data.exit_flag.load(Ordering::Relaxed) {
            if config.is_debug() {
              if !config.server_extra_quiet {
                println!("tcp exiting due to data.exit_flag");
              }
            }
            break;
          }
        }
        
        data.trim_all_listeners();
        
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

// We use a single UDP socket and if server_listen_multicast==true
// that socket also joint server_multicast_group
pub fn run_udp_sync(config: &Config, data: &Data) {
  use std::net::UdpSocket;
  use std::io::ErrorKind;
  use std::time::Duration;
  use std::collections::VecDeque;
  
  let ip_port = format!("{}:{}", config.server_ip, config.server_port);
  if !config.server_extra_quiet {
    println!("udp starting on {}", &ip_port);
  }
  
  match UdpSocket::bind(ip_port) {
    Ok(mut socket) => {
      if config.server_listen_multicast {
        println!("udp joining multicast {}", &config.server_multicast_group);
        let is_v6 = config.server_multicast_group.contains(":");
        if is_v6 {
          if let Ok(server_multicast_group) = &config.server_multicast_group.parse() {
            // TODO allow user config of ipv6 interface number?
            if let Err(e) = socket.join_multicast_v6(server_multicast_group, 0) {
              println!("Error joining multicast: {}", e);
            }
          }
        }
        else {
          if let Ok(server_multicast_group) = &config.server_multicast_group.parse() {
            if let Ok(server_ip) = &config.server_ip.parse() {
              if let Err(e) = socket.join_multicast_v4(server_multicast_group, server_ip) {
                println!("Error joining multicast: {}", e);
              }
            }
          }
        }
      }
      
      if let Err(e) = socket.set_read_timeout(Some(Duration::from_millis(1024))) {
        println!("Error setting UDP read timeout: {}", e);
      }
      if let Err(e) = socket.set_write_timeout(Some(Duration::from_millis(1024))) {
        println!("Error setting UDP write timeout: {}", e);
      }
      
      thread::scope(|s| {
        let mut handlers = VecDeque::new();
        handlers.reserve_exact(config.server_threads_in_flight + 4);
        
        let mut incoming_buf = [0u8; 65536];
        
        while !data.exit_flag.load(Ordering::Relaxed) {
          match socket.recv_from(&mut incoming_buf) {
            Ok((num_bytes, src)) => {
                let packet = incoming_buf[0..num_bytes].to_vec();
                if config.is_debug() {
                  if !config.server_extra_quiet {
                    println!("UDP: {} bytes from {:?}", num_bytes, src);
                  }
                }
                if let Ok(mut socket) = socket.try_clone() {
                  handlers.push_back(s.spawn(move |_| {
                    handle_udp_conn(&mut socket, src, packet, config, data);
                  }));
                }
                else {
                  // Fall back to sync op
                  handle_udp_conn(&mut socket, src, packet, config, data);
                }
                // Housekeeping
                if handlers.len() > config.server_threads_in_flight {
                  // First try to avoid deadlocks by calling trim_invalid_listeners
                  handlers.push_back(s.spawn(|_| {
                    data.trim_invalid_listeners();
                  }));
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
            Err(ref err) if err.kind() != ErrorKind::WouldBlock => {
                println!("UDP Server error: {}", err);
                break;
            }
            Err(_e) => {
                // Usually OS error 11
                //println!("Unknown error: {}", e);
            }
          }
          // Housekeeping after every connection is closed
          if data.exit_flag.load(Ordering::Relaxed) {
            if config.is_debug() {
              if !config.server_extra_quiet {
                println!("udp exiting due to data.exit_flag");
              }
            }
            break;
          }
        }
        
        data.trim_all_listeners();
        
      }).unwrap();
      
    }
    Err(e) => {
      println!("Error starting UDP server: {}", e);
    }
  }
}

#[cfg(not(unix))]
pub fn run_unix_sync(config: &Config, data: &Data) {
  println!("Warning: Cannot run_unix_sync on non-unix architecture");
}

#[cfg(unix)]
pub fn run_unix_sync(config: &Config, data: &Data) {
  use std::os::unix::net::{UnixListener};
  use std::collections::VecDeque;
  use std::path::Path;
  
  if !config.server_extra_quiet {
    println!("unix listening to {}", &config.server_unix_socket);
  }
  
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
            // Helps avoid deadlocks
            handlers.push_back(s.spawn(|_| {
              data.trim_invalid_listeners();
            }));
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
            if config.is_debug() {
              if !config.server_extra_quiet {
                println!("Unix exiting due to data.exit_flag");
              }
            }
            break;
          }
        }
        
        data.trim_all_listeners();
        
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
        
        // Create channel to do business logic
        let (to_business_logic, from_us) = mpsc::channel();
        let (to_us, from_business_logic) = mpsc::channel();
        let validity_flag = Arc::new(Mutex::new(AtomicBool::new(true)));
        
        thread::scope(|s| {
          let handler_validity_flag_c = validity_flag.clone();
          let bt = s.spawn(|_| {
            handle_conn(from_us, to_us, config, data, handler_validity_flag_c);
          });
          
          let client_to_business_t = s.spawn(move |_| {
            to_business_logic.send(wire_data).unwrap();
          });
          
          let ts_socket = Arc::new(Mutex::new(socket));
          let business_to_client_t = s.spawn(move |_| {
            loop {
              match from_business_logic.recv() {
                Ok(wire_data_to_client) => {
                  if let Ok(bytes) = serde_cbor::to_vec(&wire_data_to_client) {
                    if let Ok(stream) = ts_socket.lock() {
                      if let Err(e) = stream.send_to(&bytes, &src) {
                        println!("Error sending result to UDP client: {}", e);
                        break; // stop querying, client has likely exited
                      }
                      // Write packet seperation byte
                      if let Err(e) = stream.send_to(&[0xff], &src) {
                        println!("Error sending result to UDP client: {}", e);
                        break; // stop querying, client has likely exited
                      }
                    }
                  }
                }
                Err(_e) => {
                  //println!("Error in handle_udp_conn looping business back to client: {}", e); // Always channel closed error
                  break;
                }
              }
            }
            // Any listener/future logic on this connection is now invalid
            match validity_flag.lock() {
              Ok(mut validity_flag) => {
                *validity_flag.get_mut() = false;
              }
              Err(e) => {
                println!("Error validity_flag.lock() = {}", e);
              }
            }
            data.trim_invalid_listeners();
          });
          
          bt.join().unwrap();
          client_to_business_t.join().unwrap();
          business_to_client_t.join().unwrap();
        }).unwrap();
    }
  }
}

fn handle_tcp_conn(stream: Result<std::net::TcpStream, std::io::Error>, config: &Config, data: &Data) {
  use std::time::Duration;
  
  if let Ok(mut stream) = stream {
    if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(256))) {
      println!("Error setting TCP read timeout: {}", e);
    }
    if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(256))) {
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
        
        // Create channel to do business logic
        let (to_business_logic, from_us) = mpsc::channel();
        let (to_us, from_business_logic) = mpsc::channel();
        let validity_flag = Arc::new(Mutex::new(AtomicBool::new(true)));
        
        thread::scope(|s| {
          let handler_validity_flag_c = validity_flag.clone();
          let bt = s.spawn(|_| {
            handle_conn(from_us, to_us, config, data, handler_validity_flag_c);
          });
          
          let client_to_business_t = s.spawn(move |_| {
            to_business_logic.send(wire_data).unwrap();
          });
          
          let ts_stream = Arc::new(Mutex::new(stream));
          let business_to_client_t = s.spawn(move |_| {
            loop {
              match from_business_logic.recv() {
                Ok(wire_data_to_client) => {
                  if let Ok(mut stream) = ts_stream.lock() {
                    if let Ok(bytes) = serde_cbor::to_vec(&wire_data_to_client) {
                      if let Err(e) = stream.write(&bytes) {
                        println!("Error sending result to TCP client: {}", e);
                        break; // stop sending, client has likely exited
                      }
                      // Write packet seperation byte
                      if let Err(e) = stream.write(&[0xff]) {
                        println!("Error sending result to TCP client: {}", e);
                        break; // stop sending, client has likely exited
                      }
                    }
                  }
                }
                Err(_e) => {
                  //println!("Error in handle_tcp_conn looping business back to client: {}", e); // Always channel closed error
                  break;
                }
              }
            }
            // Any listener/future logic on this connection is now invalid
            match validity_flag.lock() {
              Ok(mut validity_flag) => {
                *validity_flag.get_mut() = false;
              }
              Err(e) => {
                println!("Error validity_flag.lock() = {}", e);
              }
            }
            data.trim_invalid_listeners();
          });
          
          if let Err(e) = bt.join() {
            println!("Error joining thread: {:?}", e);
          }
          if let Err(e) = client_to_business_t.join() {
            println!("Error joining thread: {:?}", e);
          }
          if let Err(e) = business_to_client_t.join() {
            println!("Error joining thread: {:?}", e);
          }
        }).unwrap();
      }
    }
    
  }
}

#[cfg(unix)]
fn handle_unix_conn(stream: Result<std::os::unix::net::UnixStream, std::io::Error>, config: &Config, data: &Data) {
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
        
        // Create channel to do business logic
        let (to_business_logic, from_us) = mpsc::channel();
        let (to_us, from_business_logic) = mpsc::channel();
        let validity_flag = Arc::new(Mutex::new(AtomicBool::new(true)));
        
        thread::scope(|s| {
          let handler_validity_flag_c = validity_flag.clone();
          let bt = s.spawn(|_| {
            handle_conn(from_us, to_us, config, data, handler_validity_flag_c);
          });
          
          let client_to_business_t = s.spawn(move |_| {
            to_business_logic.send(wire_data).unwrap();
          });
          
          let ts_stream = Arc::new(Mutex::new(stream));
          let business_to_client_t = s.spawn(move |_| {
            loop {
              match from_business_logic.recv() {
                Ok(wire_data_to_client) => {
                  if let Ok(mut stream) = ts_stream.lock() {
                    if let Ok(bytes) = serde_cbor::to_vec(&wire_data_to_client) {
                      if let Err(e) = stream.write(&bytes) {
                        println!("Error sending result to Unix client: {}", e);
                        break; // stop sending, client has likely exited
                      }
                      // Write packet seperation byte
                      if let Err(e) = stream.write(&[0xff]) {
                        println!("Error sending result to Unix client: {}", e);
                        break; // stop sending, client has likely exited
                      }
                    }
                  }
                }
                Err(_e) => {
                  //println!("Error in handle_unix_conn looping business back to client: {}", e); // Always channel closed error
                  break;
                }
              }
            }
            // Any listener/future logic on this connection is now invalid
            match validity_flag.lock() {
              Ok(mut validity_flag) => {
                *validity_flag.get_mut() = false;
              }
              Err(e) => {
                println!("Error validity_flag.lock() = {}", e);
              }
            }
            data.trim_invalid_listeners();
          });
          
          bt.join().unwrap();
          client_to_business_t.join().unwrap();
          business_to_client_t.join().unwrap();
        }).unwrap();
      }
    }
  }
}

pub fn run_websocket_sync(config: &Config, data: &Data) {
  use websocket::sync::Server;
  use std::collections::VecDeque;
  
  let ip_port = format!("{}:{}", config.server_ip, config.server_websocket_port);
  println!("websocket starting on {}", &ip_port); 
  
  match Server::bind(ip_port) {
    Ok(server) => {
      thread::scope(|s| {
        let mut handlers = VecDeque::new();
        handlers.reserve_exact(config.server_threads_in_flight + 4);
          
        for request in server.filter_map(Result::ok) {
          handlers.push_back(s.spawn(|_| {
            match request.accept() {
              Ok(client) => {
                handle_websocket_conn(client, config, data);
              }
              Err(e) => {
                println!("Error accepting websocket: {:?}", e);
              }
            }
          }));
          // Housekeeping
          if handlers.len() > config.server_threads_in_flight {
            // Helps avoid deadlocks
            handlers.push_back(s.spawn(|_| {
              data.trim_invalid_listeners();
            }));
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
            if config.is_debug() {
              if !config.server_extra_quiet {
                println!("websocket exiting due to data.exit_flag");
              }
            }
            break;
          }
        }
        
        data.trim_all_listeners();
        
        for h in handlers {
          h.join().unwrap();
        }
      }).unwrap();
    }
    Err(e) => {
      println!("Error starting websocket: {}", e);
    }
  }
}

fn handle_websocket_conn(client: websocket::client::sync::Client<std::net::TcpStream>, config: &Config, data: &Data) {
  use websocket::message::OwnedMessage;
  
  let (mut receiver, sender) = client.split().unwrap();
  
  match receiver.recv_message() {
    Err(e) => {
      println!("Error receiving first websocket msg: {}", e);
    }
    Ok(msg) => {
      match msg {
        OwnedMessage::Binary(buff) => {
          match serde_cbor::from_slice::<WireData>(&buff[..]) {
            Err(e) => {
              println!("Error reading WireData from WebSocket client: {}", e);
              return;
            }
            Ok(wire_data) => {
              
              // Create channel to do business logic
              let (to_business_logic, from_us) = mpsc::channel();
              let (to_us, from_business_logic) = mpsc::channel();
              let validity_flag = Arc::new(Mutex::new(AtomicBool::new(true)));
              
              
              thread::scope(|s| {
                let handler_validity_flag_c = validity_flag.clone();
                let bt = s.spawn(|_| {
                  handle_conn(from_us, to_us, config, data, handler_validity_flag_c);
                });
                
                let client_to_business_t = s.spawn(move |_| {
                  to_business_logic.send(wire_data).unwrap();
                });
                
                let ts_stream = Arc::new(Mutex::new(sender));
                let business_to_client_t = s.spawn(move |_| {
                  loop {
                    match from_business_logic.recv() {
                      Ok(wire_data_to_client) => {
                        if let Ok(mut stream) = ts_stream.lock() {
                          if let Ok(bytes) = serde_cbor::to_vec(&wire_data_to_client) {
                            if let Err(e) = stream.send_message(&OwnedMessage::Binary(bytes)) {
                              println!("Error sending result to WebSocket client: {}", e);
                              break; // stop sending, client has likely exited
                            }
                          }
                        }
                      }
                      Err(_e) => {
                        //println!("Error in handle_tcp_conn looping business back to client: {}", e); // Always channel closed error
                        break;
                      }
                    }
                  }
                  // Any listener/future logic on this connection is now invalid
                  match validity_flag.lock() {
                    Ok(mut validity_flag) => {
                      *validity_flag.get_mut() = false;
                    }
                    Err(e) => {
                      println!("Error validity_flag.lock() = {}", e);
                    }
                  }
                  data.trim_invalid_listeners();
                });
                
                if let Err(e) = bt.join() {
                  println!("Error joining thread: {:?}", e);
                }
                if let Err(e) = client_to_business_t.join() {
                  println!("Error joining thread: {:?}", e);
                }
                if let Err(e) = business_to_client_t.join() {
                  println!("Error joining thread: {:?}", e);
                }
              }).unwrap();
              
            }
          }
        }
        unk => {
          println!("Unsupported websocket first msg: {:?}", unk);
        }
      }
    }
  }
  
}

// This is a generic channel implementation so we can seperate business
// logic from tcp/udp/unix connection details.
fn handle_conn(from_client: mpsc::Receiver<WireData>, to_client: mpsc::Sender<WireData>, config: &Config, data: &Data, validity_flag: Arc<Mutex<AtomicBool>>) {
  match from_client.recv() {
    Err(e) => {
      println!("Error receiving in handle_conn: {}", e);
    }
    Ok(wire_data) => {
      if config.is_debug() && !config.server_extra_quiet {
        println!("wire_data = {:?}", wire_data);
      }
      // Reject all queries and published records
      // if the record appears signed (contains pub key || signature)
      // but the signature is invalid.
      if wire_data.record.is_imposter() {
        // We _ought_ to at least let the user know.
        // This gives visibility in the scenario where a valid user
        // does not understand their tools.
        let err_data = WireData {
          action: Action::unsolicited_msg,
          record: Record::new(h_map!{
            "error-message".to_string() =>
              "Error: The record received has signature keys but contains an invalid signature.".to_string()
          }),
        };
        if let Err(e) = to_client.send(err_data) {
          println!("e = {}", e);
        }
        return;
      }
      let ts_to_client = Arc::new(Mutex::new(to_client.clone()));
      match wire_data.action {
        Action::query => {
          data.search_callback(&wire_data.record.create_regex_map(), |result| {
            let wire_data = WireData {
              action: Action::result,
              record: result.clone(),
            };
            if let Ok(to_client) = ts_to_client.lock() {
              to_client.send(wire_data).unwrap();
            }
            return true; // TODO limit when using UDP?
          });
          // Tell clients connection should be closed
          let wire_data = WireData {
            action: Action::end_of_results,
            record: Record::empty()
          };
          if let Ok(to_client) = ts_to_client.lock() {
            to_client.send(wire_data).unwrap();
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
          data.listen(Listener::new(
            &wire_data.record,
            to_client,
            validity_flag.clone()
          ));
        }
        unk => {
          println!("Error: unknown action {}", unk);
          return;
        }
      }
    }
  }
}

