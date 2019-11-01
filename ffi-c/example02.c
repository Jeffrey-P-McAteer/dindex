#include <stdio.h>
#include <stdlib.h> 
#include <time.h>
#include <dindex.h>

int main(int argc, char** argv) {
  // We use rand() to exit after receiving at least 1 record.
  srand(time(0));
  
  Config* config = dindex_config(NULL /* alternatively give result of dindex_args() */);
  Record* query = dindex_record_empty();
  dindex_record_put(query, "url", ".*");
  
  printf("dIndex query record:\n");
  dindex_record_display(config, query);
  
  printf("Listening for new records...\n");
  dindex_client_listen_sync(config, query, DINDEX_LAMBDA(const char* _(Record* result) {
    dindex_record_display(config, result);
    dindex_record_free(result);
    if (rand() & 1) {
      return "Continue"; // must match one of client::ListenAction variants
    }
    else {
      printf("Stopping...\n");
      return "EndListen";
    }
  }));
  
  // Cleanup
  dindex_record_free(query);
  dindex_config_free(config);
  
  return 0;
}

