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
use cpython::{PyResult, PyDict, Python};

use structopt::StructOpt;

use crate::record::Record;
use crate::config;
use crate::config::Config;
use crate::args::Args;
use crate::disp;
use crate::client;

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
    let mut py_dict: PyDict = PyDict::new(py);
    
    py_attr_map_dict!(py, py_dict, "config_file", self.config_file.clone());
    py_attr_map_dict!(py, py_dict, "max_web_scan_depth", self.max_web_scan_depth);
    py_attr_map_dict!(py, py_dict, "verbose", self.verbose);
    py_attr_map_dict!(py, py_dict, "action", format!("{}", self.action));
    py_attr_map_dict!(py, py_dict, "signed", self.signed);
    py_attr_map_dict!(py, py_dict, "rec_args", self.rec_args.clone());
    
    return py_dict;
  }
}

impl <'source> cpython::FromPyObject<'source> for Args {
  fn extract(py: Python, obj: &'source cpython::PyObject) -> PyResult<Self> {
    //let obj: cpython::PyObject = (*obj).clone();
    let py_dict: PyDict = obj.extract(py)?;
    std::unimplemented!()
  }
}

impl cpython::ToPyObject for config::Config {
  type ObjectType = PyDict;
  fn to_py_object(&self, _py: Python) -> Self::ObjectType {
    std::unimplemented!()
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

