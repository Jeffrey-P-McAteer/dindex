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

extern crate structopt;

use victorem;
use rand;

use structopt::StructOpt;

use std::thread;

use dindex::get_config;
use dindex::Resolver;
use dindex::Record;
use dindex::Args;

fn main() {
  let args = Args::from_args();
  
  println!("{:?}", args);
  
  if args.docs {
    println!(include_str!("client_readme.md"));
    return;
  }
  
  let config = get_config();
  for resolver in config.upstream_resolvers {
    let a = args.clone(); // TODO not this
    thread::spawn(move || {
      instruct_resolver(&resolver, &a);
    });
  }
}

fn instruct_resolver(r: &Resolver, args: &Args) {
  use std::time;
  use std::time::{Duration, Instant};
  use rand::Rng;
  
  println!("Querying {}", r.get_host_port_s());
  
  let mut rng = rand::thread_rng();
  let mut client = victorem::ClientSocket::new(rng.gen_range(11111, 55555), r.get_host_port_s()).unwrap();
  
  client.send(serde_cbor::to_vec(&args.clone().into_svr_args()).unwrap()).unwrap();
  
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


