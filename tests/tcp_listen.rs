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

use std::time::Duration;

use dindex;

#[test]
fn tcp_server_listen() {
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
    name: "Localhost Server".to_string()
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
    
    // Test listening capability
    handlers.push(s.spawn(|_| {
      std::thread::sleep(Duration::from_millis(25));
      let query = {
        let mut rec = dindex::record::Record::empty();
        rec.p.insert("NAME".to_string(), "Lorem".to_string());
        rec
      };
      dindex::client::listen_sync(&test_config, &query, |rec| {
        
        let empty_s = String::new();
        let rec_url = rec.p.get(&"URL".to_string()).unwrap_or(&empty_s);
        assert_eq!(rec_url, "https://lipsum.com/");
        
        return dindex::client::ListenAction::EndListen;
      });
      
      // Instruct server to exit, test completed
      exit_flag.store(true, std::sync::atomic::Ordering::Relaxed);
      // Send it network traffic to force eval of exit_flag
      dindex::client::query_sync(&test_config, &query);
      
    }));
    
    // Thread which publishes listened-for records
    handlers.push(s.spawn(|_| {
      std::thread::sleep(Duration::from_millis(50));
      
      // Publish a record which will cause test to fail if received
      let rec_1 = {
        let mut rec = dindex::record::Record::empty();
        rec.p.insert("NAME".to_string(), "A record we should never see".to_string());
        rec.p.insert("URL".to_string(), "https://example.org/".to_string());
        rec
      };
      dindex::client::publish_sync(&test_config, &rec_1);
      
      // Publish record we want to receive
      let rec_1 = {
        let mut rec = dindex::record::Record::empty();
        rec.p.insert("NAME".to_string(), "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_string());
        rec.p.insert("URL".to_string(), "https://lipsum.com/".to_string());
        rec
      };
      dindex::client::publish_sync(&test_config, &rec_1);
    }));
    
    // If we don't get anything within 300ms the test fails
    handlers.push(s.spawn(|_| {
      // "wait" but break early if the exit flag is set to true (test success)
      let mut remaining_iters = 30;
      while remaining_iters > 0 && !exit_flag.load(std::sync::atomic::Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(10));
        remaining_iters -= 1;
      }
      
      // If the server exit flag is false the test has not received a record and we must fail
      if ! exit_flag.load(std::sync::atomic::Ordering::Relaxed) {
        let query = {
          let mut rec = dindex::record::Record::empty();
          rec.p.insert("NAME".to_string(), "Lorem".to_string());
          rec
        };
        // Instruct server to exit, test completed
        exit_flag.store(true, std::sync::atomic::Ordering::Relaxed);
        // Send it network traffic to force eval of exit_flag
        dindex::client::query_sync(&test_config, &query);
        
        // Finally fail the test due to timeout without receiving query
        assert!(false);
      }
    }));
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
}