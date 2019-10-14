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

use regex::Regex;
use num_cpus;
use crossbeam_utils::thread;

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;

use crate::record::Record;
use crate::config::Config;

pub struct Data {
  pub record_pools: Arc<Vec<Arc<RwLock<Vec<Record>>>>>,
}

impl Data {
  pub fn new(config: &Config) -> Data {
      let mut data = Data {
        record_pools: Arc::new(vec![])
      };
      let record_pools = Arc::get_mut(&mut data.record_pools).unwrap();
      // Create memory pools
      for _ in 0..config.server_num_record_pools {
        record_pools.push(
          Arc::new(RwLock::new(vec![]))
        );
      }
      return data;
  }
  pub fn insert(&self, rec: Record) {
    for pool in self.record_pools.iter() {
      if let Ok(mut pool) = pool.try_write() {
        pool.push(rec);
        break;
      }
    }
  }
  pub fn search(&self, query: &HashMap<String, Regex>) -> Vec<Record> {
    let cpus = num_cpus::get();
    let results = Arc::new(Mutex::new(vec![]));
    
    thread::scope(|s| {
      let mut handlers = vec![];
      
      let pools_per_thread = self.record_pools.len() / cpus;
      let pools_remainder = self.record_pools.len() % cpus;
      
      for thread_num in 0..cpus {
        // search from (thread_num*pools_per_thread) to ((thread_num+1)*pools_per_thread)
        let mut pool_refs = vec![];
        for pool in &self.record_pools[(thread_num*pools_per_thread)..((thread_num+1)*pools_per_thread)] {
          pool_refs.push(pool);
        }
        // Spawn thread to search all pool refs
        let thread_results = results.clone();
        handlers.push(s.spawn(move |_| {
          for p in pool_refs {
            if let Ok(p) = p.try_read() {
              for rec in p.iter() {
                if rec.matches(query) {
                  if let Ok(mut lock) = thread_results.lock() {
                    lock.push(rec.clone());
                  }
                }
              }
            }
          }
        }));
      }
      // last thread needs to search (cpus*pools_per_thread) to (cpus*pools_per_thread)+pools_remainder
      {
        let mut pool_refs = vec![];
        for pool in &self.record_pools[(cpus*pools_per_thread)..(cpus*pools_per_thread)+pools_remainder] {
          pool_refs.push(pool);
        }
        // Spawn thread to search all pool refs
        let thread_results = results.clone();
        handlers.push(s.spawn(move |_| {
          for p in pool_refs {
            if let Ok(p) = p.try_read() {
              for rec in p.iter() {
                if rec.matches(query) {
                  if let Ok(mut lock) = thread_results.lock() {
                    lock.push(rec.clone());
                  }
                }
              }
            }
          }
        }));
      }
      
      for h in handlers {
        h.join().unwrap();
      }
    }).unwrap();
    return Arc::try_unwrap(results).unwrap().into_inner().unwrap();
  }
  pub fn search_callback<F: FnMut(&Record) -> bool>(&self, query: &HashMap<String, Regex>, mut on_result: F)
    where F: Send + Copy,
  {
    let cpus = num_cpus::get();
    
    thread::scope(|s| {
      let mut handlers = vec![];
      
      let pools_per_thread = self.record_pools.len() / cpus;
      let pools_remainder = self.record_pools.len() % cpus;
      
      for thread_num in 0..cpus {
        // search from (thread_num*pools_per_thread) to ((thread_num+1)*pools_per_thread)
        let mut pool_refs = vec![];
        for pool in &self.record_pools[(thread_num*pools_per_thread)..((thread_num+1)*pools_per_thread)] {
          pool_refs.push(pool);
        }
        // Spawn thread to search all pool refs
        handlers.push(s.spawn(move |_| {
          for p in pool_refs {
            if let Ok(p) = p.try_read() {
              for rec in p.iter() {
                if rec.matches(query) {
                  if ! on_result(rec) {
                    break; // Caller says we have hit limit of records to search
                  }
                }
              }
            }
          }
        }));
      }
      // last thread needs to search (cpus*pools_per_thread) to (cpus*pools_per_thread)+pools_remainder
      {
        let mut pool_refs = vec![];
        for pool in &self.record_pools[(cpus*pools_per_thread)..(cpus*pools_per_thread)+pools_remainder] {
          pool_refs.push(pool);
        }
        // Spawn thread to search all pool refs
        handlers.push(s.spawn(move |_| {
          for p in pool_refs {
            if let Ok(p) = p.try_read() {
              for rec in p.iter() {
                if rec.matches(query) {
                  if ! on_result(rec) {
                    break; // Caller says we have hit limit of records to search
                  }
                }
              }
            }
          }
        }));
      }
      
      for h in handlers {
        h.join().unwrap();
      }
    }).unwrap();
  }
}
