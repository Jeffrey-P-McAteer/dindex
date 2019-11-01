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

use dirs;
use config;
use url::{Url};

use std::path::PathBuf;
use std::collections::HashMap;

use crate::args;

// Used for TCP and UDP listeners
pub const DINDEX_DEF_PORT: u16 = 0x1de0;
// Used for websocket listeners
pub const DINDEX_DEF_WEBSOCKET_PORT: u16 = 0x1de1;

/**
 * These are all the possible config parameters - client config
 * items come first, shared are listed second, and server parameters
 * are listed last.
 */
#[derive(Debug, Clone)]
pub struct Config {
  
  // Clients use these to transform the CLI args
  //   :webpage 'Some Title' 'http://example.org' 'Some description text'
  // into the following record
  //   {"title": "Some Title", "url", "http://example.org", "description": "Some description text"}
  pub ctypes: Vec<CType>,
  
  // Should point to RSA or ECDSA private key file;
  // clients use this to sign records
  pub client_private_key_file: String,
  
  // This is used when client is run with --http-ui option, or whenever the server is run.
  pub client_enable_http_ui: bool,
  pub client_http_port: u16,
  pub client_http_websocket_port: u16,
  pub client_http_custom_js: String,
  pub client_http_custom_css: String,
  
  // When true (either from -S flag or set in config)
  // clients sign queries and published records
  pub client_use_sig: bool,
  
  // Copied in from args, or can be specified in config .toml
  pub verbosity_level: u8,
  
  // In client: servers to query in parallel.
  // In server: federated servers to forward queries to
  pub servers: Vec<Server>,
  
  // Each entry is a complete fragment containing
  // rhai code which binds to an internal API for events/formatting logic/whatever (TODO define better)
  pub rhai_scripts: Vec<String>,
  
  // Boolean flags to turn on/off tcp/udp/unix listeners (all default to true)
  pub server_listen_tcp: bool,
  pub server_listen_udp: bool,
  pub server_listen_unix: bool,
  pub server_listen_websocket: bool,
  pub server_listen_multicast: bool,
  
  // Defaults to false, when set true server will not report
  // many useful messages. This is used to silence benchmark tests.
  pub server_extra_quiet: bool,
  
  pub server_max_listeners: usize,
  
  pub server_pid_file: String,
  
  // Servers listen on TCP and UDP on this port
  pub server_port: u16,
  pub server_websocket_port: u16,
  pub server_ip: String,
  pub server_unix_socket: String,
  pub server_multicast_group: String,
  // If more than this many threads are currently active, we will
  // block incoming connections until the oldest fraction of threads complete.
  pub server_threads_in_flight: usize,
  // Fraction of threads we wait to complete when above threadhold is reached
  pub server_threads_in_flight_fraction: f64,
  
  // URI like file:///tmp/database.json or mysql://user:pass@host/database
  // which is used to store data. Eventually a custom URL handler may be defined.
  pub server_datastore_uri: String,
  
  pub server_trusted_keys_file: String,
  
  // Servers will never remember more than this many records; oldest
  // records should be dropped first but order is not guaranteed.
  pub server_max_records: usize,
  // After N unauth websockets have connected, servers will drop oldest first.
  // No limit is applied for authenticated listening requests.
  pub server_max_unauth_websockets: usize,
  // Records are held in-memory as N RwLock-ed vectors.
  // Increasing this value will reduce write wait times,
  // decreasing (eg to 1) will mean writes must wait for ALL reads to complete.
  pub server_num_record_pools: usize,
}

#[derive(Debug, Clone)]
pub struct Server {
  pub protocol: ServerProtocol,
  // if protocol is UNIX, this is unused
  pub host: String,
  // Ignored when protocol is UNIX
  pub port: u16,
  // Only used when protocol is UNIX
  pub path: String,
  // Defaults to true, useful to silence errors when your config has a server that is usually down
  pub report_connect_errors: bool,
  pub max_latency_ms: usize,
  pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerProtocol {
  UDP, TCP, UNIX, WEBSOCKET, MULTICAST
}

#[derive(Debug, Clone)]
pub struct CType {
  pub name: String, // eg ":webpage"
  pub key_names: Vec<String>, // order matters
}

impl Config {
  pub fn is_debug(&self) -> bool {
    return cfg!(debug_assertions) || self.verbosity_level > 0;
  }
}

impl ServerProtocol {
  pub fn from_str<S: Into<String>>(s: S) -> ServerProtocol {
    let s = s.into();
    if s == "udp".to_string() || s == "UDP".to_string() {
      return ServerProtocol::UDP;
    }
    else if s == "tcp".to_string() || s == "TCP".to_string() {
      return ServerProtocol::TCP;
    }
    else if s == "websocket".to_string() || s == "WEBSOCKET".to_string() {
      return ServerProtocol::WEBSOCKET;
    }
    else if s == "multicast".to_string() || s == "MULTICAST".to_string() {
      return ServerProtocol::MULTICAST;
    }
    else {
      return ServerProtocol::UNIX;
    }
  }
}

pub fn read_config(a : &args::Args) -> Config {
  let be_verbose = cfg!(debug_assertions) || a.verbose > 0;
  let mut config = if let Some(config_file) = &a.config_file {
    get_config_detail(be_verbose, true, true, true, Ok(config_file.to_string()), a)
  }
  else {
    get_config_detail(be_verbose, true, true, true, std::env::var("DINDEX_CONF"), a)
  };
  if a.signed {
    config.client_use_sig = true;
  }
  return config;
}

pub fn get_config_detail(be_verbose: bool, check_etc: bool, check_user: bool, check_env: bool, other_config_file: Result<String, std::env::VarError>, args: &args::Args) -> Config {
  let mut settings = config::Config::default();
  
  if check_etc {
    match settings.merge(config::File::with_name("/etc/dindex")) {
      Ok(_s) => { }
      Err(e) => {
        if be_verbose {
          println!("{}", e);
        }
        return get_config_detail(be_verbose, false, check_user, check_env, other_config_file, args);
      }
    }
  }
  
  if check_user {
    let mut user_settings_path_buff = dirs::home_dir().unwrap_or(PathBuf::from(""));
    user_settings_path_buff.push(".dindex");
    
    match settings.merge(config::File::with_name( user_settings_path_buff.as_path().to_str().unwrap_or(".dindex") )) {
      Ok(_s) => { }
      Err(e) => {
        if be_verbose {
          println!("{}", e);
        }
        return get_config_detail(be_verbose, check_etc, false, check_env, other_config_file, args);
      }
    }
  }
  
  if check_env {
    match settings.merge(config::Environment::with_prefix("DINDEX")) {
      Ok(_s) => { }
      Err(e) => {
        if be_verbose {
          println!("{}", e);
        }
        return get_config_detail(be_verbose, check_etc, check_user, false, other_config_file, args);
      }
    }
  }
  
  if let Ok(config_file_str) = other_config_file {
    match settings.merge(config::File::with_name( &config_file_str )) {
      Ok(_s) => { }
      Err(e) => {
        if be_verbose {
          println!("{}", e);
        }
        return get_config_detail(be_verbose, check_etc, false, check_env, Err(std::env::VarError::NotPresent), args);
      }
    }
  }
  
  // Now read in, setting defaults where empty
  return Config {
    ctypes: s_get_ctype_vec(be_verbose, &settings, "ctypes"),
    client_private_key_file: s_get_str(be_verbose, &settings, "client_private_key_file", ""),
    client_enable_http_ui: s_get_bool(be_verbose, &settings, "client_enable_http_ui", true),
    client_http_port: s_get_i64(be_verbose, &settings, "client_http_port", 8080) as u16,
    client_http_websocket_port: s_get_i64(be_verbose, &settings, "client_http_websocket_port", 8081) as u16,
    client_http_custom_js: s_get_str(be_verbose, &settings, "client_http_custom_js", include_str!("http/example_custom_js.js")),
    client_http_custom_css: s_get_str(be_verbose, &settings, "client_http_custom_css", include_str!("http/example_custom_css.css")),
    client_use_sig: s_get_bool(be_verbose, &settings, "client_use_sig", false),
    verbosity_level: s_get_i64(be_verbose, &settings, "verbosity_level", args.verbose as i64) as u8,
    servers: s_get_server_vec(be_verbose, &settings, "servers"),
    rhai_scripts: s_get_rhai_script_vec(be_verbose, &settings, "rhai_scripts"),
    server_port: s_get_i64(be_verbose, &settings, "server_port", DINDEX_DEF_PORT as i64) as u16,
    server_websocket_port: s_get_i64(be_verbose, &settings, "server_websocket_port", DINDEX_DEF_WEBSOCKET_PORT as i64) as u16,
    server_listen_tcp: s_get_bool(be_verbose, &settings, "server_listen_tcp", true),
    server_listen_udp: s_get_bool(be_verbose, &settings, "server_listen_udp", true),
    server_listen_unix: s_get_bool(be_verbose, &settings, "server_listen_unix", true),
    server_listen_websocket: s_get_bool(be_verbose, &settings, "server_listen_websocket", true),
    server_listen_multicast: s_get_bool(be_verbose, &settings, "server_listen_multicast", true),
    server_extra_quiet: s_get_bool(be_verbose, &settings, "server_extra_quiet", false),
    server_max_listeners: s_get_i64(be_verbose, &settings, "server_max_listeners", 100) as usize,
    server_pid_file: s_get_str(be_verbose, &settings, "server_pid_file", "/tmp/dindex.pid"),
    server_ip: s_get_str(be_verbose, &settings, "server_ip", "0.0.0.0"),
    server_unix_socket: s_get_str(be_verbose, &settings, "server_unix_socket", "/tmp/dindex.sock"),
    server_multicast_group: s_get_str(be_verbose, &settings, "server_multicast_group", "239.255.29.224"), // Last 2 bytes are 0x1de0 (same as port, attempt to l33t "index")
    server_threads_in_flight: s_get_i64(be_verbose, &settings, "server_threads_in_flight", 8) as usize,
    server_threads_in_flight_fraction: s_get_f64(be_verbose, &settings, "server_threads_in_flight_fraction", 0.25),
    server_datastore_uri: s_get_str(be_verbose, &settings, "server_datastore_uri", "file:///tmp/dindex_db.json"),
    server_trusted_keys_file: s_get_str(be_verbose, &settings, "server_trusted_keys_file", "/tmp/dindex_trusted_keys"),
    server_max_records: s_get_i64(be_verbose, &settings, "server_max_records", 4096) as usize,
    server_max_unauth_websockets: s_get_i64(be_verbose, &settings, "server_max_unauth_websockets", 100) as usize,
    server_num_record_pools: s_get_i64(be_verbose, &settings, "server_num_record_pools", 8) as usize,
  };
}

// High-level helper methods

fn s_get_server_vec(be_verbose :bool, settings: &config::Config, array_name: &str) -> Vec<Server> {
  let mut servers = vec![];
  match settings.get_array(array_name) {
    Ok(vals) => {
      for s_val in vals {
        match s_val.into_table() {
          Ok(val_map) => {
            let report_connect_errors = v_get_bool_of(be_verbose, &val_map, "report_connect_errors", true);
            let name = v_get_str_of(be_verbose, &val_map, "name", "Unnamed");
            let uri_s = v_get_str_of(be_verbose, &val_map, "uri", "unix:///tmp/dindex.sock");
            if let Ok(uri) = Url::parse(&uri_s) {
              
              // Some defaults are protocol-dependent
              let protocol = ServerProtocol::from_str( uri.scheme() );
              let def_port;
              if protocol == ServerProtocol::WEBSOCKET {
                def_port = DINDEX_DEF_WEBSOCKET_PORT;
              }
              else {
                def_port = DINDEX_DEF_PORT;
              }
              
              servers.push(Server {
                protocol: protocol,
                host: uri.host().unwrap_or(url::Host::Domain("localhost")).to_string(),
                path: uri.path().to_string(),
                port: uri.port().unwrap_or(def_port) as u16,
                report_connect_errors: report_connect_errors,
                max_latency_ms: v_get_i64_of(be_verbose, &val_map, "max_latency_ms", 600) as usize,
                name: name,
              });
            }
          }
          Err(e) => {
            if be_verbose {
              println!("{}", e);
            }
          }
        }
      }
    }
    Err(e) => {
      if be_verbose {
        println!("{}", e);
      }
      // This is the default record used if nothing is configured
      servers.push(Server {
        protocol: ServerProtocol::MULTICAST,
        host: "239.255.29.224".to_string(),
        port: DINDEX_DEF_PORT,
        path: String::new(),
        report_connect_errors: true,
        max_latency_ms: 600,
        name: "Default LAN Connection".to_string()
      });
      servers.push(Server {
        protocol: ServerProtocol::TCP,
        host: "127.0.0.1".to_string(),
        port: DINDEX_DEF_PORT,
        path: String::new(),
        report_connect_errors: true,
        max_latency_ms: 600,
        name: "Default localhost TCP Connection".to_string()
      });
    }
  }
  return servers;
}

fn s_get_rhai_script_vec(be_verbose :bool, settings: &config::Config, array_name: &str) -> Vec<String> {
  let mut scripts = vec![];
  match settings.get_array(array_name) {
    Ok(vals) => {
      for s_val in vals {
        match s_val.into_table() {
          Ok(val_map) => {
            let file_s = v_get_str_of(be_verbose, &val_map, "file", "");
            if file_s.len() > 1 {
              // User specified "file", read it in
              match std::fs::read_to_string(file_s) {
                Ok(source_s) => {
                  scripts.push(source_s);
                }
                Err(e) => {
                  if be_verbose {
                    println!("{}", e);
                  }
                }
              }
            }
            else {
              // Assume user specified "source"
              let source_s = v_get_str_of(be_verbose, &val_map, "source", "");
              if source_s.len() > 1 {
                scripts.push(source_s);
              }
            }
          }
          Err(e) => {
            if be_verbose {
              println!("{}", e);
            }
          }
        }
      }
    }
    Err(e) => {
      if be_verbose {
        println!("{}", e);
      }
      scripts.push(include_str!("conf/rhai_defaults.rhai").to_string());
    }
  }
  return scripts;
}

fn s_get_ctype_vec(be_verbose :bool, settings: &config::Config, array_name: &str) -> Vec<CType> {
  let mut ctypes = vec![];
  match settings.get_array(array_name) {
    Ok(vals) => {
      for s_val in vals {
        match s_val.into_table() {
          Ok(val_map) => {
            ctypes.push(CType {
              name: v_get_str_of(be_verbose, &val_map, "name", ":unk"),
              key_names: v_get_str_vec(be_verbose, &val_map, "key_names", vec![])
            });
          }
          Err(e) => {
            if be_verbose {
              println!("{}", e);
            }
          }
        }
      }
    }
    Err(e) => {
      if be_verbose {
        println!("{}", e);
      }
      // This is the default ctype used if nothing is configured
      ctypes.push(CType {
        name: ":webpage".to_string(),
        key_names: vec![
          "title".to_string(),
          "url".to_string(),
          "description".to_string(),
        ]
      });
    }
  }
  return ctypes;
}

// Low-level helper methods to parse data

fn s_get_str_vec(be_verbose :bool, settings: &config::Config, key: &str, default: Vec<String>) -> Vec<String> {
  match settings.get_array(key) {
    Ok(val_vec) => {
      let mut s_vec: Vec<String> = vec![];
      for val in val_vec {
        match val.into_str() {
          Ok(str_val) => {
            s_vec.push(str_val);
          }
          Err(e) => {
            if be_verbose {
              println!("{}", e);
            }
          }
        }
      }
      return s_vec;
    }
    Err(e) => {
      if be_verbose {
        println!("{}", e);
      }
      return default;
    }
  }
}

fn v_get_str_vec(be_verbose: bool, settings: &HashMap<String, config::Value>, key: &str, default: Vec<String>) -> Vec<String> {
  match settings.get(key) {
    Some(val) => {
      let val = val.clone(); // TODO not this
      match val.into_array() {
        Ok(arr_val) => {
          let mut res: Vec<String> = vec![];
          for a_val in arr_val {
            match a_val.into_str() {
              Ok(a_str) => {
                res.push(a_str);
              }
              Err(e) => {
                if be_verbose {
                  println!("{}", e);
                }
              }
            }
          }
          return res;
        }
        Err(e) => {
          if be_verbose {
            println!("{}", e);
          }
          return default;
        }
      }
    }
    None => {
      if be_verbose {
        println!("No key found (v_get_str_vec): {}", key);
      }
      return default;
    }
  }
}

fn s_get_str(be_verbose: bool, settings: &config::Config, key: &str, default: &str) -> String {
  match settings.get_str(key) {
    Ok(val) => { return val; }
    Err(e) => {
      if be_verbose {
        println!("{}", e);
      }
      return default.to_string();
    }
  }
}

fn v_get_str_of(be_verbose: bool, src: &HashMap<String, config::Value>, key: &str, default: &str) -> String {
  match src.get(key) {
    Some(conf_val) => {
      match conf_val.clone().into_str() { // TODO can we design-out this clone()?
        Ok(str_val) => { return str_val; }
        Err(e) => {
          if be_verbose {
            println!("{}", e);
          }
          return default.to_string();
        }
      }
    }
    None => {
      return default.to_string();
    }
  }
}

fn v_get_bool_of(be_verbose: bool, src: &HashMap<String, config::Value>, key: &str, default: bool) -> bool {
  match src.get(key) {
    Some(conf_val) => {
      match conf_val.clone().into_bool() { // TODO can we design-out this clone()?
        Ok(bool_val) => { return bool_val; }
        Err(e) => {
          if be_verbose {
            println!("{}", e);
          }
          return default;
        }
      }
    }
    None => {
      return default;
    }
  }
}

fn s_get_i64(be_verbose: bool, settings: &config::Config, key: &str, default: i64) -> i64 {
  match settings.get_int(key) {
    Ok(val) => { return val; }
    Err(e) => {
      if be_verbose {
        println!("{}", e);
      }
      return default;
    }
  }
}

fn s_get_f64(be_verbose: bool, settings: &config::Config, key: &str, default: f64) -> f64 {
  match settings.get_float(key) {
    Ok(val) => { return val; }
    Err(e) => {
      if be_verbose {
        println!("{}", e);
      }
      return default;
    }
  }
}

fn s_get_bool(be_verbose: bool, settings: &config::Config, key: &str, default: bool) -> bool {
  match settings.get_bool(key) {
    Ok(val) => { return val; }
    Err(e) => {
      if be_verbose {
        println!("{}", e);
      }
      return default;
    }
  }
}

fn v_get_i64_of(be_verbose: bool, src: &HashMap<String, config::Value>, key: &str, default: i64) -> i64 {
  match src.get(key) {
    Some(conf_val) => {
      match conf_val.clone().into_int() { // TODO can we design-out this clone()?
        Ok(int_val) => { return int_val; }
        Err(e) => {
          if be_verbose {
            println!("{}", e);
          }
          return default;
        }
      }
    }
    None => {
      return default;
    }
  }
}

