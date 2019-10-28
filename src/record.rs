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

use serde;
use regex::Regex;

use std::collections::HashMap;

use crate::signing;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Record {
  pub p: HashMap<String, String>,
}

impl Record {
  pub fn empty() -> Record {
    Record {
      p: HashMap::new()
    }
  }
  pub fn is_empty(&self) -> bool {
    self.p.is_empty()
  }
  pub fn is_signed(&self) -> bool {
    signing::is_valid_sig(self)
  }
  pub fn pub_key(&self) -> String {
    let empty_str = String::new();
    let pub_key_val = self.p.get("public-key").unwrap_or(&empty_str);
    return format!("{}", pub_key_val);
  }
  pub fn matches(&self, query: &HashMap<String, Regex>) -> bool {
    let mut common_keys = vec![];
    for key in self.p.keys() {
      if query.contains_key(key) {
        common_keys.push(key);
      }
    }
    if common_keys.len() < 1 {
      return false;
    }
    // Only a match if ALL key reges searches match
    for key in common_keys {
      let re = query.get(key).unwrap();
      let val = self.p.get(key).unwrap();
      if ! re.is_match(val) {
        return false; // One of the keys failed, none of this is a match
      }
    }
    // All shared key regexes matched, this is a match
    return true;
  }
  pub fn create_regex_map(&self) -> HashMap<String, Regex> {
    let mut map = HashMap::new();
    for (key, val) in &self.p {
      if let Ok(r) = Regex::new(&val) {
        map.insert(key.to_string(), r);
      }
    }
    return map;
  }
}
