# Todo

_A short list of the next things being targeted for implementation_

 - Refactor tcp/udp/unix business logic into a single `fn` instead of the current copy-past-orama.
  - Possibly make generic parsing `fn` that accepts anything implementing `Read` and `Write`?
  - ^ what about UDP?

 - Add back in web-scraping utilities, incl. some kind of configurable site-watching daemon.
 
 - Implement TCP listening capability - we can test by running a server, a site-watching client, and a listening client. The listening client should _immediately_ receive changes when the site-watching client publishes them.

 - Implement Unix and UDP listening

 - Implement record signing and verification client-side
  - nice-to-have: a client command to just generate a quick default identity (`~/.dindex.identity{.pub}`?)

 - Implement signed queries server-side (when all keys are signed any trusted key be able to grant more capabilities/remove anon limits/etc)


