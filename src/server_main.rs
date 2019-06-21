use victorem;
use serde_cbor::from_slice;

use std::net::SocketAddr;
use std::time::Duration;
use std::{thread, time};
use std::collections::HashMap;

use dindex::get_config;
use dindex::Command;
use dindex::Record;

fn main() {
  let config = get_config();
  // TODO currently victorem cannot use config.listen_ip; 
  listen(config.listen_port);
}

fn listen(port: u16) {
  let mut server = victorem::GameServer::new(
    ServerGlobalData {
        last_results: None,
        records: vec![
            // TODO not this
            Record{properties: [("type".into(), "server-log".into()),("data".into(), "Server says Hello World!".into())].iter().cloned().collect()}
        ]
    },
    port
  ).unwrap();
  server.run();
}

struct ServerGlobalData {
    last_results: Option<Vec<Record>>,
    records: Vec<Record>,
}

impl ServerGlobalData {
    pub fn do_operation(&mut self, args: Vec<String>, records: Vec<Record>) -> Vec<Record> {
        if let Some(arg1) = args.get(0) {
            match arg1.as_str() {
                "query" => {
                    let mut results: Vec<Record> = vec![];
                    // This is possibly the slowest possible search impl.
                    for record in self.records.clone() { // TODO not this
                        // Check if this record matches any of the search records
                        for search_record in records.clone() { // TODO not this either
                            if record.matches(&search_record) {
                                results.push(record);
                                break;
                            }
                        }
                    }
                    return results;
                }
                "publish" => {
                    for given_record in records {
                        self.records.push(given_record);
                    }
                    return vec![
                        Record::ephemeral("Published")
                    ];
                }
                _ => {
                    return vec![
                        Record::ephemeral(format!("Unknown command {}", arg1).as_str())
                    ];
                }
            }
        }
        else {
            return vec![
                Record::ephemeral("No command given (valid commands are 'query', 'publish', )")
            ];
        }
    }
}

impl victorem::Game for ServerGlobalData {
    fn handle_command(
        &mut self,
        delta_time: Duration,
        commands: Vec<Vec<u8>>,
        from: SocketAddr,
    ) -> victorem::ContinueRunning {
        if commands.len() < 1 {
            thread::sleep(time::Duration::from_millis(50));
        }
        for v in commands {
            let cmd: Command = serde_cbor::from_slice(&v).unwrap();
            println!(
                "From Client: {} {:?}",
                from,
                cmd,
            );
            let mut args: Vec<String> = vec![];
            let mut records: Vec<Record> = vec![];
            for arg in cmd.args {
                match Record::from_str(&format!("{{\"properties\": {} }}", arg)) { // TODO fix this garbage hack
                    Ok(r) => {
                        records.push(r);
                    }
                    Err(_e) => {
                        args.push(arg);
                    }
                }
            }
            self.last_results = Some(self.do_operation(args, records));
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
                return vec![];
            }
        }
    }
    
}

