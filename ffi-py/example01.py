#!/usr/bin/env python3

import sys
import os

# Add dindex to the library search path; this is not necessary if
# dindex is already on your OS library path.
sys.path.append(os.path.abspath("../target/release/"))

import libdindex as dindex

print(dindex.config())

