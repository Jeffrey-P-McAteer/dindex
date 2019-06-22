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

use victorem;
//use serde_cbor::from_slice;

use std::net::SocketAddr;
use std::time::Duration;
use std::{thread, time};
//use std::collections::HashMap;
use std::{env, fs};

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
  println!("Listening for connections on UDP 0.0.0.0:{}", config.listen_port);
  listen(&config);
}

fn listen(config: &Config) {
  // TODO currently victorem cannot use config.listen_ip; 
  let mut server = victorem::GameServer::new(
    ServerGlobalData {
        config: config,
        last_results: None,
        records: vec![
            // TODO not this
            Record{properties: [("type".into(), "server-log".into()),("data".into(), "Server says Hello World!".into())].iter().cloned().collect()}
        ]
    },
    config.listen_port
  ).unwrap();
  server.run();
}

#[allow(dead_code)]
struct ServerGlobalData<'a> {
    config: &'a Config,
    last_results: Option<Vec<Record>>,
    records: Vec<Record>,
}

impl<'a> ServerGlobalData<'a> {
    pub fn do_operation(&mut self, args: SvrArgs) -> Vec<Record> {
        match args.action {
            dindex::ArgsAction::query => {
                let mut results: Vec<Record> = vec![];
                let query_map = args.record.gen_query_map();
                // This is possibly the slowest possible search impl.
                for record in &self.records {
                    // Check if this record matches any of the search records
                    if record.matches_faster(&query_map) {
                        results.push(record.clone());
                    }
                }
                results.push(Record::result_end_record());
                return results;
            }
            dindex::ArgsAction::publish => {
                self.records.push(args.record);
                return vec![
                    Record::ephemeral("Published")
                ];
            }
        }
        
    }
}

impl<'a> victorem::Game for ServerGlobalData<'a> {
    fn handle_command(
        &mut self,
        _delta_time: Duration,
        commands: Vec<Vec<u8>>,
        from: SocketAddr,
    ) -> victorem::ContinueRunning {
        for v in commands {
            let args: SvrArgs = serde_cbor::from_slice(&v).unwrap();
            println!(
                "From Client: {} {:?}",
                from,
                args,
            );
            
            self.last_results = Some(self.do_operation(args));
        }
        true
    }

    fn draw(&mut self, _delta_time: Duration) -> Vec<u8> {
        match &self.last_results {
            Some(results) => {
                let bytes = serde_cbor::to_vec(results).unwrap();
                self.last_results = None;
                return bytes;
            }
            None => {
                // Sleep to prevent 100% CPU thrashing
                
                thread::sleep(time::Duration::from_millis(50));
                
                return vec![];
            }
        }
    }
    
}

