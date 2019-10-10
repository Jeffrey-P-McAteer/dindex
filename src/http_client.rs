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

use rouille;
use ws;

use crossbeam_utils::thread;

use crate::config;

pub fn run_sync(config: &config::Config) {
  thread::scope(|s| {
    let h1 = s.spawn(move |_| {
      run_websocket_sync(config);
    });
    let h2 = s.spawn(move |_| {
      run_http_sync(config);
    });
    
    h1.join().unwrap();
    h2.join().unwrap();
  }).unwrap();
}

fn run_http_sync(config: &config::Config) {
  let ip_and_port = format!("127.0.0.1:{}", config.client_http_port);
  let client_http_websocket_port = config.client_http_websocket_port.clone();
  println!("Spawning http client on {}", ip_and_port);
  rouille::start_server(&ip_and_port, move |request| {
      match request.url().as_str() {
        "/" | "/index.html" => {
          rouille::Response::html(include_str!("http_index.html"))
        }
        "/style.css" => {
          rouille::Response::from_data("text/css", include_bytes!("http_style.css").as_ref() )
        }
        "/config.js" => {
          // Used to tell the client some config variables
          rouille::Response::from_data("application/javascript", format!(r#"
window.client_http_websocket_port = {};
            "#,
            client_http_websocket_port)
          )
        }
        "/app.js" => {
          rouille::Response::from_data("application/javascript", include_bytes!("http_app.js").as_ref() )
        }
        unk_path => {
          rouille::Response::text(format!("Unknown path {}", unk_path))
        }
      }
  });
}

fn run_websocket_sync(config: &config::Config) {
  let ip_and_port = format!("127.0.0.1:{}", config.client_http_websocket_port);
  ws::listen(&ip_and_port, |out| {
      move |msg: ws::Message| {
          // msg contains raw typed in search query
          if let ws::Message::Text(msg) = msg {
            if msg.len() < 1 {
              out.send("{\"records\": []}")
            }
            else if msg.len() < 2 {
              out.send("{\"records\": [\"a\"]}")
            }
            else if msg.len() < 3 {
              out.send("{\"records\": [\"a\", \"b\"]}")
            }
            else {
              out.send("{\"records\": [\"a\", \"b\", \"c\"]}")
            }
          }
          else {
            out.send("Error: cannot process non-text data.")
          }
      }
  }).unwrap();
}
