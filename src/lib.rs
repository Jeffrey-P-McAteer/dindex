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

extern crate config as crate_config;
extern crate dirs;

extern crate serde;
use serde::{Serialize, Deserialize};

extern crate clap;
use clap::arg_enum;

use structopt::StructOpt;

use regex::Regex;

use std::collections::HashMap;
use std::env;

pub mod config;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
  pub properties: HashMap<String, String>,
}

impl Record {
  pub fn empty() -> Record {
    Record {
      properties: HashMap::new()
    }
  }
  pub fn from_str(s: &str) -> Result<Record, serde_json::error::Error> {
    serde_json::from_str(s)
  }
  pub fn ephemeral(s: &str) -> Record {
    Record {
      properties: [
        ("type".into(), "ephemeral".into()),
        ("data".into(), s.into())
      ].iter().cloned().collect()
    }
  }
  
  pub fn result_end_record() -> Record {
    Record {
      properties: [
        ("type".into(), "ephemeral".into()),
        ("result-end".into(), "true".into())
      ].iter().cloned().collect()
    }
  }
  
  pub fn is_end_record(&self) -> bool {
    if let Some(result_end) = self.get_str("result-end") {
      return result_end == "true";
    }
    return false;
  }
  
  pub fn put_str(&mut self, key: &str, val: &str) {
    self.put(key.to_string(), val.to_string());
  }
  
  pub fn put(&mut self, key: String, val: String) {
    self.properties.insert(key, val);
  }
  
  pub fn get_ref(&self, key: String) -> Option<&String> {
    self.properties.get(&key)
  }
  
  pub fn get_str(&self, key: &str) -> Option<String> {
    self.get(key.to_string())
  }
  
  pub fn get(&self, key: String) -> Option<String> {
    match self.properties.get(&key) {
      Some(str_ref) => Some(str_ref.to_string()),
      None => None
    }
  }
  
  pub fn gen_query_map(&self) -> HashMap<String, Regex> {
    let mut map : HashMap<String, Regex> = HashMap::new();
    for (key, val) in &self.properties {
      map.insert(key.to_string(), Regex::new(val).unwrap());
    }
    return map;
  }
  
  pub fn matches_faster(&self, compiled_query: &HashMap<String, Regex>) -> bool {
    let mut common_keys = vec![];
    for (my_key, _) in &self.properties {
      for (their_key, _) in compiled_query {
        if my_key == their_key {
          common_keys.push(my_key.clone());
        }
      }
    }
    
    if common_keys.len() < 1 {
      return false; // cannot match, no common keys
    }
    
    let mut matching_keys = 0;
    let total_keys = common_keys.len();
    
    for common_key in common_keys {
      let my_val = self.properties.get(&common_key).unwrap();
      let re = compiled_query.get(&common_key).unwrap();
      if re.is_match(my_val) {
        matching_keys += 1;
      }
    }
    
    return matching_keys >= total_keys;
  }
  
  // Checks if this record matches the given query record (keys match, regexes, etc.)
  pub fn matches(&self, query_rec: &Record) -> bool {
    
    let mut common_keys = vec![];
    for (my_key, _) in &self.properties {
      for (their_key, _) in &query_rec.properties {
        if my_key == their_key {
          common_keys.push(my_key.clone());
        }
      }
    }
    
    if common_keys.len() < 1 {
      return false; // cannot match, no common keys
    }
    
    let mut matching_keys = 0;
    let total_keys = common_keys.len();
    
    for common_key in common_keys {
      let my_val = self.properties.get(&common_key).unwrap();
      let re = Regex::new(query_rec.properties.get(&common_key).unwrap()).unwrap();
      if re.is_match(my_val) {
        matching_keys += 1;
      }
    }
    
    return matching_keys >= total_keys;
    
  }
}

impl ::std::str::FromStr for Record {
  type Err = serde_json::error::Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    
    if s == "webpage" {
      println!("got webpage!");
      let args: Vec<String> = env::args().collect();
      let i = args.iter().position(|r| r == "webpage").unwrap();
      if let Some(url) = args.get(i+1) {
        let mut record = Record::empty();
        record.put_str("url", url);
        if let Some(title) = args.get(i+2) {
          record.put_str("title", title);
          if let Some(description) = args.get(i+3) {
            record.put_str("description", description);
          }
        }
        return Ok(record);
      }
    }
    
    let props: HashMap<String, String> = serde_json::from_str(s)?;
    Ok(Record {
      properties: props
    })
  }
}

arg_enum! {
  #[allow(non_camel_case_types)]
  #[derive(Debug, Serialize, Deserialize, Clone)]
  pub enum ArgsAction {
      query,
      publish
  }
}


#[derive(StructOpt, Debug, Serialize, Deserialize, Clone)]
#[structopt(name = "dindex", about = "A distributed index for anything and everything")]
pub struct Args {
  /// Print longer documentation
  #[structopt(short = "d", long = "docs")]
  pub docs: bool,
  
  /// Verbose mode (-v, -vv, -vvv, etc.)
  #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
  verbose: u8,
  
  #[structopt(raw(possible_values = "&ArgsAction::variants()", case_insensitive = "true"))]
  pub action: ArgsAction,
  
  pub record: Record,
  
  pub extra_args: Vec<String>,
  
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SvrArgs {
  pub action: ArgsAction,
  pub record: Record,
}

impl Args {
  pub fn into_svr_args(self) -> SvrArgs {
    SvrArgs {
      action: self.action,
      record: self.record
    }
  }
}



