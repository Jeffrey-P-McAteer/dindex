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

use std::sync::{Arc, RwLock};
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
    let results = vec![];
    for pool in self.record_pools.iter() {
      if let Ok(pool) = pool.try_read() {
        std::unimplemented!()
      }
    }
    return results;
  }
}
