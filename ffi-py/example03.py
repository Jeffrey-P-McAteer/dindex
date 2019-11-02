#!/usr/bin/env python3

import sys
import os
from datetime import datetime

# Add dindex to the library search path; this is not necessary if
# dindex is already on your OS library path.
sys.path.append(os.path.abspath("../target/release/"))

import libdindex as dindex

config = dindex.config()

now = datetime.now()

my_doc = dindex.record({
  "title": "Example Webpage",
  "url": "http://example.org",
  "description": "Lorem Ipsum Description",
  "publish-date-time": now.strftime("%Y-%m-%d %H:%M")
})

dindex.client_publish_sync(config, my_doc)

# Now my_doc should be returned from queries to any server
# we had in our config.toml

query = dindex.record({
  "url": ".*example.*"
})
print(query)

print("dIndex query record:")
dindex.record_display(config, query)

results = dindex.client_query_sync(config, query)

print("Python received {} results".format(len(results)))

print("dIndex query results:")
dindex.record_display_vec(config, results)

