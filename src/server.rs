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

pub fn run_sync(config: &Config) {
  thread::scope(|s| {
    let mut handlers = vec![];
    
    handlers.push(s.spawn(move |_| {
      run_tcp_sync(config);
    }));
    
    handlers.push(s.spawn(move |_| {
      run_udp_sync(config);
    }));
    
    handlers.push(s.spawn(move |_| {
      run_unix_sync(config);
    }));
    
    for h in handlers {
      h.join().unwrap();
    }
  }).unwrap();
}

fn run_tcp_sync(config: &Config) {
  println!("tcp starting on 0.0.0.0:{}", config.server_port);
  
}

fn run_udp_sync(config: &Config) {
  println!("udp starting on 0.0.0.0:{}", config.server_port);
  
}

fn run_unix_sync(config: &Config) {
  
}
