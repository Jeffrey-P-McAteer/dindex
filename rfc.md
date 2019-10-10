
# dIndex

This document is currently a mess of ideas, but eventually should be
formatted according to RFC 2360 (https://tools.ietf.org/html/rfc2360).

dIndex has great potential to replace all communication boilerplate
in common use today, and standardization benefits the world.

# High-level View

dIndex is a communication and memory system which uses server/client
architecture and multicast ring-style architecture. The possibility
of federation (similar to recursive DNS resolvers) also exists,
but is strictly out-of-scope of the standard.

All communication is done using a structure serialized using
the Concise Binary Object Representation (RFC 7049).

# Clients

Clients will send a CBOR object like

```
{
  "action": "publish", // one of "publish", "query", "listen"
  "record": {
    "key 1": "val 1",
    "key 2": "val 2"
  }
}
```

TODO TODO TODO

