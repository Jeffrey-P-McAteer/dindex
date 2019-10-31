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

use rhai::Engine;

use crate::config::Config;

pub struct ScriptEngine {
  pub rhai_engine: Engine,
}

impl ScriptEngine {
  pub fn new() -> ScriptEngine {
    let mut engine = Engine::new();
    // Setup our API code
    
    return ScriptEngine {
      rhai_engine: engine,
    };
  }
  pub fn run_user_scripts(&mut self, config: &Config) {
    for user_s in &config.rhai_scripts {
      if let Err(e) = self.rhai_engine.eval::<()>(&user_s) {
        println!("e = {}", e);
      }
    }
  }
}
