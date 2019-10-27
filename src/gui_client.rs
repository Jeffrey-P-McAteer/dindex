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

use web_view::*;

use crossbeam_utils::thread;


use crate::config;

pub fn run_sync(config: &config::Config) {
  thread::scope(|s| {
    let h1 = s.spawn(move |_| {
      crate::http_client::run_sync(config);
    });
    
    let h2 = s.spawn(move |_| {
      web_view::builder()
        .title("dIndex")
        .content(Content::Url(format!("http://127.0.0.1:{}", config.client_http_port)))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .unwrap();
      // TODO make a better way to exit http_client::run_sync cleanly
      std::process::exit(0);
    });
    
    h1.join().unwrap();
    h2.join().unwrap();
  }).unwrap();
}

