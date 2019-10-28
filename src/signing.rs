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

use ring::{rand, signature};
use ring::signature::{KeyPair, RsaKeyPair};
use base64;

use std::path::Path;
use std::collections::HashMap;

use crate::config::Config;
use crate::record::Record;

pub fn maybe_sign_record(config: &Config, rec: &mut Record) {
  if ! config.client_use_sig {
    return;
  }
  let identity_file_path = config.client_private_key_file.clone();
  
  match read_file(&Path::new(&identity_file_path)) {
    Ok(identity_file_bytes) => {
      match signature::RsaKeyPair::from_der(&identity_file_bytes) {
        Ok(key_pair) => {
          sign_rec(&key_pair, rec);
        }
        Err(e1) => {
          match signature::RsaKeyPair::from_pkcs8(&identity_file_bytes) {
            Ok(key_pair) => {
              sign_rec(&key_pair, rec);
            }
            Err(e2) => {
              println!("Error1 parsing private key: {}", e1);
              println!("Error2 parsing private key: {}", e2);
            }
          }
        }
      }
    }
    Err(e) => {
      println!("Error reading identity private key: {}", e);
    }
  }
  
}

fn sign_rec(keypair: &RsaKeyPair, rec: &mut Record) {
  let rng = rand::SystemRandom::new();
  let mut signatures: HashMap<String, String> = HashMap::new();
  for (key, val) in &rec.p {
    let (sign_key, sign_val) = sign_single(keypair, &rng, key.to_string(), val.to_string());
    signatures.insert(sign_key, sign_val);
  }
  
  signatures.insert("public-key".to_string(), base64::encode(&keypair.public_key()));
  
  for (sign_key, sign_val) in signatures {
    println!("Inserting {}, {}", &sign_key, &sign_val);
    rec.p.insert(sign_key, sign_val);
  }
}

fn sign_single(keypair: &RsaKeyPair, rng: &dyn rand::SecureRandom, key: String, val: String) -> (String, String) {
  let mut sig_buf: Vec<u8> = vec![0; keypair.public_modulus_len()];
  
  if let Err(e) = keypair.sign(
    &signature::RSA_PKCS1_SHA256,
    rng,
    val.as_bytes(),
    sig_buf.as_mut_slice()
  ) {
    println!("Error while signing: {}", e);
  }
  
  return (
    format!("{}-sig", key),
    base64::encode(&sig_buf),
  );
}

fn read_file(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}
