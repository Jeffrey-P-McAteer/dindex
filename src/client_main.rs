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

extern crate structopt;

use rand;

use structopt::StructOpt;

extern crate url_crawler;
extern crate webpage;

use std::thread;
use std::io::ErrorKind;

use dindex::config::get_config;
use dindex::config::Resolver;
use dindex::config::Config;
use dindex::Record;
use dindex::Args;
use dindex::SvrArgs;

fn main() {
  let args = Args::from_args();
  let config = get_config();
  
  println!("{:?}", args);
  
  if args.publish_site_pages.is_some() {
    do_publish_site_pages(config, args);
    return;
  }
  
  if args.docs {
    println!(include_str!("client_readme.md"));
    return;
  }
  
  let mut threads = vec![];
  for resolver in config.upstream_resolvers {
    let a = args.clone(); // TODO not this
    let th = thread::spawn(move || {
      instruct_resolver(&resolver, &a);
    });
    threads.push(th);
  }
  // Wait on all threads
  for th in threads {
    th.join().unwrap();
  }
}

fn instruct_resolver(r: &Resolver, args: &Args) {
  let svr_args = args.clone().into_svr_args();
  instruct_resolver_direct(r, &svr_args);
}

fn instruct_resolver_direct(r: &Resolver, args: &SvrArgs) {
  use std::time::{Duration, Instant};
  use rand::Rng;
  use std::net::UdpSocket;
  
  println!("Querying {}", r.get_host_port_s());
  
  let mut rng = rand::thread_rng();
  //let mut client = victorem::ClientSocket::new(rng.gen_range(11111, 55555), r.get_host_port_s()).unwrap();
  let local_addr = format!("0.0.0.0:{}", rng.gen_range(11111, 55555));
  let sock = UdpSocket::bind(&local_addr).expect("Failed to bind socket");
  if let Err(e) = sock.set_nonblocking(true) {
    println!("Failed to enter non-blocking mode: {}", e);
    println!("(program will hang without server response)");
  }
  
  let bytes_to_send = serde_cbor::to_vec(&args.clone()).unwrap();
  //sock.send_to(&bytes_to_send, r.get_host_port_s() ).expect("failed to send message");
  
  match sock.send_to(&bytes_to_send, r.get_host_port_s() ) {
    Ok(number_of_bytes) => {
      println!("Sent {} bytes to {}", number_of_bytes, r.get_host_port_s());
    },
    Err(e) => {
      println!("Error sending to {} - {:?}", r.get_host_port_s(), e);
      return;
    }
  }
  
  let timer = Instant::now();
  let period = Duration::from_millis(r.max_latency_ms as u64);
  
  // Create a new sleeper that trusts native thread::sleep with 100μs accuracy
  let spin_sleeper = spin_sleep::SpinSleeper::new(100_000);
  let sleep_delay = Duration::from_micros(10); // 100 = 0.1ms
  
  loop {
    
    if timer.elapsed() > period {
      println!("Timing out for resolver at {}", r.get_host_port_s());
      break;
    }
    
    let mut i = 0;
    let mut should_exit = false;
    let mut incoming_buf = [0u8; 65536];
    match sock.recv(&mut incoming_buf) {
      Ok(num_received) => {
        if let Ok(result) = serde_cbor::from_slice::<Record>(&incoming_buf[0..num_received]) {
          i += 1;
          println!("{} result {}: {:?}", r.get_host_port_s(), i, result.properties);
          if !should_exit && result.is_end_record() {
            should_exit = true;
          }
          if should_exit {
            break;
          }
        }
      }
      Err(ref err) if err.kind() != ErrorKind::WouldBlock => {
          println!("Error: {}", err);
      }
      _ => {
        spin_sleeper.sleep(sleep_delay);
      }
    }
  }
}

fn publish_to_resolver(r: &Resolver, record: &Record) {
  use dindex::ArgsAction;
  let svr_args = SvrArgs {
    action: ArgsAction::publish,
    record: record.clone(),
  };
  instruct_resolver_direct(r, &svr_args);
}

fn do_publish_site_pages(config: Config, args: Args) {
  use url_crawler::*;
  use std::time;
  
  match args.publish_site_pages {
    Some(url) => {
      println!("Crawling down to {} pages at {}", args.max, url);
      let first_url = url.clone();
      let crawler = Crawler::new(url)
        .threads(4)
        .crawl();
      
      let mut new_records: Vec<Record> = vec![];
      
      if let Ok(rec) = urlentry_to_record(url_crawler::UrlEntry::Html { url: Url::parse(&first_url).unwrap() }) {
        new_records.push(rec);
      }
      
      let mut i = 0;
      for file in crawler {
        println!("Crawled {:?}", file);
        match urlentry_to_record(file) {
          Ok(rec) => {
            new_records.push(rec);
            i += 1;
            if i > args.max {
              break;
            }
          }
          Err(e) => {
            println!("{}", e);
          }
        }
      }
      
      let mut threads = vec![];
      for new_record in &new_records {
        if new_record.is_empty() {
          continue;
        }
        println!("Publishing {:?}", new_record);
        for resolver in &config.upstream_resolvers {
          let resolver = resolver.clone(); // 
          let new_record = new_record.clone(); // OOF?
          let th = thread::spawn(move || {
            publish_to_resolver(&resolver, &new_record);
          });
          threads.push(th);
        }
        // Let some of those things go out before we jump forward
        thread::sleep(time::Duration::from_millis(1));
      }
      
      // Wait on all threads
      for th in threads {
        th.join().unwrap();
      }
      
    }
    None => {
      panic!("Should never happen");
    }
  }
}

fn urlentry_to_record(url: url_crawler::UrlEntry) -> Result<Record, ::std::io::Error> {
  use webpage::{Webpage, WebpageOptions};
  use url_crawler::UrlEntry;
  
  match url {
    UrlEntry::Html{url} => {
      let info = Webpage::from_url(url.as_str(), WebpageOptions::default())?;
      let html = info.html;
      Ok(Record::webpage_record(
        url.to_string(),
        html.title.unwrap_or(url.to_string()),
        html.description.unwrap_or(String::new()),
      ))
    }
    _ => Ok(Record::empty()) // TODO
  }
}
