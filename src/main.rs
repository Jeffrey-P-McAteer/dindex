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

// When compiling in release mode, windows .exe does not open cmd.exe
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Required for from_args() on args::Args
use structopt::StructOpt;
use fork;
use fork::{Fork};

use dindex::config;
use dindex::args;
use dindex::record;
use dindex::actions;
use dindex::actions::Action;

use dindex::http_client;
use dindex::server;
use dindex::client;
use dindex::data;
use dindex::wire;
use dindex::disp;
use dindex::signing;

use dindex::web_scan;

#[cfg(feature = "gui-client")]
use dindex::gui_client;

fn main() {
  let args = args::Args::from_args();
  let conf = config::read_config(&args);
  
  match args.action {
    Action::query => {
      let res = client::query_sync(&conf, &args.get_record(&conf));
      disp::print_results(&conf, &res);
    }
    Action::publish => {
      let rec = args.get_record(&conf);
      if rec.is_empty() {
        println!("Error: refusing to publish empty record!");
      }
      else {
        client::publish_sync(&conf, &rec);
      }
    }
    
    Action::listen => {
      let rec = args.get_record(&conf);
      client::listen_sync(&conf, &rec, |result| {
        disp::print_results(&conf, &vec![result]);
        return client::ListenAction::Continue;
      });
    }
    
    Action::run_server => {
      server::run_sync(&conf);
    }
    
    Action::double_fork_server => {
      double_fork_impl(&conf);
    }
    
    Action::run_http_client => {
      http_client::run_sync(&conf);
    }
    
    Action::run_gui_client => {
      if cfg!(feature = "gui-client") {
        #[cfg(feature = "gui-client")]
        gui_client::run_sync(&conf);
      }
      else {
        println!("This versin of dIndex was not compiled with GUI support.");
        println!("To compile with GUI support run:");
        println!("  cargo build --release --features \"gui-client\"");
      }
    }
    
    Action::run_web_scan => {
      let urls = args.rec_args.clone();
      web_scan::scan_urls(&conf, &args, urls, |rec| {
        // TODO we can eval some lua to let users filter public web endpoints
        client::publish_sync(&conf, &rec);
      });
    }
    
    Action::gen_identity => {
      let dev_stderr = "/dev/stderr".to_string();
      let output_path = args.rec_args.get(0).unwrap_or(&dev_stderr);
      signing::gen_identity(output_path);
      println!("Wrote new identity to {}", output_path);
    }
    
    Action::print_identity => {
      println!("======= Public Key =======");
      println!("{}", signing::read_pub_key_base64(&conf.client_private_key_file));
    }
    
    other => {
      println!("Cannot handle action {}", other);
    }
  }
  
}

fn double_fork_impl(config: &config::Config) {
  use std::fs;
  use nix::sys::signal::kill;
  // If a server is running kill it
  if let Ok(pid_bytes) = fs::read(&config.server_pid_file) {
    if let Ok(pid_s) = std::str::from_utf8(&pid_bytes) {
      if let Ok(pid_i) = pid_s.parse::<i32>() {
        if let Err(e) = kill(nix::unistd::Pid::from_raw(pid_i), nix::sys::signal::Signal::SIGTERM) {
          let msg = format!("{}", e);
          let is_ok_error = msg.contains("No such process");
          if ! is_ok_error {
            println!("Error killing existing server: {}", e);
          }
        }
      }
    }
  }
  
  match fork::daemon(false, false) {
    Ok(Fork::Child) => {
      server::run_sync(config);
    }
    Err(e) => {
      println!("Error forking: {:?}", e);
    }
    _ => {
      // We don't care about the parent, nothing happens
    }
  }
}
