#include <stdio.h>

// This is available b/c we added it using the flag
// -I../target/ffi-include/
#include <dindex.h>

int main(int argc, char** argv) {
  Config* config = dindex_config(NULL /* alternatively give result of dindex_args() */);
  Record* query = dindex_record_empty();
  dindex_record_put(query, "url", ".*example.*");
  
  printf("dIndex query record:\n");
  dindex_record_display(config, query);
  
  RecordVec* results = dindex_client_query_sync(config, query);
  
  printf("dIndex query results:\n");
  dindex_record_display_vec(config, results);
  
  // Cleanup
  dindex_record_vec_free(results);
  dindex_record_free(query);
  dindex_config_free(config);
  
  return 0;
}

