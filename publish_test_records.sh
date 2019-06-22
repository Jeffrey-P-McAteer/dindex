#!/bin/sh

cargo build --release --bin dindex-client


./target/release/dindex-client publish :webpage 'https://jmcateer.pw' 'a blog of some kind'
./target/release/dindex-client publish :webpage 'https://github.com' 'an awesome place to host code!'
./target/release/dindex-client publish :webpage 'https://google.com' 'an old, slow, biased search engine'

# This is how one can index an existing public site they do not control
./target/release/dindex-client publish --publish-site-pages 'http://example.org/' --max 5
./target/release/dindex-client publish --publish-site-pages 'https://jmcateer.pw/' --max 10
./target/release/dindex-client publish --publish-site-pages 'https://wikipedia.org/' --max 10
./target/release/dindex-client publish --publish-site-pages 'https://news.ycombinator.com/' --max 10

./target/release/dindex-client query :webpage '.*'

