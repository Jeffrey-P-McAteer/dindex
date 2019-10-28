
use crate::config;
use crate::record;

pub fn print_results(config: &config::Config, results: &Vec<record::Record>) {
  for res in results {
    // TODO custom formatting from config/ctypes/whatever
    println!("res = {:?}", res.p);
  }
}

pub fn print_results_ref(config: &config::Config, results: &Vec<&record::Record>) {
  for res in results {
    // TODO custom formatting from config/ctypes/whatever
    println!("res = {:?}", res.p);
  }
}


