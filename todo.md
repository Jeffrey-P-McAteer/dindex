# Todo

_A short list of the next things being targeted for implementation_

 - Rewrite signatures.rs to use 3rd-party binaries like `openssl` and `sshign`, falling back to a useful error if there are no tools installed. (this gives us portability to windorks systems)

 - Finish making fuzzing useful

 - Use Python FFI to write small LAN CLI videogame - records record player names + positions + motion

 - Implement client signature verification
 
 - Implement server dropping incoming signed records with bad invalid signature
 
 - Implement server query + listening federation

 - Implement server HTTP CGI gateway support?
   This feature would let you drop a SETUID binary in ~/public_html/
   and have it serve + receive records

 - Improve client customization with `rhai` scripts injected into processing

 - Implement SQL record storage backend? (low priority, lame + boring)

 - Fix various UDP inconsistencies (listening looks a bit broken)

