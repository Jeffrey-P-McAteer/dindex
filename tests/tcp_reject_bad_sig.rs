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
fn tcp_reject_bad_sig() {
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
    
    handlers.push(s.spawn(|_| {
      // This is where client logic is tested
      std::thread::sleep(Duration::from_millis(25));
      // Server should have bound to ports within 25ms
      
      // Test that empty server is empty
      let query_1 = {
        let mut rec = dindex::record::Record::empty();
        rec.p.insert("NAME".to_string(), ".*".to_string());
        rec
      };
      let results = dindex::client::query_sync(&test_config, &query_1);
      assert_eq!(results.len(), 0);
      
      // Add a record that contains signature keys but does not have a valid signature
      let rec_1 = {
        let mut rec = dindex::record::Record::empty();
        rec.p.insert("NAME".to_string(), "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_string());
        rec.p.insert("URL".to_string(), "https://evil.lipsum.com/".to_string());
        rec.p.insert("SIGNING:public-key".to_string(), "LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUlJQklqQU5CZ2txaGtpRzl3MEJBUUVGQUFPQ0FROEFNSUlCQ2dLQ0FRRUEwLzd5aWo4VEx0bkdhWlhUN1ZQdgpxMDlGQ0R2WTVIclZwL01kUnlYaDU5UUFsMmtwcEJvYk5weXN3dVliWkRlS1VYMVlPekRaZThUSkhYQkhtOUpyCmRQN2lMK2txSWxoRnBwZHYrSlgrWVYzcGg1bDlRamZuVm5NRkVTYUl5ZFFDci9vWW5HdEpZczNaQmJlT3FuZTAKQmVFbFErbTYvQUFpN3lNQi9qYThFekNpeGt3Wm81VVVQc1ZuNTdaUTh3aUdWNjdjWDhnZHlnaTdzNHU4Zi9lMgpMWTBCWTV6VUxSZ3Y0emZkK3F6WmtGWTZaRVdFS3MwYng0QnFqL3Nvd3BVREtkZTdaa0ZrZ0VjaDJZZXpQK1YzClhKWUl0dEJKVXAvQy94Ull0bmRVZmI5NjI3VnlNNmNFd2RIU3ZvTExoVDgyakZjZDErRmFrTUZSQVNydlIvMnEKMHdJREFRQUIKLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==".to_string());
        rec.p.insert("SIGNING:non-sig-bytes".to_string(), "TQS8PqgyH/Ey9X1ZjdarWUcpSKpPcPVRhNwkC7F7ntt2IHfemz1hoLzhK+B35cpJmKqt7mrPBXw1FPfX3F8Xf3FGxpXDSbL+8oeOxG30AB0HIFNmGoC1jEFi1haLriTX1DyK9hPpviqHfcq/2WfeuqbBcjyf3mgXf8k2bR6ZdCMnNVrXv9yqsFLIUfgh13DdkMjAVOH6s+YAnFmCPh0HtrePyxe0gIsecDQC8tn9pKLPbww2i9staHej2e/VF/3K1lsWojZ8jxN5F62fraAB31EhuzrBLhrfuMGKnxYxp9bHdsICpVMKX0fMjBu6iG6Kt4WeK1S7SXLuxZ11IDC5NQ==".to_string());
        rec
      };
      dindex::client::publish_sync(&test_config, &rec_1);
      
      let results = dindex::client::query_sync(&test_config, &query_1);
      // Results must still be empty b/c server must reject imposter record
      assert_eq!(results.len(), 0);
      
      // Add an unsigned record
      let rec_2 = {
        let mut rec = dindex::record::Record::empty();
        rec.p.insert("NAME".to_string(), "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_string());
        rec.p.insert("URL".to_string(), "https://lipsum.com/".to_string());
        rec
      };
      dindex::client::publish_sync(&test_config, &rec_2);
      
      let results = dindex::client::query_sync(&test_config, &query_1);
      // Results must have the 1 unsigned record
      assert_eq!(results.len(), 1);
      
      let empty_s = String::new();
      let rec_1_url = results[0].p.get(&"URL".to_string()).unwrap_or(&empty_s);
      assert_eq!(rec_1_url, "https://lipsum.com/");
      // ^ now we know we got the same unsigned record back
      
      // Instruct server to exit
      exit_flag.store(true, std::sync::atomic::Ordering::Relaxed);
      // Send it network traffic to force eval of exit_flag
      dindex::client::query_sync(&test_config, &query_1);
      
    }));
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
  
  
}
