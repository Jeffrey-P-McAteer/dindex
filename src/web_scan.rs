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

use url_crawler;
use url_crawler::*;

use webpage;
use webpage::{Webpage, WebpageOptions};

use crate::h_map;
use crate::record::Record;
use crate::config::Config;
use crate::args::Args;

// TODO custom traversal Lua scripts in Config?
pub fn scan_urls<F: Fn(Record) + Send + Copy>(config: &Config, args: &Args, urls: Vec<String>, callback: F) {
  for url in urls {
    scan_url(config, args, url, callback);
  }
}

pub fn scan_url<F: Fn(Record) + Send + Copy>(config: &Config, args: &Args, url: String, callback: F) {
  println!("Scanning \"{}\"", &url);
  
  let crawler = Crawler::new(url.clone())
      .threads(4)
      .crawl();
  
  if let Ok(rec) = urlentry_to_record(UrlEntry::Html { url: Url::parse(&url).unwrap() }) {
    callback(rec);
  }
  
  let mut remaining_scans = args.max_web_scan_depth;
  for file in crawler {
    println!("Scanned {:?}", file);
    match urlentry_to_record(file) {
      Ok(rec) => {
        callback(rec);
        remaining_scans -= 1;
        if remaining_scans < 1 {
          break;
        }
      }
      Err(e) => {
        println!("Error scanning: {}", e);
      }
    }
  }
  
}

fn urlentry_to_record(url: UrlEntry) -> Result<Record, ::std::io::Error> {
  match url {
    UrlEntry::Html{url} => {
      let info = Webpage::from_url(url.as_str(), WebpageOptions::default())?;
      let html = info.html;
      Ok(Record {
        p: h_map!{
          "url".to_string() => url.to_string(),
          "title".to_string() => html.title.unwrap_or(url.to_string()),
          "description".to_string() => html.description.unwrap_or(String::new())
        },
        src_server: None,
      })
    }
    _ => Ok(Record::empty()) // TODO
  }
}
