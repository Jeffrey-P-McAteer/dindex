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

use cpython::{PyResult, Python};

// use structopt::StructOpt;

// use crate::record::Record;
// use crate::config;
// use crate::config::Config;
// use crate::args::Args;
// use crate::disp;
// use crate::client;

// add bindings to the generated python module
// N.B: names: "libdindex" must be the name of the `.so` or `.pyd` file
py_module_initializer!(libdindex, initlibdindex, PyInit_libdindex, |py, m| {
  m.add(py, "__doc__", r#"
dIndex python bindings.
"#)?;
  m.add(py, "sum_as_string", py_fn!(py, sum_as_string_py(a: i64, b:i64)))?;
  Ok(())
});

// rust-cpython aware function. All of our python interface could be
// declared in a separate module.
// Note that the py_fn!() macro automatically converts the arguments from
// Python objects to Rust values; and the Rust return value back into a Python object.
fn sum_as_string_py(_: Python, a:i64, b:i64) -> PyResult<String> {
  let out = format!("{}", a + b).to_string();
  Ok(out)
}

