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

use crate::config::Config;
use crate::data::Data;

pub fn run_sync(config: &Config) {
  let data = Data::new(config);
  
  thread::scope(|s| {
    let mut handlers = vec![];
    
    handlers.push(s.spawn(|_| {
      run_tcp_sync(config, &data);
    }));
    
    handlers.push(s.spawn(|_| {
      run_udp_sync(config, &data);
    }));
    
    handlers.push(s.spawn(|_| {
      run_unix_sync(config, &data);
    }));
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
}

fn run_tcp_sync(config: &Config, data: &Data) {
  println!("tcp starting on 0.0.0.0:{}", config.server_port);
  
}

fn run_udp_sync(config: &Config, data: &Data) {
  println!("udp starting on 0.0.0.0:{}", config.server_port);
  
}

fn run_unix_sync(config: &Config, data: &Data) {
  
}
