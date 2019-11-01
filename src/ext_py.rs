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

use cpython;
use cpython::{PyResult, PyDict, PyLong, PyString, Python};

use structopt::StructOpt;

use std::collections::HashMap;

use crate::record::Record;
use crate::config;
use crate::config::Config;
use crate::args::Args;
use crate::disp;
use crate::client;
use crate::actions;

use crate::py_attr_map_dict;
use crate::attr_from_py_dict;

// add bindings to the generated python module
// N.B: names: "libdindex" must be the name of the `.so` or `.pyd` file
py_module_initializer!(libdindex, initlibdindex, PyInit_libdindex, |py, m| {
  m.add(py, "__doc__", r#"
dIndex python bindings.
"#)?;
  m.add(py, "args",
    py_fn!(py, get_args()))?;
  m.add(py, "config",
    py_fn!(py, get_config(args: Option<Args> = None)))?;
  m.add(py, "record",
    py_fn!(py, get_record(record: Record = Record::empty() ) ))?;
  m.add(py, "record_display",
    py_fn!(py, record_display(config: Config, record: Record = Record::empty() ) ))?;
  m.add(py, "record_display_vec",
    py_fn!(py, record_display_vec(config: Config, record: Vec<Record> = vec![] ) ))?;
  m.add(py, "client_query_sync",
    py_fn!(py, client_query_sync(config: Config, record: Record = Record::empty() ) ))?;
  Ok(())
});

/*
 * Useful/Required implementations of dindex structures into/out of python objects
 */

impl cpython::ToPyObject for Args {
  type ObjectType = PyDict;
  fn to_py_object(&self, py: Python) -> Self::ObjectType {
    let py_dict: PyDict = PyDict::new(py);
    
    py_attr_map_dict!(py, py_dict, "config_file", self.config_file.clone());
    py_attr_map_dict!(py, py_dict, "max_web_scan_depth", self.max_web_scan_depth);
    py_attr_map_dict!(py, py_dict, "verbose", self.verbose);
    py_attr_map_dict!(py, py_dict, "action", format!("{}", self.action));
    py_attr_map_dict!(py, py_dict, "signed", self.signed);
    py_attr_map_dict!(py, py_dict, "rec_args", self.rec_args.clone());
    
    return py_dict;
  }
}

impl <'source> cpython::FromPyObject<'source> for actions::Action {
  fn extract(py: Python, obj: &'source cpython::PyObject) -> PyResult<Self> {
    let py_str: PyString = obj.extract(py)?;
    return Ok(actions::action_from_str(
      &py_str.to_string(py).unwrap_or(std::borrow::Cow::Borrowed(&String::new()))
    ));
  }
}

impl <'source> cpython::FromPyObject<'source> for Record {
  fn extract(py: Python, obj: &'source cpython::PyObject) -> PyResult<Self> {
    let py_dict: PyDict = obj.extract(py)?;
    let mut map = HashMap::new();
    
    for (key, val) in py_dict.items(py) {
      if let Ok(key) = key.extract(py) {
        let key: String = key;
        if let Ok(val) = val.extract(py) {
          let val: String = val;
          map.insert(key, val);
        }
      }
    }
    
    return Ok(Record {
      p: map,
      src_server: None,
    });
  }
}

impl cpython::ToPyObject for Record {
  type ObjectType = PyDict;
  fn to_py_object(&self, py: Python) -> Self::ObjectType {
    let py_dict: PyDict = PyDict::new(py);

    for (key, val) in &self.p {
      if let Err(e) = py_dict.set_item(py, key, val) {
        println!("e = {:?}", e);
      }
    }

    return py_dict;
  }
}

impl cpython::ToPyObject for config::Server {
  type ObjectType = PyDict;
  fn to_py_object(&self, py: Python) -> Self::ObjectType {
    let py_dict: PyDict = PyDict::new(py);
    
    py_attr_map_dict!(py, py_dict, "protocol", format!("{:?}", self.protocol));
    py_attr_map_dict!(py, py_dict, "host", self.host.clone());
    py_attr_map_dict!(py, py_dict, "port", self.port);
    py_attr_map_dict!(py, py_dict, "path", self.path.clone());
    py_attr_map_dict!(py, py_dict, "report_connect_errors", self.report_connect_errors);
    py_attr_map_dict!(py, py_dict, "max_latency_ms", self.max_latency_ms);
    py_attr_map_dict!(py, py_dict, "name", self.name.clone());
    
    return py_dict;
  }
}

impl cpython::ToPyObject for config::CType {
  type ObjectType = PyDict;
  fn to_py_object(&self, py: Python) -> Self::ObjectType {
    let py_dict: PyDict = PyDict::new(py);
    
    py_attr_map_dict!(py, py_dict, "name", self.name.clone());
    py_attr_map_dict!(py, py_dict, "key_names", self.key_names.clone());
    
    return py_dict;
  }
}

impl <'source> cpython::FromPyObject<'source> for Args {
  fn extract(py: Python, obj: &'source cpython::PyObject) -> PyResult<Self> {
    let py_dict: PyDict = obj.extract(py)?;
    
    let config_file = attr_from_py_dict!(py, py_dict, "config_file", None, Option<String> );
    let max_web_scan_depth = attr_from_py_dict!(py, py_dict, "max_web_scan_depth", 12, usize );
    let verbose = attr_from_py_dict!(py, py_dict, "verbose", 0, u8 );
    let action = attr_from_py_dict!(py, py_dict, "action", actions::Action::no_action, actions::Action );
    let signed = attr_from_py_dict!(py, py_dict, "signed", false, bool );
    let rec_args = attr_from_py_dict!(py, py_dict, "rec_args", vec![], Vec<String> );
    
    Ok(Args {
      config_file: config_file,
      max_web_scan_depth: max_web_scan_depth,
      verbose: verbose,
      action: action,
      signed: signed,
      rec_args: rec_args,
    })
  }
}

impl <'source> cpython::FromPyObject<'source> for config::CType {
  fn extract(py: Python, obj: &'source cpython::PyObject) -> PyResult<Self> {
    let py_dict: PyDict = obj.extract(py)?;
    
    let name = attr_from_py_dict!(py, py_dict, "name", "".to_string(), String );
    let key_names = attr_from_py_dict!(py, py_dict, "key_names", vec![], Vec<String> );
    
    Ok(config::CType {
      name: name,
      key_names: key_names,
    })
  }
}

impl <'source> cpython::FromPyObject<'source> for config::Server {
  fn extract(py: Python, obj: &'source cpython::PyObject) -> PyResult<Self> {
    let py_dict: PyDict = obj.extract(py)?;
    
    let protocol = attr_from_py_dict!(py, py_dict, "protocol", config::ServerProtocol::TCP, config::ServerProtocol );
    let host = attr_from_py_dict!(py, py_dict, "host", String::new(), String );
    let port = attr_from_py_dict!(py, py_dict, "port", config::DINDEX_DEF_PORT as u16, u16 );
    let path = attr_from_py_dict!(py, py_dict, "path", String::new(), String );
    let report_connect_errors = attr_from_py_dict!(py, py_dict, "report_connect_errors", true, bool );
    let max_latency_ms = attr_from_py_dict!(py, py_dict, "max_latency_ms", 600, usize );
    let name = attr_from_py_dict!(py, py_dict, "name", String::new(), String );
    
    Ok(config::Server {
      protocol: protocol,
      host: host,
      port: port,
      path: path,
      report_connect_errors: report_connect_errors,
      max_latency_ms: max_latency_ms,
      name: name,
    })
  }
}

impl <'source> cpython::FromPyObject<'source> for config::ServerProtocol {
  fn extract(py: Python, obj: &'source cpython::PyObject) -> PyResult<Self> {
    let py_str: PyString = obj.extract(py)?;
    Ok(config::ServerProtocol::from_str(
      // f---- off type system and gimme a string
      format!("{}", py_str.to_string(py).unwrap_or(std::borrow::Cow::Borrowed(&String::new())))
    ))
  }
}

impl cpython::ToPyObject for config::Config {
  type ObjectType = PyDict;
  fn to_py_object(&self, py: Python) -> Self::ObjectType {
    let py_dict: PyDict = PyDict::new(py);
    
    py_attr_map_dict!(py, py_dict, "ctypes", self.ctypes.clone());
    py_attr_map_dict!(py, py_dict, "client_private_key_file", self.client_private_key_file.clone());
    py_attr_map_dict!(py, py_dict, "client_enable_http_ui", self.client_enable_http_ui);
    py_attr_map_dict!(py, py_dict, "client_http_port", self.client_http_port);
    py_attr_map_dict!(py, py_dict, "client_http_websocket_port", self.client_http_websocket_port);
    py_attr_map_dict!(py, py_dict, "client_http_custom_js", self.client_http_custom_js.clone());
    py_attr_map_dict!(py, py_dict, "client_http_custom_css", self.client_http_custom_css.clone());
    py_attr_map_dict!(py, py_dict, "client_use_sig", self.client_use_sig);
    py_attr_map_dict!(py, py_dict, "verbosity_level", self.verbosity_level);
    py_attr_map_dict!(py, py_dict, "servers", self.servers.clone());
    py_attr_map_dict!(py, py_dict, "rhai_scripts", self.rhai_scripts.clone());
    py_attr_map_dict!(py, py_dict, "server_listen_tcp", self.server_listen_tcp);
    py_attr_map_dict!(py, py_dict, "server_listen_udp", self.server_listen_udp);
    py_attr_map_dict!(py, py_dict, "server_listen_unix", self.server_listen_unix);
    py_attr_map_dict!(py, py_dict, "server_listen_websocket", self.server_listen_websocket);
    py_attr_map_dict!(py, py_dict, "server_listen_multicast", self.server_listen_multicast);
    py_attr_map_dict!(py, py_dict, "server_extra_quiet", self.server_extra_quiet);
    py_attr_map_dict!(py, py_dict, "server_max_listeners", self.server_max_listeners);
    py_attr_map_dict!(py, py_dict, "server_pid_file", self.server_pid_file.clone());
    py_attr_map_dict!(py, py_dict, "server_port", self.server_port);
    py_attr_map_dict!(py, py_dict, "server_websocket_port", self.server_websocket_port);
    py_attr_map_dict!(py, py_dict, "server_ip", self.server_ip.clone());
    py_attr_map_dict!(py, py_dict, "server_unix_socket", self.server_unix_socket.clone());
    py_attr_map_dict!(py, py_dict, "server_multicast_group", self.server_multicast_group.clone());
    py_attr_map_dict!(py, py_dict, "server_threads_in_flight", self.server_threads_in_flight);
    py_attr_map_dict!(py, py_dict, "server_threads_in_flight_fraction", self.server_threads_in_flight_fraction);
    py_attr_map_dict!(py, py_dict, "server_datastore_uri", self.server_datastore_uri.clone());
    py_attr_map_dict!(py, py_dict, "server_trusted_keys_file", self.server_trusted_keys_file.clone());
    py_attr_map_dict!(py, py_dict, "server_max_records", self.server_max_records);
    py_attr_map_dict!(py, py_dict, "server_max_unauth_websockets", self.server_max_unauth_websockets);
    py_attr_map_dict!(py, py_dict, "server_num_record_pools", self.server_num_record_pools);
    
    return py_dict;
  }
}

impl <'source> cpython::FromPyObject<'source> for config::Config {
  fn extract(py: Python, obj: &'source cpython::PyObject) -> PyResult<Self> {
    let py_dict: PyDict = obj.extract(py)?;
    
    let ctypes = 
      attr_from_py_dict!(py, py_dict, "ctypes", vec![], Vec<config::CType>);
    let client_private_key_file = 
      attr_from_py_dict!(py, py_dict, "client_private_key_file", String::new(), String);
    let client_enable_http_ui = 
      attr_from_py_dict!(py, py_dict, "client_enable_http_ui", false, bool);
    let client_http_port = 
      attr_from_py_dict!(py, py_dict, "client_http_port", 8080, u16);
    let client_http_websocket_port = 
      attr_from_py_dict!(py, py_dict, "client_http_websocket_port", 8081, u16);
    let client_http_custom_js = 
      attr_from_py_dict!(py, py_dict, "client_http_custom_js", include_str!("http/example_custom_js.js").to_string(), String);
    let client_http_custom_css = 
      attr_from_py_dict!(py, py_dict, "client_http_custom_css", include_str!("http/example_custom_css.css").to_string(), String);
    let client_use_sig = 
      attr_from_py_dict!(py, py_dict, "client_use_sig", false, bool);
    let verbosity_level = 
      attr_from_py_dict!(py, py_dict, "verbosity_level", 0, u8);
    let servers = 
      attr_from_py_dict!(py, py_dict, "servers", vec![], Vec<config::Server>);
    let rhai_scripts = 
      attr_from_py_dict!(py, py_dict, "rhai_scripts", vec![], Vec<String>);
    let server_listen_tcp = 
      attr_from_py_dict!(py, py_dict, "server_listen_tcp", true, bool);
    let server_listen_udp = 
      attr_from_py_dict!(py, py_dict, "server_listen_udp", true, bool);
    let server_listen_unix = 
      attr_from_py_dict!(py, py_dict, "server_listen_unix", true, bool);
    let server_listen_websocket = 
      attr_from_py_dict!(py, py_dict, "server_listen_websocket", true, bool);
    let server_listen_multicast = 
      attr_from_py_dict!(py, py_dict, "server_listen_multicast", true, bool);
    let server_extra_quiet = 
      attr_from_py_dict!(py, py_dict, "server_extra_quiet", false, bool);
    let server_max_listeners = 
      attr_from_py_dict!(py, py_dict, "server_max_listeners", 100, usize);
    let server_pid_file = 
      attr_from_py_dict!(py, py_dict, "server_pid_file", "/tmp/dindex.pid".to_string(), String);
    let server_port = 
      attr_from_py_dict!(py, py_dict, "server_port", config::DINDEX_DEF_PORT, u16);
    let server_websocket_port = 
      attr_from_py_dict!(py, py_dict, "server_websocket_port", config::DINDEX_DEF_WEBSOCKET_PORT, u16);
    let server_ip = 
      attr_from_py_dict!(py, py_dict, "server_ip", "0.0.0.0".to_string(), String);
    let server_unix_socket = 
      attr_from_py_dict!(py, py_dict, "server_unix_socket", "/tmp/dindex.sock".to_string(), String);
    let server_multicast_group = 
      attr_from_py_dict!(py, py_dict, "server_multicast_group", "239.255.29.224".to_string(), String);
    let server_threads_in_flight = 
      attr_from_py_dict!(py, py_dict, "server_threads_in_flight", 8, usize);
    let server_threads_in_flight_fraction = 
      attr_from_py_dict!(py, py_dict, "server_threads_in_flight_fraction", 0.25, f64);
    let server_datastore_uri = 
      attr_from_py_dict!(py, py_dict, "server_datastore_uri", "file:///tmp/dindex_db.json".to_string(), String);
    let server_trusted_keys_file = 
      attr_from_py_dict!(py, py_dict, "server_trusted_keys_file", "/tmp/dindex_trusted_keys".to_string(), String);
    let server_max_records = 
      attr_from_py_dict!(py, py_dict, "server_max_records", 4096, usize);
    let server_max_unauth_websockets = 
      attr_from_py_dict!(py, py_dict, "server_max_unauth_websockets", 100, usize);
    let server_num_record_pools = 
      attr_from_py_dict!(py, py_dict, "server_num_record_pools", 8, usize);
    
    Ok(config::Config {
      ctypes: ctypes,
      client_private_key_file: client_private_key_file,
      client_enable_http_ui: client_enable_http_ui,
      client_http_port: client_http_port,
      client_http_websocket_port: client_http_websocket_port,
      client_http_custom_js: client_http_custom_js,
      client_http_custom_css: client_http_custom_css,
      client_use_sig: client_use_sig,
      verbosity_level: verbosity_level,
      servers: servers,
      rhai_scripts: rhai_scripts,
      server_listen_tcp: server_listen_tcp,
      server_listen_udp: server_listen_udp,
      server_listen_unix: server_listen_unix,
      server_listen_websocket: server_listen_websocket,
      server_listen_multicast: server_listen_multicast,
      server_extra_quiet: server_extra_quiet,
      server_max_listeners: server_max_listeners,
      server_pid_file: server_pid_file,
      server_port: server_port,
      server_websocket_port: server_websocket_port,
      server_ip: server_ip,
      server_unix_socket: server_unix_socket,
      server_multicast_group: server_multicast_group,
      server_threads_in_flight: server_threads_in_flight,
      server_threads_in_flight_fraction: server_threads_in_flight_fraction,
      server_datastore_uri: server_datastore_uri,
      server_trusted_keys_file: server_trusted_keys_file,
      server_max_records: server_max_records,
      server_max_unauth_websockets: server_max_unauth_websockets,
      server_num_record_pools: server_num_record_pools,
    })
  }
}

/*
 * Functions similar to those found in ext.rs
 * but tailored for idiomatic python use.
 */

fn get_args(_: Python) -> PyResult<Args> {
  Ok( Args::from_args() )
}

fn get_config(_: Python, given_args: Option<Args>) -> PyResult<config::Config> {
  if let Some(given_args) = given_args {
    Ok(config::read_config( &given_args ))
  }
  else {
    // default structopt calls exit() if bad args so we create empty args
    // if the given action is bad
    let a = Args::from_iter_safe( std::env::args() );
    match a {
      Ok(a) => {
        Ok(config::read_config( &a ))
      }
      Err(_e) => {
        Ok(config::read_config( &Args::empty() ))
      }
    }
  }
}

// This _should_ just handle allocation, but our default argument makes
// it very ergonomic to use in python AND we don't have to detect None!
fn get_record(_: Python, rec: Record) -> PyResult<Record> {
  Ok(rec)
}

fn record_display(py: Python, config: Config, rec: Record) -> PyResult<cpython::PyObject> {
  disp::print_results_ref(&config, &vec![&rec]);
  Ok(py.None())
}

fn record_display_vec(py: Python, config: Config, rec: Vec<Record>) -> PyResult<cpython::PyObject> {
  disp::print_results(&config, &rec);
  Ok(py.None())
}

fn client_query_sync(_py: Python, config: Config, rec: Record) -> PyResult<Vec<Record>> {
  let results = client::query_sync(&config, &rec);
  Ok(results)
}
