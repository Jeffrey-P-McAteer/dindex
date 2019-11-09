/**
 *  dIndex - a distributed, organic, mechanical index for everything
 *  Copyright (C) 2019  Jeffrey McAteer <jeffrey.p.mcateer@gmail.com>
 *  
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; version 2 of the License only.
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

use crate::config;
use crate::record;

pub fn print_results(_config: &config::Config, results: &Vec<record::Record>) {
  // Sort by server name
  let mut results = results.clone();
  results.sort_by(|a, b| {
    if let Some(a_svr) = &a.src_server {
      if let Some(b_svr) = &b.src_server {
        if let Some(order) = a_svr.name.partial_cmp(&b_svr.name) {
          return order;
        }
      }
    }
    return std::cmp::Ordering::Equal;
  });
  
  let mut last_svr_name = String::new();
  for res in results {
    if let Some(svr) = &res.src_server {
      if ! svr.name.eq(&last_svr_name) {
        // Move to new group of server records, print the header
        last_svr_name = svr.name.clone();
        println!("=== {} ===", last_svr_name);
      }
    }
    println!("res = {:?}", res.p);
  }
  
}

pub fn print_results_ref(_config: &config::Config, results: &Vec<&record::Record>) {
  // Sort by server name
  let mut results = results.clone();
  results.sort_by(|a, b| {
    if let Some(a_svr) = &a.src_server {
      if let Some(b_svr) = &b.src_server {
        if let Some(order) = a_svr.name.partial_cmp(&b_svr.name) {
          return order;
        }
      }
    }
    return std::cmp::Ordering::Equal;
  });
  
  let mut last_svr_name = String::new();
  for res in results {
    if let Some(svr) = &res.src_server {
      if ! svr.name.eq(&last_svr_name) {
        // Move to new group of server records, print the header
        last_svr_name = svr.name.clone();
        println!("=== {} ===", last_svr_name);
      }
    }
    println!("res = {:?}", res.p);
  }
  
  
}


