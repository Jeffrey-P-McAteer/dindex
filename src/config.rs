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

use std::path::PathBuf;
use std::collections::HashMap;

use crate::args;

pub const DINDEX_DEF_PORT: u16 = 0x1de0;

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
  
  // In client: servers to query in parallel.
  // In server: federated servers to forward queries to
  pub servers: Vec<Server>,
  
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
  // if protocol is UNIX, this is a file path
  pub host: String,
  // Ignored when protocol is UNIX
  pub port: u16,
  pub max_latency_ms: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerProtocol {
  UDP, TCP, UNIX,
}

#[derive(Debug, Clone)]
pub struct CType {
  pub name: String, // eg ":webpage"
  pub key_names: Vec<String>, // order matters
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
    else {
      return ServerProtocol::UNIX;
    }
  }
}

pub fn read_config(a : &args::Args) -> Config {
  if let Some(config_file) = &a.config_file {
    return get_config_detail(cfg!(debug_assertions), true, true, true, Ok(config_file.to_string()));
  }
  else {
    return get_config_detail(cfg!(debug_assertions), true, true, true, std::env::var("DINDEX_CONF"));
  }
}

pub fn get_config_detail(be_verbose: bool, check_etc: bool, check_user: bool, check_env: bool, other_config_file: Result<String, std::env::VarError>) -> Config {
  let mut settings = config::Config::default();
  
  if check_etc {
    match settings.merge(config::File::with_name("/etc/dindex")) {
      Ok(_s) => { }
      Err(e) => {
        if be_verbose {
          println!("{}", e);
        }
        return get_config_detail(be_verbose, false, check_user, check_env, other_config_file);
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
        return get_config_detail(be_verbose, check_etc, false, check_env, other_config_file);
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
        return get_config_detail(be_verbose, check_etc, check_user, false, other_config_file);
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
        return get_config_detail(be_verbose, check_etc, false, check_env, Err(std::env::VarError::NotPresent));
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
    servers: s_get_server_vec(be_verbose, &settings, "servers"),
    server_max_records: s_get_i64(be_verbose, &settings, "server_max_records", 8080) as usize,
    server_max_unauth_websockets: s_get_i64(be_verbose, &settings, "server_max_unauth_websockets", 8080) as usize,
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
            servers.push(Server {
              protocol: ServerProtocol::from_str( v_get_str_of(be_verbose, &val_map, "type", "udp") ),
              host: v_get_str_of(be_verbose, &val_map, "host", "localhost"),
              port: v_get_i64_of(be_verbose, &val_map, "port", DINDEX_DEF_PORT as i64) as u16,
              max_latency_ms: v_get_i64_of(be_verbose, &val_map, "max_latency_ms", 600) as usize,
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
      // This is the default record used if nothing is configured
      servers.push(Server {
        protocol: ServerProtocol::TCP,
        host: "dindex.jmcateer.pw".to_string(),
        port: DINDEX_DEF_PORT,
        max_latency_ms: 600,
      });
    }
  }
  return servers;
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

