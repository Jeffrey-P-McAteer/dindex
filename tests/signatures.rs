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

use dindex;

#[test]
fn sign_records() {
  let test_identity_f = "/tmp/dindex-test.identity";
  dindex::signing::gen_identity(test_identity_f.clone());
  
  let mut test_config = dindex::config::get_config_detail(
    false, false, false, false,
    Err(std::env::VarError::NotPresent),
    &dindex::args::Args::empty()
  );
  test_config.client_use_sig = true;
  test_config.client_private_key_file = test_identity_f.to_string();
  test_config.server_extra_quiet = true;
  
  let mut random_record = gen_rand_record();
  
  dindex::signing::maybe_sign_record(&test_config, &mut random_record);
  
  assert!(dindex::signing::is_valid_sig(&random_record));
  
  // Now have some contrived attack scenarios
  
  let mut known_record = {
    let mut rec = dindex::record::Record::empty();
    rec.p.insert("NAME".to_string(), "Lorem Ipsum".to_string());
    rec.p.insert("NUMBER".to_string(), "1112224444".to_string());
    rec
  };
  
  let mut known_record_diff_order = {
    let mut rec = dindex::record::Record::empty();
    rec.p.insert("NUMBER".to_string(), "1112224444".to_string());
    rec.p.insert("NAME".to_string(), "Lorem Ipsum".to_string());
    rec
  };
  
  let mut unrelated_similar_record = {
    let mut rec = dindex::record::Record::empty();
    rec.p.insert("NUMBER".to_string(), "3331115555".to_string());
    rec.p.insert("NAME".to_string(), "Alice Bob".to_string());
    rec
  };
  
  let mut unrelated_unsimilar_record = {
    let mut rec = dindex::record::Record::empty();
    rec.p.insert("FOO".to_string(), "3331115555".to_string());
    rec.p.insert("BAR".to_string(), "Alice Bob".to_string());
    rec
  };
  
  dindex::signing::maybe_sign_record(&test_config, &mut known_record);
  assert!(dindex::signing::is_valid_sig(&known_record));
  
  dindex::signing::maybe_sign_record(&test_config, &mut known_record_diff_order);
  assert!(dindex::signing::is_valid_sig(&known_record_diff_order));
  
  dindex::signing::maybe_sign_record(&test_config, &mut unrelated_similar_record);
  assert!(dindex::signing::is_valid_sig(&unrelated_similar_record));
  
  dindex::signing::maybe_sign_record(&test_config, &mut unrelated_unsimilar_record);
  assert!(dindex::signing::is_valid_sig(&unrelated_unsimilar_record));
  
  // Signatures for messages should be identical no matter the order their keys are in
  assert_eq!(
    known_record.p.get(dindex::signing::SIGNING_NON_SIG_BYTES_KEY).unwrap().to_string(),
    known_record_diff_order.p.get(dindex::signing::SIGNING_NON_SIG_BYTES_KEY).unwrap().to_string()
  );
  
  // Signatures for different messages should be different
  assert_ne!(
    known_record.p.get(dindex::signing::SIGNING_NON_SIG_BYTES_KEY).unwrap().to_string(),
    unrelated_unsimilar_record.p.get(dindex::signing::SIGNING_NON_SIG_BYTES_KEY).unwrap().to_string()
  );
  
  // Signatures for different messages should be different
  assert_ne!(
    known_record.p.get(dindex::signing::SIGNING_NON_SIG_BYTES_KEY).unwrap().to_string(),
    unrelated_similar_record.p.get(dindex::signing::SIGNING_NON_SIG_BYTES_KEY).unwrap().to_string()
  );
  
  let imposter_record = {
    let mut rec = dindex::record::Record::empty();
    // Change: kept same letters, moved letters around
    rec.p.insert("NAME".to_string(), "Ipsum Lorem".to_string());
    rec.p.insert("NUMBER".to_string(), "4444111222".to_string());
    
    // Because the public key and signature are public we can "copy" them.
    // This test ensures that copied signatures will not be valid for any permutation
    // other than the original document contents.
    
    rec.p.insert(dindex::signing::SIGNING_PUB_KEY_KEY.to_string(),
      known_record.p.get(dindex::signing::SIGNING_PUB_KEY_KEY).unwrap().to_string());
    
    rec.p.insert(dindex::signing::SIGNING_NON_SIG_BYTES_KEY.to_string(),
      known_record.p.get(dindex::signing::SIGNING_NON_SIG_BYTES_KEY).unwrap().to_string());
    
    rec
  };
  
  assert!(!dindex::signing::is_valid_sig(&imposter_record));
  
}


fn gen_rand_record() -> dindex::record::Record {
  use rand::{thread_rng, Rng};
  use rand::distributions::Alphanumeric;

  let mut rng = rand::thread_rng();
  let mut rec = dindex::record::Record::empty();
  let num_pairs: usize = rng.gen_range(2, 6);
  for _ in 0..num_pairs {
    let key_len: usize = rng.gen_range(2, 64);
    let rand_key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(key_len)
        .collect();
        
    let val_len: usize = rng.gen_range(8, 512);
    let rand_val: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(val_len)
        .collect();
        
    rec.p.insert(rand_key, rand_val);
  }
  rec
}

// Chaos monkey stuff here - unicode, invalid unicode, an null bytes are valid
// key and value tokens
fn gen_rand_junk_record() -> dindex::record::Record {
  use rand::{thread_rng, Rng};
  
  let mut rng = rand::thread_rng();
  let mut rec = dindex::record::Record::empty();
  let num_pairs: usize = rng.gen_range(2, 6);
  for _ in 0..num_pairs {
    let key_len: usize = rng.gen_range(2, 64);
    let rand_key: String = thread_rng()
        .sample_iter(&UnicodeAndJunkCharsDist)
        .take(key_len)
        .collect();
        
    let val_len: usize = rng.gen_range(8, 512);
    let rand_val: String = thread_rng()
        .sample_iter(&UnicodeAndJunkCharsDist)
        .take(val_len)
        .collect();
        
    rec.p.insert(rand_key, rand_val);
  }
  rec
}

#[derive(Debug)]
pub struct UnicodeAndJunkCharsDist;

// Highly unsafe, designed to maximize the probability of breaking things.
impl rand::distributions::Distribution<char> for UnicodeAndJunkCharsDist {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> char {
        loop {
            let var = rng.next_u32();
            // if let Some(nonsense_char) = core::char::from_u32(var) {
            //   return nonsense_char;
            // }
            unsafe { // The goal is to break things guys
              return std::char::from_u32_unchecked(var);
            }
        }
    }
}



#[test]
fn junk_input_tests() {
  let test_identity_f = "/tmp/dindex-test.identity.2";
  dindex::signing::gen_identity(test_identity_f.clone());
  
  let mut test_config = dindex::config::get_config_detail(
    false, false, false, false,
    Err(std::env::VarError::NotPresent),
    &dindex::args::Args::empty()
  );
  test_config.client_use_sig = true;
  test_config.client_private_key_file = test_identity_f.to_string();
  test_config.server_extra_quiet = true;
  
  for _ in 0..1000 {
    let mut junk_data = gen_rand_junk_record();
    dindex::signing::maybe_sign_record(&test_config, &mut junk_data);
    assert!(dindex::signing::is_valid_sig(&junk_data));
  }
  
}
