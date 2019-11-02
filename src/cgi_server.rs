/*
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

// Good test one-liners:
// mkdir /tmp/cgi-bin ; ln -s $PWD/target/release/dindex /tmp/cgi-bin/index.cgi
// (cd /tmp ; python3 -m http.server --cgi --bind 127.0.0.1 )

use cgi;
use cgi::http::method::Method;
use cgi::http::header::HeaderValue;

use crate::actions;
use crate::record::Record;

// Fast function to detect if the binary is being
// run in a CGI environment.
pub fn should_perform_cgi() -> bool {
  use std::env;
  // Different CGI servers define different variables,
  // and we must remember to avoid false positives.
  env::var("REQUEST_METHOD").is_ok() || env::var("REQUEST_URI").is_ok()
}

pub fn perform_cgi() {
  cgi::handle(|req: cgi::Request| -> cgi::Response {
    let (parts, body) = req.into_parts();
    match parts.method {
      Method::GET => {
        cgi::html_response(200, include_str!("http/cgi_index.html"))
      }
      Method::POST => {
        let empty_string = String::new();
        let action_hv_default = HeaderValue::from_static("query");
        let action_hv: &HeaderValue = parts.headers.get("X-DINDEX-ACTION").unwrap_or(
          parts.headers.get("x-dindex-action").unwrap_or(
            &action_hv_default
          )
        );
        let action = actions::action_from_str(action_hv.to_str().unwrap_or(&empty_string));
        match action {
          actions::Action::query | actions::Action::publish => {
            match serde_json::from_slice::<Record>(&body) {
              Ok(record) => {
                if action == actions::Action::query {
                  
                }
                else { // must be publish
                  
                }
                cgi::html_response(200, format!("We will publish {:?}", body))
              }
              Err(e) => {
                cgi::html_response(400, format!("Error parsing record: {}", e))
              }
            }
          }
          unk => {
            cgi::html_response(400, format!("Invalid action \"{:?}\"", unk))
          }
        }
      }
      unk => {
        cgi::html_response(405, format!("Unsupported HTTP method: {}", unk))
      }
    }
  });
}
