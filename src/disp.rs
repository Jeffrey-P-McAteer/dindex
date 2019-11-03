
use crate::config;
use crate::record;

pub fn print_results(_config: &config::Config, results: &Vec<record::Record>) {
  // Sort by server name
  let mut results = results.clone();
  results.sort_by(|a, b| {
    if let Some(a_svr) = &a.src_server {
      if let Some(b_svr) = &b.src_server {
        if let Some(order) = a_svr.name.partial_cmp(&b_svr.name) {
          return order;
        }
      }
    }
    return std::cmp::Ordering::Equal;
  });
  
  let mut last_svr_name = String::new();
  for res in results {
    if let Some(svr) = &res.src_server {
      if ! svr.name.eq(&last_svr_name) {
        // Move to new group of server records, print the header
        last_svr_name = svr.name.clone();
        println!("=== {} ===", last_svr_name);
      }
    }
    println!("res = {:?}", res.p);
  }
  
}

pub fn print_results_ref(_config: &config::Config, results: &Vec<&record::Record>) {
  // Sort by server name
  let mut results = results.clone();
  results.sort_by(|a, b| {
    if let Some(a_svr) = &a.src_server {
      if let Some(b_svr) = &b.src_server {
        if let Some(order) = a_svr.name.partial_cmp(&b_svr.name) {
          return order;
        }
      }
    }
    return std::cmp::Ordering::Equal;
  });
  
  let mut last_svr_name = String::new();
  for res in results {
    if let Some(svr) = &res.src_server {
      if ! svr.name.eq(&last_svr_name) {
        // Move to new group of server records, print the header
        last_svr_name = svr.name.clone();
        println!("=== {} ===", last_svr_name);
      }
    }
    println!("res = {:?}", res.p);
  }
  
  
}


