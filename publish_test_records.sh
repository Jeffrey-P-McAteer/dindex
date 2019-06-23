#!/bin/sh

export RUST_BACKTRACE=1

#TRGT=release
TRGT=debug

if [ "$TRGT" = release ] ; then
  cargo build --release --bin dindex-client
else
  cargo build --bin dindex-client
fi

./target/$TRGT/dindex-client publish :webpage 'https://jmcateer.pw' 'a blog of some kind'
./target/$TRGT/dindex-client publish :webpage 'https://github.com' 'an awesome place to host code!'
./target/$TRGT/dindex-client publish :webpage 'https://google.com' 'an old, slow, biased search engine'

# This is how one can index an existing public site they do not control
./target/$TRGT/dindex-client publish --publish-site-pages 'http://example.org/' --max 5
./target/$TRGT/dindex-client publish --publish-site-pages 'https://jmcateer.pw/' --max 10
./target/$TRGT/dindex-client publish --publish-site-pages 'https://wikipedia.org/' --max 10
./target/$TRGT/dindex-client publish --publish-site-pages 'https://news.ycombinator.com/' --max 10

./target/$TRGT/dindex-client query :webpage '.*'

