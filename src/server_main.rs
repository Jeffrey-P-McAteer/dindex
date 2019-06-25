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

//#[macro_use]
extern crate lazy_static;

extern crate crossbeam;

use serde_cbor;

use std::net::{SocketAddr,UdpSocket};
//use std::{thread, time};
//use std::collections::HashMap;
use std::{env, fs};
use std::sync::{RwLock};
use std::time::Duration;
use std::io::ErrorKind;

use dindex::config::get_config;
use dindex::config::Config;
use dindex::Record;
use dindex::SvrArgs;
use dindex::ArgsAction;

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
  
  let svr_data = ServerGlobalData {
        config: &config,
        records: RwLock::new(vec![
            Record::server_start_record()
        ]),
  };
  
  let sock = UdpSocket::bind(config.get_ip_port())
                       .expect("Failed to bind socket");
  
  println!("Listening for connections on UDP {}", config.get_ip_port());
  
  let mut incoming_buf = [0u8; 65536];
  let mut spawned_threads = vec![];
  loop {
      match sock.recv_from(&mut incoming_buf) {
        Ok((num_bytes, src)) => {
            let packet = &incoming_buf[0..num_bytes].to_vec();
            println!("{} bytes from {:?}", num_bytes, src);
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

fn handle_packet(server: &ServerGlobalData, packet: Vec<u8>, sock: &UdpSocket, client: SocketAddr) {
    if let Ok(svr_args) = serde_cbor::from_slice::<SvrArgs>(&packet) {
      match svr_args.action {
        ArgsAction::query => {
          let max_returned_bytes = packet.len();
          let mut total_returned_bytes = 0;
          
          println!("{:?} queried {:?}", client, svr_args.record);
          do_query(&svr_args.record, &server.records, |result| {
            let reply_bytes = serde_cbor::to_vec(&result).unwrap();
            if total_returned_bytes + reply_bytes.len() > max_returned_bytes {
              println!("Hit query byte limit, refusing to reply with result");
              return;
            }
            println!("Returning to {:?} result {:?}", client, result);
            sock.send_to(&reply_bytes, client).expect("failed to send message");
            total_returned_bytes += reply_bytes.len();
          });
        },
        ArgsAction::publish => {
          println!("{:?} published {:?}", client, svr_args.record);
          do_publish(&svr_args.record, &server.records);
        }
      }
    }
    else {
      let err = Record::error_record("Error decoding query record");
      sock.send_to(&serde_cbor::to_vec(&err).unwrap(), client).expect("failed to send message");
    }
}

#[allow(dead_code)]
struct ServerGlobalData<'a> {
    config: &'a Config,
    records: RwLock<Vec<Record>>,
}

pub fn do_query<F: FnMut(Record)>(query_record: &Record, records: &RwLock<Vec<Record>>, mut on_result: F) {
  let query_map = query_record.gen_query_map();
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

pub fn do_publish(new_record: &Record, records: &RwLock<Vec<Record>>) {
  if let Ok(mut records) = records.write() {
    records.push(new_record.clone());
  }
}


