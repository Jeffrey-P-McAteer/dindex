/*
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
extern crate bencher;
use bencher::Bencher;

use rand::prelude::*;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
 
use crossbeam_utils::thread;

use std::time::Duration;

use dindex;

fn gen_rand_record() -> dindex::record::Record {
  let mut rng = rand::thread_rng();
  let mut rec = dindex::record::Record::empty();
  let num_pairs: usize = rng.gen_range(2, 6);
  for _ in 0..num_pairs {
    let key_len: usize = rng.gen_range(2, 64);
    let rand_key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(key_len)
        .collect();
        
    let val_len: usize = rng.gen_range(8, 512);
    let rand_val: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(val_len)
        .collect();
        
    rec.p.insert(rand_key, rand_val);
  }
  rec
}

fn gen_rand_record_exp(key_len: usize, num_pairs: usize) -> dindex::record::Record {
  let mut rng = rand::thread_rng();
  let mut rec = dindex::record::Record::empty();
  for _ in 0..num_pairs {
    let rand_key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(key_len)
        .collect();
        
    let val_len: usize = rng.gen_range(8, 512);
    let rand_val: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(val_len)
        .collect();
        
    rec.p.insert(rand_key, rand_val);
  }
  rec
}

fn gen_rand_query_exp(key_len: usize, num_pairs: usize) -> dindex::record::Record {
  let mut rng = rand::thread_rng();
  let mut rec = dindex::record::Record::empty();
  for _ in 0..num_pairs {
    let rand_key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(key_len)
        .collect();
        
    let val_len: usize = rng.gen_range(1, 4);
    let rand_pre: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(val_len / 2)
        .collect();
    let rand_post: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(val_len / 2)
        .collect();
        
    let rand_val = format!("{}.*{}", rand_pre, rand_post);
        
    rec.p.insert(rand_key, rand_val);
  }
  rec
}

/**
 * This test measures single-record generation time.
 * This should be used to help evaluate other tests which are forced
 * to include the record generation time where subtracting that
 * cost yields a more realistic measurement.
 */
fn single_rand_record_gen(b: &mut Bencher) {
  b.iter(|| {
    gen_rand_record()
  })
}

/**
 * This test sets up a TCP server writing to an in-memory store of records.
 * Bencher should report average single-record insert time.
 * This time includes TCP connection setup time and random record generation time.
 */
fn tcp_mem_insert_flood(b: &mut Bencher) {
  let mut test_config = dindex::config::get_config_detail(
    // this is the method that reads from env, but we specify no env in the arguments
    false, false, false, false,
    Err(std::env::VarError::NotPresent),
    &dindex::args::Args::empty()
  );
  // Write details for temporary data
  let port = 2001;
  let localhost_server = dindex::config::Server {
    protocol: dindex::config::ServerProtocol::TCP,
    host: "127.0.0.1".to_string(),
    port: port,
    path: "/tmp/dindex.test.socket".to_string(),
    max_latency_ms: 250,
    report_connect_errors: true,
  };
  test_config.servers = vec![localhost_server];
  test_config.server_port = port;
  test_config.server_ip = "127.0.0.1".to_string();
  test_config.server_listen_tcp = true;
  test_config.server_listen_udp = false;
  test_config.server_listen_unix = false;
  test_config.server_listen_websocket = false;
  test_config.server_extra_quiet = true;
  
  // Tell server not to store records outside this process's memory
  test_config.server_datastore_uri = "memory://".to_string();
  
  // Create a data store
  let mut data = dindex::data::Data::new(&test_config);
  let exit_flag = data.exit_flag.clone();
  
  // Spawn server and client threads to perform testing
  thread::scope(|s| {
    let mut handlers = vec![];
    
    handlers.push(s.spawn(|_| {
      dindex::server::run_tcp_sync(&test_config, &mut data);
    }));
    
    // Thread which publishes listened-for records
    handlers.push(s.spawn(|_| {
      std::thread::sleep(Duration::from_millis(25));
      
      b.iter(|| {
        let random_rec = gen_rand_record();
        dindex::client::publish_sync(&test_config, &random_rec);
        random_rec
      });
      
      // Instruct server to exit, test completed
      exit_flag.store(true, std::sync::atomic::Ordering::Relaxed);
      // Send it network traffic to force eval of exit_flag
      let random_rec = gen_rand_record();
      dindex::client::publish_sync(&test_config, &random_rec);
      
    }));
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
}

fn tcp_mem_query_over_1k(b: &mut Bencher) {
  let mut test_config = dindex::config::get_config_detail(
    // this is the method that reads from env, but we specify no env in the arguments
    false, false, false, false,
    Err(std::env::VarError::NotPresent),
    &dindex::args::Args::empty()
  );
  // Write details for temporary data
  let port = 2001;
  let localhost_server = dindex::config::Server {
    protocol: dindex::config::ServerProtocol::TCP,
    host: "127.0.0.1".to_string(),
    port: port,
    path: "/tmp/dindex.test.socket".to_string(),
    max_latency_ms: 250,
    report_connect_errors: true,
  };
  test_config.servers = vec![localhost_server];
  test_config.server_port = port;
  test_config.server_ip = "127.0.0.1".to_string();
  test_config.server_listen_tcp = true;
  test_config.server_listen_udp = false;
  test_config.server_listen_unix = false;
  test_config.server_listen_websocket = false;
  test_config.server_extra_quiet = true;
  
  // Tell server not to store records outside this process's memory
  test_config.server_datastore_uri = "memory://".to_string();
  
  // Create a data store
  let mut data = dindex::data::Data::new(&test_config);
  let exit_flag = data.exit_flag.clone();
  
  // Spawn server and client threads to perform testing
  thread::scope(|s| {
    let mut handlers = vec![];
    
    handlers.push(s.spawn(|_| {
      dindex::server::run_tcp_sync(&test_config, &mut data);
    }));
    
    // Wait for server to start
    std::thread::sleep(Duration::from_millis(25));
    
    // Publish 1k random records
    for _ in 0..1000 {
      let random_rec = gen_rand_record_exp(4, 8);
      dindex::client::publish_sync(&test_config, &random_rec);
    }
    
    // Benchmark random regex queries
    b.iter(|| {
      let random_rec = gen_rand_query_exp(4, 2);
      let results = dindex::client::query_sync(&test_config, &random_rec);
      results
    });
    
    // Instruct server to exit, test completed
    exit_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    // Send it network traffic to force eval of exit_flag
    let random_rec = gen_rand_record();
    dindex::client::publish_sync(&test_config, &random_rec);
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
}

benchmark_group!(benches,
  single_rand_record_gen,
  tcp_mem_insert_flood,
  tcp_mem_query_over_1k
);
benchmark_main!(benches);
