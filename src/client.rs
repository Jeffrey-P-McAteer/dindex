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

use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::config::Server;
use crate::record::Record;

pub fn query_sync(config: &Config, query: &Record) -> Vec<Record> {
  let results: Arc<Mutex<Vec<Record>>> = Arc::new(Mutex::new(vec![]));
  
  thread::scope(|s| {
    let mut handlers = vec![];
    
    for server in &config.servers {
      let t_results = results.clone();
      let t_server = server.clone();
      handlers.push(s.spawn(move |_| {
        let res_v = query_server_sync(config, &t_server, query);
        if let Ok(mut t_results) = t_results.lock() {
          t_results.extend_from_slice(&res_v[..]);
        }
      }));
    }
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
  
  return Arc::try_unwrap(results).unwrap().into_inner().unwrap();
}

pub fn query_server_sync(config: &Config, server: &Server, query: &Record) -> Vec<Record> {
  let mut results = vec![];
  //match server.
  return results;
}
