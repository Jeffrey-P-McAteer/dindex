use serde::{Serialize, Deserialize};
use victorem;
use rand;

use std::env;

use dindex::get_config;
use dindex::Resolver;
use dindex::Command;

fn main() {
  let args: Vec<String> = env::args().collect();
  let config = get_config();
  let cmd = Command {
    args: args
  };
  for resolver in config.upstream_resolvers {
    instruct_resolver(&resolver, &cmd);
  }
}

fn instruct_resolver(r: &Resolver, cmd: &Command) {
  use serde_cbor::to_vec;
  use rand::Rng;
  let mut rng = rand::thread_rng();
  
  let mut client = victorem::ClientSocket::new(rng.gen_range(11111, 55555), r.get_host_port_s()).unwrap();
  
  client.send(serde_cbor::to_vec(cmd).unwrap());
  
  
  // let mut timer = Instant::now();
  // let period = Duration::from_millis(100);
  // loop {
  //     if timer.elapsed() > period {
  //         timer = Instant::now();
  //         id += 1;
  //         let _ = client.send(format!("Ping {}", id).into_bytes());
  //     }
  //     let _ = client
  //         .recv()
  //         .map(|v| String::from_utf8(v).unwrap_or(String::new()))
  //         .map(|s| println!("From Server: {}", s));
  // }

}


