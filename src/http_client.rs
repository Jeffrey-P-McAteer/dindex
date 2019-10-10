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

use serde;
use serde_json;

use crate::config;
use crate::record;

#[macro_use]

macro_rules! h_map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);


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
  let client_http_custom_css = config.client_http_custom_css.clone();
  let client_http_custom_js = config.client_http_custom_js.clone();
  println!("Spawning http client on {}", ip_and_port);
  rouille::start_server(&ip_and_port, move |request| {
      match request.url().as_str() {
        "/" | "/index.html" => {
          rouille::Response::html(include_str!("http/http_index.html"))
        }
        "/style.css" => {
          rouille::Response::from_data("text/css", include_bytes!("http/http_style.css").as_ref() )
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
          rouille::Response::from_data("application/javascript", include_bytes!("http/http_app.js").as_ref() )
        }
        
        "/user_style.css" => {
          rouille::Response::from_data("text/css", client_http_custom_css.clone())
        }
        "/user_js.js" => {
          rouille::Response::from_data("application/javascript", client_http_custom_js.clone())
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
            
            // Debug/Test records
            let r1 = record::Record {
              p: h_map!{
                "title".to_string() => "Some Nonsense".to_string(),
                "url".to_string() => "http://example.org".to_string(),
                "description".to_string() => "Lorem ipsum nonsense".to_string()
              }
            };
            let r2 = record::Record {
              p: h_map!{
                "name".to_string() => "Jeffrey".to_string(),
                "phone_number".to_string() => "555-123-4444".to_string(),
                "description".to_string() => "Guy with too much code on his hands".to_string()
              }
            };
            
            let payload = if msg.len() < 1 {
              serde_json::to_string(&BrowserCmd::append(vec![])).unwrap_or(String::new())
            }
            else if msg.len() < 2 {
              serde_json::to_string(&BrowserCmd::append(vec![r1])).unwrap_or(String::new())
            }
            else if msg.len() < 3 {
              serde_json::to_string(&BrowserCmd::append(vec![r1, r2])).unwrap_or(String::new())
            }
            else {
              serde_json::to_string(&BrowserCmd::replace(vec![r1, r2])).unwrap_or(String::new())
            };
            out.send(payload)
          }
          else {
            out.send("Error: cannot process non-text data.")
          }
      }
  }).unwrap();
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct BrowserCmd {
  action: String,
  records: Vec<record::Record>
}

impl BrowserCmd {
  pub fn replace(recs: Vec<record::Record>) -> BrowserCmd {
    BrowserCmd {
      action: "clear".to_string(),
      records: recs
    }
  }
  pub fn append(recs: Vec<record::Record>) -> BrowserCmd {
    BrowserCmd {
      action: String::new(),
      records: recs
    }
  }
}
