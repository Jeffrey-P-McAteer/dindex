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
use cpython::{PyResult, PyDict, PyLong, Python};

use structopt::StructOpt;

use crate::record::Record;
use crate::config;
use crate::config::Config;
use crate::args::Args;
use crate::disp;
use crate::client;
use crate::actions;

use crate::py_attr_map_dict;

// add bindings to the generated python module
// N.B: names: "libdindex" must be the name of the `.so` or `.pyd` file
py_module_initializer!(libdindex, initlibdindex, PyInit_libdindex, |py, m| {
  m.add(py, "__doc__", r#"
dIndex python bindings.
"#)?;
  m.add(py, "args", py_fn!(py, get_args()))?;
  m.add(py, "config", py_fn!(py, get_config(args: Option<Args> = None)))?;
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
    
    let config_file: Option<String>;
    if let Some(f) = py_dict.get_item(py, "config_file") {
      if let Ok(s) = f.extract(py) {
        config_file = Some(s);
      }
      else {
        config_file = None;
      }
    }
    else {
      config_file = None;
    }
    
    let max_web_scan_depth: usize;
    if let Some(d) = py_dict.get_item(py, "max_web_scan_depth") {
      if let Ok(i) = d.extract(py) {
        max_web_scan_depth = i;
      }
      else {
        max_web_scan_depth = 12;
      }
    }
    else {
      max_web_scan_depth = 12;
    }
    
    let verbose: u8;
    if let Some(d) = py_dict.get_item(py, "verbose") {
      if let Ok(i) = d.extract(py) {
        verbose = i;
      }
      else {
        verbose = 0;
      }
    }
    else {
      verbose = 0;
    }
    
    let action: actions::Action;
    if let Some(d) = py_dict.get_item(py, "action") {
      if let Ok(s) = d.extract(py) {
        let s: String = s;
        action = actions::action_from_str(&s);
      }
      else {
        action = actions::Action::no_action;
      }
    }
    else {
      action = actions::Action::no_action;
    }
    
    let signed: bool;
    if let Some(d) = py_dict.get_item(py, "signed") {
      if let Ok(b) = d.extract(py) {
        signed = b;
      }
      else {
        signed = false;
      }
    }
    else {
      signed = false;
    }
    
    let rec_args: Vec<String>;
    if let Some(d) = py_dict.get_item(py, "rec_args") {
      if let Ok(v) = d.extract(py) {
        rec_args = v;
      }
      else {
        rec_args = vec![];
      }
    }
    else {
      rec_args = vec![];
    }
    
    // TODO we can probably macro-out the above logic, it's pretty regular
    
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

