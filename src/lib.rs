extern crate config;
extern crate dirs;

use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Resolver {
  host: String,
  port: u16,
  max_latency_ms: usize,
}

#[derive(Debug)]
pub struct Config {
  listen_ip: String,
  listen_port: u16,
  cache_dir: String,
  upstream_resolvers: Vec<Resolver>,
}

pub fn get_config() -> Config {
  get_config_detail(true, true, true)
}

/**
 * Reads in config from files + environment variables.
 */
pub fn get_config_detail(check_etc: bool, check_user: bool, check_env: bool) -> Config {
  let mut settings = config::Config::default();
  
  if check_etc {
    match settings.merge(config::File::with_name("/etc/dindex")) {
      Ok(_s) => { }
      Err(e) => {
        println!("{}", e);
        return get_config_detail(false, check_user, check_env);
      }
    }
  }
  
  if check_user {
    let mut user_settings_path_buff = dirs::home_dir().unwrap_or(PathBuf::from(""));
    user_settings_path_buff.push(".dindex");
    
    match settings.merge(config::File::with_name( user_settings_path_buff.as_path().to_str().unwrap_or(".dindex") )) {
      Ok(_s) => { }
      Err(e) => {
        println!("{}", e);
        return get_config_detail(check_etc, false, check_env);
      }
    }
  }
  
  if check_env {
    match settings.merge(config::Environment::with_prefix("DINDEX")) {
      Ok(_s) => { }
      Err(e) => {
        println!("{}", e);
        return get_config_detail(check_etc, check_user, false);
      }
    }
  }
  
  // Now read in, setting defaults where empty
  let mut c = Config {
    listen_ip: s_get_str(&settings, "listen_ip", "0.0.0.0"),
    listen_port: s_get_i64(&settings, "listen_port", 0x1de0) as u16,
    cache_dir: s_get_str(&settings, "cache_dir", "/tmp/dindex_cache/"),
    upstream_resolvers: vec![],
  };
  
  match settings.get_array("upstream_resolvers") {
    Ok(vals) => {
      for s_val in vals {
        match s_val.into_table() {
          Ok(val_map) => {
            c.upstream_resolvers.push(Resolver {
              host: v_get_str_of(&val_map, "host", "localhost"),
              port: v_get_i64_of(&val_map, "port", 0x1de0) as u16,
              max_latency_ms: v_get_i64_of(&val_map, "max_latency_ms", 600) as usize,
            });
          }
          Err(e) => {
            println!("{}", e);
          }
        }
      }
    }
    Err(e) => {
      println!("{}", e);
      c.upstream_resolvers.push(Resolver {
        host: "dindex.jmcateer.pw".to_string(),
        port: 0x1de0,
        max_latency_ms: 600,
      });
    }
  }
  
  return c;
}


fn s_get_str(settings: &config::Config, key: &str, default: &str) -> String {
  match settings.get_str(key) {
    Ok(val) => { return val; }
    Err(e) => {
      println!("{}", e);
      return default.to_string();
    }
  }
}

fn v_get_str_of(src: &HashMap<String, config::Value>, key: &str, default: &str) -> String {
  match src.get(key) {
    Some(conf_val) => {
      match conf_val.clone().into_str() { // TODO can we design-out this clone()?
        Ok(str_val) => { return str_val; }
        Err(e) => {
          println!("{}", e);
          return default.to_string();
        }
      }
    }
    None => {
      return default.to_string();
    }
  }
}

fn s_get_i64(settings: &config::Config, key: &str, default: i64) -> i64 {
  match settings.get_int(key) {
    Ok(val) => { return val; }
    Err(e) => {
      println!("{}", e);
      return default;
    }
  }
}

fn v_get_i64_of(src: &HashMap<String, config::Value>, key: &str, default: i64) -> i64 {
  match src.get(key) {
    Some(conf_val) => {
      match conf_val.clone().into_int() { // TODO can we design-out this clone()?
        Ok(int_val) => { return int_val; }
        Err(e) => {
          println!("{}", e);
          return default;
        }
      }
    }
    None => {
      return default;
    }
  }
}