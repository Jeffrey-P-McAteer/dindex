use serde::{Serialize, Deserialize};
use victorem;
use rand;

use std::env;

use dindex::get_config;
use dindex::Resolver;
use dindex::Config;
use dindex::Command;
use dindex::Record;

fn main() {
  let args: Vec<String> = env::args().collect();
  let config = get_config();
  let cmd = Command {
    args: args[1..].to_vec()
  };
  for resolver in config.upstream_resolvers {
    instruct_resolver(&resolver, &cmd);
  }
}

fn instruct_resolver(r: &Resolver, cmd: &Command) {
  use std::{thread, time};
  use std::time::{Duration, Instant};
  use rand::Rng;
  
  let mut rng = rand::thread_rng();
  
  let mut client = victorem::ClientSocket::new(rng.gen_range(11111, 55555), r.get_host_port_s()).unwrap();
  
  client.send(serde_cbor::to_vec(cmd).unwrap()).unwrap();
  
  let timer = Instant::now();
  let period = Duration::from_millis(r.max_latency_ms as u64);
  
  loop {
    thread::sleep(time::Duration::from_millis(10));
    
    if timer.elapsed() > period {
      println!("Timing out for resolver at {}", r.get_host_port_s());
      break;
    }
    
    match client.recv() {
      Ok(bytes) => {
        let results: Vec<Record> = serde_cbor::from_slice(&bytes).unwrap_or(vec![]);
        let is_empty = results.len() < 1;
        let mut i = 0;
        for result in results {
          println!("Result {}: {:?}", i, result.properties);
          i += 1;
        }
        if !is_empty {
          break;
        }
      }
      Err(e) => {
        match e {
          victorem::Exception::IoError(_ioe) => {
            continue; // Just waiting for data
          }
          _ => {
            println!("{}", e);
            break;
          }
        }
      }
    }
  }

}


