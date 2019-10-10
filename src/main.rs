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

mod http_client;

fn main() {
  let args = args::Args::from_args();
  let conf = config::read_config(&args);
  
  match args.action {
    args::ArgsAction::query => {
      std::unimplemented!()
    }
    args::ArgsAction::publish => {
      std::unimplemented!()
    }
    args::ArgsAction::listen => {
      std::unimplemented!()
    }
    args::ArgsAction::run_server => {
      std::unimplemented!()
    }
    args::ArgsAction::run_http_client => {
      http_client::run_sync(&conf);
    }
  }
  
}
