#include <stdio.h>
#include <time.h>

// This is available b/c we added it using the flag
// -I../target/ffi-include/
#include <dindex.h>

int main(int argc, char** argv) {
  Config* config = dindex_config(NULL /* alternatively give result of dindex_args() */);
  
  Record* my_doc = dindex_record_empty();
  dindex_record_put(my_doc, "title", "Example Webpage");
  dindex_record_put(my_doc, "url", "http://example.org");
  dindex_record_put(my_doc, "description", "Lorem Ipsum Description");
  
  { // Add a date to let the user see if our published record went full-circle.
    char date_txt[128];
    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    strftime(date_txt, sizeof(date_txt)-1, "%Y-%m-%d %H:%M", t);
    dindex_record_put(my_doc, "publish-date-time", date_txt);
  }
  
  printf("We are publishing the following record:\n");
  dindex_record_display(config, my_doc);
  
  dindex_client_publish_sync(config, my_doc);
  
  // Now my_doc should be returned from queries to any server
  // we had in our config.toml
  
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

