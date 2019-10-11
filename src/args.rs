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

use structopt;
use structopt::StructOpt;

use clap;
use clap::arg_enum;

use serde;
use serde_repr;

use crate::actions::Action;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "dindex", about = "A distributed index for anything and everything")]
pub struct Args {
  /// Specify additional config file to load
  #[structopt(long = "config")]
  pub config_file: Option<String>,
  
  /// Verbose mode (-v, -vv, -vvv, etc.)
  #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
  pub verbose: u8,
  
  /// Action to perform
  #[structopt(raw(possible_values = "&Action::variants()", case_insensitive = "true"))]
  pub action: Action,
  
  /// Sign outgoing records
  #[structopt(short = "S", long = "signed")]
  pub signed: bool,
  
  // I'd LIKE this, but we must parse manually because of user-def ctypes
  //pub record: Option<Record>,
  // Instead we will capture all extra arguments and write custom parse methods to turn them into a Record
  #[structopt(last = true)]
  pub extra_args: Vec<String>,
  
}

