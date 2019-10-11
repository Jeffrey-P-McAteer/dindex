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
use serde_repr;

use clap;
use clap::arg_enum;

arg_enum! {
  #[allow(non_camel_case_types)]
  #[derive(Debug, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Copy, Clone, PartialEq)]
  #[repr(u8)]
  pub enum Action {
      // We serialize as a number to guarantee a standard representation.
      query = 0,
      publish = 1,
      listen = 2,
      // The remaining arguments are NOT designed to be sent over the wire,
      // but instead are used by the CLI tool.
      run_server,
      run_http_client
  }
}
