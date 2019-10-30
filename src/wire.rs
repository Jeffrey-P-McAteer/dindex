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

use crate::actions::Action;
use crate::record::Record;

// This represents data send to/from servers and clients over
// any tcp, udp, or unix socket connection.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct WireData {
  pub action: Action,
  pub record: Record,
}

impl WireData {
  pub fn result(record: Record) -> WireData {
    WireData {
      action: Action::result,
      record: record,
    }
  }
  pub fn end_of_results() -> WireData {
    WireData {
      action: Action::end_of_results,
      record: Record::empty(),
    }
  }
}
