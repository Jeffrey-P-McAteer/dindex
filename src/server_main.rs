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

#[macro_use]
extern crate lazy_static;

extern crate crossbeam;

//use serde_cbor::from_slice;

use std::net::{SocketAddr,UdpSocket};
use std::{thread, time};
use std::collections::HashMap;
use std::{env, fs};
use std::sync::{RwLock, Arc};
use std::time::{Duration, Instant};
use std::io::ErrorKind;

use dindex::config::get_config;
use dindex::config::Config;
use dindex::Record;
use dindex::SvrArgs;

fn main() {
  let args: Vec<String> = env::args().collect();
  if let Some(arg1) = args.get(1) {
      if &arg1[..] == "install" {
        let exe = env::current_exe().unwrap();
        let exe = exe.as_path().to_string_lossy();
        println!("Installing a systemd unit using this binary ({})", exe);
        let contents = format!(r#"
[Unit]
Description=dIndex server
After=network.target

[Service]
Type=simple
User=nobody
WorkingDirectory=/tmp/
ExecStart={}
Restart=on-failure

[Install]
WantedBy=multi-user.target

"#, exe);
        fs::write("/etc/systemd/system/dindex.service", contents).expect("Unable to write file");
        println!("Start + enable the server with:");
        println!("  sudo systemctl enable --now dindex");
        return;
      }
  }
  
  let config = get_config();
  
  let svr_data = RwLock::new(ServerGlobalData {
        config: &config,
        pending_results: HashMap::new(),
        records: Arc::new(RwLock::new(vec![
            Record::server_start_record()
        ])),
        pending_cleanup_timer: Instant::now(),
  });
  
  let sock = UdpSocket::bind(config.get_ip_port())
                       .expect("Failed to bind socket");
  sock.set_nonblocking(true)
      .expect("Failed to enter non-blocking mode");
  println!("Listening for connections on UDP {}", config.get_ip_port());
  
  let mut incoming_buf = [0u8; 65536];
  let mut spawned_threads = vec![];
  loop {
      match sock.recv_from(&mut incoming_buf) {
        Ok((num_bytes, src)) => {
            let packet = &incoming_buf[0..num_bytes].to_vec();
            let th = crossbeam::thread::scope(|_s| {
                handle_packet(&svr_data, packet.to_vec(), &sock, src);
            });
            spawned_threads.push(th);
        }
        Err(ref err) if err.kind() != ErrorKind::WouldBlock => {
            println!("Server error: {}", err);
            break;
        }
        Err(_e) => {
            // Usually OS error 11
            //println!("Unknown error: {}", e);
        }
      }
  }
}

fn handle_packet(server: &RwLock<ServerGlobalData>, packet: Vec<u8>, sock: &UdpSocket, client: SocketAddr) {
    sock.send_to(&packet, client).expect("failed to send message");
    
}

struct ClientResults {
    pub from: SocketAddr,
    pub begun: bool,
    pub completed: bool,
    pub results: Vec<Record>,
}

impl ClientResults {
    pub fn new(from: SocketAddr) -> ClientResults {
        ClientResults {
            from: from,
            begun: false,
            completed: false,
            results: vec![],
        }
    }
}

#[allow(dead_code)]
struct ServerGlobalData<'a> {
    config: &'a Config,
    pending_results: HashMap<String, Arc<RwLock<ClientResults>>>,
    records: Arc<RwLock<Vec<Record>>>,
    pending_cleanup_timer: Instant,
}

pub fn do_operation<F: Fn(Record)>(args: SvrArgs, records: Arc<RwLock<Vec<Record>>>, on_result: F) {
    match args.action {
        dindex::ArgsAction::query => {
            let query_map = args.record.gen_query_map();
            // This is possibly the slowest possible search impl.
            if let Ok(records) = records.read() {
                for record in &records[..] {
                    // Check if this record matches any of the search records
                    if record.matches_faster(&query_map) {
                        on_result(record.clone());
                    }
                }
                on_result(Record::result_end_record());
            }
        }
        dindex::ArgsAction::publish => {
            if let Ok(mut records) = records.write() {
                records.push(args.record);
                on_result(Record::ephemeral("Published"));
            }
            else {
                on_result(Record::ephemeral("Failed to publish"));
            }
        }
    }
}

// impl<'a> victorem::Game for ServerGlobalData<'a> {
//     fn handle_command(
//         &mut self,
//         _delta_time: Duration,
//         commands: Vec<Vec<u8>>,
//         from: SocketAddr,
//     ) -> victorem::ContinueRunning {
//         for v in commands {
//             if let Ok(args) = serde_cbor::from_slice::<SvrArgs>(&v) {
//                 println!("From Client: {} {:?}", from, args,);
                
//                 let client_ip_port_s = format!("{}", from);
//                 let results = Arc::new(RwLock::new(ClientResults::new(from)));
//                 self.pending_results.insert(client_ip_port_s, results.clone());
                
//                 // process the new request async
//                 let records_ref = self.records.clone();
//                 thread::spawn(move || {
//                     do_operation(args, records_ref, move |result| {
//                         from.
//                     });
//                 });
                
//             }
//         }
        
//         // Routinely free memory from hashmap
//         let period = Duration::from_millis(20_000);
//         if self.pending_cleanup_timer.elapsed() > period {
//             let mut to_remove_vec: Vec<String> = vec![];
//             for (result_key, result_val) in self.pending_results {
//                 if let Ok(val) = result_val.read() {
//                     if val.completed {
//                         to_remove_vec.push(result_key);
//                     }
//                 }
//             }
//             for to_remove in to_remove_vec {
//                 println!("Removing pending data for {}", to_remove);
//                 self.pending_results.remove(&to_remove);
//             }
//         }
        
//         true
//     }

//     fn draw(&mut self, _delta_time: Duration) -> Vec<u8> {
//         vec![]
//         // for (client_key, results) in self.pending_results {
            
//         // }
//         // match &self.last_results {
//         //     Some(results) => {
//         //         let bytes = serde_cbor::to_vec(results).unwrap();
//         //         self.last_results = None;
//         //         return bytes;
//         //     }
//         //     None => {
//         //         // Sleep to prevent 100% CPU thrashing
                
//         //         thread::sleep(time::Duration::from_millis(50));
                
//         //         return vec![];
//         //     }
//         // }
//     }
    
// }
