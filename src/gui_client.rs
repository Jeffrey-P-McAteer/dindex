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

use piston::window::WindowSettings;
use piston::event_loop::{Events, EventSettings};
use piston_window::{Event, Input};
use glutin_window::GlutinWindow;

use crate::config;

pub fn run_sync(config: &config::Config) {
  let settings = WindowSettings::new("dIndex", (600, 400))
        .exit_on_esc(true);
  let mut window: GlutinWindow = settings.build()
        .expect("Could not create window");
  let mut events = Events::new(EventSettings::new());
  while let Some(e) = events.next(&mut window) {
    match e {
      // Event::Input(Input::Resize(rs), _option_timestamp) => {
      //   rs.window_size
      // }
      unk_e => {
        println!("unk_e={:?}", unk_e);
      }
    }
  }
}

