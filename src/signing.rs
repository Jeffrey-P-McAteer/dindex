/**
 *  dIndex - a distributed, organic, mechanical index for everything
 *  Copyright (C) 2019  Jeffrey McAteer <jeffrey.p.mcateer@gmail.com>
 *  
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; version 2 of the License only.
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

use openssl;
use openssl::sign::{Signer, Verifier};
use openssl::rsa::Rsa;
use openssl::pkey::{PKey, Private, Public};
use openssl::hash::MessageDigest;

use base64;

use std::fs;
use std::path::Path;
use std::collections::HashMap;

use crate::config::Config;
use crate::record::Record;

// Reserved key, holds base64 public key
pub const SIGNING_PUB_KEY_KEY: &str = "SIGNING:public-key";
// Reserved key, holds base64 signature of non_sig_bytes() for a record
pub const SIGNING_NON_SIG_BYTES_KEY: &str = "SIGNING:non-sig-bytes";

pub fn gen_identity(output_file: &str) {
  let rsa = Rsa::generate(2048).unwrap();
  match rsa.private_key_to_pem() {
    Ok(priv_pem_bytes) => {
      if let Err(e) = fs::write(output_file, priv_pem_bytes) {
        println!("Error writing identity file: {}", e);
      }
    }
    Err(e) => {
      println!("Error exporting PEM: {}", e);
    }
  }
}

pub fn maybe_sign_record(config: &Config, rec: &mut Record) {
  if ! config.client_use_sig {
    return;
  }
  let identity_file_path = config.client_private_key_file.clone();
  
  match read_file(&Path::new(&identity_file_path)) {
    Ok(identity_file_bytes) => {
      match try_parse_rsa(&identity_file_bytes) {
        Ok(rsa_pair) => {
          match PKey::from_rsa(rsa_pair) {
            Ok(generic_key_pair) => {
              sign_rec(&generic_key_pair, rec);
              if config.is_debug() && !config.server_extra_quiet {
                println!("Record after signing: {:?}", rec.p);
              }
            }
            Err(e) => {
              println!("Error making RSA keys generic: {}", e);
            }
          }
        }
        Err(e) => {
          println!("Error parsing identity file: {:?}", e);
        }
      }
    }
    Err(e) => {
      println!("Error reading identity private key: {}", e);
    }
  }
  
}

pub fn read_pub_key_base64(identity_file_path: &str) -> String {
  match read_file(&Path::new(identity_file_path)) {
    Ok(identity_file_bytes) => {
      match try_parse_rsa(&identity_file_bytes) {
        Ok(rsa_pair) => {
          match PKey::from_rsa(rsa_pair) {
            Ok(generic_key_pair) => {
              return base64::encode(&generic_key_pair.public_key_to_pem().unwrap_or(vec![]));
            }
            Err(e) => {
              println!("Error making RSA keys generic: {}", e);
            }
          }
        }
        Err(e) => {
          println!("Error parsing identity file: {:?}", e);
        }
      }
    }
    Err(e) => {
      println!("Error reading identity private key: {}", e);
    }
  }
  return String::new();
}

pub fn try_parse_rsa(bytes: &[u8]) -> Result<Rsa<Private>, ()> {
  if let Ok(ret) = Rsa::private_key_from_pem(bytes) {
    return Ok(ret);
  }
  if let Ok(ret) = Rsa::private_key_from_der(bytes) {
    return Ok(ret);
  }
  
  Err(())
}

pub fn try_parse_rsa_pub(bytes: &[u8]) -> Result<Rsa<Public>, ()> {
  if let Ok(ret) = Rsa::public_key_from_pem(bytes) {
    return Ok(ret);
  }
  if let Ok(ret) = Rsa::public_key_from_der(bytes) {
    return Ok(ret);
  }
  if let Ok(ret) = Rsa::public_key_from_der_pkcs1(bytes) {
    return Ok(ret);
  }
  
  Err(())
}

fn sign_rec(keypair: &PKey<Private>, rec: &mut Record) {
  let mut signatures: HashMap<String, String> = HashMap::new();
  
  signatures.insert(SIGNING_PUB_KEY_KEY.to_string(), base64::encode(&keypair.public_key_to_pem().unwrap()));
  signatures.insert(SIGNING_NON_SIG_BYTES_KEY.to_string(), sign_nonsig_bytes(keypair, rec));
  
  for (sign_key, sign_val) in signatures {
    rec.p.insert(sign_key, sign_val);
  }
}

fn sign_nonsig_bytes(keypair: &PKey<Private>, rec: &Record) -> String {
  let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
  
  // Signatures are done on the key concatinated with value, in that order (non-sig key value)
  signer.update(&non_sig_bytes(rec)).unwrap();
  let signature = signer.sign_to_vec().unwrap();
  
  return base64::encode(&signature);
}

fn read_file(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn has_sig_fields(rec: &Record) -> bool {
  // We OR instead of AND because we want to prevent
  // the possibility of a badly-configured server publishing partial
  // invalid records which are then treated has "not having sig fields"
  // when a public key is listed.
  return rec.p.contains_key(SIGNING_PUB_KEY_KEY) || rec.p.contains_key(SIGNING_NON_SIG_BYTES_KEY);
}

pub fn is_valid_sig(rec: &Record) -> bool {
  if ! rec.p.contains_key(SIGNING_PUB_KEY_KEY) {
    return false;
  }
  if ! rec.p.contains_key(SIGNING_NON_SIG_BYTES_KEY) {
    return false;
  }
  let empty_str = String::new();
  let pub_key_base64 = rec.p.get(SIGNING_PUB_KEY_KEY).unwrap_or(&empty_str);
  let pub_key_bytes = base64::decode(pub_key_base64).unwrap_or(pub_key_base64.as_bytes().to_vec());
  match try_parse_rsa_pub(&pub_key_bytes) {
    Ok(rsa_pub_key) => {
      match PKey::from_rsa(rsa_pub_key) {
        Ok(pkey) => {
          let base64_unsigned_sig = rec.p.get(SIGNING_NON_SIG_BYTES_KEY).unwrap_or(&empty_str);
          if ! check_nonsig_bytes(&pkey, rec, base64_unsigned_sig) {
            return false;
          }
          return true;
        }
        Err(e) => {
          println!("Error making RSA pub key generic: {}", e);
        }
      }
    }
    Err(e) => {
      println!("Error parsing RSA pub key: {:?}", e);
    }
  }
  // Some error occured, fail safe
  return false;
}

pub fn check_single_sig(pub_key: &PKey<Public>, value_key: &str, value: &str, sig_base64: &str) -> bool {
  let mut verifier = Verifier::new(MessageDigest::sha256(), &pub_key).unwrap();
  verifier.update(value_key.as_bytes()).unwrap();
  verifier.update(value.as_bytes()).unwrap();
  let sig = base64::decode(sig_base64).unwrap_or(vec![]);
  return verifier.verify(&sig).unwrap();
}

pub fn check_nonsig_bytes(pub_key: &PKey<Public>, rec: &Record, sig_base64: &str) -> bool {
  let mut verifier = Verifier::new(MessageDigest::sha256(), &pub_key).unwrap();
  verifier.update(&non_sig_bytes(rec)).unwrap();
  let sig = base64::decode(sig_base64).unwrap_or(vec![]);
  return verifier.verify(&sig).unwrap();
}

// Parses out all non T-sig and public-key strings, sorting alphanumerically by keys
// then concatinating all KEY+VAL pairs.
pub fn non_sig_bytes(rec: &Record) -> Vec<u8> {
  use std::collections::BTreeMap;
  let mut bytes = vec![];
  
  let mut sorted_map = BTreeMap::new();
  for (key, val) in &rec.p {
    if !key_is_used_in_signing(key) {
      // TODO can we reference as a perf improvement?
      sorted_map.insert(key.clone(), val.clone());
    }
  }
  
  for (key, val) in sorted_map.iter() {
    bytes.extend(key.as_bytes());
    bytes.extend(val.as_bytes());
  }
  
  return bytes;
}

pub fn is_auth_by_server(rec: &Record, config: &Config) -> bool {
  use std::io::BufReader;
  use std::io::BufRead;
  use std::fs::File;
  
  if !is_valid_sig(rec) {
    return false; // Cannot be trusted by server if anon sigs aren't even correct
  }
  
  let new_s = String::new();
  let rec_pub_key_s = rec.p.get(SIGNING_PUB_KEY_KEY).unwrap_or(&new_s);
  
  if rec_pub_key_s.len() < 1 {
    return false; // no pub key given
  }
  
  match File::open(&config.server_trusted_keys_file) {
    Ok(f) => {
      let buff = BufReader::new(&f);
      for (_num, line) in buff.lines().enumerate() {
        if let Ok(line) = line {
          if line.starts_with("#") || line.trim().len() < 1 {
            continue;
          }
          if &line == rec_pub_key_s {
            return true;
          }
        }
      }
    }
    Err(e) => {
      println!("Error opening server_trusted_keys_file: {}", e);
    }
  }
  // Some error or nothing in auth file matches, fail safe
  return false;
}

// As reserved keys pile up, this method tracks reserved
// key patterns which are not considered user data when signing.
pub fn key_is_used_in_signing(key: &str) -> bool {
  key == SIGNING_PUB_KEY_KEY || key == SIGNING_NON_SIG_BYTES_KEY
}

