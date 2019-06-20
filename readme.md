
# dIndex

This repo holds software which manages a _distributed index_, in the most
abstract meaning of the term. The index doesn't need to hold websites; it can hold
phone number records and classroom assignments that professors publish.

dIndex records have 2 important properties:

  * Extensibility; you should be able to add your own properties and data types to a record
  * Programmability; queries should be repeatable and mechanical

This means that you can query on more properties than simply text content, a url, or an article title.

Query capabilities planned include:

 - [ ] document titles
 - [ ] document file type (`.pdf`, `.jpg`, `.mp4`, etc)
 - [ ] GPG signature status (`signed`, `unsigned`, `invalid signature`)
 - [ ] GPG signature source (`somebody@site.com`)
 - [ ] source hostname
 - [ ] popularity (computed by total number of unique submitting clients per hour? algo uncertain.)
 - [ ] document/asset/file total size (`<=12kb`, '`>15mb`, etc.)

This plan does not address the problem of spam content submission to the index;
ideally the search capabilities will be robust enough to make filtering through
spam easy to do at a user level. Building in spam "protection" would require making
the search algorithm non-deterministic, which is something I want to avoid.

I think spam will be best combated by the inclusion of GPG signatures early in the design.
Well-known news sources can make their content more trustworthy by signing it,
and users can query only signed content to filter through spam.
This moves the spam problem from the architecture of the program to the program's use,
and it frees users to subscribe to their own black/white-lists of various content publishers.
 

# Architecture

dIndex is made up of a server program (`dindex-server`), a client program (`dindex-client`, and a library.

The client is responsible for parsing user queries and displaying results.
The client may also submit new content to be stored in the index.

The server handles responding to queries and updating the index when clients tell it to do so,
and it also federates searches to other servers. This means no single server
needs to have all records for the records to be available to clients.

# Security implications

Because dIndex uses UDP for communication it could potentially be used
in a DoS amplification attack. To prevent this `dindex-server` will have
a whitelist of RSA, ECDSA, and GPG keys that it trusts.
By default the size of responses will be limited to the number of bytes received from the client, and anonomous clients will pad their queries accordingly. If the query comes in and is signed by a trusted key, this limitation is removed.

Other limits such as the total bytes per IP per second may be set by the `dindex-server` operator,
be they a single person, a university, or some government organization.

This will make amplification attacks possible only if the attacker can make the
server trust a large number of keys they control. Managing how these keys are
trusted is outside the scope of `dindex-server`. Universities may simply assign a key to each
student, companies can tie keys to credit cards to prove uniqueness, whatever. 



# Example use

TODO





