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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Record {
  pub p: HashMap<String, String>,
}

impl Record {
  pub fn matches(&self, query: &HashMap<String, Regex>) -> bool {
    false // TODO
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
