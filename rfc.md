
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

# Client - Server communication

Clients send a COBR object to publish, query, or begin listening for matching records.

Servers send back a CBOR object with 0xff at the end. 0xff is used to split multiple
records, such as when clients make a query and there are multiple results.

this _should_ never cause a problem with the COBR objects; RFC 7049 defines 0xff as
a "break" stop code, so the meaning should match semantically.

# Record Signing

A signed record differs from a regular record in that:

 - it contains a key "public-key" with a value that is a base64 string of an RSA public key (TODO doc format details)
 - for every key ending in "-sig" there is a corresponding key without "-sig", and the "-sig" value is a base64 encoded RSA_PKCS1_SHA256 signature of the non-sig key's value.

The following keys are therefore reserved for use signing records:

 - `public-key`
 - `T-sig` for all `T`


