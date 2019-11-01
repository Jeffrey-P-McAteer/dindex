#!/usr/bin/env python3

import sys
import os

# Add dindex to the library search path; this is not necessary if
# dindex is already on your OS library path.
sys.path.append(os.path.abspath("../target/release/"))

import libdindex as dindex

config = dindex.config()
print(config)

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

