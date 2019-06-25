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

extern crate config;
extern crate dirs;

extern crate serde;

extern crate clap;

use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Resolver {
  pub host: String,
  pub port: u16,
  pub max_latency_ms: usize,
}

impl Resolver {
  pub fn get_host_port_s(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }
}


#[derive(Debug)]
pub struct Config {
  pub listen_ip: String,
  pub listen_port: u16,
  pub query_padding_bytes: usize,
  pub anon_max_bytes_sent_per_ip_per_sec: usize,
  pub trusted_ip_sources: Vec<String>,
  pub identity_private_key_file: String,
  pub identity_public_key_file: String,
  pub cache_dir: String,
  pub cache_max_bytes: usize,
  pub upstream_resolvers: Vec<Resolver>,
}

impl Config {
  pub fn get_ip_port(&self) -> String {
    format!("{}:{}", self.listen_ip, self.listen_port)
  }
}

pub fn get_config() -> Config {
  get_config_detail(false, true, true, true)
}

/**
 * Reads in config from files + environment variables.
 */
pub fn get_config_detail(be_verbose: bool, check_etc: bool, check_user: bool, check_env: bool) -> Config {
  let mut settings = config::Config::default();
  
  if check_etc {
    match settings.merge(config::File::with_name("/etc/dindex")) {
      Ok(_s) => { }
      Err(e) => {
        if be_verbose {
          println!("{}", e);
        }
        return get_config_detail(be_verbose, false, check_user, check_env);
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
        return get_config_detail(be_verbose, check_etc, false, check_env);
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
        return get_config_detail(be_verbose, check_etc, check_user, false);
      }
    }
  }
  
  // Now read in, setting defaults where empty
  let mut c = Config {
    listen_ip:
        s_get_str(be_verbose, &settings, "listen_ip", "0.0.0.0"),
    listen_port:
        s_get_i64(be_verbose, &settings, "listen_port", 0x1de0) as u16,
    query_padding_bytes:
        s_get_i64(be_verbose, &settings, "query_padding_bytes", 4096) as usize,
    anon_max_bytes_sent_per_ip_per_sec:
        s_get_i64(be_verbose, &settings, "anon_max_bytes_sent_per_ip_per_sec", 16384) as usize,
    trusted_ip_sources:
        s_get_str_vec(be_verbose, &settings, "trusted_ip_sources", vec!["127.0.0.1".to_string()]),
    identity_private_key_file:
        s_get_str(be_verbose, &settings, "identity_private_key_file", "/dev/null"),
    identity_public_key_file:
        s_get_str(be_verbose, &settings, "identity_public_key_file", "/dev/null"),
    cache_dir:
        s_get_str(be_verbose, &settings, "cache_dir", "/tmp/dindex_cache/"),
    cache_max_bytes:
        s_get_i64(be_verbose, &settings, "cache_max_bytes", 16384) as usize,
    upstream_resolvers:
        vec![],
  };
  
  match settings.get_array("upstream_resolvers") {
    Ok(vals) => {
      for s_val in vals {
        match s_val.into_table() {
          Ok(val_map) => {
            c.upstream_resolvers.push(Resolver {
              host: v_get_str_of(be_verbose, &val_map, "host", "localhost"),
              port: v_get_i64_of(be_verbose, &val_map, "port", 0x1de0) as u16,
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
      c.upstream_resolvers.push(Resolver {
        host: "dindex.jmcateer.pw".to_string(),
        port: 0x1de0,
        max_latency_ms: 600,
      });
    }
  }
  
  return c;
}

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
