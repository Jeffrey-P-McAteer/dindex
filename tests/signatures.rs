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
  
  let mut random_record = gen_rand_record();
  
  dindex::signing::maybe_sign_record(&test_config, &mut random_record);
  
  assert!(dindex::signing::is_valid_sig(&random_record));
  
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