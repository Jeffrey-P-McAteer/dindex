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

use serde_json;
//use clap::arg_enum;

//use serde;
//use serde_repr;

use crate::actions::Action;
use crate::config::Config;
use crate::record::Record;

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
  
  /// CType or JSON data used for publishing and querying payloads (eg :website '.*keyword.*' or '{"key": "value"}')
  pub rec_args: Vec<String>,
}

impl Args {
  // Parses a record from rec_args, applying ctypes along the way.
  // If no known types match, concatinates rec_args and parses as JSON.
  // Failing that, returns an empty record
  pub fn get_record(&self, config: &Config) -> Record {
    return parse_record(&self.rec_args, self.verbose, config);
  }
}

pub fn parse_record(args: &Vec<String>, verbose: u8, config: &Config) -> Record {
  if let Some(ctype_name) = args.get(0) {
    for ctype in &config.ctypes {
      if ctype.name.eq(ctype_name) {
        // Found a matching ctype, put key names in
        let arg_vals = &args[1..];
        let mut rec = Record::empty();
        for (key_name, extra_arg_val) in ctype.key_names.iter().zip(arg_vals.iter()) {
          rec.p.insert(key_name.to_string(), extra_arg_val.to_string());
        }
        if !rec.p.is_empty() {
          if verbose > 0 {
            println!("arg record = {:?}", &rec);
          }
          return rec;
        }
      }
    }
  }
  // No ctypes matched in rec_args[0], concatinate all + parse as JSON
  let joined = args.join(" ");
  let joined = format!("{{\"p\":{} }}", joined); // Wrap it so we can use serde directly
  if let Ok(rec) = serde_json::from_str(&joined) {
    if verbose > 0 {
      println!("arg record = {:?}", &rec);
    }
    return rec;
  }
  
  let rec = Record::empty();
  if verbose > 0 {
    println!("arg record = {:?}", &rec);
  }
  return rec;
}
