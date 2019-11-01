#!/usr/bin/env python3

import sys
import os

import random

# Add dindex to the library search path; this is not necessary if
# dindex is already on your OS library path.
sys.path.append(os.path.abspath("../target/release/"))

import libdindex as dindex

config = dindex.config()

query = dindex.record({
  "url": ".*example.*"
})

print("dIndex query record:")
dindex.record_display(config, query)

# Create a callback; a lambda may suffice for simple logic but is untested
def on_record_match(rec):
  print("Python was listening and heard a matching record:")
  dindex.record_display(config, rec)
  if random.random() < 0.5:
    return "Continue" # must match one of client::ListenAction variants
  else:
    print("Python is sending EndListen...")
    return "EndListen"

dindex.client_listen_sync(config, query, on_record_match)

print("Python is done listening for records")

