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
pub const signing_pub_key_key: &str = "SIGNING:public-key";
// Reserved key, holds base64 signature of non_sig_bytes() for a record
pub const signing_non_sig_bytes_key: &str = "SIGNING:non-sig-bytes";
// Reserved for all keys that match the pattern T-sig for all T
pub const signing_sig_suffix: &str = "-sig";

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
  // for (key, val) in &rec.p {
  //   let (sign_key, sign_val) = sign_single(keypair, key.to_string(), val.to_string());
  //   signatures.insert(sign_key, sign_val);
  // }
  
  signatures.insert(signing_pub_key_key.to_string(), base64::encode(&keypair.public_key_to_pem().unwrap()));
  signatures.insert(signing_non_sig_bytes_key.to_string(), sign_nonsig_bytes(keypair, rec));
  
  for (sign_key, sign_val) in signatures {
    //println!("Inserting {}, {}", &sign_key, &sign_val);
    rec.p.insert(sign_key, sign_val);
  }
}

fn sign_single(keypair: &PKey<Private>, key: String, val: String) -> (String, String) {
  let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
  
  // Signatures are done on the key concatinated with value, in that order (non-sig key value)
  signer.update(key.as_bytes()).unwrap();
  signer.update(val.as_bytes()).unwrap();
  let signature = signer.sign_to_vec().unwrap();
  
  return (
    format!("{}-sig", key),
    base64::encode(&signature),
  );
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

pub fn is_valid_sig(rec: &Record) -> bool {
  if ! rec.p.contains_key(signing_pub_key_key) {
    return false;
  }
  if ! rec.p.contains_key(signing_non_sig_bytes_key) {
    return false;
  }
  let empty_str = String::new();
  let pub_key_base64 = rec.p.get(signing_pub_key_key).unwrap_or(&empty_str);
  let pub_key_bytes = base64::decode(pub_key_base64).unwrap_or(pub_key_base64.as_bytes().to_vec());
  match try_parse_rsa_pub(&pub_key_bytes) {
    Ok(rsa_pub_key) => {
      match PKey::from_rsa(rsa_pub_key) {
        Ok(pkey) => {
          // for (key, val) in &rec.p {
          //   if key == signing_pub_key_key {
          //     continue;
          //   }
          //   if key.ends_with(signing_sig_suffix) {
          //     let unsig_key = &key[0..key.len()-4];
          //     //println!("key={}  unsig_key={}", &key, &unsig_key);
          //     let unsigned_val = rec.p.get(unsig_key).unwrap_or(&empty_str);
          //     if ! check_single_sig(&pkey, unsig_key, unsigned_val, &val) {
          //       return false;
          //     }
          //   }
          // }
          // Now check entire message non signing_sig_suffix field signature
          let base64_unsigned_sig = rec.p.get(signing_non_sig_bytes_key).unwrap_or(&empty_str);
          if ! check_nonsig_bytes(&pkey, rec, base64_unsigned_sig) {
            return false;
          }
          
          // Every check passed, every signing_sig_suffix field is signed with the key from signing_pub_key_key
          // NB: what about fields without a signing_sig_suffix pair?
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

// Parses out all non T-sig and public-key strings, concatinating their
// byte representations. Used to create an HMAC of sorts to prevent
// an attack where signed key-value pairs can be spliced together in different Records.
// Bytes are sorted smallest -> largest to prevent having to deal with ordering of key-value pairs.
pub fn non_sig_bytes(rec: &Record) -> Vec<u8> {
  let mut bytes = vec![];
  for (key, val) in &rec.p {
    if !key_is_used_in_signing(key) {
      bytes.extend(val.as_bytes());
    }
  }
  bytes.sort();
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
  let rec_pub_key_s = rec.p.get(signing_pub_key_key).unwrap_or(&new_s);
  
  if rec_pub_key_s.len() < 1 {
    return false; // no pub key given
  }
  
  match File::open(&config.server_trusted_keys_file) {
    Ok(f) => {
      let buff = BufReader::new(&f);
      for (num, line) in buff.lines().enumerate() {
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
  key == signing_pub_key_key || key == signing_non_sig_bytes_key || key.ends_with(signing_sig_suffix)
}

