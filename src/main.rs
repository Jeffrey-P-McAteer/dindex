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

// Required for from_args() on args::Args
use structopt::StructOpt;

mod config;
mod args;
mod record;
mod actions;

mod http_client;
mod server;
mod client;
mod data;
mod wire;

fn main() {
  let args = args::Args::from_args();
  let conf = config::read_config(&args);
  
  match args.action {
    actions::Action::query => {
      let res = client::query_sync(&conf, &args.get_record(&conf));
      print_results(&conf, &res);
    }
    actions::Action::publish => {
      std::unimplemented!()
    }
    actions::Action::listen => {
      std::unimplemented!()
    }
    actions::Action::run_server => {
      server::run_sync(&conf);
    }
    actions::Action::run_http_client => {
      http_client::run_sync(&conf);
    }
    other => {
      println!("Cannot handle action {}", other);
    }
  }
  
}

fn print_results(config: &config::Config, results: &Vec<record::Record>) {
  for res in results {
    // TODO custom formatting from config/ctypes/whatever
    println!("res = {:?}", res.p);
  }
}
