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


/** Common type flags go here */

pub const WEBPAGE: &'static str = ":webpage";
pub const WEBPAGE_KEYS: &'static [&'static str] = &[
  "url", "title", "description"
];

pub const EMAIL: &'static str = ":email";
pub const EMAIL_KEYS: &'static [&'static str] = &[
  "name", "email"
];

pub const PHONE: &'static str = ":phone";
pub const PHONE_KEYS: &'static [&'static str] = &[
  "name", "phone"
];

pub const IMAGE: &'static str = ":image";
pub const IMAGE_KEYS: &'static [&'static str] = &[
  "image-url", "description"
];

use std::collections::HashMap;

pub fn get_all_ctypes() -> HashMap<&'static str, &'static [&'static str]> {
  let mut hm = HashMap::new();
  
  hm.insert(WEBPAGE, WEBPAGE_KEYS);
  hm.insert(EMAIL, EMAIL_KEYS);
  hm.insert(PHONE, PHONE_KEYS);
  
  return hm;
}

